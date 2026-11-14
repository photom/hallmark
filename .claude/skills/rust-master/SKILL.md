---
name: rust-master
description: Project Rust conventions for naming, module layout, error handling, async code, and enforcing Bounded Contexts with Rust's privacy and module system. Use when writing or reviewing Rust code in this repository.
---

# Rust conventions

## Naming and the Ubiquitous Language

### Standard Rust naming

- Types and traits: `UpperCamelCase`
- Variables, functions, and modules: `snake_case`
- Constants and statics: `SCREAMING_SNAKE_CASE`

### Domain language

- Do not repeat the module name in a type name.
- Type names must match the domain term exactly, with no technical padding or suffix.
- Write domain terms in full. Abbreviate only where the domain itself does.
- Keep protocol constant names as the specification spells them, for example DKIM tag names such as `bh` and SPF mechanism names such as `include`.

## Error handling

- The library's public API exposes typed error enums built with `thiserror` 2.x. A library crate must not expose `Box<dyn Error>` aliases; keep `anyhow`-style boxed errors for binaries, examples, and tests.
- Give each domain its own error enum.
- Do not call `unwrap`, `expect`, `panic!`, `unreachable!`, or slice indexing in library code. Clippy denies them outside tests, because no message, DNS answer, or record may make hallmark panic.
- Propagate errors with `?`.

## Modern Rust baseline

- Use edition 2024 (Rust 1.85 or later) for new crates, and declare the MSRV with `rust-version` in `Cargo.toml`.
- Write `async fn` in traits directly (Rust 1.75 or later) and do not add the `async-trait` crate. In a public trait whose futures must be `Send`, for example because callers spawn them on a multi-threaded Tokio runtime, declare `fn method(&self) -> impl Future<Output = T> + Send`. rustc's `async_fn_in_trait` lint warns about `async fn` in public traits.
- Pin with `std::pin::pin!` (Rust 1.68 or later) in place of `tokio::pin!` or `futures::pin_mut!`.
- Use `std::sync::LazyLock` (Rust 1.80) and `OnceLock` (Rust 1.70) in place of `lazy_static` and `once_cell`.
- Suppress lints with `#[expect(lint, reason = "...")]` (Rust 1.81 or later) in place of `#[allow]`. The compiler then reports a suppression that is no longer needed.

## Async code

The hallmark core has no async runtime and does no I/O: the caller drives DNS and streams, and a check holds no future and no socket (ADR 0001). Async code lives in the library's driver, in `hallmark-wasm`, in examples, and in tests.

### Runtime independence

- The driver awaits the resolver's futures and depends on no async runtime (DNS-003). Do not make `crates/hallmark` depend on Tokio or any other runtime.
- `hallmark-wasm` runs on the browser's event loop, where DNS over HTTPS goes through `fetch` and its promises.

### Cancellation

- A caller cancels a check by dropping it or by dropping the driver's future. Write the driver so that a drop at any `.await` leaves nothing to clean up.
- In examples and tests that run on Tokio, stop tasks with `tokio::select!` and `tokio_util::sync::CancellationToken`.

### Executor safety

- On Tokio, do not run synchronous I/O or CPU-heavy work on the executor. Move it to `spawn_blocking`.
- Do not hold a `std::sync` mutex guard across an `.await`. The guard makes the future `!Send`.

### Async tests

- Test the driver without Tokio, so the tests show that it needs no runtime.
- Use `#[tokio::test]` for async tests of examples that run on Tokio.

## Bounded Contexts in modules

### Privacy

- Keep types and fields private unless code outside the module needs them.
- Use `pub(crate)` for crate-wide visibility and `pub(super)` for the parent module.
- Make only Aggregate Roots part of the public API.

### Module isolation

- Group related domain objects in one module.
- Code outside the module talks only to the Aggregate Root.
- Implementation details stay private to the module.

### Workspace layout

- For strict Bounded Context boundaries, split the contexts into separate crates in the Cargo workspace, one complete context per crate.
- Keep dependencies between context crates to a minimum.

## Tooling and quality

### Before every commit

- Run `cargo fmt`.
- Run `cargo clippy` and fix every warning.
- Make sure all tests pass.

### Documentation

- Give every public API a rustdoc comment with an example that runs as a doctest.
- Document the safety requirements of `unsafe` code.
- Explain design decisions that are not obvious in the module-level docs.

## Related skills

- canon-tdd: TDD for Rust development.
- design-master: the Bounded Context strategy behind the module rules.
- external-integration: resolvers, DNS over HTTPS, retries and caching.
