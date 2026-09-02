#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const TEST_FILE_PATTERN = /\.(?:test|spec)\.(?:[cm]?tsx?)$/;
const STATUS_VALUES = new Set([
  'unported',
  'partial',
  'red',
  'green',
  'blocked',
]);

function fail(message) {
  console.error(`[rust-test-inventory] ${message}`);
  process.exitCode = 1;
}

function parseArgs(argv) {
  const options = {
    check: undefined,
    writeJson: undefined,
    writeMarkdown: undefined,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--check') {
      options.check = argv[++index];
    } else if (argument === '--write-json') {
      options.writeJson = argv[++index];
    } else if (argument === '--write-markdown') {
      options.writeMarkdown = argv[++index];
    } else if (argument === '--help' || argument === '-h') {
      console.log(`Usage: node scripts/rust-port-test-inventory.mjs [options]\n\nOptions:\n  --check <path>           Validate a checked-in port manifest\n  --write-json <path>      Write the current TypeScript test inventory\n  --write-markdown <path>  Write a human-readable inventory report\n  --help                   Show this message`);
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${argument}`);
    }

    if (
      (argument === '--check' ||
        argument === '--write-json' ||
        argument === '--write-markdown') &&
      !argv[index]
    ) {
      throw new Error(`${argument} requires a path`);
    }
  }

  return options;
}

function trackedFiles() {
  return execFileSync('git', ['ls-files', '-z'], {
    cwd: ROOT,
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  })
    .split('\0')
    .filter(Boolean)
    .sort((left, right) => left.localeCompare(right));
}

function sha256(buffer) {
  return createHash('sha256').update(buffer).digest('hex');
}

function surfaceFor(sourcePath) {
  const segments = sourcePath.split('/');
  if (segments[0] === 'packages' && segments[1]) {
    return `packages/${segments[1]}`;
  }
  if (segments[0] === 'workbench' && segments[1]) {
    return `workbench/${segments[1]}`;
  }
  if (segments[0] === 'docs') return 'docs';
  if (segments[0] === '.github') return '.github';
  return 'root';
}

function kindFor(sourcePath) {
  if (sourcePath.includes('/e2e/') || /(?:^|\/)e2e(?:\.|-)/.test(sourcePath)) {
    return 'e2e';
  }
  if (sourcePath.startsWith('workbench/')) return 'workbench';
  if (sourcePath.startsWith('.github/')) return 'ci';
  if (sourcePath.startsWith('docs/')) return 'docs';
  return 'unit';
}

function countTestDeclarations(source) {
  const matches = source.match(
    /\b(?:it|test)(?:\.(?:each|skip|only|todo|fails))?\s*\(/g
  );
  return matches?.length ?? 0;
}

function currentCommit() {
  return execFileSync('git', ['rev-parse', 'HEAD'], {
    cwd: ROOT,
    encoding: 'utf8',
  }).trim();
}

function buildInventory() {
  const entries = trackedFiles()
    .filter((sourcePath) => TEST_FILE_PATTERN.test(sourcePath))
    .map((sourcePath) => {
      const absolutePath = path.join(ROOT, sourcePath);
      const bytes = readFileSync(absolutePath);
      const source = bytes.toString('utf8');
      return {
        typescript: sourcePath,
        sourceSha256: sha256(bytes),
        surface: surfaceFor(sourcePath),
        kind: kindFor(sourcePath),
        lineCount: source.split(/\r?\n/).length,
        declaredTestCount: countTestDeclarations(source),
      };
    });

  return {
    schemaVersion: 1,
    sourceCommit: currentCommit(),
    sourceCount: entries.length,
    declaredTestCount: entries.reduce(
      (total, entry) => total + entry.declaredTestCount,
      0
    ),
    entries,
  };
}

function writeOutput(outputPath, contents) {
  const absolutePath = path.resolve(ROOT, outputPath);
  mkdirSync(path.dirname(absolutePath), { recursive: true });
  writeFileSync(absolutePath, contents);
}

function markdownReport(inventory) {
  const groups = new Map();
  for (const entry of inventory.entries) {
    const current = groups.get(entry.surface) ?? {
      files: 0,
      tests: 0,
      kinds: new Set(),
    };
    current.files += 1;
    current.tests += entry.declaredTestCount;
    current.kinds.add(entry.kind);
    groups.set(entry.surface, current);
  }

  const rows = [...groups.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(
      ([surface, group]) =>
        `| \`${surface}\` | ${group.files} | ${group.tests} | ${[...group.kinds].sort().join(', ')} |`
    );

  return `# TypeScript test inventory\n\nGenerated from commit \`${inventory.sourceCommit}\`.\n\nThis report counts tracked TypeScript test files before production Rust code is\nimplemented. The declaration count is a conservative source-level count of\n\`it(...)\` and \`test(...)\` calls; parameterized tests may execute more cases.\n\n- Test files: **${inventory.sourceCount}**\n- Declared tests: **${inventory.declaredTestCount}**\n\n| Surface | Test files | Declared tests | Kinds |\n| --- | ---: | ---: | --- |\n${rows.join('\n')}\n`;
}

