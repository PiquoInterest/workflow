import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { StepSchema } from '../../packages/world/src/steps.js';

type RustSuccess = { ok: true; value: unknown };
type RustFailure = {
  ok: false;
  error: { code: string; message: string };
};
type RustOutcome = RustSuccess | RustFailure;

const binary =
  process.env.WORKFLOW_RUST_STEP_CONFORMANCE_BIN ??
  path.resolve('target/debug/examples/steps_conformance');

const baseStep = {
  runId: 'wrun_1',
  stepId: 'step_1',
  stepName: 'step//./src/workflows/order//processPayment',
  attempt: 1,
  specVersion: 7,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:01.000Z',
};

function rust(value: unknown): RustOutcome {
  const output = execFileSync(binary, [], {
    encoding: 'utf8',
    input: JSON.stringify({ value }),
    maxBuffer: 1024 * 1024,
  });
  return JSON.parse(output) as RustOutcome;
}

function normalized(value: unknown): unknown {
  return JSON.parse(JSON.stringify(value));
}

const contradictoryStates = [
  {
    name: 'pending step carrying terminal output and completion time',
    value: {
      ...baseStep,
      status: 'pending',
      output: { secret: 'pending-output' },
      completedAt: '2026-01-01T00:00:02.000Z',
    },
  },
  {
    name: 'running step carrying retry and completion timestamps',
    value: {
      ...baseStep,
      status: 'running',
      retryAfter: '2026-01-01T00:00:03.000Z',
      completedAt: '2026-01-01T00:00:04.000Z',
    },
  },
  {
    name: 'modern completed step without output',
    value: {
      ...baseStep,
      status: 'completed',
      completedAt: '2026-01-01T00:00:05.000Z',
    },
  },
  {
    name: 'modern failed step without error but with output',
    value: {
      ...baseStep,
      status: 'failed',
      output: { secret: 'failed-output' },
      completedAt: '2026-01-01T00:00:06.000Z',
    },
  },
  {
    name: 'cancelled step without completion time but with output',
    value: {
      ...baseStep,
      status: 'cancelled',
      output: { secret: 'cancelled-output' },
    },
  },
  {
    name: 'running step smuggling a forbidden retry field through null',
    value: {
      ...baseStep,
      status: 'running',
      retryAfter: null,
    },
  },
] as const;

const unsafeAttempts = [
  { name: 'negative', attempt: -1 },
  { name: 'fractional', attempt: 1.5 },
  {
    name: 'above Number.MAX_SAFE_INTEGER',
    attempt: Number.MAX_SAFE_INTEGER + 1,
  },
] as const;

const safeAttempts = [
  { name: 'zero', attempt: 0 },
  { name: 'one', attempt: 1 },
  { name: 'Number.MAX_SAFE_INTEGER', attempt: Number.MAX_SAFE_INTEGER },
] as const;

const validStates = [
  {
    ...baseStep,
    status: 'pending',
    error: { kind: 'retry' },
    retryAfter: '2026-01-01T00:00:10.000Z',
  },
  {
    ...baseStep,
    status: 'running',
    error: { kind: 'earlier-retry' },
  },
  {
    ...baseStep,
    status: 'completed',
    output: { kind: 'result' },
    error: { kind: 'earlier-retry' },
    completedAt: '2026-01-01T00:00:11.000Z',
  },
  {
    ...baseStep,
    status: 'failed',
    error: { kind: 'terminal' },
    completedAt: '2026-01-01T00:00:12.000Z',
  },
  {
    ...baseStep,
    status: 'cancelled',
    error: { kind: 'earlier-retry' },
    completedAt: '2026-01-01T00:00:13.000Z',
  },
  {
    ...baseStep,
    status: 'completed',
    specVersion: 1,
    completedAt: '2026-01-01T00:00:14.000Z',
  },
  {
    ...baseStep,
    status: 'failed',
    specVersion: 1,
    completedAt: '2026-01-01T00:00:15.000Z',
  },
] as const;

describe('WF-RUST-005 TypeScript step-state security proof', () => {
  it.each(contradictoryStates)(
    'proves TypeScript accepts while Rust rejects $name',
    ({ value }) => {
      expect(StepSchema.safeParse(value).success).toBe(true);

      const outcome = rust(value);
      expect(outcome.ok).toBe(false);
      if (outcome.ok) {
        throw new Error('Rust unexpectedly accepted a contradictory step');
      }
      expect(outcome.error.code).toBe('invalid_step_state');
      for (const secret of [
        'pending-output',
        'failed-output',
        'cancelled-output',
      ]) {
        expect(outcome.error.message).not.toContain(secret);
      }
    }
  );

  it.each(validStates)(
    'preserves the valid TypeScript projection for $status at spec $specVersion',
    (value) => {
      expect(rust(value)).toEqual({
        ok: true,
        value: normalized(StepSchema.parse(value)),
      });
    }
  );
});

describe('WF-RUST-012 step-attempt security proof', () => {
  it.each(unsafeAttempts)(
    'proves TypeScript accepts while Rust rejects a $name attempt',
    ({ attempt }) => {
      const value = { ...baseStep, status: 'running', attempt };
      expect(StepSchema.safeParse(value).success).toBe(true);

      const outcome = rust(value);
      expect(outcome.ok).toBe(false);
      if (outcome.ok) {
        throw new Error('Rust unexpectedly accepted an unsafe attempt');
      }
      expect(outcome.error).toEqual({
        code: 'invalid_step_state',
        message: 'attempt must be a non-negative safe integer',
      });
    }
  );

  it.each(safeAttempts)(
    'preserves and normalizes the $name attempt',
    ({ attempt }) => {
      const value = { ...baseStep, status: 'running', attempt };
      expect(rust(value)).toEqual({
        ok: true,
        value: normalized(StepSchema.parse(value)),
      });
    }
  );
});
