import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { afterEach, describe, expect, it } from 'vitest';
import {
  applyAttributeChanges,
  validateAttributeChanges,
} from '../../packages/world/src/attributes-validation.js';
import {
  _resetEnvWarnCacheForTests,
  envFlag,
  envNumber,
} from '../../packages/world/src/env-config.js';
import {
  classifyEntityEvent,
  entityEventClass,
  getEventDataRefFields,
  isSealedNoopEvent,
} from '../../packages/world/src/event-metadata.js';
import {
  isChildEntityCreationEvent,
  isChildEntityCreationEventType,
  isHookEventRequiringExistence,
  isHookLifecycleEventType,
  isRunEventType,
  isStepEventType,
  isTerminalRunEventType,
  isTerminalStepEventType,
  isWaitEventType,
  stripEventDataRefs,
} from '../../packages/world/src/events.js';
import {
  getQueueTopicPrefix,
  parseQueueName,
  QueuePayloadSchema,
  QueuePrefix,
  ValidQueueName,
} from '../../packages/world/src/queue.js';
import { BulkCancelWorkflowRunsRequestSchema } from '../../packages/world/src/runs.js';
import {
  eventIdToSlot,
  isSlotBody,
  isSlotEventId,
  slotToEventId,
} from '../../packages/world/src/slot-identity.js';
import {
  isLegacySpecVersion,
  mintedSpecVersion,
  requiresNewerWorld,
} from '../../packages/world/src/spec-version.js';

type RustSuccess = { ok: true; value: unknown };
type RustFailure = {
  ok: false;
  error: { code: string; message: string };
};
type RustOutcome = RustSuccess | RustFailure;

type Outcome = { ok: true; value: unknown } | { ok: false; error: unknown };

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

function typescript(operation: () => unknown): Outcome {
  try {
    return { ok: true, value: normalized(operation()) };
  } catch (error) {
    return { ok: false, error };
  }
}

function expectParity(
  op: string,
  input: unknown,
  operation: () => unknown
): void {
  const rustOutcome = rust(op, input);
  const tsOutcome = typescript(operation);
  expect(rustOutcome.ok).toBe(tsOutcome.ok);
  if (rustOutcome.ok && tsOutcome.ok) {
    expect(rustOutcome.value).toEqual(tsOutcome.value);
  }
}

function withEnvNumber(
  name: string,
  raw: string | undefined,
  fallback: number,
  options: { min?: number; max?: number; integer?: boolean } = {}
): number {
  const previous = process.env[name];
  try {
    if (raw === undefined) delete process.env[name];
    else process.env[name] = raw;
    return envNumber(name, fallback, options);
  } finally {
    if (previous === undefined) delete process.env[name];
    else process.env[name] = previous;
  }
}

afterEach(() => {
  _resetEnvWarnCacheForTests();
});

