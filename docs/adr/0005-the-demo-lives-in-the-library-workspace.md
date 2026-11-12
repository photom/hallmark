# ADR 0005: The demo lives in the library's workspace

- Status: Accepted
- Date: 2026-11-12
- Affects: the repository layout, CI, GitHub Pages, and the hallmark-milter repository

## Context

The browser demo has two jobs: it shows what hallmark does, and it works as a test bench for the library. It reads the trace, which will change often while the library is young. It needs WebAssembly bindings and a DNS-over-HTTPS resolver that the core must not contain, because the core depends on no browser crate (WSM-002).

hallmark-milter is the other consumer. It is a program you deploy, with its own dependencies (rust-milter, Tokio, configuration) and its own release schedule.

## Options considered

- (a) The demo in its own repository, depending on a published hallmark.
- (b) The demo in the hallmark repository, as part of a Cargo workspace, with the browser code in its own crate.
- (c) hallmark-milter in the same workspace as well.

## Decision

Option (b), and hallmark-milter stays in its own repository.

- `crates/hallmark` is the core library, published to crates.io. It depends on no browser crate.
- `crates/hallmark-wasm` holds the `wasm-bindgen` bindings and the DNS-over-HTTPS resolver. It is not published at first.
- `demo/` holds the static site. A GitHub Actions workflow builds it and publishes it to GitHub Pages, with no `gh-pages` branch.
- CI builds the core and `hallmark-wasm` for `wasm32-unknown-unknown` on every change, and runs browser tests against the demo once the demo exists, so a library change that breaks the demo fails its own pull request.
- The demo stays small: plain TypeScript or JavaScript with few npm dependencies.
- hallmark-milter depends on the published hallmark crate and never on its internals.

## Consequences

Positive:
- A change to the trace format updates the library and the demo in one commit.
- GitHub Pages publishes from this public repository on a free plan.

Negative:
- The repository carries JavaScript tooling next to the Rust crates, and Dependabot will raise npm updates too.
- If the demo grows into a large front-end application with its own contributors, it moves to its own repository, and that move needs a new ADR.

## References

- [requirements.md: Repository and deliverables](../reference/requirements.md#repository-and-deliverables)
- [requirements.md: Browser demo](../reference/requirements.md#browser-demo)
