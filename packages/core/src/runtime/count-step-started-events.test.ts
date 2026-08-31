import { type Event, SPEC_VERSION_CURRENT } from '@workflow/world';
import { describe, expect, it } from 'vitest';
import {
  countStepStartedEvents,
  nextStepAttempt,
} from './count-step-started-events.js';

describe('nextStepAttempt', () => {
  it.each([
    [0, 1],
    [1, 2],
    [Number.MAX_SAFE_INTEGER - 1, Number.MAX_SAFE_INTEGER],
  ])('advances %s to %s without precision loss', (prior, next) => {
    expect(nextStepAttempt(prior)).toBe(next);
  });

  it.each([
    -1,
    1.5,
    Number.MAX_SAFE_INTEGER,
    Number.MAX_SAFE_INTEGER + 1,
    Number.NaN,
    Number.POSITIVE_INFINITY,
  ])('rejects an unsafe prior count: %s', (prior) => {
    expect(() => nextStepAttempt(prior)).toThrow(
      new RangeError(
        'prior step attempt count must be a non-negative safe integer below Number.MAX_SAFE_INTEGER'
      )
    );
  });
});

describe('countStepStartedEvents', () => {
  const stepId = 'step_TARGET';
  let seq = 0;
  const start = (ownerMessageId?: string, correlationId = stepId): Event =>
    ({
      eventType: 'step_started',
      runId: 'wrun_count_test',
      eventId: `evnt_${String(seq++).padStart(4, '0')}`,
      createdAt: new Date(),
      specVersion: SPEC_VERSION_CURRENT,
      correlationId,
      eventData: {
        stepName: 'step//file//fn',
        ...(ownerMessageId !== undefined ? { ownerMessageId } : {}),
      },
    }) as Event;

  it('returns 0 for null/undefined/empty logs', () => {
    expect(countStepStartedEvents(null, stepId)).toBe(0);
    expect(countStepStartedEvents(undefined, stepId)).toBe(0);
    expect(countStepStartedEvents([], stepId)).toBe(0);
  });

  it('unscoped: counts every step_started for the step, ignoring other steps and event types', () => {
    const events: Event[] = [
      start('msg_A'),
      start(undefined),
      start('msg_B', 'step_OTHER'),
      {
        eventType: 'step_completed',
        runId: 'wrun_count_test',
        eventId: 'evnt_done',
        createdAt: new Date(),
        specVersion: SPEC_VERSION_CURRENT,
        correlationId: stepId,
        eventData: { result: undefined },
      } as unknown as Event,
    ];
    expect(countStepStartedEvents(events, stepId)).toBe(2);
  });

  it('ownedBy: counts only starts stamped with the given messageId', () => {
    const events: Event[] = [
      start('msg_OWNER'),
      start('msg_RACER_1'),
      start('msg_RACER_2'),
      start(undefined),
      start('msg_OWNER'),
    ];
    expect(
      countStepStartedEvents(events, stepId, {
        type: 'ownedBy',
        messageId: 'msg_OWNER',
      })
    ).toBe(2);
  });

  it('totalAttempts: bare starts plus the largest single owner, so racer one-off stamps do not accumulate', () => {
    const events: Event[] = [
      start('msg_OWNER'),
      start('msg_OWNER'),
      start('msg_RACER_1'),
      start('msg_RACER_2'),
      start(undefined),
    ];
    expect(
      countStepStartedEvents(events, stepId, { type: 'totalAttempts' })
    ).toBe(3);
  });

  it('regression (workflow#3069): racing invocations must not exhaust the owned-recovery retry ceiling', () => {
    const events: Event[] = [
      start('msg_OWNER'),
      start('msg_RACER_1'),
      start('msg_RACER_2'),
      start(undefined),
    ];
    const maxRetries = 3;

    const unscopedAttempt = nextStepAttempt(
      countStepStartedEvents(events, stepId)
    );
    expect(unscopedAttempt).toBeGreaterThan(maxRetries + 1);

    const ownedAttempt = nextStepAttempt(
      countStepStartedEvents(events, stepId, {
        type: 'ownedBy',
        messageId: 'msg_OWNER',
      })
    );
    expect(ownedAttempt).toBe(2);
    expect(ownedAttempt).toBeLessThanOrEqual(maxRetries + 1);

    const totalAttempt = nextStepAttempt(
      countStepStartedEvents(events, stepId, { type: 'totalAttempts' })
    );
    expect(totalAttempt).toBe(3);
    expect(totalAttempt).toBeLessThanOrEqual(maxRetries + 1);
  });

  it('still bounds real timeout retries: each recovery re-run by the owner counts toward the ceiling', () => {
    const events: Event[] = [
      start('msg_OWNER'),
      start('msg_OWNER'),
      start('msg_OWNER'),
      start('msg_OWNER'),
    ];
    const maxRetries = 3;
    const attempt = nextStepAttempt(
      countStepStartedEvents(events, stepId, {
        type: 'ownedBy',
        messageId: 'msg_OWNER',
      })
    );
    expect(attempt).toBeGreaterThan(maxRetries + 1);
  });

  it('mixed owned→bare timeout sequence trips the combined background ceiling', () => {
    const events: Event[] = [
      start('msg_OWNER'),
      start('msg_OWNER'),
      start('msg_OWNER'),
      start(undefined),
      start(undefined),
    ];
    const maxRetries = 3;
    const attempt = nextStepAttempt(
      countStepStartedEvents(events, stepId, { type: 'totalAttempts' })
    );
    expect(attempt).toBe(6);
    expect(attempt).toBeGreaterThan(maxRetries + 1);
  });
});
