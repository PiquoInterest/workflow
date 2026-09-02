import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { getRunCapabilities } from '../../packages/core/src/capabilities.js';

interface CapabilityView {
  supportedFormats: string[];
  framedByteStreams: boolean;
}

const binary =
  process.env.WORKFLOW_RUST_CAPABILITIES_CONFORMANCE_BIN ??
  path.resolve('target/debug/examples/capabilities_conformance');

function rust(version: string | undefined): CapabilityView {
  const output = execFileSync(
    binary,
    version === undefined ? [] : [version],
    {
      encoding: 'utf8',
      timeout: 5_000,
      maxBuffer: 1024 * 1024,
    }
  );
  return JSON.parse(output) as CapabilityView;
}

function typescript(version: string | undefined): CapabilityView {
  const capabilities = getRunCapabilities(version);
  return {
    supportedFormats: [...capabilities.supportedFormats],
    framedByteStreams: capabilities.framedByteStreams,
  };
}

const versions: Array<string | undefined> = [
  undefined,
  '',
  'dev',
  'not-a-version',
  '4',
  '4.2',
  '3.0.0',
  '4.1.0-beta.63',
  '4.2.0-beta.64',
  'v4.2.0-beta.64',
  '4.2.0',
  '4.99.99',
  '5.0.0-beta.2',
  '5.0.0-beta.14',
  '5.0.0-beta.15',
  '5.0.0-beta.17',
  '5.0.0-beta.18',
  '5.0.0-beta.36',
  '5.0.0-beta.37',
  '5.0.0',
  '5.1.0',
  '6.0.0',
  '  v5.0.0-beta.37+build.9  ',
  '01.0.0',
  '5.0.0-beta.01',
  '9007199254740992.0.0',
  `${' '.repeat(256)}5.0.0`,
];

describe('Rust run-capability negotiation parity', () => {
  for (const version of versions) {
    it(`matches TypeScript for ${JSON.stringify(version)}`, () => {
      expect(rust(version)).toEqual(typescript(version));
    });
  }
});
