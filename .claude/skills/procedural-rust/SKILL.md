---
name: procedural-rust
description: Write hallmark's Rust code in the procedural style of ADR 0003 (match, if let, let-else, ?, loops; no closures, Fn bounds, function pointers, or methods that take a function) and without panics (ADR 0004), and run the checks. Use when writing any Rust in this repository, or when Clippy reports a disallowed method or a panic lint.
---

# Procedural Rust in hallmark

ADR 0003 records the style rule and ADR 0004 the panic rule. This skill gives the rule on one screen, the traps, and the checks.

## The rule on one screen

- Branch with `match`, `if let`, `let ... else`; return errors with `?`; repeat with `for`, `while`, `loop`.
- No method that takes a function. `clippy.toml` lists them under `disallowed-methods`, and CI fails on any call. Methods that take no function (`unwrap_or`, `ok`, `then_some`, `enumerate`, `skip`, `zip`, `collect`) stay allowed. Passing a function name instead of a closure, as in `.map(Selector::new)`, breaks the rule too, even where no check catches it.
- No closures. ADR 0003 lists the external APIs that require one; there are none yet. `async` blocks are fine; `async |x|` closures are not.
- No `Fn`, `FnMut`, `FnOnce` or `AsyncFn*` bounds, and no `Box<dyn Fn...>`.
- No function pointer types (`fn(u8) -> bool`). Use an enum and a `match`.
- No `allow` or `expect` of `clippy::disallowed_methods`.
- No panics in library code: no `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, and no slice indexing (`buf[i]`, `&buf[a..b]`). Use `get`, `split_at_checked`, `strip_prefix`, pattern matching on slices, and checked arithmetic. Tests may use them; `clippy.toml` allows them in tests.

## Traps

- Clippy's `question_mark` rejects `let Some(x) = x else { return None; };`. Write `let x = x?;`.
- `sort_by_key` takes a function. Derive `Ord` and call `sort`.
- proptest's `prop_map` takes a function. Draw the raw values and build the input in the test body.
- Slice patterns replace most indexing: `if let [first, rest @ ..] = bytes { ... }`.
- Arithmetic on lengths from the input can overflow. Use `checked_add` and friends, and turn `None` into the protocol's error result.
- Clippy reads only `.rs` files. Review the Rust blocks of `docs/tutorials/` and `docs/how-to/` yourself.
- A change that needs a public API decision is the user's to make. Stop and ask, with the options and a recommendation.

## Check your code

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo clippy -p hallmark -p hallmark-wasm --target wasm32-unknown-unknown --all-features -- -D warnings
cargo test --workspace --all-features
```

## Fix a finding

1. Rewrite the finding with `match`, `if let`, a loop, or a checked operation. Keep behavior unchanged; the existing scenarios are the safety net. If a rewrite needs a new behavior or a new test, add the scenario to `docs/testing/` first (canon-tdd).
2. Run every check above until it reports nothing.
3. Review the diff as a reviewer would: same behavior on every path, no leftover helper, clear names, doc comments that still match.
