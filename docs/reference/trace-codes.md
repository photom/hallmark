# Trace codes

The codes that trace events carry, and the slots under the root of the tree (TRC-011, [ADR 0034](../adr/0034-trace-step-codes-are-strings-listed-in-a-reference-page.md)). [Trace format](trace-format.md) describes the events themselves.

## Contents

- [How to read an entry](#how-to-read-an-entry)
- [Rules](#rules)
- [Slots under the root](#slots-under-the-root)
- [Codes](#codes)
  - [`trace`](#trace)
- [Hints](#hints)
  - [`dkim.hint.body_changed`](#dkimhintbody_changed)
  - [`dkim.hint.list_header_change`](#dkimhintlist_header_change)
  - [`dkim.hint.simple_header_change`](#dkimhintsimple_header_change)
  - [`dkim2.hint.quoted_local_part`](#dkim2hintquoted_local_part)
  - [`dmarc.hint.forwarded`](#dmarchintforwarded)

## How to read an entry

Each code has an entry with these fields:

| Field | Meaning |
|---|---|
| Kind | `fact`, `caution` or `hint` (TRC-003) |
| Position | The step's place in the tree |
| Parameters | Each parameter's name and type ([Trace format](trace-format.md#values)), in the order they appear in `params`, and for a data value who controls it |
| Outcomes | The outcomes the step can have, or none |
| Cites | The sections the step cites, or none |
| English | The text that the library's English renderer makes, with parameter names in braces |
| Requirement | The requirement that calls for the step, when one names it |

## Rules

- A code is lowercase ASCII, with parts joined by `.` and words within a part joined by `_`. The first part names the area: `trace`, `spf`, `dkim`, `dkim2`, `dmarc`, `dns`, `header` or `body`.
- An outcome is lowercase. Results keep the names that their specification gives them, such as `permerror` (RFC 7208 §2.6) or `permfail` (RFC 6376 §3.9).
- The human-readable strings of draft-09 §14 go in a parameter named `draft_text`, in the words the draft gives them (DK2-013, TRC-011).
- A code is never reused or renamed. Adding a code, an outcome or a parameter is a minor version change, and removing or changing one is a major one ([Trace format](trace-format.md#versions)).
- A code enters this page with the scenarios that need it, before the code that records it is written.
- A test checks this page against the list of codes in the crate.

## Slots under the root

Each kind of check has a fixed slot under the root, which it keeps whether or not the other checks run in the same call. So a position names the same step in every call (GEN-001, DMC-005).

| Position | Code | Step |
|---|---|---|
| `[]` | `trace` | The root of every trace |
| `[1]` | `spf.helo` | The SPF check of the HELO identity (SPF-002, RFC 7208 §2.3) |
| `[2]` | `spf.mail_from` | The SPF check of the MAIL FROM identity (SPF-001, RFC 7208 §2.4) |
| `[3]` | `dkim.verify` | DKIM verification, with each signature as a child in the order of the header section |
| `[4]` | `dkim2.verify` | DKIM2 verification |
| `[5]` | `dmarc` | DMARC evaluation |
| `[6]` | `dkim.sign` | DKIM signing, with each signature it makes as a child |
| `[7]` | `dkim2.sign` | DKIM2 signing |

The entries for these checks and for the steps below them come with each check's scenarios.

## Codes

### `trace`

| Field | Value |
|---|---|
| Kind | `fact` |
| Position | `[]` |
| Parameters | `level`: keyword, `summary` or `full`; `events`: count; `cut_short`: boolean |
| Outcomes | none |
| Cites | none |
| English | "Trace at the {level} level, with {events} events." When `cut_short` is true, the renderer adds "A size limit cut the trace short." |

## Hints

A hint is a labeled guess about the cause of a result, and it never changes the result (TRC-003). Each hint sits under the step that recorded the fact it rests on, and fires only when that fact holds. The first release has these five.

### `dkim.hint.body_changed`

| Field | Value |
|---|---|
| Kind | `hint` |
| Position | A child of the body hash step of a DKIM signature |
| Parameters | none |
| Outcomes | none |
| Cites | `RFC 6377 §3.3`, `RFC 7960 §3.1.2.1`, `RFC 7960 §3.2.3` |
| English | "A mediator may have changed the body after signing, such as a mailing list that added or removed lines or MIME parts, or an MTA that re-encoded 8-bit content as 7-bit." |
| Requirement | DKV-013 |

### `dkim.hint.list_header_change`

| Field | Value |
|---|---|
| Kind | `hint` |
| Position | A child of the signature check step of a DKIM signature |
| Parameters | `fields`: list of keywords, from `subject`, `reply-to` and `sender`, in that order |
| Outcomes | none |
| Cites | `RFC 6377 §3.3`, `RFC 7960 §3.2.3` |
| English | "A mailing list may have changed a signed header field: {fields}." |
| Requirement | DKV-012 |

### `dkim.hint.simple_header_change`

| Field | Value |
|---|---|
| Kind | `hint` |
| Position | A child of the signature check step of a DKIM signature |
| Parameters | none |
| Outcomes | none |
| Cites | `RFC 6376 §3.4`, `RFC 7960 §2.3`, `RFC 7960 §3.1.2.2` |
| English | "A relay may have refolded, re-spaced or rewritten a signed header field, which simple header canonicalization doesn't tolerate." |
| Requirement | DKV-014 |

### `dkim2.hint.quoted_local_part`

| Field | Value |
|---|---|
| Kind | `hint` |
| Position | A child of the chain-of-custody step that compared the two paths |
| Parameters | `recorded`: data, from the sender, the local part that `mf=` or `rt=` recorded; `envelope`: data, from the sender, the local part in MAIL FROM or RCPT TO |
| Outcomes | none |
| Cites | `RFC 5321 §4.1.2`, `draft-ietf-dkim-dkim2-spec-09 §14.4` |
| English | "The local parts {recorded} and {envelope} differ only in quoting, and RFC 5321 §4.1.2 treats them as one mailbox." |
| Requirement | ADS-024 |

### `dmarc.hint.forwarded`

| Field | Value |
|---|---|
| Kind | `hint` |
| Position | A child of the alignment step of the DMARC check |
| Parameters | `spf`: position, the SPF result for MAIL FROM; `dkim`: position, the aligned DKIM signature that passed |
| Outcomes | none |
| Cites | `RFC 7960 §2.2`, `RFC 7960 §3.2.1` |
| English | "The message may have been forwarded by a mediator that kept the original MAIL FROM: SPF failed, but an aligned DKIM signature passed." |
| Requirement | DMC-009 |
