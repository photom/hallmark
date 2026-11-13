# ADR 0014: A From domain literal gives DMARC no author domain

- Status: Accepted
- Date: 2026-11-13
- Affects: DMARC evaluation (DMC-002, ADS-005) and the trace

## Context

A From address can name a host by its IP address, as in `From: a@[192.0.2.1]` (RFC 5322 §3.4.1).

DMARC needs a domain. RFC 9989 §5.3.1 extracts "the domain in the RFC5322.From header field", and policy discovery queries the name made by "prepending the label "_dmarc" to the Author Domain" (§4.10.1). hallmark can't build that name from a literal, and neither RFC 9989 nor RFC 7489 §6.6.1 says what to do with one.

RFC 9989 §5.3.1 covers the case where no domain is found: "DMARC validation is not possible and the process terminates". Draft-09 §11 says a From header field with a domain literal can never be Authentic in DKIM2.

## Options considered

- (a) A domain literal counts as no domain, so a From field with only literals gives DMARC no author domain.
- (b) Report DMARC `permerror`.
- (c) Report DMARC `fail`.

## Decision

Option (a).

- A mailbox whose domain is a literal contributes no domain to the author domain count of DMC-002.
- If the From field holds no other domain, hallmark doesn't apply DMARC, the same as for a From field with no domain at all, and the trace says that the From address uses a domain literal.

Options (b) and (c) would invent results that neither DMARC specification defines for this case.

## Consequences

Positive:
- A domain literal gets the same handling as any From field that names no domain, as RFC 9989 §5.3.1 defines it.
- The trace states the reason, so the caller can decide what to do with the message.

Negative:
- A message whose From field holds only a literal escapes DMARC, as a message with no From domain does. hallmark leaves the disposition of such messages to the caller.

## References

- [RFC 5322 §3.4.1](https://www.rfc-editor.org/rfc/rfc5322.html#section-3.4.1)
- [RFC 9989 §4.10.1 and §5.3.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-5.3.1)
- [RFC 7489 §6.6.1](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.6.1)
- [draft-ietf-dkim-dkim2-spec-09 §11](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-11)
- [requirements.md: Addresses and domain names](../reference/requirements.md#addresses-and-domain-names)
