import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  CorruptedEventLogError,
  EntityConflictError,
  FatalError,
  HookConflictError,
  HookNotFoundError,
  MaxEventsExceededError,
  PreconditionFailedError,
  ReplayDivergenceError,
  RetryableError,
  RunExpiredError,
  RunNotSupportedError,
  RuntimeDecryptionError,
  SerializationError,
  StepNotRegisteredError,
  StreamExpiredError,
  ThrottleError,
  TooEarlyError,
  WorkflowBuildError,
  WorkflowDeploymentMismatchError,
  WorkflowError,
  WorkflowNotRegisteredError,
  WorkflowRunCancelledError,
  WorkflowRunFailedError,
  WorkflowRunNotCompletedError,
  WorkflowRunNotFoundError,
  WorkflowRuntimeError,
  WorkflowWorldError,
} from '../../packages/errors/src/index.js';
import {
  WORKFLOW_DESERIALIZE,
  WORKFLOW_SERIALIZE,
} from '../../packages/serde/src/index.js';
import { parseDurationToDate } from '../../packages/utils/src/time.js';

type RustSuccess = { ok: true; value: unknown };
type RustFailure = {
  ok: false;
  error: { code: string; message: string };
};
type RustOutcome = RustSuccess | RustFailure;

const binary =
  process.env.WORKFLOW_RUST_CONFORMANCE_BIN ??
  path.resolve('target/debug/examples/conformance');

function rust(op: string, input: unknown): RustOutcome {
  const output = execFileSync(binary, [], {
    encoding: 'utf8',
    input: JSON.stringify({ op, input }),
    maxBuffer: 1024 * 1024,
  });
  return JSON.parse(output) as RustOutcome;
}

function normalized(value: unknown): unknown {
  if (value === undefined) return null;
  return JSON.parse(JSON.stringify(value));
}

function descriptor(error: Error): unknown {
  const fields: Record<string, unknown> = {};
  for (const key of Object.keys(error).sort()) {
    if (['cause', 'fatal', 'message', 'name', 'stack'].includes(key)) continue;
    const value = (error as unknown as Record<string, unknown>)[key];
    if (value === undefined) continue;
    fields[key] = value instanceof Date ? value.getTime() : normalized(value);
  }
  return {
    name: error.name,
    message: error.message,
    fatal: FatalError.is(error),
    fields,
  };
}