function validateManifest(inventory, manifestPath) {
  const absolutePath = path.resolve(ROOT, manifestPath);
  if (!existsSync(absolutePath)) {
    fail(`manifest does not exist: ${manifestPath}`);
    return;
  }

  const manifest = JSON.parse(readFileSync(absolutePath, 'utf8'));
  if (manifest.schemaVersion !== 1 || !Array.isArray(manifest.entries)) {
    fail('manifest must use schemaVersion 1 and contain an entries array');
    return;
  }

  const actualByPath = new Map(
    inventory.entries.map((entry) => [entry.typescript, entry])
  );
  const seen = new Set();

  for (const entry of manifest.entries) {
    if (!entry || typeof entry.typescript !== 'string') {
      fail('every manifest entry needs a TypeScript path');
      continue;
    }
    if (seen.has(entry.typescript)) {
      fail(`duplicate manifest entry: ${entry.typescript}`);
      continue;
    }
    seen.add(entry.typescript);

    const actual = actualByPath.get(entry.typescript);
    if (!actual) {
      fail(`manifest references a missing TypeScript test: ${entry.typescript}`);
      continue;
    }
    if (entry.sourceSha256 !== actual.sourceSha256) {
      fail(`source hash changed without a manifest review: ${entry.typescript}`);
    }
    if (!STATUS_VALUES.has(entry.status)) {
      fail(`invalid status for ${entry.typescript}: ${String(entry.status)}`);
    }
    if (!Array.isArray(entry.rustTests)) {
      fail(`rustTests must be an array for ${entry.typescript}`);
      continue;
    }
    if (entry.status !== 'unported' && entry.status !== 'blocked') {
      if (entry.rustTests.length === 0) {
        fail(`${entry.typescript} is ${entry.status} but has no Rust test target`);
      }
      for (const rustTest of entry.rustTests) {
        if (typeof rustTest !== 'string' || !existsSync(path.join(ROOT, rustTest))) {
          fail(`missing Rust test target for ${entry.typescript}: ${String(rustTest)}`);
        }
      }
    }
  }

  for (const actual of inventory.entries) {
    if (!seen.has(actual.typescript)) {
      fail(`untracked TypeScript test file: ${actual.typescript}`);
    }
  }

  if (!process.exitCode) {
    const counts = Object.fromEntries(
      [...STATUS_VALUES].map((status) => [
        status,
        manifest.entries.filter((entry) => entry.status === status).length,
      ])
    );
    console.log(
      `[rust-test-inventory] ${inventory.sourceCount} TypeScript test files tracked: ${JSON.stringify(counts)}`
    );
  }
}

const options = parseArgs(process.argv.slice(2));
const inventory = buildInventory();

if (options.writeJson) {
  writeOutput(options.writeJson, `${JSON.stringify(inventory, null, 2)}\n`);
}
if (options.writeMarkdown) {
  writeOutput(options.writeMarkdown, markdownReport(inventory));
}
if (options.check) {
  validateManifest(inventory, options.check);
}
if (!options.writeJson && !options.writeMarkdown && !options.check) {
  process.stdout.write(`${JSON.stringify(inventory, null, 2)}\n`);
}
