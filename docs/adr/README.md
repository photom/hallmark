# Architecture decision records

Decisions that shape hallmark's design and specification, with their context and consequences. These records sit outside the Diátaxis structure of the rest of `docs/`: each one is a dated record of a decision, kept as it was made.

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-the-core-does-no-io-and-the-caller-drives-it.md) | The core does no I/O, and the caller drives it | Accepted |
| [0002](0002-the-code-that-decides-writes-the-trace.md) | The code that decides writes the trace | Accepted |
| [0003](0003-rust-code-is-written-in-a-procedural-style.md) | Rust code is written in a procedural style | Accepted |
| [0004](0004-no-input-makes-hallmark-panic.md) | No input makes hallmark panic | Accepted |
| [0005](0005-the-demo-lives-in-the-library-workspace.md) | The demo lives in the library's workspace | Accepted |
| [0006](0006-dkim2-targets-draft-09-behind-an-unstable-feature.md) | DKIM2 targets draft-09 behind an unstable feature | Accepted |
| [0007](0007-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md) | DMARC supports RFC 7489 and DMARCbis, with DMARCbis the default | Accepted |
| [0008](0008-dkim2-recipe-line-numbers-count-the-raw-body.md) | DKIM2 recipe line numbers count the raw body | Accepted |
| [0009](0009-requirements-and-tests-come-only-from-the-specifications.md) | Requirements and tests come only from the specifications | Accepted |

Conventions:
- One file per decision, named `NNNN-short-title.md`, numbered in order.
- Sections: status and date, context, options considered, decision, consequences, references.
- Status is `Proposed`, `Accepted`, `Superseded by ADR NNNN`, or `Deprecated`.
- An accepted record is not rewritten; only its status line changes.
- When a decision turns out wrong or changes, a new ADR restates the whole decision, so that it can be read alone. The old record's status becomes `Superseded by ADR NNNN`, and its body stays as it was.
- Specifications and other documents link only to records in force, and describe current behavior without its history.
