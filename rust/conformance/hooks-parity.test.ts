import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  HOOK_RESUME_DEDUP_VERSION,
  HOOK_RESUME_INPUT_VERSION,
  HookResumeCapabilitiesSchema,
  HookResumeContextSchema,
} from '../../packages/world/src/hooks.js';

type RustSuccess = { ok: true; value: unknown };
type RustFailure = {
  ok: false;
  error: { code: string; message: string };
};
type RustOutcome = RustSuccess | RustFailure;
type TypeScriptOutcome =
  | { ok: true; value: unknown }
  | { ok: false; error: unknown };

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

function typescript(operation: () => unknown): TypeScriptOutcome {
  try {
    return {
      ok: true,
      value: JSON.parse(JSON.stringify(operation())),
    };
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
  const typescriptOutcome = typescript(operation);
  expect(rustOutcome.ok).toBe(typescriptOutcome.ok);
  if (rustOutcome.ok && typescriptOutcome.ok) {
    expect(rustOutcome.value).toEqual(typescriptOutcome.value);
  }
}

const fullContext = {
  deploymentId: 'deployment_1',
  workflowName: 'processOrder',
  runSpecVersion: 5,
  workflowCoreVersion: '5.0.0',
  traceCarrier: { traceparent: '00-abc-def-01' },
  encryptionPublicKey: 'ZmFrZS1wdWJsaWMta2V5',
  hookResumeInputVersion: 1,
};

describe('Rust hook resume protocol parity', () => {
  it('matches protocol version constants', () => {
    expect(rust('hookProtocolVersions', {})).toEqual({
      ok: true,
      value: {
        hookResumeInputVersion: HOOK_RESUME_INPUT_VERSION,
        hookResumeDedupVersion: HOOK_RESUME_DEDUP_VERSION,
      },
    });
  });

  for (const input of [
    {
      deploymentId: 'deployment_1',
      workflowName: 'processOrder',
    },
    fullContext,
    {
      ...fullContext,
      resumeCapabilities: { hookResumeDedupVersion: 1 },
      unexpected: 'stripped',
    },
    {
      ...fullContext,
      traceCarrier: { traceparent: 42 },
    },
    {
      workflowName: 'missingDeployment',
    },
  ]) {
    it(`matches HookResumeContextSchema for ${JSON.stringify(input)}`, () => {
      expectParity(
        'parseHookResumeContext',
        { value: input },
        () => HookResumeContextSchema.parse(input)
      );
    });
  }

  for (const input of [
    { hookResumeDedupVersion: 1 },
    { hookResumeDedupVersion: 1, unknown: true },
    {},
    { hookResumeDedupVersion: '1' },
  ]) {
    it(`matches HookResumeCapabilitiesSchema for ${JSON.stringify(input)}`, () => {
      expectParity(
        'parseHookResumeCapabilities',
        { value: input },
        () => HookResumeCapabilitiesSchema.parse(input)
      );
    });
  }
});
