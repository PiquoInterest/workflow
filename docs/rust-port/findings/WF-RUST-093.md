# WF-RUST-093: Local build subprocesses need hard lifetime and output bounds

**Status:** TypeScript characterization retained; Rust regression suite is TDD RED.

## Affected surface

- `packages/core/e2e/local-build.test.ts`
- the workbench build matrix
- Rust replacement process runner and artifact validator

## Original behavior

The TypeScript helper starts a child with `spawn()` and continuously appends both
stdout and stderr to three JavaScript strings. It has no helper-level timeout,
process-group termination, or output cap. Vitest's 180-second test timeout can
reject the test while a descendant build process remains alive, and a noisy or
wedged builder can consume memory for as long as it keeps writing.

The helper that reads optional generated files also catches every error as
`null`. The current ESM assertion eventually fails for a missing value, but the
original I/O cause and distinction between absence and unreadability are lost.

## Security and correctness impact

A compromised dependency or malfunctioning build tool can outlive the test,
interfere with later workbenches, retain fixture files, and exhaust runner memory
through unbounded output. Detached descendants also make teardown behavior and
build reproducibility depend on process timing rather than the test contract.

## Required Rust invariant

Every preflight and build command must:

1. use direct argv execution rather than a shell;
2. run in a process group or equivalent job object;
3. have a hard 180-second deadline;
4. terminate and reap the complete process tree on timeout or cancellation;
5. cap accepted stdout and stderr bytes under one shared 8 MiB budget while
   retaining their arrival order for diagnostics;
6. run fixture cleanup on every exit unless CI deliberately preserves inputs
   required by the just-built artifact;
7. treat non-not-found artifact read failures as errors.

## Regression evidence

`rust/tdd/workflow-core/tests/local_build.rs` expands the parameterized source
case across all 13 workbench projects and adds explicit process, output,
artifact, diagnostics, ESM, legacy-route, and cleanup contracts.

## Closure condition

The finding closes only when the real Rust child-process runner, process-tree
cancellation path, bounded output collector, builder invocation, artifact
reader, and cleanup guard pass the translated tests. A precomputed build
observation is not sufficient.
