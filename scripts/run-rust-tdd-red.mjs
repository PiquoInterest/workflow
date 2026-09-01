#!/usr/bin/env node

import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const configPath = path.resolve(ROOT, process.argv[2] ?? 'rust/tdd-red.json');
const overridesPath = path.resolve(
  ROOT,
  process.argv[3] ?? 'rust/test-port-overrides.json'
);

function fragmentDirectoryFor(filePath) {
  const extension = path.extname(filePath);
  return `${filePath.slice(0, -extension.length)}.d`;
}

function loadDocuments(primaryPath, arrayKey, label) {
  const paths = [primaryPath];
  const fragmentDirectory = fragmentDirectoryFor(primaryPath);

  if (existsSync(fragmentDirectory)) {
    if (!statSync(fragmentDirectory).isDirectory()) {
      throw new Error(`${fragmentDirectory} must be a directory`);
    }
    const fragments = readdirSync(fragmentDirectory, { withFileTypes: true })
      .filter((entry) => entry.isFile() && entry.name.endsWith('.json'))
      .map((entry) => entry.name)
      .sort((left, right) => left.localeCompare(right));
    paths.push(...fragments.map((name) => path.join(fragmentDirectory, name)));
  }

  return paths.flatMap((documentPath) => {
    const document = JSON.parse(readFileSync(documentPath, 'utf8'));
    if (document.schemaVersion !== 1 || !Array.isArray(document[arrayKey])) {
      throw new Error(
        `${path.relative(ROOT, documentPath)} must use schemaVersion 1 and contain ${arrayKey}`
      );
    }
    return document[arrayKey].map((entry) => ({
      ...entry,
      __sourceDocument: path.relative(ROOT, documentPath),
      __sourceLabel: label,
    }));
  });
}

const cases = loadDocuments(configPath, 'cases', 'expected-RED case');
const overrideEntries = loadDocuments(
  overridesPath,
  'entries',
  'test-port override'
);

let failures = 0;
const configuredPaths = new Set();
for (const testCase of cases) {
  if (typeof testCase?.typescript !== 'string') continue;
  if (configuredPaths.has(testCase.typescript)) {
    console.error(
      `[rust-tdd-red] duplicate case: ${testCase.typescript} (${testCase.__sourceDocument})`
    );
    failures += 1;
  }
  configuredPaths.add(testCase.typescript);
}

const overridePaths = new Set();
const expectedRedPaths = new Set();
for (const entry of overrideEntries) {
  if (typeof entry?.typescript !== 'string') continue;
  if (overridePaths.has(entry.typescript)) {
    console.error(
      `[rust-tdd-red] duplicate override: ${entry.typescript} (${entry.__sourceDocument})`
    );
    failures += 1;
  }
  overridePaths.add(entry.typescript);
  if (entry.status === 'red') expectedRedPaths.add(entry.typescript);
}

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

for (const testCase of cases) {
  const { typescript, command, failureMarker } = testCase;
  if (
    typeof typescript !== 'string' ||
    !Array.isArray(command) ||
    command.length === 0 ||
    command.some((part) => typeof part !== 'string') ||
    typeof failureMarker !== 'string' ||
    failureMarker.length === 0
  ) {
    console.error(
      `[rust-tdd-red] invalid case in ${testCase.__sourceDocument}: ${JSON.stringify(testCase)}`
    );
    failures += 1;
    continue;
  }

  const [program, ...args] = command;
  const result = spawnExpectedRed(program, args);
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

function spawnExpectedRed(program, args) {
  const { spawnSync } = awaitImportChildProcess();
  return spawnSync(program, args, {
    cwd: ROOT,
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
  });
}

function awaitImportChildProcess() {
  return childProcessModule;
}

import * as childProcessModule from 'node:child_process';
