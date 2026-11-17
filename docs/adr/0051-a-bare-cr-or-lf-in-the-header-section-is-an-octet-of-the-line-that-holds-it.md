# ADR 0051: A bare CR or LF in the header section is an octet of the line that holds it

- Status: Accepted
- Date: 2026-11-17
- Affects: the header section parser, fields from a milter, header canonicalization for DKIM and DKIM2, the trace

## Context

[ADR 0037](0037-the-core-takes-crlf-line-endings-only-and-a-bare-cr-or-lf-in-the-body-is-an-octet.md) has the core take CRLF line endings only and makes a bare CR or LF in the body an octet of the body. It left a bare CR or LF inside the header section open.

RFC 5321 §2.3.8 says that conforming implementations "MUST NOT recognize or generate any other character or character sequence as a line terminator" than CRLF. RFC 5322 §2.2 says a field body "MUST NOT include CR and LF except when used in folding and unfolding". Its obsolete syntax, which §4 says a receiver "MUST" accept and parse, adds bare CR, bare LF and NUL to unstructured field bodies (`obs-unstruct`, §4.1), and to quoted pairs (`obs-qp`). §4.1 notes that bare CR and LF appear either as wrong line separators or as plain control characters.

RFC 6376 §5.3 and draft-09 §17 tell a signer to convert bare CR and LF to CRLF before it signs. No specification tells a verifier to convert anything. The `relaxed` header algorithm of RFC 6376 §3.4.2 unfolds only CRLF followed by WSP, and its other steps touch only WSP.

Example: the header section `A: 1\nB: 2\r\n\r\n` holds a bare LF after `A: 1` (scenario FLD-052).

## Options considered

- (a) A bare CR or LF is an octet of the line that holds it. It ends no line and starts no field.
- (b) A line that holds a bare CR or LF is a malformed line, as a line that isn't a header field is (HDR-004).
- (c) A bare CR or LF ends a line, as CRLF does.
- (d) The core converts each bare CR or LF to CRLF before it parses.

## Decision

Option (a).

- The header section parser splits lines at CRLF only. A bare CR or LF is an octet of its line, so `A: 1\nB: 2\r\n` is one field named `A` whose value is `1\nB: 2`.
- In a field name, a bare CR or LF breaks the `field-name` rule, so the line is a malformed line (HDR-004). A line that starts with a bare CR or LF is a malformed line too, because neither byte is WSP, so the line can't continue a folded field.
- In a field value, the octet stays. An unstructured field takes it through `obs-unstruct`. The parser of a structured field, such as From or DKIM-Signature, reads it by that field's grammar, which allows it only inside a quoted pair (`obs-qp`), if at all, so elsewhere it is a parse error with the rule's usual outcome.
- Canonicalization passes it through unchanged, because it is neither CRLF nor WSP. `simple` keeps it, `relaxed` keeps the WSP before it (RFC 6376 §3.4.2), the DKIM2 header hash takes it in the `relaxed` form (draft-09 §6.2), and the DKIM2 signing input keeps it while it deletes all WSP (§12.8). A last line that ends with a bare CR and no line ending gets a CRLF after it, as HCN-008 says: `A: 1\r` becomes `A: 1\r\r\n`.
- A field from a milter follows the same rule. The check rebuilds the raw bytes from the value and the form the caller states (STR-008), and a CR or LF that isn't part of a line break of that form stays an octet.
- The trace records a caution (TRC-003) with the count and the first places of bare CR and LF in the header section, citing RFC 5322 §2.2 and RFC 5321 §2.3.8. In signing, the caution also cites RFC 6376 §5.3 and draft-09 §17. The caution changes no result.

Option (b) disagrees with `obs-unstruct`, which a receiver must accept, and a signer that hashed such a field as a field gets a different header hash. Option (c) breaks the MUST NOT of RFC 5321 §2.3.8 and splits one field into two. Option (d) hashes bytes that the message doesn't hold and repairs input, which [ADR 0010](0010-header-parsers-follow-the-abnf-and-never-repair-input.md) rules out.

## Consequences

Positive:
- The header section and the body follow one rule for bare CR and LF.
- `simple` canonicalization hashes the bytes as they arrived, as RFC 6376 §3.4.1 requires.
- If a bare CR or LF breaks a signature, the trace shows where it sits.

Negative:
- A bare LF that a broken sender meant as a line ending joins two fields into one, so the second field is invisible to `h=` selection and to the parsers.
- In the LF form of a milter field, an LF followed by WSP counts as a fold, so a bare LF followed by WSP in the original comes back as CRLF. The trace records that assumption (STR-008).

## References

- [RFC 5321 §2.3.8](https://www.rfc-editor.org/rfc/rfc5321.html#section-2.3.8)
- [RFC 5322 §2.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2), [§4](https://www.rfc-editor.org/rfc/rfc5322.html#section-4) and [§4.1](https://www.rfc-editor.org/rfc/rfc5322.html#section-4.1)
- [RFC 6376 §3.4.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.1), [§3.4.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.2) and [§5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.3)
- [draft-ietf-dkim-dkim2-spec-09 §6.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-6.2), [§12.8](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-12.8) and [§17](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-17)
- [ADR 0010](0010-header-parsers-follow-the-abnf-and-never-repair-input.md), [ADR 0037](0037-the-core-takes-crlf-line-endings-only-and-a-bare-cr-or-lf-in-the-body-is-an-octet.md)
- [requirements.md: Header parsing](../reference/requirements.md#header-parsing)
