import { describe, expect, it } from 'vitest';
import { getErrorMessage } from './get-error-message.js';

describe('getErrorMessage legacy JSON serialization', () => {
  it('executes an object-supplied toJSON callback', () => {
    let calls = 0;
    const value = {
      toJSON() {
        calls += 1;
        return { rewritten: true };
      },
    };

    expect(getErrorMessage(value)).toBe('{"rewritten":true}');
    expect(calls).toBe(1);
  });

  it('throws while normalizing a cyclic object', () => {
    const cyclic: Record<string, unknown> = {};
    cyclic.self = cyclic;

    expect(() => getErrorMessage(cyclic)).toThrow(TypeError);
  });

  it('throws while normalizing a bigint', () => {
    expect(() => getErrorMessage(1n)).toThrow(TypeError);
  });
});
