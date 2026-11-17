# ADR 0004: No input makes hallmark panic

- Status: Accepted
- Date: 2026-11-12
- Affects: all library code in `crates/hallmark` and `crates/hallmark-wasm`, the workspace lints, `clippy.toml`, and the fuzz targets

## Context

hallmark reads messages, DNS answers, and DNS records written by strangers, some of them hostile. A panic in a milter process can take down mail handling for everyone. In WebAssembly built for `wasm32-unknown-unknown`, a panic aborts the module, and the page has to load it again. Rust's common ways to panic hide in ordinary code: `unwrap`, `expect`, slice indexing, string slicing at a byte that is not a character boundary, and `unreachable!`.

## Options considered

- (a) Catch panics at the API boundary with `catch_unwind`.
- (b) Review alone.
- (c) Deny the panicking constructs in library code with Clippy, and fuzz every parser.

## Decision

Option (c).

- The workspace enables the Clippy lints `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `string_slice`, `unreachable`, `todo`, and `unimplemented`, and CI turns them into errors with `-D warnings`.
- `clippy.toml` allows `unwrap`, `expect`, `panic!`, and indexing in tests, where a panic is a failed assertion.
- Library code reads input with `get`, `split_at_checked`, `strip_prefix`, and slice patterns, and does arithmetic on lengths from the input with checked operations. An overflow or an out-of-range value becomes the protocol's error result.
- Every parser has a fuzz target (SAF-003). CI runs them for a bounded time on every change and for longer on a schedule.

## Consequences

Positive:
- Clippy rejects the common panics before review.
- Fuzzing covers the panics that lints can't see, such as an overflow in unchecked arithmetic or a loop that never ends.

Negative:
- Code that reads bytes is longer than code that indexes.
- Unchecked arithmetic (`a + b` on lengths) is not linted, because `arithmetic_side_effects` would flag every addition. Review and fuzzing cover it.
- `catch_unwind` stays unused: it would not help in WebAssembly, where panics abort.

## References

- [requirements.md: Hostile input](../reference/requirements.md#hostile-input)
- [procedural-rust skill](../../.claude/skills/procedural-rust/SKILL.md)
- [SECURITY.md](../../SECURITY.md), which counts a panic on any input as a vulnerability
