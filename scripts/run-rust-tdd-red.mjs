#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const configPath = path.resolve(ROOT, process.argv[2] ?? 'rust/tdd-red.json');
const overridesPath = path.resolve(
  ROOT,
  process.argv[3] ?? 'rust/test-port-overrides.json'
);
const config = JSON.parse(readFileSync(configPath, 'utf8'));
const overrides = JSON.parse(readFileSync(overridesPath, 'utf8'));

if (config.schemaVersion !== 1 || !Array.isArray(config.cases)) {
  throw new Error('rust/tdd-red.json must use schemaVersion 1 and contain cases');
}
if (overrides.schemaVersion !== 1 || !Array.isArray(overrides.entries)) {
  throw new Error(
    'rust/test-port-overrides.json must use schemaVersion 1 and contain entries'
  );
}

let failures = 0;
const configuredPaths = new Set();
for (const testCase of config.cases) {
  if (typeof testCase?.typescript !== 'string') continue;
  if (configuredPaths.has(testCase.typescript)) {
    console.error(`[rust-tdd-red] duplicate case: ${testCase.typescript}`);
    failures += 1;
  }
  configuredPaths.add(testCase.typescript);
}

const expectedRedPaths = new Set(
  overrides.entries
    .filter((entry) => entry?.status === 'red')
    .map((entry) => entry.typescript)
);
for (const typescript of expectedRedPaths) {
  if (!configuredPaths.has(typescript)) {
    console.error(
      `[rust-tdd-red] manifest override is RED without a runner case: ${typescript}`
    );
    failures += 1;
  }
}
for (const typescript of configuredPaths) {
  if (!expectedRedPaths.has(typescript)) {
    console.error(
      `[rust-tdd-red] runner case is not marked RED in overrides: ${typescript}`
    );
    failures += 1;
  }
}

for (const testCase of config.cases) {
  const { typescript, command, failureMarker } = testCase;
  if (
    typeof typescript !== 'string' ||
    !Array.isArray(command) ||
    command.length === 0 ||
    command.some((part) => typeof part !== 'string') ||
    typeof failureMarker !== 'string' ||
    failureMarker.length === 0
  ) {
    console.error(`[rust-tdd-red] invalid case: ${JSON.stringify(testCase)}`);
    failures += 1;
    continue;
  }

  const [program, ...args] = command;
  const result = spawnSync(program, args, {
    cwd: ROOT,
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
  });
  const output = `${result.stdout ?? ''}${result.stderr ?? ''}`;

  if (result.error) {
    console.error(`[rust-tdd-red] ${typescript}: could not execute test command`);
    console.error(result.error);
    failures += 1;
    continue;
  }
  if (result.signal) {
    console.error(`[rust-tdd-red] ${typescript}: terminated by ${result.signal}`);
    failures += 1;
    continue;
  }
  if (result.status === 0) {
    console.error(
      `[rust-tdd-red] ${typescript}: unexpectedly GREEN; move it out of the RED manifest`
    );
    failures += 1;
    continue;
  }
  if (!output.includes(failureMarker)) {
    console.error(
      `[rust-tdd-red] ${typescript}: failed for the wrong reason; marker not found`
    );
    console.error(output);
    failures += 1;
    continue;
  }

  console.log(`[rust-tdd-red] expected RED confirmed: ${typescript}`);
}

if (failures > 0) process.exit(1);
