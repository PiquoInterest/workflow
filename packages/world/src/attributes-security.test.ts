import { describe, expect, it } from 'vitest';
import { applyAttributeChanges } from './attributes-validation.js';

describe('applyAttributeChanges security invariants', () => {
  it('materializes __proto__ as ordinary data without changing the prototype', () => {
    const before = {};
    const after = applyAttributeChanges(before, [
      { key: '__proto__', value: 'ordinary-data' },
    ]);

    expect(Object.getPrototypeOf(after)).toBe(Object.prototype);
    expect(Object.hasOwn(after, '__proto__')).toBe(true);
    expect(after.__proto__).toBe('ordinary-data');
    expect(before).toEqual({});
  });

  it('does not allow an object-shaped runtime value to mutate the prototype', () => {
    // This bypasses the TypeScript type on purpose to model an untrusted caller
    // reaching the mutation helper without first invoking validation.
    const attackerValue = { polluted: true } as unknown as string;
    const after = applyAttributeChanges(undefined, [
      { key: '__proto__', value: attackerValue },
    ]);

    expect(Object.getPrototypeOf(after)).toBe(Object.prototype);
    expect(Object.hasOwn(after, '__proto__')).toBe(true);
    expect(after.__proto__).toBe(attackerValue);
    expect(({} as { polluted?: boolean }).polluted).toBeUndefined();
  });
});