describe('Rust World attribute parity', () => {
  const validCases = [
    {
      changes: [
        { key: 'phase', value: 'init' },
        { key: 'stale', value: null },
      ],
    },
    {
      changes: [{ key: '💥'.repeat(128), value: '💥'.repeat(64) }],
    },
    {
      changes: [{ key: '$framework.kind', value: 'agent' }],
      allowReservedAttributes: true,
    },
    {
      changes: [{ key: 'k0', value: 'updated' }],
      existingKeys: Array.from({ length: 64 }, (_, index) => `k${index}`),
    },
  ];

  for (const fixture of validCases) {
    it(`accepts ${JSON.stringify(fixture).slice(0, 80)}`, () => {
      expectParity('validateAttributeChanges', fixture, () =>
        validateAttributeChanges(fixture.changes, {
          existingKeys: fixture.existingKeys,
          allowReservedAttributes: fixture.allowReservedAttributes,
        })
      );
    });
  }

  const invalidCases = [
    { changes: [{ key: '', value: 'value' }] },
    { changes: [{ key: 'k'.repeat(257), value: 'value' }] },
    { changes: [{ key: 'value', value: '💥'.repeat(65) }] },
    { changes: [{ key: '$private', value: 'value' }] },
    {
      changes: [
        { key: 'phase', value: 'init' },
        { key: 'phase', value: 'done' },
      ],
    },
    {
      changes: Array.from({ length: 65 }, (_, index) => ({
        key: `k${index}`,
        value: 'v',
      })),
    },
  ];

  for (const fixture of invalidCases) {
    it(`rejects ${JSON.stringify(fixture).slice(0, 80)}`, () => {
      expectParity('validateAttributeChanges', fixture, () =>
        validateAttributeChanges(fixture.changes)
      );
    });
  }

  it('applies upserts and deletions without mutating the input', () => {
    const existing = { a: '1', stale: 'x' };
    const changes = [
      { key: 'stale', value: null },
      { key: 'fresh', value: 'yes' },
    ];
    expectParity('applyAttributeChanges', { existing, changes }, () =>
      applyAttributeChanges(existing, changes)
    );
    expect(existing).toEqual({ a: '1', stale: 'x' });
  });

  it('treats __proto__ as an ordinary own data key', () => {
    const changes = [{ key: '__proto__', value: 'ordinary-data' }];
    expectParity('applyAttributeChanges', { changes }, () => {
      const output = applyAttributeChanges(undefined, changes);
      expect(Object.hasOwn(output, '__proto__')).toBe(true);
      expect(output.__proto__).toBe('ordinary-data');
      return output;
    });
  });
});

describe('Rust World environment and spec-version parity', () => {
  const numberCases = [
    { raw: undefined, fallback: 100 },
    { raw: '', fallback: 100 },
    { raw: '42', fallback: 100 },
    { raw: '0.05', fallback: 100, max: 1 },
    { raw: '2.5', fallback: 100, integer: true },
    { raw: 'abc', fallback: 100 },
    { raw: 'Infinity', fallback: 100 },
    { raw: '-5', fallback: 100 },
    { raw: '9999', fallback: 100, max: 500 },
    { raw: '0x10', fallback: 100 },
    { raw: '   ', fallback: 100 },
  ];

  for (const fixture of numberCases) {
    it(`matches envNumber for ${JSON.stringify(fixture)}`, () => {
      const name = 'WORKFLOW_RUST_PARITY_NUMBER';
      expectParity('envNumber', { name, ...fixture }, () =>
        withEnvNumber(name, fixture.raw, fixture.fallback, fixture)
      );
    });
  }

  for (const raw of [
    undefined,
    '',
    '0',
    'false',
    'FALSE',
    '1',
    'true',
    'yes',
  ]) {
    it(`matches envFlag for ${String(raw)}`, () => {
      const name = 'WORKFLOW_RUST_PARITY_FLAG';
      const environment = raw === undefined ? {} : { [name]: raw };
      expectParity('envFlag', { name, raw, fallback: true }, () =>
        envFlag(name, true, environment)
      );
    });
  }

  for (const raw of [undefined, '', '0', 'false', '1', 'true', 'yes-please']) {
    it(`matches mintedSpecVersion for ${String(raw)}`, () => {
      const environment = raw === undefined ? {} : { WORKFLOW_SEALED_LOG: raw };
      expectParity('mintedSpecVersion', { environment }, () =>
        mintedSpecVersion(environment)
      );
    });
  }

  for (const version of [null, 1, 2, 7, 8]) {
    it(`matches version predicates for ${String(version)}`, () => {
      expectParity('isLegacySpecVersion', { version }, () =>
        isLegacySpecVersion(version)
      );
      expectParity('requiresNewerWorld', { version }, () =>
        requiresNewerWorld(version)
      );
    });
  }
});

