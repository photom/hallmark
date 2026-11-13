# ADR 0003: Rust code is written in a procedural style

- Status: Accepted
- Date: 2026-11-12
- Affects: all Rust code in the repository, the workspace lints, `clippy.toml`, and CI

## Context

Rust code can branch and loop in two styles. One uses `match`, `if let`, and `for`. The other chains `Option` and `Result` combinators and iterator adapters, each with a closure: `map_err(|error| ...)`, `ok_or_else(|| ...)`, `iter().filter(|x| ...).any(|y| ...)`. A chain hides its branches and early returns inside closure arguments, and the maintainer finds such code hard to read and review. Clippy's default lints push toward the second style: `manual_map`, `manual_find`, `map_entry`, and similar lints turn an explicit `match` or loop into a combinator with a closure.

Code passed around as a value hides which code runs in the same way. A closure stored as `Box<dyn Fn>`, or a function pointer in a table, puts the code behind a field or an index.

hallmark's parsers and canonicalizers read hostile input, and reviewers need to see every branch.

## Options considered

- (a) No rule. Reviewers judge each change.
- (b) A written convention, checked only in review.
- (c) The procedural style, with the methods that take a function checked by Clippy.

## Decision

Option (c).

- Code branches with `match`, `if let`, and `let ... else`, returns errors with `?`, and repeats with `for`, `while`, and `loop`.
- Code calls no method that takes a function as an argument: the `Option` and `Result` combinators, `bool::then`, the `Iterator` adapters and consumers that take a closure, `iter::from_fn`, `repeat_with`, and `successors`, and the `retain`, `dedup_by`, and `Entry` methods of the collections. `clippy.toml` lists them under `disallowed-methods`, and CI runs Clippy with `-D warnings`. Methods that take no function stay allowed, alone or chained.
- Code writes no closures. No external API that hallmark uses requires one yet. When one does, a new ADR names it as the exception.
- Code writes no bound on `Fn`, `FnMut`, `FnOnce`, or the `AsyncFn` traits, stores no closure as `Box<dyn Fn>`, and uses no function pointer type. A table of code becomes an enum with a `match`.
- No source file allows or expects `clippy::disallowed_methods`.
- The workspace allows the Clippy lints whose fix uses a method with a closure: `manual_filter`, `manual_filter_map`, `manual_find`, `manual_find_map`, `manual_map`, `manual_retain`, and `map_entry`.
- The rule covers the example code in `docs/tutorials/` and `docs/how-to/` as well; review checks it.

## Consequences

Positive:
- Branches and early returns appear as statements, where a reader can follow them.
- The method rule is mechanical, so Clippy checks it in full and reviewers do not judge chains.

Negative:
- Some code gets longer: a five-line `match` replaces a one-line `map`, and a loop replaces `any`.
- Clippy does not see closures, `Fn` bounds, or function pointers. Review catches them until the repository adds a syntax check.
- serde's derive generates `Option::map` for `#[serde(with = ...)]` fields. If the trace format uses such fields, Clippy will report the generated calls, and a new ADR will decide how to handle them.

## References

- [procedural-rust skill](../../.claude/skills/procedural-rust/SKILL.md)
- [ADR 0004](0004-no-input-makes-hallmark-panic.md), the panic rule checked by the same Clippy run
