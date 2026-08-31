import type { StringValue } from 'ms';

import {
  parseDurationToDate as parseDurationToDateUnchecked,
} from './time-implementation.js';

/**
 * Parses a retry duration and rejects invalid JavaScript Date results.
 *
 * The compatibility implementation performs the legacy `ms` parsing and
 * date-like-object handling. This boundary additionally verifies TimeClip so
 * NaN, infinity, and finite-but-out-of-range timestamps cannot reach queue or
 * persistence code.
 */
export function parseDurationToDate(
  param: StringValue | Date | number
): Date {
  const result = parseDurationToDateUnchecked(param);
  if (Number.isFinite(result.getTime())) {
    return result;
  }

  const isDateInput =
    param instanceof Date ||
    (param !== null &&
      typeof param === 'object' &&
      typeof (param as { getTime?: unknown }).getTime === 'function');

  if (isDateInput) {
    throw new Error(
      'Invalid duration Date. Expected a valid Date with a finite timestamp.'
    );
  }

  throw new Error(
    'Invalid duration. Resulting date is outside the supported range.'
  );
}
