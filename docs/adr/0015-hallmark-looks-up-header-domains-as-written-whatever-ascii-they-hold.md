# ADR 0015: hallmark looks up header domains as written, whatever ASCII they hold

- Status: Accepted
- Date: 2026-11-13
- Affects: domain names from header fields (ADS-004), DMARC and DKIM2

## Context

The two address grammars allow different characters in a domain:
- RFC 5322 §3.4.1 builds a header domain from atext, which includes characters such as `_`, `+` and `!`. A From address such as `a@foo_bar.invalid` is valid in a message. The same section notes that implementations must "conform to the syntax of addresses for the context in which they are used".
- RFC 5321 §4.1.2 allows only letters, digits and hyphens in an envelope domain, and says that "the underscore character is not permitted".

DNS itself doesn't restrict labels to those characters. RFC 2181 §11 says that apart from the length limits, "any binary string whatever can be used as the label of any resource record". RFC 9989 doesn't restrict the characters of an author domain.

## Options considered

- (a) Look up a header domain as written, subject only to the DNS length limits.
- (b) Require letters, digits and hyphens in header domains too, and treat anything else as malformed.

## Decision

Option (a).

- A header domain whose ASCII labels hold characters other than letters, digits and hyphens keeps them in its lookup form, and checks use it as written. Non-ASCII labels follow ADR 0013.
- Envelope domains keep the grammar of RFC 5321 §4.1.2, so an envelope domain with an underscore fails to parse.
- The trace records which grammar hallmark read each domain under.

Option (b) would turn a valid message into a malformed one, and the RFC that governs header fields doesn't call for it.

## Consequences

Positive:
- No From field loses its DMARC evaluation only because of the characters in its domain.
- Each grammar applies where its RFC says it applies.

Negative:
- A header domain and an envelope domain follow different character rules, so the same text can be valid in one and not in the other. The trace shows which rule applied.

## References

- [RFC 5322 §3.4.1](https://www.rfc-editor.org/rfc/rfc5322.html#section-3.4.1)
- [RFC 5321 §4.1.2](https://www.rfc-editor.org/rfc/rfc5321.html#section-4.1.2)
- [RFC 2181 §11](https://www.rfc-editor.org/rfc/rfc2181.html#section-11)
- [requirements.md: Addresses and domain names](../reference/requirements.md#addresses-and-domain-names)
