# ADR 0028: A malformed DKIM2 Domain Policy Record gives PERMERROR

- Status: Accepted
- Date: 2026-11-14
- Affects: the DKIM2 From check, the Domain Policy Record parser, the DKIM2 result, and the trace

## Context

DKIM2 makes a From header field Authentic when its domain matches the signing domain under the relaxed domain match of draft-09 §7. If the match holds only because the signing domain is a subdomain of the From domain, the verifier fetches the From domain's Domain Policy Record (dns-03 §5, DK2-012). A `strict` flag in that record cancels the match.

dns-03 §5 says that TXT records at the name "that do not begin with v=DKIM1 are ignored. If more than one remains, this is an error." It also says: "If the "strict" flag is absent, or there is no f= tag, or there is no Domain Policy Record, subdomains are authorized." The record is a tag=value list with the rules of draft-09 §8 (dns-03 §2.4), so a repeated tag makes it invalid.

draft-09 §14.8 lists the outcomes of the From check. Among them are `TEMPERROR: DKIM2-Signature i=<x> policy for <value> unavailable` and `PERMERROR: DKIM2-Signature i=<x> multiple records for <value> policy`. It lists nothing for a record that begins with `v=DKIM1` and then breaks the tag=value rules. For key records, §14.5 says: "A key record that is malformed cannot be used and the result is a PERMERROR."

## Options considered

- (a) PERMERROR, as for a malformed key record (§14.5) and for several policy records (§14.8).
- (b) Treat the record as absent, so subdomains are authorized.
- (c) Treat the record as if it had the `strict` flag.

## Decision

Option (a).

- A Domain Policy Record is malformed if it begins with `v=DKIM1` and then breaks the tag=value rules of draft-09 §8 (HDR-006), or if its `f=` value doesn't match the ABNF of dns-03 §5.2. A flag hallmark doesn't know is not an error, because §5.2 says "Unrecognized flags are ignored."
- A malformed record gives PERMERROR for the signature whose From check fetched it.
- draft-09 §14.8 gives no text for this case, so the result carries none (DK2-013). The trace records the rule the record broke and that the draft has no text for it.
- An open item asks the ietf-dkim working group which outcome the draft intends.

## Consequences

Positive:
- A typo in a record that was meant to set `strict` doesn't let subdomains make the From field Authentic.
- A malformed policy record and a malformed key record give the same state.

Negative:
- A typo in such a record stops subdomain signers from making the From field Authentic until the owner fixes it.
- A verifier that ignores the malformed record returns a different result.
- The result has no human-readable text from the draft for this case.

## References

- [draft-ietf-dkim-dkim2-dns-03 §2.4](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-2.4), [§5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-5) and [§5.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-5.2)
- [draft-ietf-dkim-dkim2-spec-09 §7](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-7), [§8](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-8), [§14.5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.5) and [§14.8](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.8)
- [ADR 0006](0006-dkim2-targets-draft-09-behind-an-unstable-feature.md), the drafts hallmark targets
- [requirements.md: DKIM2, DNS records](../reference/requirements.md#dns-records)
