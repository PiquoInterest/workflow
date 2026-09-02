import { afterEach, describe, expect, it, vi } from 'vitest';
import { debugLog, isWorkflowDebugEnabled } from './debug-log.js';

describe('legacy debug selector security characterization', () => {
  afterEach(() => {
    vi.unstubAllEnvs();
    vi.restoreAllMocks();
  });

  it('accepts unrelated and negative selectors that merely contain workflow:', () => {
    // This intentionally records the current unsafe TypeScript behavior. The
    // Rust implementation must parse selector boundaries and reject both
    // values instead of using substring matching.
    for (const selector of ['myworkflow:*', 'app:*,-workflow:*']) {
      vi.stubEnv('DEBUG', selector);
      expect(isWorkflowDebugEnabled(), selector).toBe(true);
    }
  });

  it('emits diagnostic arguments even when workflow logging is explicitly negated', () => {
    vi.stubEnv('DEBUG', 'app:*,-workflow:*');
    const debugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {});
    const sensitiveDetail = { runId: 'wrun_private' };

    debugLog('diagnostic breadcrumb', sensitiveDetail);

    expect(debugSpy).toHaveBeenCalledWith(
      'diagnostic breadcrumb',
      sensitiveDetail
    );
  });
});
