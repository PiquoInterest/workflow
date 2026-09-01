import type { Event, WorkflowRun } from '@workflow/world';
import type { PayloadKey } from './serialization/encryption.js';
import {
  type PreparedReplayPayload,
  prepareReplayPayload,
  type ReplayPayloadPreparer,
} from './serialization.js';

const MAX_MEMOIZED_PRIMITIVE_LENGTH = 4096;
type ReplayPayloadField = 'result' | 'error' | 'payload';

export const PAYLOAD_CONFLICT_MESSAGE =
  'replay payload cache key was reused with different binary data';
export const PAYLOAD_CONFLICT_CODE = 'REPLAY_PAYLOAD_CONFLICT' as const;
export const REENTRANT_PREPARATION_MESSAGE =
  'replay payload preparation re-entered the same cache key';
export const REENTRANT_PREPARATION_CODE =
  'REPLAY_PAYLOAD_REENTRANT' as const;

export class ReplayPayloadConflictError extends Error {
  readonly code = PAYLOAD_CONFLICT_CODE;

  constructor() {
    super(PAYLOAD_CONFLICT_MESSAGE);
    this.name = 'ReplayPayloadConflictError';
  }
}

export class ReplayPayloadReentrantError extends Error {
  readonly code = REENTRANT_PREPARATION_CODE;

  constructor() {
    super(REENTRANT_PREPARATION_MESSAGE);
    this.name = 'ReplayPayloadReentrantError';
  }
}

interface PreparationEntry {
  readonly originalBytes: Uint8Array;
  readonly preparation: Promise<PreparedReplayPayload>;
  readonly resolvePreparation: (value: PreparedReplayPayload) => void;
  readonly rejectPreparation: (reason: unknown) => void;
  invokingPreparer: boolean;
  settled: boolean;
  conflict?: ReplayPayloadConflictError;
  conflictPreparation?: Promise<PreparedReplayPayload>;
}

interface EnsuredPreparation {
  readonly entry: PreparationEntry;
  readonly preparation: Promise<PreparedReplayPayload>;
  readonly scheduleInPrewarm: boolean;
}

function isMemoizablePrimitive(value: unknown): boolean {
  if (value === null) return true;
  const type = typeof value;
  if (type === 'object' || type === 'function') return false;
  if (type === 'string') {
    return (value as string).length <= MAX_MEMOIZED_PRIMITIVE_LENGTH;
  }
  if (type === 'bigint') {
    return (value as bigint).toString().length <= MAX_MEMOIZED_PRIMITIVE_LENGTH;
  }
  return true;
}

function hasSameBinaryData(expected: Uint8Array, actual: Uint8Array): boolean {
  if (expected.byteLength !== actual.byteLength) return false;

  let difference = 0;
  for (let index = 0; index < expected.byteLength; index++) {
    difference |= expected[index]! ^ actual[index]!;
  }
  return difference === 0;
}

/**
 * Invocation-scoped cache for replay payload hydration.
 *
 * A workflow invocation may replay the same event log through several fresh
 * VMs. This cache keeps the VM-independent decrypt/decompress result across
 * those replays. Deserialization still runs against each VM's globals so every
 * replay receives fresh object graphs and correctly revived Workflow objects.
 *
 * Successful prepared plaintext remains resident for the invocation lifetime.
 * Its memory cost is the sum of decrypted and decompressed payload sizes, but
 * it never crosses workflow runs or queue deliveries.
 */
export class ReplayPayloadCache {
  private readonly preparedPayloads = new Map<string, PreparationEntry>();
  private readonly primitiveStepResults = new Map<string, unknown>();
  private nextUnscannedEventIndex = 0;

  constructor(
    private readonly encryptionKey: PayloadKey | undefined,
    private readonly preparer: ReplayPayloadPreparer = prepareReplayPayload
  ) {}

