# Test scenarios

The test scenarios for hallmark, written before the code as the Canon TDD test list. Like the [architecture decision records](../adr/README.md), these documents sit outside the Diátaxis structure of the rest of `docs/`. Each scenario links to a [requirement](../reference/requirements.md) and, where one applies, to the section of the RFC or draft it checks (TST-001).

## Contents

- [Files](#files)
- [Format](#format)
- [Rules](#rules)
- [Requirements checked outside tests](#requirements-checked-outside-tests)
- [Third-party test data](#third-party-test-data)

## Files

Ordered as the components will be built:

| File | Area | ID prefix |
|------|------|-----------|
| [parsers.md](parsers.md) | Rules that every parser shares: errors, byte ranges, numbers, limits and nesting | `PRS` |
| [trace.md](trace.md) | Trace levels and limits, step codes, the order of steps, marked values, cautions and hints, events and their positions, the format's bytes, and the body in the trace | `STEP` |
| [canonicalization.md](canonicalization.md) | Body and header canonicalization for DKIM, DKIM2's body hash, header hash and signing input, line endings, chunk boundaries, `l=`, and the body canonicalizer | `CAN` |
| [header-fields.md](header-fields.md) | Splitting the header section into fields, unfolding, obsolete forms, and fields from a milter | `FLD` |
| [tag-lists.md](tag-lists.md) | Tag=value lists in DKIM and DKIM2, and the decoders of their values | `TAG` |
| [authentication-results.md](authentication-results.md) | Parsing and generating Authentication-Results | `ARH` |
| [addresses.md](addresses.md) | Header addresses, envelope paths and HELO names, the lookup form of domain names, and the checks that read addresses | `ADDR`, `PATH`, `DOM` |
| [checks.md](checks.md) | Driving a check: input order, questions and answers, caller errors, and the driver | `RUN` |
| [dns.md](dns.md) | Answer shapes, question names, temporary failures, snapshots and demo cases, pasted records, and public suffix answers | `RES` |
| [spf-records.md](spf-records.md) | The SPF record parser: selection, terms, addresses, macros, target names, explanations, and the address answers SPF reads | `SPR` |
| [dmarc-records.md](dmarc-records.md) | The DMARC record parser: selection, tag syntax, invalid values, the test of whether the author domain exists, reporting URIs, and RFC 7489 mode | `DMR` |
| [key-records.md](key-records.md) | DKIM key records, DKIM2 key records and Domain Policy Records | `KEY` |
| [dkim-verification.md](dkim-verification.md) | The checks on a DKIM-Signature field before its key is looked up | `VER` |
| [test-modes.md](test-modes.md) | Test modes and the published policy in DKIM and DMARC results | `POL` |
| [signing.md](signing.md) | Signers, the verification of the signatures they return, the library's signer for keys in memory, and the changes a DKIM2 Reviser describes | `SIGN` |
| [demo.md](demo.md) | The browser demo: line endings of pasted text and loaded files, values shown as text, and keys left out of saved cases | `DEMO` |
| [api.md](api.md) | The validation rules of the API reference: input order, envelope parts, answers, options, limits, keys, trace and snapshot documents, the canonicalizer, Authentication-Results and the driver | `API` |

Each new area gets its file before its first test.

## Format

Each file groups its scenarios into tables with these columns:

| Column | Content |
|--------|---------|
| ID | `<PREFIX>-<NNN>`, unique and never reused. Gaps between numbers leave room for related scenarios |
| Level | `unit`, `property` (property-based test), `fuzz`, `doctest` (code compiled from a document), or `browser` (the demo in a browser) |
| Given | The starting state |
| When | The single action under test |
| Then | The observable result to check |
| Spec | Links to the requirement and to the RFC or draft section that the scenario checks |

Byte strings in the tables use `\r`, `\n`, and `\t` for CR, LF, and HTAB. Hashes are SHA-256, written in base64 as they appear in a `bh=` tag.

## Rules

1. Every requirement in `docs/reference/requirements.md` has at least one scenario before its code is written, except those in [Requirements checked outside tests](#requirements-checked-outside-tests).
2. A test's function name starts with its scenario ID in lowercase, such as `can_001_simple_empty_body_is_one_crlf`, so each scenario can be traced to its test and selected with `cargo test can_001`.
3. A test is written from its scenario before the code that makes it pass.
4. A case found during implementation is added here first, with a new ID, then tested.
5. When a requirement changes, the scenarios that link to it are updated in the same change.
6. A scenario whose expected result depends on an open decision is marked "open" in its Then column and links to the question.
7. A scenario that turns out to be impossible is withdrawn, not deleted: it keeps its row and ID, its Then column starts with "Withdrawn:" and gives the reason, and it has no test.
8. Tests make no network calls. hallmark's own test domains are under `.test` ([ADR 0058](../adr/0058-test-names-are-under-test.md)), and their DNS answers come from snapshots. A test that runs third-party test data keeps the data's names and answers DNS from the data's records.

## Requirements checked outside tests

These requirements govern the repository, CI, the dependencies or how the code is written, so no test of the library can check them. The table names what checks each one.

| Requirement | Checked by |
|------|------|
| [REP-001](../reference/requirements.md#repository-and-deliverables) | The layout: `demo/` sits outside `crates/hallmark`, the directory that the package is built from |
| [REP-002](../reference/requirements.md#repository-and-deliverables) | The Pages workflow, once the demo exists |
| [REP-003](../reference/requirements.md#repository-and-deliverables) | Review of the demo's dependencies |
| [REP-004](../reference/requirements.md#repository-and-deliverables) | The Pages workflow, and the link check in it and in the Rustdoc job |
| [STB-001](../reference/requirements.md#stability-and-licensing) | Review of the changes before each release |
| [STB-002](../reference/requirements.md#stability-and-licensing) | Review of the workspace manifest, and the cargo-deny job for the licenses of dependencies |
| [STB-003](../reference/requirements.md#stability-and-licensing) | The MSRV job |
| [TST-001](../reference/requirements.md#quality-and-process) | Review of each scenario and each test |
| [TST-002](../reference/requirements.md#quality-and-process) | Review, and the table in [Third-party test data](#third-party-test-data) |
| [TST-004](../reference/requirements.md#quality-and-process) | Review of each test |
| [TST-005](../reference/requirements.md#quality-and-process) | A CI job that comes with the first fuzz target |
| [TST-006](../reference/requirements.md#quality-and-process) | Review of the code |
| [TST-007](../reference/requirements.md#quality-and-process) | The test job, whose matrix holds both systems |
| [WSM-001](../reference/requirements.md#webassembly) | The WebAssembly build job |
| [WSM-002](../reference/requirements.md#webassembly) | Review of the manifest of `hallmark` |
| [WSM-003](../reference/requirements.md#webassembly) | The WebAssembly build job |
| [TIM-006](../reference/requirements.md#verification-time) | The WebAssembly build job, which builds the same `Options` that the native tests use |
| [IO-002](../reference/requirements.md#io-only-through-the-caller) | Review of the API reference |
| [SEC-003](../reference/requirements.md#private-keys) | The cargo-deny jobs, which report advisories, and an ADR for each timing advisory |
| [TRC-001](../reference/requirements.md#trace-and-reasoning) | Review of the code: the code that decides writes the step ([ADR 0031](../adr/0031-the-code-that-decides-writes-the-trace-as-events-with-tree-positions.md)) |

## Third-party test data

hallmark's tests run this published conformance data (TST-002, [ADR 0039](../adr/0039-the-openspf-test-suite-is-copied-unchanged-into-a-test-crate-that-is-never-published.md)). Each copy matches the listed version byte for byte and sits next to its license file in `crates/hallmark-conformance`, a crate that is never published.

| Data | Source | Version | Files | License |
|------|--------|---------|-------|---------|
| The openspf test suite for RFC 7208: 203 cases | The [copy](https://github.com/sdgathman/pyspf/tree/4bf96ea63af4999663809bae9b5530bce25e6f16/test) that the suite's main author keeps in the pyspf repository | Commit `4bf96ea`, 2019-08-31 | `rfc7208-tests.yml`, `rfc7208-tests.LICENSE` | BSD-3-Clause, copyright Stuart D Gathman, Julian Mehnle and Scott Kitterman |

The openspf files come into the repository with the first test of the SPF check.
