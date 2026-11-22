---
name: procedural-rust
description: Write hallmark's Rust code under ADR 0064 (match, if let, let-else, ?, loops; no closures, Fn bounds, function pointers, or methods that take a function; no needless allocations; assumptions held in types instead of defaults; #[expect] with a reason; small functions) in pull requests of at most 400 lines of Rust, and without panics (ADR 0004), and run the checks. Use when writing any Rust in this repository, or when Clippy reports a disallowed method, a panic lint, or a size lint.
---

# Procedural Rust in hallmark

ADR 0064 records the style, efficiency, type, exception and function size rules, and ADR 0004 the panic rule. This skill gives the rules on one screen, the limit on the size of your pull requests, the traps, and the checks.

## The rules on one screen

Style:
- Branch with `match`, `if let`, `let ... else`; return errors with `?`; repeat with `for`, `while`, `loop`.
- No method that takes a function. `clippy.toml` lists them under `disallowed-methods`, and CI fails on any call. Methods that take no function (`ok`, `then_some`, `enumerate`, `skip`, `zip`, `collect`) stay allowed, except `unwrap_or` and `unwrap_or_default`. Passing a function name instead of a closure, as in `.map(Selector::new)`, breaks the rule too, even where no check catches it.
- No closures. No external API needs one yet; when one does, a new ADR names it as the exception. `async` blocks are fine; `async |x|` closures are not.
- No `Fn`, `FnMut`, `FnOnce` or `AsyncFn*` bounds, and no `Box<dyn Fn...>`.
- No function pointer types (`fn(u8) -> bool`). Use an enum and a `match`.

Panics:
- No panics in library code: no `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, and no slice indexing (`buf[i]`, `&buf[a..b]`). Use `get`, `split_at_checked`, `strip_prefix`, pattern matching on slices, and checked arithmetic. Tests may use them; `clippy.toml` allows them in tests. `unwrap_or` and `unwrap_or_default` stay disallowed in tests too, because `allow-unwrap-in-tests` doesn't cover `disallowed-methods`.

Assumptions in types:
- A field that always holds a value once its type is built is not an `Option`. Take every required value in the constructor, or give each state its own type.
- Values that are present together are one `Option<(A, B)>`, one `Option` of a struct, or an enum, never two `Option`s.
- An absent value with more than one meaning is an enum, not `Option<Option<T>>`.
- No silent default. Write a `match`. A default that a specification gives sits in its own arm with a comment that cites the section, such as `// RFC 6376 §3.5: c= defaults to simple/simple`.
- No `else` branch and no `_` arm drops an error. Turn it into the protocol's result or the caller's error.

Efficiency:
- Don't `collect` what you only iterate, don't `clone` what you drop next, and take by reference what you only read. Clippy checks these.
- Don't `clone` to get past the borrow checker. Borrow, split the borrow, or move the value.
- Inside a loop over input, don't search a collection that grows with the input (`Vec::contains`, a nested `for` over the same items). Keep a map, a set, an index or sorted data.

Exceptions and size:
- An exception is `#[expect(clippy::lint, reason = "...")]` on the smallest item. `#[allow]` and an `expect` without a reason fail Clippy, and CI fails on an `expect` that no longer matches.
- No `allow` or `expect` of `clippy::disallowed_methods`. `lib.rs` of the `hallmark` crate forbids the lint, so the compiler rejects one there, unit tests included. Integration tests and benchmarks are separate crates that `lib.rs` doesn't reach: start each file under `tests/` and `benches/` with `#![forbid(clippy::disallowed_methods)]`.
- A function that only tests call so far is `dead_code` outside `cfg(test)`. Write `#[cfg_attr(not(test), expect(dead_code, reason = "..."))]`. A bare `#[expect(dead_code)]` fails under `cfg(test)`, where the lint doesn't fire.
- A lint that fires in only one feature set gets its exception in that set only, such as `#[cfg_attr(not(feature = "unstable-dkim2"), expect(clippy::lint, reason = "..."))]`. CI runs Clippy with every feature and with none.
- A function has at most 80 lines and 4 levels of nesting. The `impl` block and the function body count as one level each, so a method holds at most 2 levels of control structures. Split it, or return early with `let ... else` and `?`.
- A pull request changes at most 400 lines of Rust under `crates/`, counting added and removed lines and leaving out generated files. Split larger work, and ask the user before going over the limit.
- The description of each pull request says what the change assumes, which invariants its types hold, and why it takes this approach.

## Traps

- Clippy's `question_mark` rejects `let Some(x) = x else { return None; };`. Write `let x = x?;`.
- `sort_by_key` takes a function. Derive `Ord` and call `sort`.
- proptest's `prop_map` takes a function. Draw the raw values and build the input in the test body.
- Slice patterns replace most indexing: `if let [first, rest @ ..] = bytes { ... }`.
- Arithmetic on lengths from the input can overflow. Use `checked_add` and friends, and turn `None` into the protocol's error result.
- Clippy catches only `unwrap_or` and `unwrap_or_default`. A fallback such as `if let Some(v) = x { v } else { 0 }`, or a `match` arm with a default and no cited section, passes every check. Before you push, find each default in your diff and confirm that a specification gives it and that its arm cites the section.
- A lint that you can't satisfy is a design question first. Change the types or split the function before you write an `expect`.
- Tests follow the procedural style too: no closures, no `iter().any(...)`, no `unwrap_or`. Write the assertion with a loop or a `match`. A table-driven test over 80 lines gets `#[expect(clippy::too_many_lines, reason = "...")]`.
- Clippy reads only `.rs` files. Review the Rust blocks of `docs/tutorials/` and `docs/how-to/` yourself.
- A change that needs a public API decision is the user's to make. Stop and ask, with the options and a recommendation.

## Check your code

In the TDD cycle, run only `cargo test` in Red and Green. Run the commands below after Green, fix what they report in Refactor, and run them again after Refactor until they report nothing.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --no-default-features -- -D warnings
cargo clippy -p hallmark -p hallmark-wasm --target wasm32-unknown-unknown --all-features -- -D warnings
cargo test --workspace --all-features
```

## Fix a finding

1. Rewrite the finding with `match`, `if let`, a loop, a checked operation, or a type that makes the assumption unnecessary. Keep behavior unchanged; the existing scenarios are the safety net. If a rewrite needs a new behavior or a new test, add the scenario to `docs/testing/` first (canon-tdd).
2. Run every check above until it reports nothing.
3. Review the diff as a reviewer would: same behavior on every path, no leftover helper, no new `clone` or default, clear names, doc comments that still match.
