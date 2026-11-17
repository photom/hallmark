# ADR 0046: DMARC asks whether the author domain exists only when `np` can change the policy

- Status: Superseded by ADR 0050
- Date: 2026-11-16
- Affects: DMARC policy discovery in RFC 9989 mode, the check's DNS questions, the trace, and snapshots

## Context

RFC 9989 §4.7 defines the `np` tag, the policy for subdomains that don't exist. §3.2.13 says that a domain doesn't exist when "the response code received for a query for a domain name is NXDOMAIN", and that its subdomains don't exist either, as in RFC 8020. Appendix A.4 adds that any resource record makes a domain exist, and that a NODATA response means the name exists without records of the type asked. RFC 9989 doesn't say which query a receiver uses to test the author domain.

The questions of policy discovery don't answer it. They ask for TXT records at `_dmarc.` followed by the author domain and its parents (§4.10). An NXDOMAIN for `_dmarc.random123.void.invalid` says only that this name doesn't exist, and nothing about `random123.void.invalid`.

The test matters only in some cases. Under ADR 0023, a subdomain that exists takes `sp`, then `p`, and a subdomain that doesn't exist takes `np`, then `sp`, then `p`. If the DMARC Policy Record has no valid `np`, both cases take the same tag. If policy discovery finds a record at the author domain itself, that domain exists. RFC 7489 has no `np`.

RFC 9989 erratum 8995, still Reported, proposes that a DNSSEC compact denial (RFC 9824) also proves that a name doesn't exist. [hallmark#3](https://github.com/photom/hallmark/issues/3) tracks it.

## Options considered

- (a) Ask one question at the author domain, only when the record that policy discovery found sits above the author domain and has a valid `np`.
- (b) Ask the question for every message, next to the first `_dmarc` question, once the header fields are complete.

For the type of the question: A, AAAA, MX or TXT. Any of them tells NXDOMAIN from NODATA.

## Decision

Option (a), with an A question.

- In RFC 9989 mode, the check asks for A records at the author domain only when policy discovery found the DMARC Policy Record at a name above the author domain, and that record has a valid `np`. Otherwise `np` can't change the policy, and the check asks nothing.
- An answer that the name doesn't exist (DNS-002) means the author domain doesn't exist, and `np` applies. An answer with records, or with no records of that type, means the author domain exists, and `sp` or `p` applies (ADR 0023).
- A temporary failure for the question gives `temperror`, as a temporary failure during policy discovery does (GEN-005).
- hallmark counts only NXDOMAIN as proof, as RFC 9989 §3.2.13 says, until erratum 8995 is settled.
- The trace records the question, the answer and whether the author domain exists. A snapshot keeps the answer like any other.
- RFC 7489 mode asks no such question.

Option (b) adds a question to nearly every message, though the answer changes the policy only for records with `np`. Option (a) costs one more round of questions, after policy discovery, for the messages that need it. Resolvers often hold A records in their caches, so the question costs little.

## Consequences

Positive:
- `np` applies as RFC 9989 defines it, with a question whose answer the trace shows.
- Most messages send no extra question.

Negative:
- A message whose policy depends on `np` waits for one more round of DNS.
- Under a wildcard record, every subdomain exists in DNS, so `np` never applies there.

## References

- [RFC 9989 §3.2.13](https://www.rfc-editor.org/rfc/rfc9989.html#section-3.2.13), [§4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7), [§4.10](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10) and [Appendix A.4](https://www.rfc-editor.org/rfc/rfc9989.html#appendix-A.4)
- [RFC 8020 §2](https://www.rfc-editor.org/rfc/rfc8020.html#section-2)
- [RFC 9989 erratum 8995](https://www.rfc-editor.org/errata/eid8995) and [hallmark#3](https://github.com/photom/hallmark/issues/3)
- [ADR 0023](0023-dmarc-follows-rfc-9989-section-4-10-1-for-invalid-policy-tags-and-section-4-7-for-nonexistent-subdomains.md)
- [requirements.md: DNS, DNS records](../reference/requirements.md#dns-records)