  /**
   * Start every missing binary preparation before workflow execution. Failures
   * are intentionally retained: the ordered event consumer must observe the
   * original rejection before that entry becomes retryable.
   */
  async prewarm(workflowRun: WorkflowRun, events: Event[]): Promise<void> {
    const preparations: Promise<PreparedReplayPayload>[] = [];
    const scheduled = new Set<Promise<PreparedReplayPayload>>();
    const start = (cacheKey: string, value: unknown): void => {
      // Legacy flattened values may be mutated by devalue's unflatten and are
      // therefore prepared only by their eventual consumer, never cached.
      if (!(value instanceof Uint8Array)) return;

      // Each replay scans the full event log, so awaiting cached promises here
      // would add O(N^2) promise reactions over an N-step invocation. Only wait
      // for preparations first discovered by this prewarm pass, plus a newly
      // discovered integrity conflict for a key that was already cached.
      const ensured = this.ensurePreparation(cacheKey, value);
      if (
        ensured.scheduleInPrewarm &&
        !scheduled.has(ensured.preparation)
      ) {
        scheduled.add(ensured.preparation);
        preparations.push(ensured.preparation);
      }
    };

    start(this.workflowInputKey(workflowRun.runId), workflowRun.input);
    // This cache is scoped to one invocation. Incremental loads and write
    // response deltas only ever append, so the scanned length locates the
    // events added since the previous replay. A reload that can insert events
    // BELOW that length (a stale-snapshot restart replacing the log with a
    // corrected one) must call `resetScan()` first, or the inserted events are
    // never scanned. Prepared entries stay valid across that: they are keyed by
    // event id, not by position.
    for (
      let index = this.nextUnscannedEventIndex;
      index < events.length;
      index++
    ) {
      const event = events[index]!;
      switch (event.eventType) {
        case 'step_completed':
          start(
            this.eventPayloadKey(event.eventId, 'result'),
            event.eventData?.result
          );
          break;
        case 'step_failed':
          start(
            this.eventPayloadKey(event.eventId, 'error'),
            event.eventData?.error
          );
          break;
        case 'hook_received':
          start(
            this.eventPayloadKey(event.eventId, 'payload'),
            event.eventData?.payload
          );
          break;
      }
    }
    this.nextUnscannedEventIndex = events.length;

    // Prewarming is speculative and must not fail replay before the matching
    // event is consumed. allSettled also attaches rejection handlers eagerly.
    await Promise.allSettled(preparations);
  }

  /**
   * Forget how much of the event log has been scanned, so the next
   * {@link prewarm} walks it from the start again.
   *
   * Required before a replay whose event log was reloaded rather than extended:
   * a corrected log inserts the events the previous load was missing, which
   * shifts every later position, so a positional resume would skip exactly the
   * events the reload was for. Already-prepared payloads are kept: they are
   * keyed by event id, so re-scanning re-observes them for free.
   */
  resetScan(): void {
    this.nextUnscannedEventIndex = 0;
  }

  /** Return the workflow input after shared host-side preparation. */
  prepareWorkflowInput(
    workflowRun: WorkflowRun
  ): Promise<PreparedReplayPayload> {
    return this.consumePreparation(
      this.workflowInputKey(workflowRun.runId),
      workflowRun.input
    );
  }

  /**
   * Return an event payload after shared host-side preparation. A rejected
   * preparation is evicted only after this ordered consumer requests it, so a
   * later replay can retry without hiding the original failure.
   */
  prepareEventPayload(
    eventId: string,
    field: ReplayPayloadField,
    value: unknown
  ): Promise<PreparedReplayPayload> {
    return this.consumePreparation(this.eventPayloadKey(eventId, field), value);
  }

  /**
   * Reuse final step values only when sharing them across VMs is unobservable.
   * Objects and large strings/bigints always run `hydrate` again, producing a
   * fresh VM-specific value from the separately cached prepared payload.
   */
  async getStepResult(
    eventId: string,
    hydrate: () => Promise<unknown>
  ): Promise<unknown> {
    if (this.primitiveStepResults.has(eventId)) {
      return this.primitiveStepResults.get(eventId);
    }

    const value = await hydrate();
    if (isMemoizablePrimitive(value)) {
      this.primitiveStepResults.set(eventId, value);
    }
    return value;
  }

  /**
   * Consumer-facing lookup. Binary payloads share preparation; legacy values
   * bypass the cache because their flattened representation may be mutated.
   */
  private consumePreparation(
    cacheKey: string,
    value: unknown
  ): Promise<PreparedReplayPayload> {
    if (!(value instanceof Uint8Array)) return this.runPreparation(value);

    const ensured = this.ensurePreparation(cacheKey, value);
    void ensured.preparation.catch(() => {
      // A contradictory key is terminal for the invocation. Never turn it back
      // into a retryable cache miss, even if the original preparation fails or
      // completes after the conflict was discovered.
      if (ensured.entry.conflict) return;
      if (ensured.preparation !== ensured.entry.preparation) return;
      if (this.preparedPayloads.get(cacheKey) === ensured.entry) {
        this.preparedPayloads.delete(cacheKey);
      }
    });
    return ensured.preparation;
  }

