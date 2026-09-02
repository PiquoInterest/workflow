#!/usr/bin/env node

import {
  existsSync,
  readFileSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const RESTORE_COMMIT = '653047900576f427e301209105519ff0f555a5e0';
const BRANCH = 'pro/rust-repository-port';
const COMPATIBILITY = 'packages/ai/src/get-error-message.test.ts';
const SECURITY = 'packages/ai/src/get-error-message-security.test.ts';
const EVIDENCE =
  ' GREEN verified by the permanent read-only AI lane in workflow run ' +
  '33584024158 at 31426849bbd4e67771a7bc3fa6c2e915ae963f06. ' +
  'That run passed the TypeScript oracles, production and translated Rust ' +
  'tests, Rustfmt, and Clippy with warnings denied.';

if (process.env.GITHUB_ACTIONS !== 'true') {
  throw new Error('the one-shot promotion helper may run only in GitHub Actions');
}

process.chdir(ROOT);

function run(command, args, options = {}) {
  return execFileSync(command, args, {
    cwd: ROOT,
    encoding: 'utf8',
    stdio: options.capture ? 'pipe' : 'inherit',
    ...options,
  });
}

function readJson(relativePath) {
  return JSON.parse(readFileSync(path.join(ROOT, relativePath), 'utf8'));
}

function writeJson(relativePath, value) {
  writeFileSync(
    path.join(ROOT, relativePath),
    `${JSON.stringify(value, null, 2)}\n`,
    'utf8'
  );
}

function selectExactlyOne(entries, typescript, expectedStatus) {
  const matches = entries.filter((entry) => entry.typescript === typescript);
  if (matches.length !== 1 || matches[0].status !== expectedStatus) {
    throw new Error(
      `${typescript} must have exactly one ${expectedStatus} override before promotion`
    );
  }
  return matches[0];
}

function promoteOverride(relativePath, typescript) {
  const overrides = readJson(relativePath);
  const entry = selectExactlyOne(overrides.entries, typescript, 'red');
  entry.status = 'green';
  if (!entry.notes.endsWith('.')) {
    entry.notes += '.';
  }
  entry.notes += EVIDENCE;
  writeJson(relativePath, overrides);
}

function removeExpectedRedCase() {
  const relativePath = 'rust/tdd-red.json';
  const red = readJson(relativePath);
  const selected = red.cases.filter(
    (entry) => entry.typescript === COMPATIBILITY
  );
  if (selected.length !== 1) {
    throw new Error('compatibility expected-RED case is missing or duplicated');
  }
  red.cases = red.cases.filter(
    (entry) => entry.typescript !== COMPATIBILITY
  );
  writeJson(relativePath, red);

  const fragmentPath = path.join(
    ROOT,
    'rust/tdd-red.d/ai-get-error-message-security.json'
  );
  const fragment = JSON.parse(readFileSync(fragmentPath, 'utf8'));
  if (
    fragment.cases.length !== 1 ||
    fragment.cases[0].typescript !== SECURITY
  ) {
    throw new Error('security expected-RED fragment changed unexpectedly');
  }
  unlinkSync(fragmentPath);
}

function collapseRedShim() {
  const relativePath = 'rust/tdd/workflow-ai/src/lib.rs';
  const filePath = path.join(ROOT, relativePath);
  let source = readFileSync(filePath, 'utf8');
  const imports =
    'use std::marker::PhantomData;\n' +
    'use std::sync::atomic::{AtomicUsize, Ordering};\n' +
    'use std::sync::{Arc, Weak};\n';
  if (!source.includes(imports)) {
    throw new Error('AI RED-shim imports changed unexpectedly');
  }
  source = source.replace(imports, 'use std::marker::PhantomData;\n');

  const startMarker = '#[derive(Debug, Clone)]\npub struct CallableProbe';
  const endMarker =
    '#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum IteratorCase';
  const start = source.indexOf(startMarker);
  const end = source.indexOf(endMarker);
  if (start < 0 || end <= start) {
    throw new Error('AI RED-shim hostile-value block changed unexpectedly');
  }
  source =
    source.slice(0, start) +
    'pub use workflow_ai::{\n' +
    '    CallableProbe, ErrorValue, SharedErrorObject, get_error_message,\n' +
    '};\n\n' +
    source.slice(end);
  writeFileSync(filePath, source, 'utf8');
}

function appendSecurityLedger() {
  const relativePath = 'security.txt';
  const filePath = path.join(ROOT, relativePath);
  let source = readFileSync(filePath, 'utf8');
  if (source.includes('WF-RUST-103')) {
    throw new Error('WF-RUST-103 is already present in security.txt');
  }
  source = `${source.trimEnd()}\n\n#\n# WF-RUST-103: TypeScript AI error normalization delegates arbitrary\n# non-nullish values to JSON.stringify. That executes an object-controlled\n# toJSON hook and throws for cyclic objects and BigInts while another failure is\n# already being handled. The TypeScript characterization keeps all three legacy\n# behaviors reproducible. Rust preserves ordinary JSON compatibility but treats\n# callable fields as inert data, tracks shared-object identity, bounds recursive\n# traversal, canonicalizes BigInts, and returns fixed placeholders for cycles,\n# released references, excessive depth, and invalid BigInt text. Expected-RED\n# run 33583543050 and permanent GREEN run 33584024158 cover the TypeScript\n# oracles, translated Rust suites, direct production regressions, Rustfmt, and\n# Clippy. Full analysis is in docs/rust-port/findings/WF-RUST-103.md.\n`;
  writeFileSync(filePath, source, 'utf8');
}

function appendFindingsLedger() {
  const relativePath =
    'docs/rust-port/TYPESCRIPT_LOGIC_AND_SECURITY_FIXES.md';
  const filePath = path.join(ROOT, relativePath);
  let source = readFileSync(filePath, 'utf8');
  if (source.includes('## WF-RUST-103:')) {
    throw new Error('WF-RUST-103 is already present in the findings ledger');
  }
  source = `${source.trimEnd()}\n\n## WF-RUST-103: Error normalization executed hooks and could mask failures\n\n**Status:** Closed at the production Rust AI boundary; TypeScript remains the\nlegacy compatibility and security oracle.\n\n**Affected code:** \`packages/ai/src/get-error-message.ts\` and the Rust\n\`workflow-ai\` error-normalization contract.\n\n**Old behavior:** Every non-nullish, non-string, non-Error value was passed to\n\`JSON.stringify\`. An object-supplied \`toJSON\` callback executed during failure\nhandling, while cyclic values and BigInts threw a second \`TypeError\`.\n\n**Impact:** Diagnostic normalization could perform re-entrant side effects,\nreplace the original diagnostic, or mask it with a secondary availability\nfailure. This does not claim code injection: the callback is already executable\ninside the process, but normalization invoked it implicitly.\n\n**Fix:** Rust uses an inert typed value model, cycle detection, a depth bound,\ncanonical BigInt diagnostics, and fixed non-reflective placeholders. Ordinary\nobjects, arrays, primitives, escaping, and nested \`undefined\` retain the\nTypeScript compatibility contract.\n\n**Regression evidence:**\n\n- \`packages/ai/src/get-error-message.test.ts\`\n- \`packages/ai/src/get-error-message-security.test.ts\`\n- \`rust/tdd/workflow-ai/tests/get_error_message.rs\`\n- \`rust/tdd/workflow-ai/tests/get_error_message_security.rs\`\n- \`crates/workflow-ai/tests/get_error_message_security.rs\`\n- expected-RED workflow run \`33583543050\`\n- permanent GREEN workflow run \`33584024158\`\n- \`docs/rust-port/findings/WF-RUST-103.md\`\n`;
  writeFileSync(filePath, source, 'utf8');
}

function regenerateAndValidateManifest() {
  run('node', [
    'scripts/rust-port-test-inventory.mjs',
    '--write-json',
    '/tmp/rust-port-test-inventory.json',
  ]);
  run('node', [
    'scripts/bootstrap-rust-test-port-manifest.mjs',
    '/tmp/rust-port-test-inventory.json',
    'rust/test-port-manifest.json',
  ]);
  run('node', [
    'scripts/rust-port-test-inventory.mjs',
    '--check',
    'rust/test-port-manifest.json',
  ]);

  const manifest = readJson('rust/test-port-manifest.json');
  const expected = {
    unported: 223,
    partial: 0,
    red: 80,
    green: 39,
    blocked: 0,
  };
  if (JSON.stringify(manifest.statusCounts) !== JSON.stringify(expected)) {
    throw new Error(
      `unexpected promoted counts: ${JSON.stringify(manifest.statusCounts)}`
    );
  }
}

function runPromotedTests() {
  run('cargo', [
    'fmt',
    '--manifest-path',
    'rust/tdd/workflow-ai/Cargo.toml',
  ]);
  run('cargo', [
    'test',
    '--locked',
    '--manifest-path',
    'crates/workflow-ai/Cargo.toml',
    '--all-targets',
  ]);
  run('cargo', [
    'test',
    '--locked',
    '--manifest-path',
    'rust/tdd/workflow-ai/Cargo.toml',
    '--test',
    'get_error_message',
  ]);
  run('cargo', [
    'test',
    '--locked',
    '--manifest-path',
    'rust/tdd/workflow-ai/Cargo.toml',
    '--test',
    'get_error_message_security',
  ]);
  run('cargo', [
    'clippy',
    '--locked',
    '--manifest-path',
    'crates/workflow-ai/Cargo.toml',
    '--all-targets',
    '--',
    '-D',
    'warnings',
  ]);
  run('cargo', [
    'clippy',
    '--locked',
    '--manifest-path',
    'rust/tdd/workflow-ai/Cargo.toml',
    '--test',
    'get_error_message',
    '--test',
    'get_error_message_security',
    '--',
    '-D',
    'warnings',
  ]);
}

function restoreTemporaryFiles() {
  const restore = (relativePath) => {
    const content = run(
      'git',
      ['show', `${RESTORE_COMMIT}:${relativePath}`],
      { capture: true }
    );
    writeFileSync(path.join(ROOT, relativePath), content, 'utf8');
  };
  restore('scripts/apply-rust-test-port-overrides.mjs');
  restore('.github/workflows/rust-test-manifest.yml');

  for (const relativePath of [
    '.github/workflows/rust-ai-error-normalization-promote-once.yml',
    '.github/workflows/rust-source-snapshot-once.yml',
    'scripts/promote-ai-error-normalization-once.mjs',
  ]) {
    const filePath = path.join(ROOT, relativePath);
    if (existsSync(filePath)) {
      unlinkSync(filePath);
    }
  }
}

function commitAndPush() {
  run('git', ['diff', '--check']);
  run('git', ['fetch', 'origin', BRANCH]);
  const localHead = run('git', ['rev-parse', 'HEAD'], { capture: true }).trim();
  const remoteHead = run(
    'git',
    ['rev-parse', `origin/${BRANCH}`],
    { capture: true }
  ).trim();
  if (localHead !== remoteHead) {
    throw new Error(
      `branch head changed during promotion: local ${localHead}, remote ${remoteHead}`
    );
  }

  restoreTemporaryFiles();
  run('git', ['diff', '--check']);
  run('git', ['config', 'user.name', 'github-actions[bot]']);
  run('git', [
    'config',
    'user.email',
    '41898282+github-actions[bot]@users.noreply.github.com',
  ]);
  run('git', ['add', '-A']);
  const staged = run('git', ['diff', '--cached', '--name-only'], {
    capture: true,
  }).trim();
  if (!staged) {
    throw new Error('promotion produced no staged changes');
  }
  run('git', [
    'commit',
    '--signoff',
    '-m',
    'test(rust-ai): promote error normalization to GREEN',
  ]);
  run('git', ['push', 'origin', `HEAD:${BRANCH}`]);
}

promoteOverride('rust/test-port-overrides.json', COMPATIBILITY);
promoteOverride(
  'rust/test-port-overrides.d/ai-get-error-message-security.json',
  SECURITY
);
removeExpectedRedCase();
collapseRedShim();
appendSecurityLedger();
appendFindingsLedger();
regenerateAndValidateManifest();
runPromotedTests();
commitAndPush();
