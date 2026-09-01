import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { normalizeAttributeChanges } from '../../packages/core/src/attribute-changes.js';
import {
  ATTRIBUTE_KEY_MAX_LENGTH,
  ATTRIBUTE_MAX_PER_RUN,
  ATTRIBUTE_VALUE_MAX_BYTES,
} from '../../packages/world/src/attributes-validation.js';

interface Success {
  ok: true;
  changes: Array<{ key: string; value: string | null }>;
}

interface Failure {
  ok: false;
  error: string;
}

type Outcome = Success | Failure;
type Entry = readonly [string, string | undefined];

interface Fixture {
  name: string;
  input: unknown;
  options?: { allowReservedAttributes?: boolean };
  rustArguments: string[];
}

const binary =
  process.env.WORKFLOW_RUST_ATTRIBUTE_CHANGES_CONFORMANCE_BIN ??
  path.resolve(
    'crates/workflow-core/target/debug/examples/attribute_changes_conformance'
  );

function decodeHex(encoded: string): string {
  if (!/^(?:[0-9a-f]{2})*$/.test(encoded)) {
    throw new Error('Rust attribute conformance runner returned invalid hex');
  }
  return Buffer.from(encoded, 'hex').toString('utf8');
}

function runRust(arguments_: string[]): Outcome {
  const output = execFileSync(binary, arguments_, {
    encoding: 'utf8',
    timeout: 5_000,
    maxBuffer: 1024 * 1024,
  }).trimEnd();
  const [kind, ...fields] = output.split('\t');

  if (kind === 'err' && fields.length === 1) {
    return { ok: false, error: decodeHex(fields[0]!) };
  }
  if (kind !== 'ok' || fields.length % 2 !== 0) {
    throw new Error('Rust attribute conformance runner returned malformed output');
  }

  const changes: Success['changes'] = [];
  for (let index = 0; index < fields.length; index += 2) {
    const key = decodeHex(fields[index]!);
    const valueToken = fields[index + 1]!;
    const value =
      valueToken === 'n'
        ? null
        : valueToken.startsWith('s')
          ? decodeHex(valueToken.slice(1))
          : undefined;
    if (value === undefined) {
      throw new Error(
        'Rust attribute conformance runner returned an invalid value token'
      );
    }
    changes.push({ key, value });
  }
  return { ok: true, changes };
}

function runTypeScript(
  input: unknown,
  options: { allowReservedAttributes?: boolean } = {}
): Outcome {
  try {
    return {
      ok: true,
      changes: normalizeAttributeChanges(
        input as Record<string, string | undefined>,
        options
      ),
    };
  } catch (error) {
    return { ok: false, error: (error as Error).message };
  }
}

function recordFixture(
  name: string,
  entries: Entry[],
  allowReservedAttributes = false
): Fixture {
  const input = Object.fromEntries(entries) as Record<
    string,
    string | undefined
  >;
  const rustArguments = [
    'record',
    allowReservedAttributes ? '1' : '0',
  ];
  for (const [key, value] of Object.entries(input)) {
    rustArguments.push(key, value === undefined ? 'n' : `s:${value}`);
  }
  return {
    name,
    input,
    options: allowReservedAttributes
      ? { allowReservedAttributes: true }
      : undefined,
    rustArguments,
  };
}

const exactCapEntries: Entry[] = Array.from(
  { length: ATTRIBUTE_MAX_PER_RUN },
  (_, index) => [`key_${index}`, 'v'] as const
);
const aboveCapEntries: Entry[] = Array.from(
  { length: ATTRIBUTE_MAX_PER_RUN + 1 },
  (_, index) => [`key_${index}`, 'v'] as const
);

const fixtures: Fixture[] = [
  recordFixture('ordered changes and deletion', [
    ['phase', 'init'],
    ['stale', undefined],
  ]),
  recordFixture('empty record', []),
  { name: 'null input', input: null, rustArguments: ['null'] },
  {
    name: 'array input',
    input: ['phase', 'init'],
    rustArguments: ['array'],
  },
  { name: 'string input', input: 'phase=init', rustArguments: ['string'] },
  { name: 'number input', input: 42, rustArguments: ['number'] },
  recordFixture('ASCII key at the boundary', [
    ['k'.repeat(ATTRIBUTE_KEY_MAX_LENGTH), 'v'],
  ]),
  recordFixture('ASCII key above the boundary', [
    ['k'.repeat(ATTRIBUTE_KEY_MAX_LENGTH + 1), 'v'],
  ]),
  recordFixture('astral key at the JavaScript UTF-16 boundary', [
    ['💥'.repeat(ATTRIBUTE_KEY_MAX_LENGTH / 2), 'v'],
  ]),
  recordFixture('astral key above the JavaScript UTF-16 boundary', [
    ['💥'.repeat(ATTRIBUTE_KEY_MAX_LENGTH / 2 + 1), 'v'],
  ]),
  recordFixture('UTF-8 value at the byte boundary', [
    ['note', 'é'.repeat(ATTRIBUTE_VALUE_MAX_BYTES / 2)],
  ]),
  recordFixture('UTF-8 value above the byte boundary', [
    ['note', 'é'.repeat(ATTRIBUTE_VALUE_MAX_BYTES / 2 + 1)],
  ]),
  recordFixture('reserved namespace denied', [['$system', 'x']]),
  recordFixture(
    'reserved namespace explicitly allowed',
    [['$agent.kind', 'durable']],
    true
  ),
  recordFixture('exact per-run batch cap', exactCapEntries),
  recordFixture('above per-run batch cap', aboveCapEntries),
];

describe('Rust attribute normalization parity', () => {
  for (const fixture of fixtures) {
    it(fixture.name, () => {
      expect(runRust(fixture.rustArguments)).toEqual(
        runTypeScript(fixture.input, fixture.options)
      );
    });
  }
});
