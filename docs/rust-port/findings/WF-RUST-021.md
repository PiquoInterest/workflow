# WF-RUST-021: Authenticated encryption and typed crypto failures

**Status:** TypeScript behavior is covered; Rust implementation is TDD RED.

## Security boundary

Workflow payload encryption uses a 32-byte AES key and an authenticated GCM
envelope containing a 12-byte nonce and 16-byte authentication tag. Truncated,
tampered, or wrong-key ciphertext must never produce plaintext. Raw Web Crypto
`OperationError` or `InvalidAccessError` values must not escape the runtime
boundary because generic error classification can misattribute them as user
errors.

## Required Rust invariant

The Rust implementation must use a maintained authenticated-encryption
provider, generate a fresh cryptographically secure nonce for every encryption,
require exactly 32 key bytes, authenticate before exposing plaintext, and wrap
all provider failures in the typed runtime crypto error. Context may include the
operation and total byte length but must not mislabel nonce bytes as an outer
serialization format prefix.

## Regression evidence

- TypeScript source: `packages/core/src/encryption.test.ts`.
- Rust expected-RED translation: `rust/tdd/workflow-core/tests/encryption.rs`.

The translated tests cover round trip length, short keys, truncated envelopes,
a flipped authentication-tag byte, a different reader key, bogus minimum-size
ciphertext, and a decrypt-only key used for encryption.

## Closure condition

This finding closes only when the production Rust serialization path uses the
verified crypto implementation, the typed errors feed run classification, and
cross-language encrypted fixtures prove TypeScript and Rust can read each
other's envelopes without nonce reuse.
