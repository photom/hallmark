# ADR 0036: The minimum supported Rust version starts at 1.88 and rises when a dependency needs it

- Status: Accepted
- Date: 2026-11-15
- Affects: the manifest of the `hallmark` crate, CI, STB-003, and hallmark-milter

## Context

STB-003 asks the core to state a minimum supported Rust version (MSRV) and to test it in CI. The workspace uses edition 2024, which needs Rust 1.85 or later. `rust-toolchain.toml` pins Rust 1.99, the current stable release, for development and for every CI job.

rust-milter, the other library that hallmark-milter depends on, declares Rust 1.85 with `rust-version`. A CI job checks its library with that version and leaves out the tests, which need a newer Rust. rust-milter's dependencies, such as Tokio, still build with 1.85.

hallmark needs pure Rust cryptography that builds for WebAssembly (WSM-003). The crates aren't chosen yet, but the newest releases of the likely candidates need more than 1.85. On crates.io on 2026-11-15, `sha2` 0.11.1 declares Rust 1.88, and `rsa` 0.10.0 and `ml-dsa` 0.1.2 declare 1.87. `ed25519-dalek` 3.0.0, `zeroize` 1.9.1, `pkcs1` 0.8.0, `spki` 0.8.1 and `der` 0.8.3 declare 1.85. The workspace uses Cargo's resolver "3", which prefers the newest release that fits the declared `rust-version`. With 1.85, Cargo would resolve hallmark's own lockfile to `sha2` 0.11.0 and `ml-dsa` 0.1.1, and no final release of `rsa` 0.10 builds with 1.85.

Rust 1.88 also made `let` chains stable in edition 2024. With them, the procedural style of ADR 0003 can join `if let` conditions with `&&` in a single `if`.

## Options considered

- (a) Rust 1.85, as rust-milter declares. It is the lowest version the edition allows, and it holds the cryptography crates on older releases.
- (b) Rust 1.88, raised when a release of a dependency needs a newer Rust.
- (c) No stated version, so the library supports only the toolchain that `rust-toolchain.toml` pins. STB-003 asks for a stated version, and callers such as hallmark-milter would have to follow each bump of that file.

## Decision

Option (b), with the structure that rust-milter uses.

- `crates/hallmark` declares `rust-version = "1.88"`. `hallmark-wasm` isn't published and declares none.
- A CI job named MSRV installs Rust 1.88 and runs `cargo check` on the `hallmark` library, with all features and with none. Tests may need a newer Rust, and the WebAssembly build keeps running on the pinned toolchain (WSM-001).
- `rust-toolchain.toml` keeps pinning the current stable release for development and for every other CI job, and a maintainer bumps it by hand.
- If a security or bug-fix release of a dependency needs a newer Rust, the MSRV rises to that version in the crate's manifest, the CI job and STB-003. hallmark doesn't hold a dependency back to keep an older MSRV.

## Consequences

Positive:
- The library can take the newest releases of the cryptography crates above, with their fixes.
- Code can use `let` chains.
- Clippy takes its MSRV from `rust-version`. Its `incompatible_msrv` lint, which warns by default, flags items of the standard library that are newer than 1.88, and the Clippy job turns that warning into an error. A contributor who runs Clippy on the pinned toolchain sees the problem without installing Rust 1.88.
- hallmark-milter needs Rust 1.88 or later. rust-milter's 1.85 lies below that, so the two libraries don't conflict.

Negative:
- A caller on a Rust older than 1.88, which came out in June 2025, can't build hallmark.
- The MSRV follows the dependencies, so a caller who stays on one Rust version can lose hallmark updates when a dependency raises its own.
- Raising the MSRV for another reason, such as a newer language feature, changes this decision and needs a new ADR.
- A contributor who runs the MSRV check locally needs the Rust 1.88 toolchain as well as the pinned one.

## References

- [requirements.md: Stability and licensing](../reference/requirements.md#stability-and-licensing) (STB-003)
- [requirements.md: WebAssembly](../reference/requirements.md#webassembly) (WSM-001, WSM-003)
- [ADR 0003: Rust code is written in a procedural style](0003-rust-code-is-written-in-a-procedural-style.md)
- [Rust 1.88.0 release notes: Language](https://github.com/rust-lang/rust/blob/master/RELEASES.md#1.88.0-Language)
- [rust-milter test environment: Toolchain](https://github.com/photom/rust-milter/blob/main/docs/reference/test-environment.md#toolchain)
