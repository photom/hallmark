# ADR 0026: A DKIM selector with several key records passes if any of them verifies the signature

- Status: Accepted
- Date: 2026-11-14
- Affects: DKIM verification, the DKIM result, the limits of SAF-002, and the trace

## Context

A DKIM key lives in a TXT record at `<selector>._domainkey.<domain>` (RFC 6376 §3.6.2.1). A domain can publish more than one TXT record there, and RFC 6376 gives two answers for that case. §3.6.2.2 says: "TXT RRs MUST be unique for a particular selector name; that is, if there are multiple records in an RRset, the results are undefined." Step 4 of §6.1.2 says:

> If the query for the public key returns multiple key records, the Verifier can choose one of the key records or may cycle through the key records, performing the remainder of these steps on each record at the discretion of the implementer. The order of the key records is unspecified.

RFC 8463 §6 adds that "there can only be a single key record in the DNS for each selector". dns-03 §3, which would update RFC 6376 (§6), drops records whose `v=` isn't `DKIM1` and says: "If more than one record then remains, this is an error." For DKIM2, draft-09 §14.5 makes that a PERMERROR with the text `public key <s1> has multiple records`.

DNS gives the records of an answer in no fixed order, and two resolvers can return the same records in different orders. hallmark's results must not depend on that order (TRC-012).

## Options considered

- (a) Use the first record in the answer.
- (b) Try each record, and pass if any of them verifies the signature.
- (c) Give PERMFAIL, as dns-03 §3 and the DKIM2 rule do.

## Decision

Option (b), for DKIM. DKIM2 keeps the PERMERROR of draft-09 §14.5.

- hallmark first drops each record that begins with a `v=` tag whose value isn't `DKIM1` (RFC 6376 §3.6.1, dns-03 §3).
- If more records remain than the limit of key records per selector (SAF-002), the signature gets PERMFAIL before hallmark tries any of them, as SAF-002 does for every limit. A check that tried only some of them would depend on their order.
- Otherwise hallmark runs the rest of §6.1.2 and §6.1.3 with each record. The signature passes if any record verifies it. If none does, the signature fails, and the result lists each record with its own reason, such as a key syntax error or a revoked key.
- If several records verify the signature, the result lists all of them. The signature counts as testing (DKV-009) if any of them has `t=y`.
- The trace records every record and its outcome, and adds a caution (TRC-003) that cites RFC 6376 §3.6.2.2 and dns-03 §3.

## Consequences

Positive:
- The result is the same whatever order the resolver gives the records in.
- A domain that publishes an old and a new key at one selector during a key change still gets its signatures verified.

Negative:
- hallmark may verify one signature several times, up to the limit.
- A verifier that follows dns-03 §3 returns PERMFAIL for the same message.
- DKIM and DKIM2 treat the same records in different ways.

## References

- [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1), [§3.6.2.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.2.1), [§3.6.2.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.2.2) and [§6.1.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.2)
- [RFC 8463 §6](https://www.rfc-editor.org/rfc/rfc8463.html#section-6)
- [draft-ietf-dkim-dkim2-dns-03 §3](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-3) and [§6](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-6)
- [draft-ietf-dkim-dkim2-spec-09 §14.5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.5)
- [requirements.md: DKIM verification, DNS records](../reference/requirements.md#dns-records)
