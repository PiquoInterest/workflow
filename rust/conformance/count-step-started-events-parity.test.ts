import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  countStepStartedEvents,
  nextStepAttempt,
  type StepStartScope,
} from '../../packages/core/src/runtime/count-step-started-events.js';

interface Success {
  ok: true;
  value: number;
}

interface Failure {
  ok: false;
  error: string;
}

type Outcome = Success | Failure;

interface EventFixture {
  kind: 'started' | 'completed';
  stepId: string;
  owner?: string;
}

const binary =
  process.env.WORKFLOW_RUST_STEP_ATTEMPT_CONFORMANCE_BIN ??
  path.resolve(
    'crates/workflow-core/target/debug/examples/count_step_started_events_conformance'
  );

function runRust(arguments_: string[]): Outcome {
  return JSON.parse(
    execFileSync(binary, arguments_, {
      encoding: 'utf8',
      timeout: 5_000,
      maxBuffer: 1024 * 1024,
    })
  ) as Outcome;
}

function normalize(call: () => number): Outcome {
  try {
    return { ok: true, value: call() };
  } catch (error) {
    return { ok: false, error: (error as Error).message };
  }
}

function numberArgument(value: number): string {
  if (Number.isNaN(value)) return 'NaN';
  if (value === Number.POSITIVE_INFINITY) return 'Infinity';
  if (value === Number.NEGATIVE_INFINITY) return '-Infinity';
  if (Object.is(value, -0)) return '-0';
  return String(value);
}

function tsEvents(events: EventFixture[]) {
  return events.map((event, index) => ({
    eventType: event.kind === 'started' ? 'step_started' : 'step_completed',
    runId: 'wrun_conformance',
    eventId: `evnt_${index}`,
    correlationId: event.stepId,
    createdAt: new Date(0),
    eventData:
      event.kind === 'started'
        ? {
            stepName: 'step//fixture//work',
            ...(event.owner === undefined
              ? {}
              : { ownerMessageId: event.owner }),
          }
        : { result: undefined },
  })) as Parameters<typeof countStepStartedEvents>[0];
}

function encodeEvent(event: EventFixture): string {
  return `${event.kind}|${event.stepId}|${event.owner ?? ''}`;
}

function scopeArgument(scope: StepStartScope | undefined): string {
  if (scope === undefined) return 'unscoped';
  if (scope.type === 'totalAttempts') return 'total';
  return `owned|${scope.messageId}`;
}

const numberCases = [
  0,
  -0,
  1,
  Number.MAX_SAFE_INTEGER - 1,
  -1,
  1.5,
  Number.MAX_SAFE_INTEGER,
  Number.MAX_SAFE_INTEGER + 1,
  Number.NaN,
  Number.POSITIVE_INFINITY,
  Number.NEGATIVE_INFINITY,
];

describe('Rust step-attempt transition parity', () => {
  for (const value of numberCases) {
    it(`matches TypeScript for ${String(value)}`, () => {
      expect(runRust(['next', numberArgument(value)])).toEqual(
        normalize(() => nextStepAttempt(value))
      );
    });
  }
});

const STEP_ID = 'step_TARGET';
const countCases: Array<{
  name: string;
  events: EventFixture[];
  scope?: StepStartScope;
}> = [
  { name: 'empty', events: [] },
  {
    name: 'filters other steps and terminal events',
    events: [
      { kind: 'started', stepId: STEP_ID, owner: 'msg_A' },
      { kind: 'started', stepId: STEP_ID },
      { kind: 'started', stepId: 'step_OTHER', owner: 'msg_B' },
      { kind: 'completed', stepId: STEP_ID },
    ],
  },
  {
    name: 'selected owner only',
    events: [
      { kind: 'started', stepId: STEP_ID, owner: 'msg_OWNER' },
      { kind: 'started', stepId: STEP_ID, owner: 'msg_RACER_1' },
      { kind: 'started', stepId: STEP_ID },
      { kind: 'started', stepId: STEP_ID, owner: 'msg_OWNER' },
    ],
    scope: { type: 'ownedBy', messageId: 'msg_OWNER' },
  },
  {
    name: 'bare plus largest owner',
    events: [
      { kind: 'started', stepId: STEP_ID, owner: 'msg_OWNER' },
      { kind: 'started', stepId: STEP_ID, owner: 'msg_OWNER' },
      { kind: 'started', stepId: STEP_ID, owner: 'msg_RACER_1' },
      { kind: 'started', stepId: STEP_ID, owner: 'msg_RACER_2' },
      { kind: 'started', stepId: STEP_ID },
    ],
    scope: { type: 'totalAttempts' },
  },
  {
    name: 'mixed owner and bare timeout attempts',
    events: [
      { kind: 'started', stepId: STEP_ID, owner: 'msg_OWNER' },
      { kind: 'started', stepId: STEP_ID, owner: 'msg_OWNER' },
      { kind: 'started', stepId: STEP_ID, owner: 'msg_OWNER' },
      { kind: 'started', stepId: STEP_ID },
      { kind: 'started', stepId: STEP_ID },
    ],
    scope: { type: 'totalAttempts' },
  },
];

describe('Rust step-start counting parity', () => {
  for (const testCase of countCases) {
    it(testCase.name, () => {
      const rust = runRust([
        'count',
        STEP_ID,
        scopeArgument(testCase.scope),
        ...testCase.events.map(encodeEvent),
      ]);
      const typescript: Outcome = {
        ok: true,
        value: countStepStartedEvents(
          tsEvents(testCase.events),
          STEP_ID,
          testCase.scope
        ),
      };
      expect(rust).toEqual(typescript);
    });
  }
});
