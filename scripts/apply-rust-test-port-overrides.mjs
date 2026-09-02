#!/usr/bin/env node

import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const [manifestArgument, overridesArgument] = process.argv.slice(2);
const STATUS_VALUES = new Set([
  'unported',
  'partial',
  'red',
  'green',
  'blocked',
]);

if (!manifestArgument || !overridesArgument) {
  throw new Error(
    'Usage: node scripts/apply-rust-test-port-overrides.mjs <manifest.json> <overrides.json>'
  );
}

const manifestPath = path.resolve(ROOT, manifestArgument);
const overridesPath = path.resolve(ROOT, overridesArgument);
const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
const overrides = JSON.parse(readFileSync(overridesPath, 'utf8'));

if (overrides.schemaVersion !== 1 || !Array.isArray(overrides.entries)) {
  throw new Error('test-port overrides must use schemaVersion 1 and contain entries');
}

const manifestByPath = new Map(
  manifest.entries.map((entry) => [entry.typescript, entry])
);
const seen = new Set();

for (const override of overrides.entries) {
  if (!override || typeof override.typescript !== 'string') {
    throw new Error('every override needs a TypeScript path');
  }
  if (seen.has(override.typescript)) {
    throw new Error(`duplicate override: ${override.typescript}`);
  }
  seen.add(override.typescript);

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

manifest.statusCounts = Object.fromEntries(
  [...STATUS_VALUES].map((status) => [
    status,
    manifest.entries.filter((entry) => entry.status === status).length,
  ])
);
manifest.sourceCount = manifest.entries.length;
manifest.declaredTestCount = manifest.entries.reduce(
  (total, entry) => total + entry.declaredTestCount,
  0
);

writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
