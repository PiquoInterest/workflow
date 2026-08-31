import { describe, expect, it } from 'vitest';
import { StepSchema } from './steps.js';

const baseStep = {
  runId: 'wrun_1',
  stepId: 'step_1',
  stepName: 'step//./src/workflows/order//processPayment',
  attempt: 1,
  specVersion: 7,
  createdAt: new Date('2026-01-01T00:00:00.000Z'),
  updatedAt: new Date('2026-01-01T00:00:01.000Z'),
};

/**
 * These are intentionally unsafe states accepted by the current TypeScript
 * schema. This characterization test is the proof for WF-RUST-005, not an
 * endorsement of the behavior. The Rust boundary must reject every fixture.
 */
export const contradictoryStepStates = [
  {
    name: 'pending step carrying terminal output and completion time',
    value: {
      ...baseStep,
      status: 'pending',
      output: { secret: 'pending-output' },
      completedAt: new Date('2026-01-01T00:00:02.000Z'),
    },
  },
  {
    name: 'running step carrying retry and completion timestamps',
    value: {
      ...baseStep,
      status: 'running',
      retryAfter: new Date('2026-01-01T00:00:03.000Z'),
      completedAt: new Date('2026-01-01T00:00:04.000Z'),
    },
  },
  {
    name: 'modern completed step without an output field',
    value: {
      ...baseStep,
      status: 'completed',
      completedAt: new Date('2026-01-01T00:00:05.000Z'),
    },
  },
  {
    name: 'modern failed step without an error but with output',
    value: {
      ...baseStep,
      status: 'failed',
      output: { secret: 'failed-output' },
      completedAt: new Date('2026-01-01T00:00:06.000Z'),
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
] as const;

/**
 * `z.number()` rejects non-finite values but accepts these finite values even
 * though a durable retry counter cannot safely use them. Rust intentionally
 * narrows this boundary under WF-RUST-012.
 */
export const unsafeStepAttempts = [
  { name: 'a negative attempt', attempt: -1 },
  { name: 'a fractional attempt', attempt: 1.5 },
  {
    name: 'an attempt above Number.MAX_SAFE_INTEGER',
    attempt: Number.MAX_SAFE_INTEGER + 1,
  },
] as const;

const nonFiniteStepAttempts = [
  { name: 'NaN', attempt: Number.NaN },
  { name: 'positive infinity', attempt: Number.POSITIVE_INFINITY },
  { name: 'negative infinity', attempt: Number.NEGATIVE_INFINITY },
] as const;

describe('StepSchema security characterization', () => {
  it.each(contradictoryStepStates)(
    'currently accepts $name',
    ({ value }) => {
      expect(StepSchema.safeParse(value).success).toBe(true);
    }
  );
});

describe('StepSchema attempt-counter characterization', () => {
  it.each(unsafeStepAttempts)('currently accepts $name', ({ attempt }) => {
    expect(
      StepSchema.safeParse({ ...baseStep, status: 'running', attempt }).success
    ).toBe(true);
  });

  it.each(nonFiniteStepAttempts)('already rejects $name', ({ attempt }) => {
    expect(
      StepSchema.safeParse({ ...baseStep, status: 'running', attempt }).success
    ).toBe(false);
  });
});