function makeTypeScriptError(input: Record<string, unknown>): Error {
  const kind = String(input.kind);
  const message = String(input.message ?? 'boom');
  switch (kind) {
    case 'WorkflowError':
      return new WorkflowError(message, {
        slug: input.slug as 'corrupted-event-log' | undefined,
      });
    case 'WorkflowWorldError':
      return new WorkflowWorldError(message, {
        status: input.status as number | undefined,
        code: input.code as string | undefined,
        url: input.url as string | undefined,
        retryAfter: input.retryAfter as number | undefined,
      });
    case 'WorkflowRunFailedError':
      return new WorkflowRunFailedError(String(input.runId), input.error, {
        errorCode: input.errorCode as string | undefined,
      });
    case 'WorkflowRunNotCompletedError':
      return new WorkflowRunNotCompletedError(
        String(input.runId),
        String(input.status)
      );
    case 'WorkflowRuntimeError':
      return new WorkflowRuntimeError(message, {
        slug: input.slug as 'replay-divergence' | undefined,
      });
    case 'CorruptedEventLogError':
      return new CorruptedEventLogError(message);
    case 'ReplayDivergenceError':
      return new ReplayDivergenceError(message, {
        eventId: String(input.eventId),
      });
    case 'MaxEventsExceededError':
      return new MaxEventsExceededError(
        Number(input.eventCount),
        Number(input.limit)
      );
    case 'RuntimeDecryptionError':
      return new RuntimeDecryptionError(message, {
        context: input.context as {
          operation?: 'encrypt' | 'decrypt';
          byteLength?: number;
          formatPrefix?: string;
        },
      });
    case 'WorkflowBuildError':
      return new WorkflowBuildError(message, {
        hint: input.hint as string | undefined,
      });
    case 'SerializationError':
      return new SerializationError(message, {
        hint: input.hint as string | undefined,
      });
    case 'StepNotRegisteredError':
      return new StepNotRegisteredError(String(input.stepName));
    case 'WorkflowNotRegisteredError':
      return new WorkflowNotRegisteredError(String(input.workflowName));
    case 'WorkflowDeploymentMismatchError':
      return new WorkflowDeploymentMismatchError(
        String(input.runId),
        String(input.expectedDeploymentId),
        String(input.actualDeploymentId),
        { recoveryAttempts: Number(input.recoveryAttempts ?? 0) }
      );
    case 'WorkflowRunNotFoundError':
      return new WorkflowRunNotFoundError(String(input.runId));
    case 'HookConflictError':
      return new HookConflictError(
        String(input.token),
        input.conflictingRunId as string | undefined
      );
    case 'HookNotFoundError':
      return new HookNotFoundError(String(input.token));
    case 'EntityConflictError':
      return new EntityConflictError(message);
    case 'RunExpiredError':
      return new RunExpiredError(message);
    case 'StreamExpiredError':
      return new StreamExpiredError(
        message,
        input.runId as string | undefined,
        input.streamId as string | undefined,
        input.expiredAtMs === undefined
          ? undefined
          : new Date(Number(input.expiredAtMs))
      );
    case 'TooEarlyError':
      return new TooEarlyError(message, {
        retryAfter: input.retryAfter as number | undefined,
      });
    case 'ThrottleError':
      return new ThrottleError(message, {
        retryAfter: input.retryAfter as number | undefined,
      });
    case 'PreconditionFailedError':
      return new PreconditionFailedError(message, {
        retryAfter: input.retryAfter as number | undefined,
        details: input.details,
      });
    case 'WorkflowRunCancelledError':
      return new WorkflowRunCancelledError(String(input.runId));
    case 'RunNotSupportedError':
      return new RunNotSupportedError(
        Number(input.runSpecVersion),
        Number(input.worldSpecVersion)
      );
    case 'FatalError':
      return new FatalError(message);
    case 'RetryableError':
      return new RetryableError(message, {
        retryAfter:
          input.retryKind === 'string'
            ? (String(input.retryAfter) as `${number}s`)
            : input.retryKind === 'date'
              ? new Date(Number(input.retryAfter))
              : Number(input.retryAfter),
      });
    default:
      throw new Error(`Unknown TypeScript error fixture: ${kind}`);
  }
}

