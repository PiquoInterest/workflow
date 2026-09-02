import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { afterEach, describe, expect, it } from 'vitest';
// @ts-expect-error -- plain JS lint rule, no type declarations
import { scanPackage } from '../../../scripts/lint/module-scope-state.mjs';

const tempPackages: string[] = [];

function packageWith(source: string): string {
  const dir = fs.mkdtempSync(
    path.join(os.tmpdir(), 'module-scope-state-security-')
  );
  tempPackages.push(dir);
  fs.mkdirSync(path.join(dir, 'src'));
  fs.writeFileSync(path.join(dir, 'src/state.ts'), source);
  return dir;
}

afterEach(() => {
  for (const dir of tempPackages.splice(0)) {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

describe('module-scope state legacy bypass characterization', () => {
  it('trusts an unrelated method whose property name is globalSingleton', () => {
    const dir = packageWith(
      [
        'const localFactory = {',
        '  globalSingleton: () => ({ count: 0 }),',
        '};',
        'const state = localFactory.globalSingleton();',
        'export function bump() {',
        '  state.count++;',
        '}',
        '',
      ].join('\n')
    );

    // Legacy evidence: the TypeScript rule checks only the final property name,
    // not whether the helper came from @workflow/utils.
    expect(scanPackage(dir, dir)).toEqual([]);
  });

  it('trusts a local object merely because a binary operand is globalThis', () => {
    const dir = packageWith(
      [
        'const state = globalThis && { count: 0 };',
        'export function bump() {',
        '  state.count++;',
        '}',
        '',
      ].join('\n')
    );

    // `globalThis && localObject` evaluates to the local object, but the legacy
    // recursive binary-expression check treats either operand as sufficient.
    expect(scanPackage(dir, dir)).toEqual([]);
  });

  it('misses a static registry mutated through a literal computed property', () => {
    const dir = packageWith(
      [
        'export class Registry {',
        '  static transports = new Map<string, number>();',
        '  static open(id: string) {',
        "    Registry['transports'].set(id, 1);",
        '  }',
        '}',
        '',
      ].join('\n')
    );

    // Legacy evidence: every element access discards the field segment, even
    // when the key is a literal and therefore statically knowable.
    expect(scanPackage(dir, dir)).toEqual([]);
  });
});
