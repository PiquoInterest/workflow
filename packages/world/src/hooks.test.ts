import { describe, expect, it } from 'vitest';
import {
  HOOK_RESUME_DEDUP_VERSION,
  HOOK_RESUME_INPUT_VERSION,
  HookResumeCapabilitiesSchema,
  HookResumeContextSchema,
  HookSchema,
  PersistedHookSchema,
} from './hooks.js';

const baseHook = {
  runId: 'wrun_1',
  hookId: 'hook_1',
  token: 'order:1',
  ownerId: 'owner_1',
  projectId: 'project_1',
  environment: 'production',
  createdAt: new Date('2026-01-02T03:04:05.000Z'),
};

const resumeContext = {
  deploymentId: 'deployment_1',
  workflowName: 'processOrder',
  runSpecVersion: 5,
  workflowCoreVersion: '5.0.0',
  traceCarrier: { traceparent: '00-abc-def-01' },
  encryptionPublicKey: 'ZmFrZS1wdWJsaWMta2V5',
  hookResumeInputVersion: HOOK_RESUME_INPUT_VERSION,
};

const resumeCapabilities = {
  hookResumeDedupVersion: HOOK_RESUME_DEDUP_VERSION,
};

const invalidProtocolVersions = [0, -1, 1.5, 0x1_0000_0000];

describe('hook resume protocol versions', () => {
  it('keeps the producer and backend capability versions stable', () => {
    expect(HOOK_RESUME_INPUT_VERSION).toBe(1);
    expect(HOOK_RESUME_DEDUP_VERSION).toBe(1);
  });
});

describe('HookResumeContextSchema', () => {
  it('parses and preserves a resumeContext', () => {
    expect(HookResumeContextSchema.parse(resumeContext)).toEqual(resumeContext);
  });

  it('strips response-only capability fields and unrelated unknown fields', () => {
    expect(
      HookResumeContextSchema.parse({
        ...resumeContext,
        resumeCapabilities,
        unexpected: 'not persisted',
      })
    ).toEqual(resumeContext);
  });

  it('rejects a non-string trace carrier value', () => {
    expect(() =>
      HookResumeContextSchema.parse({
        ...resumeContext,
        traceCarrier: { traceparent: 42 },
      })
    ).toThrow();
  });

  it('requires bounded integer protocol versions', () => {
    for (const version of invalidProtocolVersions) {
      expect(() =>
        HookResumeContextSchema.parse({
          ...resumeContext,
          runSpecVersion: version,
        })
      ).toThrow();
      expect(() =>
        HookResumeContextSchema.parse({
          ...resumeContext,
          hookResumeInputVersion: version,
        })
      ).toThrow();
    }

    expect(
      HookResumeContextSchema.parse({
        ...resumeContext,
        runSpecVersion: 0xffff_ffff,
      }).runSpecVersion
    ).toBe(0xffff_ffff);
  });
});

describe('HookResumeCapabilitiesSchema', () => {
  it('parses the live backend attestation and strips unknown fields', () => {
    expect(
      HookResumeCapabilitiesSchema.parse({
        ...resumeCapabilities,
        staleServerField: true,
      })
    ).toEqual(resumeCapabilities);
  });

  it('requires a bounded integer dedup protocol version', () => {
    expect(() => HookResumeCapabilitiesSchema.parse({})).toThrow();
    expect(() =>
      HookResumeCapabilitiesSchema.parse({ hookResumeDedupVersion: '1' })
    ).toThrow();
    for (const version of invalidProtocolVersions) {
      expect(() =>
        HookResumeCapabilitiesSchema.parse({
          hookResumeDedupVersion: version,
        })
      ).toThrow();
    }

    expect(
      HookResumeCapabilitiesSchema.parse({
        hookResumeDedupVersion: 0xffff_ffff,
      }).hookResumeDedupVersion
    ).toBe(0xffff_ffff);
  });
});

describe('HookSchema resumeContext', () => {
  it('parses and preserves a resumeContext', () => {
    const parsed = HookSchema.parse({ ...baseHook, resumeContext });
    expect(parsed.resumeContext).toEqual(resumeContext);
  });

  it('is optional — a hook without it still parses', () => {
    const parsed = HookSchema.parse(baseHook);
    expect(parsed.resumeContext).toBeUndefined();
  });

  it('an old client strips the unknown resumeContext field', () => {
    const legacyHookSchema = HookSchema.omit({ resumeContext: true });
    const parsed = legacyHookSchema.parse({ ...baseHook, resumeContext });
    expect('resumeContext' in parsed).toBe(false);
    expect(parsed).toMatchObject({
      runId: baseHook.runId,
      hookId: baseHook.hookId,
      token: baseHook.token,
    });
  });
});

describe('PersistedHookSchema', () => {
  it('strips transient backend capabilities before persistence', () => {
    const parsed = PersistedHookSchema.parse({
      ...baseHook,
      resumeContext,
      resumeCapabilities,
    });

    expect(parsed.resumeContext).toEqual(resumeContext);
    expect('resumeCapabilities' in parsed).toBe(false);
  });

  it('keeps HookSchema capable of parsing the response-only attestation', () => {
    const parsed = HookSchema.parse({
      ...baseHook,
      resumeContext,
      resumeCapabilities,
    });
    expect(parsed.resumeCapabilities).toEqual(resumeCapabilities);
  });
});
