# ADR 0069: Fuzz targets run with cargo-fuzz and reach the internal parsers through a hidden `fuzzing` feature

- Status: Accepted
- Date: 2026-11-27
- Affects: the `fuzz/` crate, the `fuzzing` feature and the `fuzz` module of the `hallmark` crate, the CI workflows, and `deny.toml`

## Context

[ADR 0004](0004-no-input-makes-hallmark-panic.md) and SAF-003 give every parser a fuzz target, and TST-005 runs the targets in CI for a bounded time on every change and for longer on a schedule. Fuzzing covers what the lints can't see, such as an overflow in unchecked arithmetic or a loop that never ends. No document picks the tool, and none says how a target reaches a parser. A fuzz target is a separate crate that sees only the public API, while most of hallmark's parsers are crate-private: the tag-list parser and its value decoders, base64 and JSON.

## Options considered

The tool:

- (a) cargo-fuzz, which runs libFuzzer. It is coverage-guided and needs a nightly toolchain for its sanitizer flags.
- (b) cargo-afl, which runs AFL++. It is coverage-guided and works on stable, but its toolchain takes longer to set up and run in a short CI job.
- (c) Property tests with proptest only. They need no new toolchain, but they aren't coverage-guided, so they find fewer deep faults, and they may not meet "fuzz target" in SAF-003.

How a target reaches a crate-private parser:

- (d) A Cargo feature that builds a hidden module of thin entry points around the parsers.
- (e) The public API only, such as a whole message given to a check. This adds no surface, but inputs reach the inner parsers less directly.
- (f) The targets themselves inside the library, behind a `cfg`, with one hidden entry point for each. This keeps fuzz code in the library source.

## Decision

Options (a) and (d).

- The targets live in `fuzz/`, a crate outside the workspace, laid out as cargo-fuzz expects. It depends on `libfuzzer-sys`, and on `hallmark` with the features `fuzzing` and `unstable-dkim2`.
- The `fuzzing` feature builds `hallmark::fuzz`, a module marked `#[doc(hidden)]`. Each entry point runs the parsers of one area on the input, and returns whether every result keeps the promises of those parsers, such as each error offset and each range lying within the input. An entry point never panics, as ADR 0004 asks of library code. The fuzz target asserts what it returns.
- The feature isn't for callers. It stays out of the documentation and of the semantic versioning promise, and any release can change the module.
- Each target is one `fuzz_target!`, which takes its input in the syntax of a closure. That is the one exception to the rule of [ADR 0064](0064-rust-code-is-procedural-and-lints-types-and-size-limits-keep-it-efficient-and-reviewable.md) against closures, and it applies only to the targets in `fuzz/`.
- A `Fuzz` workflow runs every target for 60 seconds on each pull request and push to `main`, and for 600 seconds on a weekly schedule and on a manual run. Only that job installs a nightly toolchain and cargo-fuzz. The other jobs keep the toolchain that `rust-toolchain.toml` names.
- The same workflow runs cargo-deny on the fuzz crate with the workspace's `deny.toml`, as [ADR 0049](0049-hallmark-depends-on-cryptography-crates-zeroize-and-thiserror-and-writes-everything-else-itself.md) asks for the dependencies of fuzz targets. `libfuzzer-sys` holds LLVM's libFuzzer under the NCSA license, which `deny.toml` allows for that crate alone, because the fuzz crate is never published or shipped with hallmark.

## Consequences

- A fault fails the `Fuzz` job. libFuzzer prints the input that caused it and saves it as a file, which the job uploads, and the file reproduces the fault with cargo-fuzz. The job keeps the corpus in the CI cache, so each run starts from what the earlier runs found.
- The job pins the version of cargo-fuzz, and uses the current nightly toolchain.
- The `tags` modules no longer expect dead code when the `fuzzing` feature is on, because the entry points use all their items: they read every field of each result and error. `base64` and `json` keep their expectation, because parts of them serve only the trace writer, which no code calls yet. A module that the entry points use in full drops its expectation under the feature in the same way.
- The fuzz crate has no lock file of its own yet, so cargo resolves its dependencies on each run until one is committed.
- Each new parser adds its entry point to `hallmark::fuzz` and its target to `fuzz/` (SAF-003).

## References

- [ADR 0004](0004-no-input-makes-hallmark-panic.md)
- [ADR 0049](0049-hallmark-depends-on-cryptography-crates-zeroize-and-thiserror-and-writes-everything-else-itself.md)
- [ADR 0064](0064-rust-code-is-procedural-and-lints-types-and-size-limits-keep-it-efficient-and-reviewable.md)
- [SAF-001](../reference/requirements.md#hostile-input), [SAF-003](../reference/requirements.md#hostile-input) and [TST-005](../reference/requirements.md#quality-and-process)
- [TAG-060](../testing/tag-lists.md#fuzzing)
