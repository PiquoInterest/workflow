#!/usr/bin/env node

import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const [inventoryArgument, manifestArgument] = process.argv.slice(2);
const STATUS_VALUES = new Set([
  'unported',
  'partial',
  'red',
  'green',
  'blocked',
]);

if (!inventoryArgument || !manifestArgument) {
  throw new Error(
    'Usage: node scripts/bootstrap-rust-test-port-manifest.mjs <inventory.json> <manifest.json>'
  );
}

const inventoryPath = path.resolve(ROOT, inventoryArgument);
const manifestPath = path.resolve(ROOT, manifestArgument);
const inventory = JSON.parse(readFileSync(inventoryPath, 'utf8'));

const existing = existsSync(manifestPath)
  ? JSON.parse(readFileSync(manifestPath, 'utf8'))
  : { schemaVersion: 1, entries: [] };
const previousByPath = new Map(
  existing.entries.map((entry) => [entry.typescript, entry])
);

const entries = inventory.entries.map((source) => {
  const previous = previousByPath.get(source.typescript);
  return {
    typescript: source.typescript,
    sourceSha256: source.sourceSha256,
    surface: source.surface,
    kind: source.kind,
    lineCount: source.lineCount,
    declaredTestCount: source.declaredTestCount,
    status: previous?.status ?? 'unported',
    rustTests: previous?.rustTests ?? [],
    notes: previous?.notes ?? '',
  };
});

const knownPaths = new Set(entries.map((entry) => entry.typescript));
const removed = existing.entries
  .filter((entry) => !knownPaths.has(entry.typescript))
  .map((entry) => entry.typescript);
if (removed.length > 0) {
  throw new Error(
    `Refusing to silently drop removed TypeScript tests from the manifest: ${removed.join(', ')}`
  );
}

const manifestByPath = new Map(
  entries.map((entry) => [entry.typescript, entry])
);
const seenOverrides = new Set();
for (const override of loadOverrideEntries()) {
  if (!override || typeof override.typescript !== 'string') {
    throw new Error('every test-port override needs a TypeScript path');
  }
  if (seenOverrides.has(override.typescript)) {
    throw new Error(`duplicate test-port override: ${override.typescript}`);
  }
  seenOverrides.add(override.typescript);

  const entry = manifestByPath.get(override.typescript);
  if (!entry) {
    throw new Error(`override references an unknown test: ${override.typescript}`);
  }
  if (!STATUS_VALUES.has(override.status)) {
    throw new Error(
      `invalid override status for ${override.typescript}: ${String(override.status)}`
    );
  }
  if (!Array.isArray(override.rustTests)) {
    throw new Error(`rustTests must be an array for ${override.typescript}`);
  }
  for (const rustTest of override.rustTests) {
    if (typeof rustTest !== 'string' || !existsSync(path.join(ROOT, rustTest))) {
      throw new Error(
        `missing Rust test target for ${override.typescript}: ${String(rustTest)}`
      );
    }
  }

  entry.status = override.status;
  entry.rustTests = override.rustTests;
  entry.notes = override.notes ?? '';
}

const statusCounts = Object.fromEntries(
  [...STATUS_VALUES].map((status) => [
    status,
    entries.filter((entry) => entry.status === status).length,
  ])
);

const manifest = {
  schemaVersion: 1,
  sourceCount: entries.length,
  declaredTestCount: entries.reduce(
    (total, entry) => total + entry.declaredTestCount,
    0
  ),
  statusCounts,
  entries,
};

mkdirSync(path.dirname(manifestPath), { recursive: true });
writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);

function loadOverrideEntries() {
  const primaryPath = path.join(ROOT, 'rust/test-port-overrides.json');
  const fragmentDirectory = path.join(ROOT, 'rust/test-port-overrides.d');
  const overridePaths = [primaryPath];

  if (existsSync(fragmentDirectory)) {
    if (!statSync(fragmentDirectory).isDirectory()) {
      throw new Error(`${fragmentDirectory} must be a directory`);
    }
    const fragments = readdirSync(fragmentDirectory, { withFileTypes: true })
      .filter((entry) => entry.isFile() && entry.name.endsWith('.json'))
      .map((entry) => entry.name)
      .sort((left, right) => left.localeCompare(right));
    overridePaths.push(
      ...fragments.map((name) => path.join(fragmentDirectory, name))
    );
  }

  return overridePaths.flatMap((overridePath) => {
    const overrides = JSON.parse(readFileSync(overridePath, 'utf8'));
    if (overrides.schemaVersion !== 1 || !Array.isArray(overrides.entries)) {
      throw new Error(
        `${path.relative(ROOT, overridePath)} must use schemaVersion 1 and contain entries`
      );
    }
    return overrides.entries;
  });
}
