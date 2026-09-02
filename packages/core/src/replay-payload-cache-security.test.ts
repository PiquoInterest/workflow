import type { WorkflowRun } from '@workflow/world';
import { describe, expect, it, vi } from 'vitest';
import { ReplayPayloadCache } from './replay-payload-cache.js';
import type { ReplayPayloadPreparer } from './serialization.js';

const EXPECTED_CONFLICT_NAME = 'ReplayPayloadConflictError';
const EXPECTED_CONFLICT_CODE = 'REPLAY_PAYLOAD_CONFLICT';
const EXPECTED_CONFLICT_MESSAGE =
  'replay payload cache key was reused with different binary data';
const EXPECTED_REENTRANT_NAME = 'ReplayPayloadReentrantError';
const EXPECTED_REENTRANT_CODE = 'REPLAY_PAYLOAD_REENTRANT';
const EXPECTED_REENTRANT_MESSAGE =
  'replay payload preparation re-entered the same cache key';

class AliasingUint8Array extends Uint8Array {
  override slice(): Uint8Array {
    return this;
  }
}

function makeRun(runId: string, input: unknown): WorkflowRun {
  const now = new Date();
  return {
    runId,
    status: 'running',
    deploymentId: 'dpl_replay_cache_security',
    workflowName: 'workflow//test//cache-security',
    input,
    attributes: {},
    startedAt: now,
    createdAt: now,
    updatedAt: now,
  };
}

async function captureRejection(promise: Promise<unknown>): Promise<unknown> {
  return promise.then(
    () => {
      throw new Error('expected replay payload rejection');
    },
    (error: unknown) => error
  );
}

function expectRedactedError(
  error: unknown,
  expected: Readonly<{ name: string; code: string; message: string }>,
  forbiddenValues: string[]
): void {
  expect(error).toBeInstanceOf(Error);
  const candidate = error as Error & { code?: unknown };
  expect(candidate.name).toBe(expected.name);
  expect(candidate.code).toBe(expected.code);
  expect(candidate.message).toBe(expected.message);
  for (const value of forbiddenValues) {
    expect(candidate.message).not.toContain(value);
  }
}

function expectRedactedConflict(
  error: unknown,
  forbiddenValues: string[]
): void {
  expectRedactedError(
    error,
    {
      name: EXPECTED_CONFLICT_NAME,
      code: EXPECTED_CONFLICT_CODE,
      message: EXPECTED_CONFLICT_MESSAGE,
    },
    forbiddenValues
  );
}

function expectRedactedReentrant(
  error: unknown,
  forbiddenValues: string[]
): void {
  expectRedactedError(
    error,
    {
      name: EXPECTED_REENTRANT_NAME,
      code: EXPECTED_REENTRANT_CODE,
      message: EXPECTED_REENTRANT_MESSAGE,
    },
    forbiddenValues
  );
}

