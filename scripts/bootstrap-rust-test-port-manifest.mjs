#!/usr/bin/env node

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const [inventoryArgument, manifestArgument] = process.argv.slice(2);

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

const statusCounts = Object.fromEntries(
  ['unported', 'partial', 'red', 'green', 'blocked'].map((status) => [
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
