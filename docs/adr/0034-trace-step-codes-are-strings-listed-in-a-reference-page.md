# ADR 0034: Trace step codes are strings listed in a reference page

- Status: Accepted
- Date: 2026-11-15
- Affects: every check that records trace steps, the trace format, the tests, and the demo

## Context

Each trace step carries codes and parameters, and the text a person reads is rendered from them (TRC-011). Tests match codes, so a change of wording breaks no test (TRC-009). The demo renders the trace, in English or another language, and outside tools read it too. TRC-011 calls for "a documented list" of codes, which doesn't exist yet.

A reader also needs a step's citations, and one code can cite different sections in different cases: the caution of HCN-004 cites RFC 6376 §3.5, §3.7 and §5.4. If checks run in separate calls (GEN-001), a DMARC alignment step names the SPF or DKIM result it used (DMC-005), so a step's position has to mean the same thing in every call.

## Options considered

- (a) Codes are strings such as `dkim.key_lookup`, listed in a reference page.
- (b) Codes are numbers, listed in a reference page.
- (c) Codes live only in the Rust code, as enum names.

## Decision

Option (a).

- Each event carries a `kind` and a `code`, an `outcome` when the code has outcomes, `params` when the step has parameters, and `cite` when the step cites a section.
- The kind is `fact`, `caution` or `hint` (TRC-003).
- A code is lowercase ASCII: parts joined by `.`, words within a part joined by `_`. The first part names the area: `trace`, `spf`, `dkim`, `dkim2`, `dmarc`, `dns`, `header` or `body`.
- An outcome is a lowercase string from the code's list. Results keep the names their specification gives them, in lowercase, such as `permerror` (RFC 7208 §2.6) or `permfail` (RFC 6376 §3.9).
- `cite` is a list of strings that each name a document and a section, such as `RFC 6376 §3.7`, `draft-ietf-dkim-dkim2-spec-09 §6.2` or `RFC 6376 erratum 5839`. Each event carries its citations, because the same code can cite different sections.
- The human-readable strings of draft-09 §14 (DK2-013) go in a parameter named `draft_text`, in the words the draft gives them (TRC-011).
- `docs/reference/trace-codes.md` lists every code with its kind, position, parameters and their types, outcomes, citations and English template. A test checks the page against the list of codes in the crate, so the two can't drift apart.
- The steps under the root of the tree have fixed slots, one for each kind of check, listed in that page. A check keeps its slot whether or not the other checks run in the same call, so a position names the same step in every call.
- A code is never reused or renamed. Adding a code, an outcome or a parameter is a minor version change; removing or changing one is a major one ([ADR 0033](0033-trace-events-are-i-json-texts-in-a-json-text-sequence.md)).
- The list grows with each feature: a code enters it with the scenarios that need it, before its code is written.

Option (b) makes every log line and every test need a lookup table, and a number tells a reader nothing. Option (c) leaves the demo and outside tools without a list to read, and lets a rename in the Rust code change the format without anyone noticing.

## Consequences

Positive:
- Codes read well in logs and in tests.
- Readers, the demo included, have one page to work from, and the test keeps it in step with the code.
- Positions under the root mean the same thing in every call, so a step in one trace can point to a step in another.

Negative:
- String codes make each event larger than numbers would.
- Every new step needs an entry in the page before its code is written.

## References

- [Trace codes](../reference/trace-codes.md)
- [requirements.md: Trace and reasoning](../reference/requirements.md#trace-and-reasoning)
- [ADR 0031](0031-the-code-that-decides-writes-the-trace-as-events-with-tree-positions.md)
