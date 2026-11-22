# ADR 0064: Rust code is procedural, and lints, types and size limits keep it efficient and reviewable

- Status: Accepted
- Date: 2026-11-22
- Supersedes: [ADR 0003](0003-rust-code-is-written-in-a-procedural-style.md)
- Affects: all Rust code in the repository, the workspace lints, `clippy.toml`, and CI

## Context

Rust code can branch and loop in two styles. One uses `match`, `if let`, and `for`. The other chains `Option` and `Result` combinators and iterator adapters, each with a closure: `map_err(|error| ...)`, `ok_or_else(|| ...)`, `iter().filter(|x| ...).any(|y| ...)`. A chain hides its branches and early returns inside closure arguments, and the maintainer finds such code hard to read and review. Clippy's default lints push toward the second style: `manual_map`, `manual_find`, `map_entry`, and similar lints turn an explicit `match` or loop into a combinator with a closure.

Code passed around as a value hides which code runs in the same way. A closure stored as `Box<dyn Fn>`, or a function pointer in a table, puts the code behind a field or an index.

hallmark's parsers and canonicalizers read hostile input, and reviewers need to see every branch.

Code also goes wrong in ways that compile and pass the tests:

- It allocates where it needn't: a `collect` whose result is only iterated, a `clone` of a value that is dropped next, an argument taken by value and only read. A `clone` that only gets past the borrow checker looks like any other `clone`.
- It searches a list inside a loop over the same input, which takes time quadratic in a count that the sender controls.
- It assumes that a value is present. [ADR 0004](0004-no-input-makes-hallmark-panic.md) rejects the forms that panic, such as `unwrap` and indexing. Code that can't panic can still hide the missing value behind `unwrap_or_default()`, a fallback value or an empty `else`, and give a result that no specification allows.
- It follows the common case of a protocol instead of its RFC. The tests come from the RFC text (TST-001, TST-003), and fuzzing covers what the tests miss (SAF-003).
- It grows past what a reviewer can hold: long functions and deep nesting.

Code that a person or a coding assistant writes quickly shows these faults, and review misses some of them. Clippy and CI check every change the same way.

## Options considered

- (a) No rule. Reviewers judge each change.
- (b) Written conventions, checked only in review.
- (c) The procedural style and the rules below: Clippy and CI check what they can, types make the assumptions unnecessary, and review covers the rest.

## Decision

Option (c).

### Procedural style

- Code branches with `match`, `if let`, and `let ... else`, returns errors with `?`, and repeats with `for`, `while`, and `loop`.
- Code calls no method that takes a function as an argument: the `Option` and `Result` combinators, `bool::then`, the `Iterator` adapters and consumers that take a closure, `iter::from_fn`, `repeat_with`, and `successors`, and the `retain`, `dedup_by`, and `Entry` methods of the collections. `clippy.toml` lists them under `disallowed-methods`. Methods that take no function stay allowed, alone or chained, except those that the rules on assumptions below disallow.
- Code writes no closures. No external API that hallmark uses requires one yet. When one does, a new ADR names it as the exception.
- Code writes no bound on `Fn`, `FnMut`, `FnOnce`, or the `AsyncFn` traits, stores no closure as `Box<dyn Fn>`, and uses no function pointer type. A table of code becomes an enum with a `match`.
- The workspace allows the Clippy lints whose fix uses a method with a closure: `manual_filter`, `manual_filter_map`, `manual_find`, `manual_find_map`, `manual_map`, `manual_retain`, and `map_entry`.

### Efficiency

- `clippy::all` brings the `perf` group. The workspace also enables `needless_collect` and `redundant_clone` from the nursery group, and `needless_pass_by_value` and `implicit_clone` from the pedantic group.
- Code doesn't clone a value to get past the borrow checker. It borrows, splits the borrow, or moves the value, and review checks each `clone`.
- Inside a loop over input, code doesn't search a collection that grows with the input, such as with `Vec::contains` or a nested `for` over the same items. It keeps a map, a set, an index or sorted data. The limits of SAF-002 bound every count, and review checks the loops.
- Each hot path, such as body canonicalization and the header section splitter, gets an allocation-limit test and a benchmark when it is written. The change that writes the first hot path adds `dhat` as a test dependency, which [ADR 0049](0049-hallmark-depends-on-cryptography-crates-zeroize-and-thiserror-and-writes-everything-else-itself.md) allows, adds the benchmark harness, and says how CI runs them.

