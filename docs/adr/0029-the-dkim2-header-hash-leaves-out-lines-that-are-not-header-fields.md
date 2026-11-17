# ADR 0029: The DKIM2 header hash leaves out lines that are not header fields

- Status: Accepted
- Date: 2026-11-15
- Affects: the DKIM2 header hash in verification and signing, and the trace

## Context

The header section parser keeps a line that isn't a header field as a separate malformed line with its raw bytes, and never joins it to the field before it (HDR-004, [ADR 0010](0010-header-parsers-follow-the-abnf-and-never-repair-input.md)). Examples are a line with no colon, a field name with a byte outside printable US-ASCII (RFC 5322 §2.2), and a first line that begins with white space and so continues no field.

DKIM picks the fields it hashes by name, through `h=`, so such a line never enters a DKIM hash. DKIM2 hashes every header field of the message except those that §4 and §6.2 ignore (draft-09 §6.2). Draft-09 §2.7 defines the term: "As defined in [RFC5322], a header field is a single logical line in the message header consisting of a field name, a colon, and a field body (value)." The steps of §6.2 work on field names: they lowercase them and sort the fields by them. The draft says nothing about a line with no field name. A header hash that doesn't match gives FAIL (§14.7).

## Options considered

- (a) Leave the line out of the header hash, and record a caution.
- (b) Fail the header hash of any message that has such a line.
- (c) Hash the line, in its raw or relaxed form, at some place in the sorted list.

## Decision

Option (a).

- Verification and signing leave each line that isn't a header field out of the DKIM2 header hash. Header recipes number fields by name (§5.1), so a header set that a recipe rebuilds never holds such a line either.
- The trace records a caution (TRC-003) for each line left out, with its raw bytes marked as a value the sender controls (TRC-013), citing draft-09 §2.7 and §6.2.
- The parser's error for the line (HDR-010) stays in the trace as before.

Option (b) would stop a signer from signing such a message at all, and the draft gives no error text for that case. Option (c) needs a place in a list sorted by field name for a line that has no field name, and the draft defines none, so two implementations would choose different places.

## Consequences

Positive:
- The header hash follows §6.2 as written.
- A message with one malformed line still gets a DKIM2 result.
- DKIM and DKIM2 treat the line alike: neither hashes it.

Negative:
- A line that isn't a header field can be added, changed or removed after signing without changing the DKIM2 header hash. The caution shows each such line, and the caller decides what it means (GEN-006).
- A verifier that hashes the line gets a different header hash for the same message.

## References

- [draft-ietf-dkim-dkim2-spec-09 §2.7](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-2.7), [§4](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-4), [§5.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-5.1), [§6.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-6.2) and [§14.7](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.7)
- [RFC 5322 §2.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2)
- [requirements.md: Header parsing, Header canonicalization](../reference/requirements.md#header-canonicalization)
- [ADR 0010](0010-header-parsers-follow-the-abnf-and-never-repair-input.md)
