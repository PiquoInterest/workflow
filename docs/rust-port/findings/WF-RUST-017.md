# WF-RUST-017: DurableAgent tool approval bypass

**Status:** Open, TDD RED. The issue is characterized in TypeScript and the safe
Rust target is specified, but no production Rust agent loop exists yet.

## Affected behavior

`packages/core/e2e/e2e-agent.test.ts` documents that a tool configured with
`needsApproval` executes immediately. Its current result contains two model
steps because both the tool call and tool result are present. The broader
compatibility evidence is in `rust/tdd/workflow-ai/tests/durable_agent_compat.rs`.

## Security impact

Tool approval is an authorization boundary. Ignoring it can execute an
externally visible or destructive tool action before the user, policy engine,
or application grants permission. Retrying or replaying the workflow can also
repeat that unapproved side effect unless the decision and execution are
persisted and deduplicated.

## Rust security target

The Rust implementation must:

1. validate the tool input before evaluating a static or dynamic approval rule;
2. persist a durable approval-request state before any local tool execution;
3. expose one pending call and zero tool results while approval is absent;
4. resume only after an authenticated decision associated with the exact run,
   step, and tool-call ID;
5. make duplicate, replayed, late, or conflicting decisions idempotent; and
6. guarantee that rejection never invokes the tool implementation.

## Regression evidence

- TypeScript characterization:
  `packages/core/e2e/e2e-agent.test.ts`, `tool approval (GAP)`.
- Rust compatibility and callback tests:
  `rust/tdd/workflow-ai/tests/durable_agent_compat.rs`.
- Full-runtime Rust TDD translation:
  `rust/tdd/workflow-core/tests/e2e_agent/gaps.rs`.

The Rust test suite keeps the TypeScript bypass as an explicit oracle case and
adds a separate production target requiring `pending = true`, one tool call,
zero tool results, and `tool_executed = false`. The production Rust runtime must
satisfy the latter. It must not turn the insecure compatibility observation
into normal Rust behavior.

## Closure condition

This finding closes only when the real Rust DurableAgent state machine, storage
boundary, resume path, and replay logic pass the secure target and an end-to-end
test proves that no tool side effect occurs before approval.