describe('Rust World event metadata parity', () => {
  for (const eventType of [
    'step_completed',
    'step_failed',
    'run_completed',
    'attr_set',
    'constructor',
    'toString',
    'unknown',
  ]) {
    it(`matches metadata for ${eventType}`, () => {
      expectParity('entityEventClass', { eventType }, () =>
        entityEventClass(eventType)
      );
      expectParity('getEventDataRefFields', { eventType }, () =>
        getEventDataRefFields(eventType)
      );
      expectParity('isSealedNoopEvent', { eventType }, () =>
        isSealedNoopEvent({ eventType })
      );
    });
  }

  for (const fixture of [
    { eventType: 'run_started' },
    { eventType: 'attr_set', correlationId: 'attr_A' },
    { eventType: 'attr_set' },
    { eventType: 'step_completed', correlationId: 'step_A' },
  ]) {
    it(`matches entity classification for ${JSON.stringify(fixture)}`, () => {
      expectParity('classifyEntityEvent', fixture, () =>
        classifyEntityEvent(fixture)
      );
    });
  }
});

describe('Rust World event helper parity', () => {
  const predicates = {
    isRunEventType,
    isTerminalRunEventType,
    isStepEventType,
    isTerminalStepEventType,
    isHookLifecycleEventType,
    isHookEventRequiringExistence,
    isWaitEventType,
    isChildEntityCreationEventType,
  };

  for (const [op, predicate] of Object.entries(predicates)) {
    for (const eventType of [
      'run_created',
      'run_completed',
      'step_created',
      'step_failed',
      'hook_received',
      'wait_created',
      'noop',
      'unknown',
    ]) {
      it(`${op} matches for ${eventType}`, () => {
        expectParity(op, { eventType }, () => predicate(eventType));
      });
    }
  }

  for (const event of [
    { eventType: 'step_created' },
    {
      eventType: 'step_started',
      eventData: { stepName: 'work', input: null },
    },
    { eventType: 'step_started', eventData: { stepName: 'work' } },
  ]) {
    it(`matches child-creation detection for ${JSON.stringify(event)}`, () => {
      expectParity('isChildEntityCreationEvent', { event }, () =>
        isChildEntityCreationEvent(event as never)
      );
    });
  }

  const stripFixtures = [
    {
      event: {
        eventType: 'step_completed',
        eventData: { stepName: 'work', result: [1, 2, 3] },
        eventId: 'evnt_1',
      },
      resolveData: 'none' as const,
    },
    {
      event: {
        eventType: 'run_cancelled',
        eventData: { cancelReason: 'cleanup' },
        eventId: 'evnt_2',
      },
      resolveData: 'none' as const,
    },
    {
      event: {
        eventType: 'step_completed',
        eventData: null,
        eventId: 'evnt_3',
      },
      resolveData: 'none' as const,
    },
    {
      event: {
        eventType: 'step_completed',
        eventData: { stepName: 'work', result: [1, 2, 3] },
        eventId: 'evnt_4',
      },
      resolveData: 'all' as const,
    },
  ];

  for (const fixture of stripFixtures) {
    it(`matches ref stripping for ${fixture.event.eventId}`, () => {
      expectParity('stripEventDataRefs', fixture, () =>
        stripEventDataRefs(fixture.event as never, fixture.resolveData)
      );
    });
  }
});

describe('Rust World slot identity parity', () => {
  for (const slot of [1, 2, 9, 10, 99, 100, 1_000, Number.MAX_SAFE_INTEGER]) {
    it(`round-trips slot ${slot}`, () => {
      expectParity('slotToEventId', { slot }, () => slotToEventId(slot));
      const eventId = slotToEventId(slot);
      expectParity('eventIdToSlot', { eventId }, () => eventIdToSlot(eventId));
    });
  }

  for (const slot of [0, -1, 1.5, Number.MAX_SAFE_INTEGER + 2]) {
    it(`rejects invalid slot ${slot}`, () => {
      expectParity('slotToEventId', { slot }, () => slotToEventId(slot));
    });
  }

  const body = String(42).padStart(26, '0');
  for (const eventId of [`evnt_${body}`, `wevt_${body}`, body, 'not-an-id']) {
    it(`matches slot detection for ${eventId}`, () => {
      expectParity('isSlotEventId', { eventId }, () => isSlotEventId(eventId));
      expectParity('eventIdToSlot', { eventId }, () => eventIdToSlot(eventId));
    });
  }

  for (const candidate of [
    body,
    '0000000001'.padEnd(26, '0'),
    `${'0'.repeat(25)}A`,
    '',
  ]) {
    it(`matches body detection for ${candidate || '<empty>'}`, () => {
      expectParity('isSlotBody', { body: candidate }, () =>
        isSlotBody(candidate)
      );
    });
  }
});