  /**
   * Start binary preparation once and bind the logical key to an immutable
   * snapshot of those bytes. Identical bytes share the exact promise; any later
   * conflict becomes a terminal, redacted integrity error.
   */
  private ensurePreparation(
    cacheKey: string,
    value: Uint8Array
  ): EnsuredPreparation {
    const cached = this.preparedPayloads.get(cacheKey);
    if (cached) {
      if (!hasSameBinaryData(cached.originalBytes, value)) {
        const conflict = this.markPayloadConflict(cached);
        return {
          entry: cached,
          preparation: conflict.preparation,
          scheduleInPrewarm: conflict.created,
        };
      }
      if (cached.invokingPreparer) {
        return {
          entry: cached,
          preparation: Promise.reject(new ReplayPayloadReentrantError()),
          scheduleInPrewarm: true,
        };
      }
      return {
        entry: cached,
        preparation: cached.conflictPreparation ?? cached.preparation,
        scheduleInPrewarm: false,
      };
    }

    const { entry, preparationInput } = this.createPreparation(value);
    // Publish the identity binding before invoking custom preparation code. A
    // synchronous re-entry therefore observes this cell instead of creating a
    // second preparation for the same logical key.
    this.preparedPayloads.set(cacheKey, entry);
    this.startPreparation(entry, preparationInput);
    return {
      entry,
      preparation: entry.preparation,
      scheduleInPrewarm: true,
    };
  }

  private createPreparation(value: Uint8Array): {
    readonly entry: PreparationEntry;
    readonly preparationInput: Uint8Array;
  } {
    // The cache owns the identity bytes. A caller cannot mutate the original
    // Uint8Array after insertion and silently change what gets prepared. The
    // preparer receives a second copy so it cannot rewrite that identity.
    const originalBytes = new Uint8Array(value);
    const preparationInput = new Uint8Array(originalBytes);
    let resolvePreparation!: (value: PreparedReplayPayload) => void;
    let rejectPreparation!: (reason: unknown) => void;
    const preparation = new Promise<PreparedReplayPayload>((resolve, reject) => {
      resolvePreparation = resolve;
      rejectPreparation = reject;
    });
    const entry: PreparationEntry = {
      originalBytes,
      preparation,
      resolvePreparation,
      rejectPreparation,
      invokingPreparer: false,
      settled: false,
    };

    return { entry, preparationInput };
  }

  private startPreparation(
    entry: PreparationEntry,
    preparationInput: Uint8Array
  ): void {
    entry.invokingPreparer = true;
    const prepared = this.runPreparation(preparationInput);
    entry.invokingPreparer = false;

    void prepared.then(
      (value) => {
        if (entry.settled) return;
        entry.settled = true;
        entry.resolvePreparation(value);
      },
      (error: unknown) => {
        if (entry.settled) return;
        entry.settled = true;
        entry.rejectPreparation(error);
      }
    );
  }

  private markPayloadConflict(entry: PreparationEntry): {
    readonly preparation: Promise<PreparedReplayPayload>;
    readonly created: boolean;
  } {
    if (entry.conflict && entry.conflictPreparation) {
      return { preparation: entry.conflictPreparation, created: false };
    }

    const conflict = new ReplayPayloadConflictError();
    entry.conflict = conflict;
    if (!entry.settled) {
      entry.settled = true;
      entry.rejectPreparation(conflict);
      entry.conflictPreparation = entry.preparation;
    } else {
      entry.conflictPreparation = Promise.reject(conflict);
    }

    // Prewarm and ordered consumers attach their own handlers. This guard also
    // keeps a conflict discovered by an internal caller from becoming an
    // unhandled-rejection side channel.
    void entry.conflictPreparation.catch(() => {});
    return { preparation: entry.conflictPreparation, created: true };
  }

  /** Normalize synchronous and asynchronous preparers to one promise contract. */
  private async runPreparation(value: unknown): Promise<PreparedReplayPayload> {
    return this.preparer(value, this.encryptionKey);
  }

  private workflowInputKey(runId: string): string {
    return `run:${runId}:input`;
  }

  private eventPayloadKey(eventId: string, field: ReplayPayloadField): string {
    return `event:${eventId}:${field}`;
  }
}
