import { describe, expect, it } from 'vitest';
import { parseDurationToDate } from './time';

describe('parseDurationToDate', () => {
  it('should parse duration strings correctly', () => {
    const result = parseDurationToDate('5s');
    expect(result).toBeInstanceOf(Date);
    expect(result.getTime()).toBeGreaterThan(Date.now());
  });

  it('should parse numbers as milliseconds', () => {
    const result = parseDurationToDate(1000);
    expect(result).toBeInstanceOf(Date);
    expect(result.getTime()).toBeGreaterThan(Date.now());
  });

  it('should handle Date objects', () => {
    const futureDate = new Date(Date.now() + 5000);
    const result = parseDurationToDate(futureDate);
    expect(result).toEqual(futureDate);
  });

  it('should handle valid date-like objects from deserialization', () => {
    const timestamp = Date.now() + 5000;
    const result = parseDurationToDate({ getTime: () => timestamp } as Date);
    expect(result.getTime()).toBe(timestamp);
  });

  it('should throw on invalid duration strings', () => {
    // @ts-expect-error - invalid duration string
    expect(() => parseDurationToDate('invalid')).toThrow();
  });

  it('should throw on negative numbers', () => {
    expect(() => parseDurationToDate(-1000)).toThrow();
  });

  it('rejects invalid Date objects instead of returning a poisoned retry timestamp', () => {
    expect(() => parseDurationToDate(new Date(Number.NaN))).toThrow(
      'Expected a valid Date with a finite timestamp'
    );
  });

  it('rejects date-like objects with non-finite timestamps', () => {
    expect(() =>
      parseDurationToDate({ getTime: () => Number.POSITIVE_INFINITY } as Date)
    ).toThrow('Expected a valid Date with a finite timestamp');
  });

  it('rejects finite durations whose resulting Date overflows TimeClip', () => {
    expect(() => parseDurationToDate(Number.MAX_VALUE)).toThrow(
      'Resulting date is outside the supported range'
    );
  });
});
