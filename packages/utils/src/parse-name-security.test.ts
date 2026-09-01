import { describe, expect, it } from 'vitest';
import { formatStepName, formatWorkflowName } from './parse-name';

const LOG_BREAKING_CHARACTERS = /[\u0000-\u001f\u007f-\u009f\u2028\u2029]/u;

describe('machine-name log formatting security', () => {
  it('escapes controls in parsed function and module names', () => {
    const formatted = formatStepName(
      'step//./jobs/\u001b[31mred//run\r\nforged\t\u2028'
    );

    expect(formatted).toBe(
      'run\\r\\nforged\\t\\u2028 (./jobs/\\u001b[31mred)'
    );
    expect(formatted).not.toMatch(LOG_BREAKING_CHARACTERS);
  });

  it('escapes controls when formatting falls back to a legacy name', () => {
    const formatted = formatWorkflowName('legacy\nforged\u001b]8;;target\u0007');

    expect(formatted).toBe('legacy\\nforged\\u001b]8;;target\\u0007');
    expect(formatted).not.toMatch(LOG_BREAKING_CHARACTERS);
  });

  it('preserves ordinary parsed and fallback rendering', () => {
    expect(formatStepName('step//./jobs/order//run')).toBe(
      'run (./jobs/order)'
    );
    expect(formatWorkflowName('legacy-workflow-name')).toBe(
      'legacy-workflow-name'
    );
  });
});
