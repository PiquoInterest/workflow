import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '..',
  '..'
);

function run(command, args, options = {}) {
  const rendered = [command, ...args].join(' ');
  process.stdout.write(`\n[rust-port] ${rendered}\n`);
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    env: { ...process.env, ...options.env },
    encoding: 'utf8',
    stdio: 'inherit',
    shell: false,
  });

  if (result.error) {
    console.error(`[rust-port] failed to execute ${command}:`, result.error);
    process.exit(1);
  }
  if (result.signal) {
    console.error(
      `[rust-port] ${rendered} terminated by signal ${result.signal}`
    );
    process.exit(1);
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

// Limit rustfmt to the migration crate. Formatting the whole workspace would
// also rewrite the pre-existing SWC crates according to the runner's newer
// rustfmt release, making this gate depend on unrelated source formatting.
run('cargo', [
  'fmt',
  '--package',
  'workflow-world',
  '--',
  '--check',
]);
run('cargo', ['test', '-p', 'workflow-world', '--all-targets']);
run('cargo', [
  'clippy',
  '-p',
  'workflow-world',
  '--all-targets',
  '--',
  '-D',
  'warnings',
]);
run('cargo', [
  'build',
  '-p',
  'workflow-world',
  '--example',
  'conformance',
]);

const executable = path.join(
  repoRoot,
  'target',
  'debug',
  'examples',
  process.platform === 'win32' ? 'conformance.exe' : 'conformance'
);

run(
  process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm',
  ['exec', 'vitest', 'run', 'rust/conformance/world-parity.test.ts'],
  {
    env: {
      WORKFLOW_RUST_CONFORMANCE_BIN: executable,
    },
  }
);
