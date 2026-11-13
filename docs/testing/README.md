# Test scenarios

The test scenarios for hallmark, written before the code as the Canon TDD test list. Like the [architecture decision records](../adr/README.md), these documents sit outside the Diátaxis structure of the rest of `docs/`. Each scenario links to a [requirement](../reference/requirements.md) and, where one applies, to the section of the RFC or draft it checks ([ADR 0009](../adr/0009-requirements-and-tests-come-only-from-the-specifications.md)).

## Files

Ordered as the components will be built:

| File | Area | ID prefix |
|------|------|-----------|
| [canonicalization.md](canonicalization.md) | Body canonicalization for DKIM and DKIM2, line endings, chunk boundaries, and `l=` | `CAN` |
| [header-fields.md](header-fields.md) | Splitting the header section into fields, unfolding, obsolete forms, and fields from a milter | `FLD` |
| [tag-lists.md](tag-lists.md) | Tag=value lists in DKIM and DKIM2, and the decoders of their values | `TAG` |
| [authentication-results.md](authentication-results.md) | Parsing and generating Authentication-Results | `ARH` |

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

1. Every requirement in `docs/reference/requirements.md` has at least one scenario before its code is written.
2. A test's function name starts with its scenario ID in lowercase, such as `can_001_simple_empty_body_is_one_crlf`, so each scenario can be traced to its test and selected with `cargo test can_001`.
3. A test is written from its scenario before the code that makes it pass.
4. A case found during implementation is added here first, with a new ID, then tested.
5. When a requirement changes, the scenarios that link to it are updated in the same change.
6. A scenario whose expected result depends on an open decision is marked "open" in its Then column and links to the question.
7. A scenario that turns out to be impossible is withdrawn, not deleted: it keeps its row and ID, its Then column starts with "Withdrawn:" and gives the reason, and it has no test.
8. Test domains are under `.invalid`, and DNS answers come from snapshots. Tests make no network calls.
