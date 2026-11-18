# ADR 0052: The API reference is rustdoc, published on GitHub Pages

- Status: Accepted
- Date: 2026-11-18
- Affects: the API reference, the `hallmark` and `hallmark-wasm` crates, CI, GitHub Pages, `docs/testing/`, and every document that links to the API reference

## Context

The API reference was a set of Markdown pages in `docs/reference/api/`. A page gave the inputs of an API in one sentence, such as "A field name and a value, as bytes", and a reader couldn't tell how many parameters the method takes, or their types and order. Rust signatures written into those pages would state the API twice, once in Markdown and once in the crate, and the two copies can drift apart.

rustdoc builds a reference from the crate: each type with its fields or variants, each function with its signature, and the doc comments. The compiler checks the signatures, and `cargo doc` with `-D warnings` checks the links between items.

hallmark has no implementation yet. Rust needs a body for every function outside a trait, and ADR 0004 denies `unimplemented!` and `todo!` in library code.

rustdoc output isn't stored in the repository, so a scenario in `docs/testing/` can't reach it with a relative link. The repository is public, and REP-002 already publishes the demo on GitHub Pages.

## Options considered

- (a) Keep the Markdown reference, and write the signatures into it.
- (b) Write the reference as rustdoc, adding each item to the crate when it is implemented. The Markdown pages hold the rest until then.
- (c) Write the whole reference as rustdoc now. An item that isn't implemented yet sits under `#[cfg(doc)]`, so that only rustdoc compiles it, and `docs/reference/api/` goes away.

## Decision

Option (c).

- The API reference is the rustdoc of the `hallmark` crate. Its doc comments hold what the Markdown pages held: the meaning and form of each parameter, the fields and variants of each type, the validation rules with their IDs, the error codes, and the requirements and scenarios behind them.
- An item that isn't implemented yet carries `#[cfg(doc)]`, and the bodies of its functions are `unimplemented!()`. rustdoc compiles it. `cargo build`, `cargo test` and Clippy don't, so the library holds no `unimplemented!` and ADR 0004 holds as written.
- A module with no implemented item carries `#[cfg(doc)]` on its declaration. When an item is implemented, it leaves `#[cfg(doc)]` and keeps its doc comment. A module that holds implemented items keeps `#[cfg(doc)]` only on the items still to come.
- The doc comment of an implemented item may hold examples, which run as doctests. An item under `#[cfg(doc)]` holds none, because doctests compile without `cfg(doc)`. Tutorials and how-to guides keep the longer examples.
- `docs/reference/api/` goes away. The rest of `docs/` stays in Markdown, the requirements, the design, the trace format and the trace codes included, because those documents describe behavior and formats that don't depend on Rust.
- CI builds the rustdoc of the workspace from `main` and publishes it on GitHub Pages under `/api/`, in the same site as the demo of REP-002. The root of the site belongs to the demo. Until the demo exists, the root links to the API reference.
- A document links to an API item by its Pages URL and rustdoc anchor, such as `https://photom.github.io/hallmark/api/hallmark/struct.Check.html#method.answer`. CI checks each such link against the `target/doc` that the same run builds. It also checks every relative link in the Markdown documents, with its anchor.

Option (a) states the API twice. Option (b) splits the reference between Markdown and rustdoc until the implementation is complete.

## Consequences

Positive:
- The signatures and types are written once, and the compiler checks them.
- Readers get the layout of rustdoc: each type, each method with its signature, and its errors.
- A broken link between items, or from a document to an item, fails CI.

Negative:
- Until an item is implemented, only `cargo doc` compiles its declaration. `cargo build` and Clippy don't see it.
- The site shows `main`, so a link in an older commit points at the current API.
- Renaming an item changes its anchor, and the documents that link to it change in the same pull request.
- The validation rules sit in Markdown tables inside doc comments, which read less well in the source than in the HTML.

## References

- [ADR 0003](0003-rust-code-is-written-in-a-procedural-style.md), the procedural style that the declarations follow
- [ADR 0004](0004-no-input-makes-hallmark-panic.md), the panic rule for library code
- [ADR 0005](0005-the-demo-lives-in-the-library-workspace.md), the demo in the library's workspace
- [requirements.md: Repository and deliverables](../reference/requirements.md#repository-and-deliverables)
- [The rustdoc book](https://doc.rust-lang.org/rustdoc/)
