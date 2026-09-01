import { describe, expect, it } from 'vitest';
import { formatStepName, formatWorkflowName } from './parse-name';

function containsLogBreakingCharacter(value: string): boolean {
  for (const character of value) {
    const codePoint = character.codePointAt(0);
    if (
      codePoint !== undefined &&
      (codePoint <= 0x1f ||
        (codePoint >= 0x7f && codePoint <= 0x9f) ||
        codePoint === 0x2028 ||
        codePoint === 0x2029)
    ) {
      return true;
    }
  }
  return false;
}

describe('machine-name log formatting security', () => {
  it('escapes controls in parsed function and module names', () => {
    const formatted = formatStepName(
      'step//./jobs/\u001b[31mred//run\r\nforged\t\u2028'
    );

    expect(formatted).toBe(
      'run\\r\\nforged\\t\\u2028 (./jobs/\\u001b[31mred)'
    );
    expect(containsLogBreakingCharacter(formatted)).toBe(false);
  });

  it('escapes controls when formatting falls back to a legacy name', () => {
    const formatted = formatWorkflowName('legacy\nforged\u001b]8;;target\u0007');

    expect(formatted).toBe('legacy\\nforged\\u001b]8;;target\\u0007');
    expect(containsLogBreakingCharacter(formatted)).toBe(false);
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
