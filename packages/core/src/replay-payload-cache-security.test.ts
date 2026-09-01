import type { WorkflowRun } from '@workflow/world';
import { describe, expect, it, vi } from 'vitest';
import {
  ReplayPayloadCache,
  type ReplayPayloadPreparer,
} from './replay-payload-cache.js';

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

describe('ReplayPayloadCache conflicting-key characterization', () => {
  it('aliases conflicting bytes to the first cached workflow input', async () => {
    const firstPayload = new Uint8Array([1]);
    const conflictingPayload = new Uint8Array([2]);
    const preparer = vi.fn<ReplayPayloadPreparer>((value) => ({ data: value }));
    const cache = new ReplayPayloadCache(undefined, preparer);

    const first = cache.prepareWorkflowInput(
      makeRun('wrun_conflicting_payload', firstPayload)
    );
    const conflicting = cache.prepareWorkflowInput(
      makeRun('wrun_conflicting_payload', conflictingPayload)
    );

    expect(conflicting).toBe(first);
    await expect(first).resolves.toEqual({ data: firstPayload });
    await expect(conflicting).resolves.toEqual({ data: firstPayload });
    expect(preparer).toHaveBeenCalledOnce();
    expect(preparer).not.toHaveBeenCalledWith(conflictingPayload, undefined);
  });

  it('aliases conflicting bytes to the first cached event payload', async () => {
    const firstPayload = new Uint8Array([1]);
    const conflictingPayload = new Uint8Array([2]);
    const preparer = vi.fn<ReplayPayloadPreparer>((value) => ({ data: value }));
    const cache = new ReplayPayloadCache(undefined, preparer);

    const first = cache.prepareEventPayload(
      'evnt_conflicting_payload',
      'result',
      firstPayload
    );
    const conflicting = cache.prepareEventPayload(
      'evnt_conflicting_payload',
      'result',
      conflictingPayload
    );

    expect(conflicting).toBe(first);
    await expect(first).resolves.toEqual({ data: firstPayload });
    await expect(conflicting).resolves.toEqual({ data: firstPayload });
    expect(preparer).toHaveBeenCalledOnce();
    expect(preparer).not.toHaveBeenCalledWith(conflictingPayload, undefined);
  });
});