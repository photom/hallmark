# ADR 0024: DMARC follows RFC 9989 §4.10.1 for invalid policy tags and §4.7 for nonexistent subdomains

- Status: Accepted
- Date: 2026-11-14
- Affects: the DMARC record parser, the DMARC result, and the trace

## Context

Three passages of RFC 9989 disagree about a record whose policy tags are missing or invalid. The `p` tag in §4.7 says: "If this tag is not present in an otherwise syntactically valid DMARC Policy Record, then the record is treated as if it included "p=none"". §4.8 says: "Syntax errors in the remainder of the record MUST be discarded in favor of default values (if any) or ignored outright." §4.10.1 says:

> If a retrieved DMARC Policy Record does not contain a valid "p" tag, or contains an "sp" or "np" tag that is not valid, then:
> - If a "rua" tag is present and contains at least one syntactically valid reporting URI, the Mail Receiver MUST act as if a record containing "p=none" was retrieved and continue processing.
> - Otherwise, the Mail Receiver applies no DMARC processing to this message.

For `v=DMARC1` alone, §4.7 applies DMARC with `p=none`, and §4.10.1 applies no DMARC. For `v=DMARC1; p=reject; sp=bogus` with no `rua`, §4.8 ignores the bad `sp` and keeps `reject`, and §4.10.1 applies no DMARC. Erratum 9236 reports the first conflict and erratum 9237 the second. Both are in the Reported state, so nobody has confirmed a correction.

A third passage concerns a nonexistent subdomain under a record with `sp` and no `np`. §4.7 says that if `np` is absent, "the policy specified by the "sp" tag (if the "sp" tag is present) or the policy specified by the "p" tag (if the "sp" tag is not present) MUST be applied for non-existent subdomains." §4.10.1 says "In the absence of applicable "sp" or "np" tags, the "p" tag policy is used for subdomains", which a reader can take to mean `p` whenever `np` is absent. Erratum 9235, also Reported, describes this.

RFC 7489 has the first two conflicts with SHOULD in place of MUST: §6.3 ("Syntax errors in the remainder of the record SHOULD be discarded") against step 6 of §6.6.3, which covers `p` and `sp`. RFC 7489 has no `np`.

## Options considered

For a missing or invalid `p`, and an invalid `sp` or `np`:
- (a) §4.10.1 governs these three tags, and §4.8 governs every other tag.
- (b) §4.7 and §4.8 govern: a missing `p` means `none`, and an invalid `sp` or `np` is ignored. §4.10.1 applies only to a `p` with an invalid value.
- (c) A record with any invalid policy tag counts as no record.

For a nonexistent subdomain with no `np`:
- (a) Follow §4.7: `sp`, then `p`.
- (b) Read §4.10.1 as going from `np` straight to `p`.

## Decision

Option (a) for both. §4.10.1 names the three policy tags, so it is the more specific rule, and §4.7 states the order for nonexistent subdomains with a MUST.

- A missing `p`, a `p` with an invalid value, or an `sp` or `np` with an invalid value starts the §4.10.1 rule. ADR 0025 says which values are invalid.
- A "syntactically valid reporting URI" is an entry of `rua` that matches `dmarc-uri` or `obs-dmarc-uri` in §4.8: an RFC 3986 URI in which commas and exclamation marks are percent-encoded, optionally followed by an obsolete size. hallmark checks the syntax only and sends no reports.
- If `rua` holds at least one such URI, the policy that applies is `none`, whichever of `p`, `sp` and `np` would have applied. The record's other valid tags keep their values, so `adkim`, `aspf`, `t` and `psd` still count. If `rua` holds none, the result says that DMARC was not applied and why.
- For a nonexistent subdomain, the policy comes from `np`, or from `sp` if `np` is absent, or from `p` if both are absent.
- The result lists every policy tag as published, invalid text included, and names the rule that set the policy (DMC-006).
- RFC 7489 mode takes the same rule from step 6 of §6.6.3 for `p` and `sp`.
- If the errata are verified or rejected, a new ADR replaces this one.

## Consequences

Positive:
- One rule covers all three policy tags, in both modes.
- The result and the trace show the published values, so a caller that reads the record another way can see why hallmark chose its policy.

Negative:
- For `v=DMARC1; p=reject; sp=bogus`, hallmark applies no DMARC without `rua`, and `none` with it, though the domain owner published `reject`. A verifier that follows §4.8 applies `reject`.
- A record that is only `v=DMARC1` gives no DMARC result, while §4.7 alone would apply `p=none`.
- hallmark needs an RFC 3986 URI parser, though it sends no reports.

## References

- [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7), [§4.8](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.8) and [§4.10.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10.1)
- [RFC 9989 errata 9235](https://www.rfc-editor.org/errata/eid9235), [9236](https://www.rfc-editor.org/errata/eid9236) and [9237](https://www.rfc-editor.org/errata/eid9237), all Reported
- [RFC 7489 §6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.3) and [§6.6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.6.3)
- [RFC 3986 §3](https://www.rfc-editor.org/rfc/rfc3986.html#section-3)
- [ADR 0020](0020-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md), the two DMARC modes
- [ADR 0025](0025-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md), the DMARC record syntax
- [requirements.md: DMARC, DNS records](../reference/requirements.md#dns-records)