function expectErrorParity(input: Record<string, unknown>): void {
  const rustOutcome = rust('makeError', input);
  expect(rustOutcome.ok).toBe(true);
  if (rustOutcome.ok) {
    expect(rustOutcome.value).toEqual(descriptor(makeTypeScriptError(input)));
  }
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe('Rust serde symbol registry parity', () => {
  it('uses the same global symbol registry keys', () => {
    const outcome = rust('serdeSymbols', {});
    expect(outcome).toEqual({
      ok: true,
      value: {
        deserialize: Symbol.keyFor(WORKFLOW_DESERIALIZE),
        serialize: Symbol.keyFor(WORKFLOW_SERIALIZE),
      },
    });
  });
});

describe('Rust error constructor parity', () => {
  const fixtures: Record<string, unknown>[] = [
    { kind: 'WorkflowError', message: 'boom' },
    {
      kind: 'WorkflowError',
      message: 'event mismatch',
      slug: 'corrupted-event-log',
    },
    {
      kind: 'WorkflowWorldError',
      message: 'backend rejected request',
      status: 429,
      code: 'throttled',
      url: 'https://example.test/world',
      retryAfter: 2.5,
    },
    {
      kind: 'WorkflowRunFailedError',
      runId: 'run-1',
      error: 'user exploded',
      errorCode: 'USER_ERROR',
    },
    { kind: 'WorkflowRunNotCompletedError', runId: 'run-1', status: 'running' },
    { kind: 'WorkflowRuntimeError', message: 'runtime failed' },
    { kind: 'CorruptedEventLogError', message: 'event mismatch' },
    {
      kind: 'ReplayDivergenceError',
      message: 'consumer mismatch',
      eventId: 'event-1',
    },
    { kind: 'MaxEventsExceededError', eventCount: 101, limit: 100 },
    {
      kind: 'RuntimeDecryptionError',
      message: 'decrypt failed',
      context: { operation: 'decrypt', byteLength: 42, formatPrefix: 'encr' },
    },
    {
      kind: 'WorkflowBuildError',
      message: 'Build failed',
      hint: 'install the package\nand retry',
    },
    {
      kind: 'SerializationError',
      message: 'cannot encode class',
      hint: 'register a serializer',
    },
    { kind: 'StepNotRegisteredError', stepName: 'sendEmail' },
    { kind: 'WorkflowNotRegisteredError', workflowName: 'onboard' },
    {
      kind: 'WorkflowDeploymentMismatchError',
      runId: 'run-1',
      expectedDeploymentId: 'dpl-a',
      actualDeploymentId: 'dpl-b',
      recoveryAttempts: 2,
    },
    { kind: 'WorkflowRunNotFoundError', runId: 'run-missing' },
    { kind: 'HookConflictError', token: 'hook-1', conflictingRunId: 'run-2' },
    { kind: 'HookNotFoundError', token: 'hook-missing' },
    { kind: 'EntityConflictError', message: 'already terminal' },
    { kind: 'RunExpiredError', message: 'run expired' },
    {
      kind: 'StreamExpiredError',
      message: 'stream expired',
      runId: 'run-1',
      streamId: 'stream-1',
      expiredAtMs: 1_700_000_000_000,
    },
    { kind: 'TooEarlyError', message: 'not yet', retryAfter: 3 },
    { kind: 'ThrottleError', message: 'slow down', retryAfter: 4 },
    {
      kind: 'PreconditionFailedError',
      message: 'stale event log',
      retryAfter: 1,
      details: { missing: ['event-4'] },
    },
    { kind: 'WorkflowRunCancelledError', runId: 'run-1' },
    { kind: 'RunNotSupportedError', runSpecVersion: 8, worldSpecVersion: 7 },
    { kind: 'FatalError', message: 'do not retry' },
  ];

  for (const fixture of fixtures) {
    it(`matches ${String(fixture.kind)}`, () => {
      expectErrorParity(fixture);
    });
  }

  it('matches RetryableError with a fixed clock', () => {
    vi.spyOn(Date, 'now').mockReturnValue(1_700_000_000_000);
    expectErrorParity({
      kind: 'RetryableError',
      message: 'try again',
      retryKind: 'number',
      retryAfter: 250,
      nowMs: 1_700_000_000_000,
    });
  });
});

describe('Rust duration parsing parity and hardening', () => {
  const nowMs = 1_700_000_000_000;

  for (const fixture of [
    { kind: 'string', value: '5s' },
    { kind: 'number', value: 1500 },
    { kind: 'date', value: 1_700_000_005_000 },
  ]) {
    it(`matches ${fixture.kind} duration input`, () => {
      vi.spyOn(Date, 'now').mockReturnValue(nowMs);
      const outcome = rust('parseDurationToUnixMs', { ...fixture, nowMs });
      expect(outcome).toEqual({
        ok: true,
        value: parseDurationToDate(
          fixture.kind === 'string'
            ? (fixture.value as '5s')
            : fixture.kind === 'date'
              ? new Date(Number(fixture.value))
              : Number(fixture.value)
        ).getTime(),
      });
    });
  }

  for (const fixture of [
    { kind: 'string', value: 'invalid' },
    { kind: 'number', value: -1 },
    { kind: 'number', value: Number.MAX_VALUE },
    { kind: 'date', value: 8_640_000_000_000_001 },
  ]) {
    it(`rejects ${JSON.stringify(fixture)}`, () => {
      vi.spyOn(Date, 'now').mockReturnValue(nowMs);
      const rustOutcome = rust('parseDurationToUnixMs', { ...fixture, nowMs });
      let typescriptOk = true;
      try {
        parseDurationToDate(
          fixture.kind === 'string'
            ? (fixture.value as 'invalid')
            : fixture.kind === 'date'
              ? new Date(Number(fixture.value))
              : Number(fixture.value)
        );
      } catch {
        typescriptOk = false;
      }
      expect(rustOutcome.ok).toBe(typescriptOk);
      expect(rustOutcome.ok).toBe(false);
    });
  }
});
