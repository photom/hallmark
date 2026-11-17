# ADR 0044: A DKIM signature that breaks the `s` flag of its key gets PERMFAIL (domain mismatch)

- Status: Accepted
- Date: 2026-11-16
- Affects: DKIM verification, the DKIM key record parser, the DKIM result, and the trace

## Context

The `t=` tag of a DKIM key record holds flags (RFC 6376 §3.6.1). It differs from the `t=` tag of a DKIM-Signature field, which holds the signature's timestamp (§3.5). One flag is `s`: "Any DKIM-Signature header fields using the "i=" tag MUST have the same domain value on the right-hand side of the "@" in the "i=" tag and the value of the "d=" tag. That is, the "i=" domain MUST NOT be a subdomain of "d="." §3.10 says the same: with the `s` flag, "the domain of the AUID ("i=" flag) MUST be the same as that of the SDID (d=) domain".

RFC 6376 names no result for a signature that breaks this rule. The key checks of §6.1.2 end with the hash algorithms and the key type, and don't mention the flag. §6.1.1 has a close rule for every signature: the `d=` domain must be the same as or a parent of the `i=` domain, and if not, "the Verifier should return PERMFAIL (domain mismatch)". The `s` flag narrows that rule to "the same".

## Options considered

- (a) PERMFAIL (domain mismatch), the result §6.1.1 gives when `i=` and `d=` don't fit.
- (b) Treat the key record as one that doesn't apply to the signature, so a selector with no other record gives PERMFAIL (no key for signature).
- (c) Verify as usual, and add a caution.

## Decision

Option (a).

- If the key record's `t=` lists `s`, and the domain of the signature's `i=` isn't the same as `d=`, the signature gets PERMFAIL (domain mismatch).
- The domains compare in their lookup form without regard to case (ADS-003). A signature with no `i=` takes the default `@` followed by `d=` (§3.5), so it can't break the rule.
- The trace records the flag, both domains and the result, and cites RFC 6376 §3.6.1 and §3.10.
- With several key records at the selector, the record with the `s` flag gets this reason in the list of ADR 0025, and another record can still verify the signature.

Option (b) would report that no key exists when a key does, and the result would no longer tell the two cases apart. Option (c) would pass a signature that §3.6.1 forbids with a MUST.

## Consequences

Positive:
- A domain that sets the `s` flag keeps subdomain `i=` values from passing, as §3.6.1 intends.
- The result uses a reason that RFC 6376 already defines for a mismatch between `i=` and `d=`.

Negative:
- RFC 6376 doesn't give this reason for this case, so another verifier may report a different reason for the same signature.

## References

- [RFC 6376 §3.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.5), [§3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1), [§3.10](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.10), [§6.1.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.1) and [§6.1.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.2)
- [ADR 0025](0025-a-dkim-selector-with-several-key-records-passes-if-any-of-them-verifies-the-signature.md)
- [requirements.md: DKIM verification, DNS records](../reference/requirements.md#dns-records)