describe('ReplayPayloadCache conflicting-key integrity', () => {
  it('binds workflow input keys to immutable bytes and rejects later conflicts', async () => {
    const runId = 'wrun_conflicting_payload';
    const firstPayload = new AliasingUint8Array([1]);
    const conflictingPayload = new Uint8Array([2]);
    const preparer = vi.fn<ReplayPayloadPreparer>((value) => {
      const bytes = value as Uint8Array;
      const data = bytes.slice();
      bytes[0] = 8;
      return { data };
    });
    const cache = new ReplayPayloadCache(undefined, preparer);

    const first = cache.prepareWorkflowInput(makeRun(runId, firstPayload));
    firstPayload[0] = 9;

    await expect(first).resolves.toEqual({ data: new Uint8Array([1]) });
    expect(preparer).toHaveBeenCalledOnce();
    expect(preparer.mock.calls[0]?.[0]).not.toBe(firstPayload);

    const identical = cache.prepareWorkflowInput(
      makeRun(runId, new Uint8Array([1]))
    );
    expect(identical).toBe(first);
    await expect(identical).resolves.toEqual({ data: new Uint8Array([1]) });

    const conflict = await captureRejection(
      cache.prepareWorkflowInput(makeRun(runId, conflictingPayload))
    );
    expectRedactedConflict(conflict, [runId, '1', '2', '8', '9']);
    expect(preparer).toHaveBeenCalledOnce();
    expect(preparer).not.toHaveBeenCalledWith(conflictingPayload, undefined);

    const repeated = await captureRejection(
      cache.prepareWorkflowInput(makeRun(runId, new Uint8Array([1])))
    );
    expect(repeated).toBe(conflict);
    expect(preparer).toHaveBeenCalledOnce();
  });

  it('makes an in-flight event conflict terminal for every waiter', async () => {
    const eventId = 'evnt_in_flight_conflict';
    let releasePreparation!: () => void;
    const preparer = vi.fn<ReplayPayloadPreparer>(
      (value) =>
        new Promise((resolve) => {
          releasePreparation = () => resolve({ data: value });
        })
    );
    const cache = new ReplayPayloadCache(undefined, preparer);

    const first = cache.prepareEventPayload(
      eventId,
      'result',
      new Uint8Array([1])
    );
    const firstFailure = captureRejection(first);
    expect(preparer).toHaveBeenCalledOnce();

    const conflict = await captureRejection(
      cache.prepareEventPayload(eventId, 'result', new Uint8Array([2]))
    );
    expectRedactedConflict(conflict, [eventId, '1', '2']);
    expect(preparer).toHaveBeenCalledOnce();

    releasePreparation();
    expect(await firstFailure).toBe(conflict);

    const repeated = await captureRejection(
      cache.prepareEventPayload(eventId, 'result', new Uint8Array([1]))
    );
    expect(repeated).toBe(conflict);
    expect(preparer).toHaveBeenCalledOnce();
  });

  it('rejects synchronous same-key preparer re-entry without evicting success', async () => {
    const eventId = 'evnt_reentrant_preparation';
    let reentrantLookup!: Promise<unknown>;
    let cache!: ReplayPayloadCache;
    const preparer = vi.fn<ReplayPayloadPreparer>((value) => {
      reentrantLookup = cache.prepareEventPayload(eventId, 'result', value);
      return { data: value };
    });
    cache = new ReplayPayloadCache(undefined, preparer);

    await expect(
      cache.prepareEventPayload(eventId, 'result', new Uint8Array([3]))
    ).resolves.toEqual({ data: new Uint8Array([3]) });
    const reentrantError = await captureRejection(reentrantLookup);
    expectRedactedReentrant(reentrantError, [eventId, '3']);
    expect(preparer).toHaveBeenCalledOnce();

    await expect(
      cache.prepareEventPayload(eventId, 'result', new Uint8Array([3]))
    ).resolves.toEqual({ data: new Uint8Array([3]) });
    expect(preparer).toHaveBeenCalledOnce();
  });

  it('detects conflicting bytes during synchronous preparer re-entry', async () => {
    const eventId = 'evnt_reentrant_conflict';
    let reentrantLookup!: Promise<unknown>;
    let cache!: ReplayPayloadCache;
    let shouldReenter = true;
    const preparer = vi.fn<ReplayPayloadPreparer>((value) => {
      if (shouldReenter) {
        shouldReenter = false;
        reentrantLookup = cache.prepareEventPayload(
          eventId,
          'result',
          new Uint8Array([5])
        );
      }
      return { data: value };
    });
    cache = new ReplayPayloadCache(undefined, preparer);

    const outer = cache.prepareEventPayload(
      eventId,
      'result',
      new Uint8Array([4])
    );
    const conflict = await captureRejection(reentrantLookup);
    expectRedactedConflict(conflict, [eventId, '4', '5']);
    expect(await captureRejection(outer)).toBe(conflict);
    expect(preparer).toHaveBeenCalledOnce();

    const repeated = await captureRejection(
      cache.prepareEventPayload(eventId, 'result', new Uint8Array([4]))
    );
    expect(repeated).toBe(conflict);
    expect(preparer).toHaveBeenCalledOnce();
  });
});
