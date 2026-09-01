import { describe, expect, it, vi } from 'vitest';
import {
  ReplayPayloadCache,
  type ReplayPayloadPreparer,
} from './replay-payload-cache.js';

describe('ReplayPayloadCache conflicting-key characterization', () => {
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
