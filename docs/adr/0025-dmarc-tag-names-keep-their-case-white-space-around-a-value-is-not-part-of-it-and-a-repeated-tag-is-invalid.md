# ADR 0025: DMARC tag names keep their case, white space around a value is not part of it, and a repeated tag is invalid

- Status: Accepted
- Date: 2026-11-14
- Affects: the DMARC record parser and the trace

## Context

A DMARC record has two descriptions that don't agree on every point. RFC 9989 §4.7 says that records "follow the extensible "tag-value" syntax for DNS-based key records defined in DKIM [RFC6376]", and RFC 7489 §6.3 says the same. §4.8 then gives an ABNF of its own. Four cases fall between them.

Case of tag names. RFC 6376 §3.2 says "Tags MUST be interpreted in a case-sensitive manner." The ABNF of RFC 9989 §4.8 has `dmarc-tag = 1*ALPHA equals 1*dmarc-value` and a table that names the tags in lowercase. The ABNF of RFC 7489 §6.4 writes each name as a quoted string, such as `"p"`, and a quoted string ignores case (RFC 5234 §2.3). For values, RFC 6376 §3.2 says they are case-sensitive "unless the specific tag description of semantics specifies case insensitivity". The DMARC ABNF gives each value as quoted strings, such as `"none" / "quarantine" / "reject"`, and marks the one exception with RFC 7405 syntax: `dmarc-version = "v" equals %s"DMARC1" ; case sensitive`.

White space at the end of a value. RFC 9989 §4.8 allows white space around `=` and `;` (`equals = *WSP "=" *WSP`, `dmarc-sep = *WSP ";" *WSP`), and `dmarc-value = %x20-3A / %x3C-7E` includes the space. At the end of a record, `p=reject ` therefore has the value `reject `, which no rule accepts. A tab there matches nothing at all. RFC 6376 §3.2, as erratum 5070 corrects it, puts white space around a value outside the value: `tag-spec = [FWS] tag-name [FWS] "=" [FWS] [tag-value [FWS]]`.

A repeated tag. The ABNF allows a tag to appear twice. RFC 6376 §3.2 says "if a tag name does occur more than once, the entire tag-list is invalid." RFC 9989 discards a whole record only for its `v` tag (§4.7), and §4.8 says other syntax errors "MUST be discarded in favor of default values (if any) or ignored outright".

The place of `p` in RFC 7489. §6.3 says that "the "v" and "p" tags MUST be present and MUST appear in that order". The ABNF of §6.4 puts `dmarc-request` right after `dmarc-version`, with the comment that "components other than dmarc-version and dmarc-request may appear in any order". RFC 9989 has no rule about the place of `p`.

## Options considered

- Tag names: (a) case-sensitive in both modes, as RFC 6376 §3.2 says; (b) case-insensitive in both modes, as quoted strings in the RFC 7489 ABNF are; (c) case-sensitive in RFC 9989 mode and case-insensitive in RFC 7489 mode.
- White space at the edges of a value: (a) not part of the value, so a record may also end with white space; (b) part of the value, as the RFC 9989 ABNF reads.
- A repeated tag: (a) each copy is invalid and the rest of the record stands; (b) the whole record is invalid, as RFC 6376 §3.2 says; (c) the first copy wins; (d) the last copy wins.
- `p` in RFC 7489 mode: (a) anywhere after `v`; (b) only as the second tag.

## Decision

Option (a) in each case.

- Tag names are case-sensitive in both modes, so `P=reject` is an unknown tag and the record has no `p`.
- Keyword values ignore ASCII case: `p`, `sp`, `np`, `adkim`, `aspf`, `t`, `psd` and `fo`. The value of `v` must be exactly `DMARC1`.
- White space at either end of a value is not part of it, and a record may end with white space. White space inside a value stays.
- An empty value is invalid, because the ABNF asks for at least one character.
- If a tag appears more than once, every copy is invalid, and the tag counts as invalid. For `p`, `sp` and `np`, that starts the rule of ADR 0024. Other tags take their defaults. A second `v` tag doesn't change the version check of the first tag, and hallmark ignores it.
- In RFC 7489 mode, `p` may appear anywhere after `v`.
- The trace records each tag as published, with its byte range (PAR-002), and says why hallmark ignored or rejected it.

## Consequences

Positive:
- The tag names follow the DKIM tag-value syntax that both DMARC specifications point to, and the values follow the ABNF, which marks its only case-sensitive value.
- A trailing space or tab in a published record doesn't change the policy.
- No copy of a repeated tag wins by its position.

Negative:
- A verifier that matches tag names without regard to case reads `P=reject` as `reject`, while hallmark finds no `p`.
- A verifier that lets the first or last copy of a repeated tag win reads a policy where hallmark finds an invalid tag.
- In RFC 7489 mode, hallmark accepts a record whose `p` isn't the second tag, which the §6.4 ABNF doesn't allow.

## References

- [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7) and [§4.8](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.8)
- [RFC 7489 §6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.3) and [§6.4](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.4)
- [RFC 6376 §3.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.2) and [erratum 5070](https://www.rfc-editor.org/errata/eid5070)
- [RFC 5234 §2.3](https://www.rfc-editor.org/rfc/rfc5234.html#section-2.3) and [RFC 7405 §2.1](https://www.rfc-editor.org/rfc/rfc7405.html#section-2.1)
- [ADR 0024](0024-dmarc-follows-rfc-9989-section-4-10-1-for-invalid-policy-tags-and-section-4-7-for-nonexistent-subdomains.md), invalid policy tags
- [requirements.md: DMARC, DNS records](../reference/requirements.md#dns-records)
