import { describe, expect, it } from 'vitest';
import { WorkflowError } from './index.js';

describe('WorkflowError', () => {
  it('uses its public class name and passes its own type guard', () => {
    const error = new WorkflowError('boom');

    expect(error.name).toBe('WorkflowError');
    expect(WorkflowError.is(error)).toBe(true);
  });

  it('keeps documentation framing stable', () => {
    const error = new WorkflowError('event history is invalid', {
      slug: 'corrupted-event-log',
    });

    expect(error.message).toBe(
      'event history is invalid\n╰▶ docs: https://workflow-sdk.dev/err/corrupted-event-log'
    );
  });
});
