# ADR 0030: A last header line without a line ending is a header field that canonicalization ends with CRLF

- Status: Accepted
- Date: 2026-11-15
- Affects: the header section parser, every header canonicalization of DKIM and DKIM2, and the trace

## Context

The header section ends at the first empty line, or at the end of the input when the message has no body (HDR-001, RFC 5322 §3.5). RFC 5322 §2.2 says: "Header fields are lines beginning with a field name, followed by a colon (":"), followed by a field body, and terminated by CRLF." The input can still end in the middle of its last line, with no line ending. A header-only message pasted into the demo without a final newline does that, and so does a file saved without one.

[ADR 0010](0010-header-parsers-follow-the-abnf-and-never-repair-input.md) says that hallmark never repairs input that matches no grammar, and that a line that isn't a header field stays a malformed line.

The canonicalizations expect a CRLF at the end of every field:

- RFC 6376 §3.7, step 1 of the header hash: "Each header field MUST be terminated with a single CRLF."
- RFC 6376 §3.4.2 and draft-09 §6.2: "Implementations MUST NOT remove the CRLF at the end of the header field value."
- Draft-09 §12.8: "The colon separator and the CRLF MUST be retained."

For the body, both specifications add a missing CRLF at the end: "If there is no body or no trailing CRLF on the message body, a CRLF is added" (RFC 6376 §3.4.3, draft-09 §6.1). Neither says anything about the last header field.

## Options considered

- (a) Treat the line as a header field, keep its raw bytes, and end its canonical forms with CRLF.
- (b) Treat it as a malformed line under ADR 0010.
- (c) Treat it as a header field and hash it without a CRLF.

## Decision

Option (a).

- The last line of the input is a header field if it matches the field grammar apart from its missing line ending. Its raw bytes stay as they arrived.
- Each canonical form ends the field with one CRLF: DKIM `simple` and `relaxed`, the DKIM2 header hash and the DKIM2 signing input. The DKIM-Signature field being verified or created gets none, because RFC 6376 §3.7 hashes that field without a trailing CRLF in any case.
- The parser records the missing line ending as a fact with its offset, citing RFC 5322 §2.2, and the trace records the CRLF that canonicalization added.
- This decision covers only the last line of the input. ADR 0010 governs every other line, and a line that breaks the field grammar in any other way stays a malformed line.

Under option (b), DKIM would never select the field, DMARC would lose a From field that comes last, and DKIM2 would leave the line out of its header hash ([ADR 0029](0029-the-dkim2-header-hash-leaves-out-lines-that-are-not-header-fields.md)), although the line holds a valid name, colon and value. Under option (c), the hash would break the rule of §3.7 quoted above and would match no signer that saw the line with its line ending.

## Consequences

Positive:
- A message pasted without its final newline gives the same hashes as the message that was signed.
- The trace shows that the line ending was missing and that hallmark added a CRLF to the canonical form.

Negative:
- hallmark hashes a CRLF that isn't in the input.
- A verifier that treats the line as malformed gets different results, for DMARC too if the line is the From field.

## References

- [RFC 5322 §2.2 and §3.5](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2)
- [RFC 6376 §3.4.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.2), [§3.4.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.3) and [§3.7](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.7)
- [draft-ietf-dkim-dkim2-spec-09 §6.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-6.1), [§6.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-6.2) and [§12.8](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-12.8)
- [requirements.md: Header parsing, Header canonicalization](../reference/requirements.md#header-canonicalization)
- [ADR 0010](0010-header-parsers-follow-the-abnf-and-never-repair-input.md)