### Assumptions in types

- A field that always holds a value once its type is built is not an `Option`. A constructor takes every required value, or each state gets its own type.
- Values that are present together form one `Option` of a struct or a tuple, or an enum, never several `Option`s.
- An absent value with more than one meaning is an enum. The workspace enables `option_option`.
- Code doesn't replace a missing value with a default. `clippy.toml` disallows `unwrap_or` and `unwrap_or_default` of `Option` and `Result`. A default that a specification gives, such as `c=simple/simple` (RFC 6376 §3.5), is a `match` arm with a comment that cites the section.
- No `else` branch and no `_` arm drops an error. The error becomes the protocol's result or the caller's error (ADR 0004).

### Lint exceptions

- An exception is `#[expect(lint, reason = "...")]` on the smallest item that needs it. The workspace enables `allow_attributes`, which rejects `#[allow]`, and `allow_attributes_without_reason`, which rejects an attribute without a reason. An `expect` whose lint no longer fires gives a warning of its own, so a stale exception fails CI.
- No source file allows or expects `clippy::disallowed_methods`. The `hallmark` crate forbids the lint in `lib.rs`, so the compiler rejects such an attribute in the library and its unit tests. An integration test or a benchmark is a crate of its own, which `lib.rs` doesn't reach, so each file under `tests/` and `benches/` forbids the lint too. `hallmark-wasm` can't forbid it, because the macros of wasm-bindgen emit `#[allow(clippy::all)]` on imported functions, and an `allow` of a forbidden lint is an error (E0453). Review checks `hallmark-wasm` and the files under `tests/` and `benches/`.

### Size

- The workspace enables `too_many_lines`, and `clippy.toml` sets its limit to 80 lines and the limit of `excessive_nesting` to 4 levels. `cognitive_complexity` stays off, because Clippy's documentation says the measure can't be computed.

### Checks

- The workspace denies `clippy::all` and every Clippy lint it adds, so `cargo clippy` fails on any of them without extra flags. Clippy lints don't run in `cargo build` or `cargo test`, so they don't stop a test run while code is being written.
- CI runs Clippy with `-D warnings` as well, which turns the remaining warnings, such as `missing_docs` and a stale `expect`, into errors.
- The rules cover the example code in `docs/tutorials/` and `docs/how-to/` as well. Review checks it, because Clippy reads only `.rs` files.

## Consequences

Positive:
- Branches and early returns appear as statements, where a reader can follow them.
- The method rule and the efficiency, exception and size lints are mechanical. Clippy and CI check them in full on every change, and reviewers don't judge chains.
- Types carry what code would otherwise assume, so a missing value becomes a compile error or a protocol result instead of a silent default.
- Functions stay within what a reviewer can read.

Negative:
- Some code gets longer: a five-line `match` replaces a one-line `map` or `unwrap_or`, and a loop replaces `any`.
- Nursery lints can report false positives, and each one needs an exception with a reason.
- A long table-driven test can pass 80 lines and has to be split or carry an exception.
- Clippy doesn't see closures, `Fn` bounds, function pointers, clones that get past the borrow checker, or searches inside loops. Review catches them.
- serde's derive generates `Option::map` for `#[serde(with = ...)]` fields. If the trace format uses such fields, Clippy will report the generated calls, and a new ADR will decide how to handle them.
- The code that `#[wasm_bindgen]` generates for exported functions, structs and `impl` blocks calls `Option::unwrap_or_default` and `Iterator::map`, and can take arguments by value. When `hallmark-wasm` starts to use wasm-bindgen, a new ADR will decide how its generated code meets these rules.

## References

- [procedural-rust skill](../../.claude/skills/procedural-rust/SKILL.md)
- [ADR 0004](0004-no-input-makes-hallmark-panic.md), the panic rule checked by the same Clippy run
- [ADR 0049](0049-hallmark-depends-on-cryptography-crates-zeroize-and-thiserror-and-writes-everything-else-itself.md), the dependencies of the `hallmark` crate
- [requirements.md: Hostile input](../reference/requirements.md#hostile-input), SAF-002 and SAF-003
- [Clippy lints](https://rust-lang.github.io/rust-clippy/master/index.html)
