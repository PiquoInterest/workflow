import { describe, expect, it } from 'vitest';
import {
  BULK_CANCEL_MAX_RUN_IDS,
  BulkCancelWorkflowRunsResultSchema,
} from './runs.js';

function validResult() {
  return {
    summary: {
      requested: 3,
      cancelled: 1,
      alreadyCancelled: 0,
      notCancellable: 1,
      notFound: 1,
      failed: 0,
    },
    results: [
      { runId: 'wrun_cancelled', outcome: 'cancelled' as const },
      {
        runId: 'wrun_terminal',
        outcome: 'not_cancellable' as const,
        status: 'completed',
      },
      { runId: 'wrun_missing', outcome: 'not_found' as const },
    ],
  };
}

describe('BulkCancelWorkflowRunsResultSchema response integrity', () => {
  it('accepts an exact projection of the per-run outcomes', () => {
    const value = validResult();
    expect(BulkCancelWorkflowRunsResultSchema.parse(value)).toEqual(value);
  });

  it.each([
    { name: 'requested', override: { requested: 2 } },
    { name: 'cancelled', override: { cancelled: 2 } },
    { name: 'alreadyCancelled', override: { alreadyCancelled: 1 } },
    { name: 'notCancellable', override: { notCancellable: 0 } },
    { name: 'notFound', override: { notFound: 0 } },
    { name: 'failed', override: { failed: 1 } },
  ])('rejects a mismatched $name counter', ({ override }) => {
    const value = validResult();
    expect(
      BulkCancelWorkflowRunsResultSchema.safeParse({
        ...value,
        summary: { ...value.summary, ...override },
      }).success
    ).toBe(false);
  });

  it.each([-1, 1.5])('rejects the invalid requested count %s', (requested) => {
    const value = validResult();
    expect(
      BulkCancelWorkflowRunsResultSchema.safeParse({
        ...value,
        summary: { ...value.summary, requested },
      }).success
    ).toBe(false);
  });

  it('rejects duplicate run IDs', () => {
    const value = validResult();
    const duplicate = {
      ...value,
      summary: {
        requested: 2,
        cancelled: 1,
        alreadyCancelled: 0,
        notCancellable: 0,
        notFound: 1,
        failed: 0,
      },
      results: [
        { runId: 'sensitive-run-id', outcome: 'cancelled' as const },
        { runId: 'sensitive-run-id', outcome: 'not_found' as const },
      ],
    };
    expect(BulkCancelWorkflowRunsResultSchema.safeParse(duplicate).success).toBe(
      false
    );
  });

  it('rejects an empty aggregate response', () => {
    expect(
      BulkCancelWorkflowRunsResultSchema.safeParse({
        summary: {
          requested: 0,
          cancelled: 0,
          alreadyCancelled: 0,
          notCancellable: 0,
          notFound: 0,
          failed: 0,
        },
        results: [],
      }).success
    ).toBe(false);
  });

  it('rejects more results than one valid request can contain', () => {
    const results = Array.from(
      { length: BULK_CANCEL_MAX_RUN_IDS + 1 },
      (_, index) => ({
        runId: `wrun_${index}`,
        outcome: 'cancelled' as const,
      })
    );
    expect(
      BulkCancelWorkflowRunsResultSchema.safeParse({
        summary: {
          requested: results.length,
          cancelled: results.length,
          alreadyCancelled: 0,
          notCancellable: 0,
          notFound: 0,
          failed: 0,
        },
        results,
      }).success
    ).toBe(false);
  });
});