describe('Rust World queue contract parity', () => {
  for (const fixture of [
    { kind: 'workflow', namespace: undefined },
    { kind: 'workflow', namespace: 'custom' },
    { kind: 'workflow', namespace: 'myframework123' },
    { kind: 'workflow', namespace: '123abc' },
    { kind: 'workflow', namespace: 'Custom' },
    { kind: 'step', namespace: undefined },
  ]) {
    it(`matches prefix construction for ${JSON.stringify(fixture)}`, () => {
      expectParity('getQueueTopicPrefix', fixture, () =>
        getQueueTopicPrefix(fixture.kind as never, fixture.namespace)
      );
    });
  }

  for (const value of [
    '__wkf_workflow_',
    '__custom_wkf_workflow_',
    '__wkf_step_',
    '__Custom_wkf_workflow_',
    'bad_prefix',
  ]) {
    it(`matches QueuePrefix parsing for ${value}`, () => {
      expectParity(
        'isValidQueuePrefix',
        { value },
        () => QueuePrefix.safeParse(value).success
      );
    });
  }

  for (const value of [
    '__wkf_workflow_myFlow',
    '__custom_wkf_workflow_myFlow',
    '__wkf_workflow_',
    '__wkf_step_myStep',
    'not_a_queue_name',
  ]) {
    it(`matches queue-name validation for ${value}`, () => {
      const rustOutcome = rust('isValidQueueName', { value });
      const tsOutcome = typescript(
        () => ValidQueueName.safeParse(value).success
      );
      expect(rustOutcome).toEqual(tsOutcome);
    });
  }

  for (const value of [
    '__wkf_workflow_myFlow',
    '__custom_wkf_workflow_myFlow',
  ]) {
    it(`matches queue-name parsing for ${value}`, () => {
      expectParity('parseQueueName', { value }, () =>
        parseQueueName(value as never)
      );
    });
  }

  for (const payload of [
    {
      __healthCheck: true,
      correlationId: 'corr_123',
      runId: 'wrun_01ABC',
    },
    { __healthCheck: true, correlationId: 'corr_123' },
    { runId: 'wrun_01ABC', stepId: 'step_1' },
    {
      runId: 'wrun_01ABC',
      runInput: {
        input: { foo: 'bar' },
        deploymentId: 'dpl_123',
        workflowName: 'myWorkflow',
        specVersion: 7,
        environment: 'preview',
        futureField: 'ignored',
      },
    },
  ]) {
    it(`matches valid queue payload ${JSON.stringify(payload)}`, () => {
      expectParity('parseQueuePayload', { payload }, () =>
        QueuePayloadSchema.parse(payload)
      );
    });
  }
});

describe('Rust World bulk cancellation parity', () => {
  const valid = [
    { runIds: ['wrun_1'], cancelReason: 'cleanup' },
    {
      runIds: Array.from({ length: 500 }, (_, index) => `wrun_${index}`),
    },
  ];
  for (const request of valid) {
    it(`accepts request with ${request.runIds.length} ids`, () => {
      expectParity('validateBulkCancelRequest', { request }, () => {
        BulkCancelWorkflowRunsRequestSchema.parse(request);
      });
    });
  }

  const invalid = [
    { runIds: [] },
    { runIds: ['wrun_1', 'wrun_1'] },
    {
      runIds: Array.from({ length: 501 }, (_, index) => `wrun_${index}`),
    },
    { runIds: ['wrun_1'], cancelReason: 'x'.repeat(513) },
  ];
  for (const request of invalid) {
    it(`rejects invalid request ${JSON.stringify(request).slice(0, 80)}`, () => {
      expectParity('validateBulkCancelRequest', { request }, () => {
        BulkCancelWorkflowRunsRequestSchema.parse(request);
      });
    });
  }
});
