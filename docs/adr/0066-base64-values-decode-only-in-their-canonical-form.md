# ADR 0066: base64 values decode only in their canonical form

- Status: Accepted
- Date: 2026-11-25
- Affects: the base64 decoder in `crates/hallmark`, and every DKIM and DKIM2 tag whose value is base64, such as `b=`, `bh=` and the `p=` of a key record

## Context

DKIM and DKIM2 carry signatures, body hashes and public keys as base64. Neither specification defines the algorithm; each one points to another RFC and adds a syntax:

- RFC 6376 §3.2 defines "base64" text as in RFC 2045 §6.8. §2.10 gives the ABNF `base64string = ALPHADIGITPS *([FWS] ALPHADIGITPS) [ [FWS] "=" [ [FWS] "=" ] ]`, and §3.5 says white space in `b=` and `bh=` is ignored.
- Draft-09 §2.14 says that base64strings "are defined in [RFC4648]", repeats the ABNF, and adds: "A base64string MUST be padded with zero bits and trailing = characters provided if needed." FWS within a base64string "will be ignored".

The cited RFCs describe what an encoder writes. RFC 2045 §6.8 always completes the last group of four symbols: the zero bits "are added (on the right)", and `=` pads the group. RFC 4648 §3.2 says implementations "MUST include appropriate pad characters", and §3.5 says pad bits "MUST be set to zero by conforming encoders".

They leave most of a decoder's behavior open:

- No RFC says what a decoder does with a value that lacks its padding, such as `QQ` for the octet `A`. The ABNF makes `=` optional, because a value that fills its last group has none.
- RFC 4648 §3.5 lets decoders reject pad bits that aren't zero, and says the referring specification "may mandate a specific behaviour". `QR==` then decodes to the same octet as `QQ==`.
- RFC 4648 §3.3 says decoders reject characters outside the alphabet unless the referring specification says otherwise, while RFC 2045 §6.8 says decoders ignore them.

## Options considered

- (a) Decode only the canonical form: the padding that completes the last group, and pad bits of zero, in DKIM and DKIM2.
- (b) Decode what the bits allow: accept a value without padding and drop any pad bits, in both.
- (c) Follow each protocol's wording: require padding and zero pad bits in DKIM2, and accept both in DKIM.

## Decision

Option (a).

- The decoder implements the `base64string` rule of RFC 6376 §2.10 and draft-09 §2.14. It ignores one FWS between two symbols and one before each `=`, where an FWS is RFC 5322's without obs-FWS (RFC 6376 §2.8, draft-09 §2.12). Any other octet, an FWS with more than one CRLF, and an FWS before the first symbol or after the last symbol or `=` are syntax errors.
- A last group of two symbols ends with `==`, a last group of three ends with `=`, and a value whose symbols fill whole groups has no `=`. A missing or surplus `=`, or a last group of one symbol, is an error.
- The pad bits of the last symbol must be zero (RFC 4648 §3.5).
- The decoder checks the symbols and the FWS first, then the padding, then the pad bits. The first check that fails gives the error, with the offset of the first octet that broke its rule (PAR-001). The check then applies its protocol's rule for a malformed value, such as PERMFAIL (signature syntax error) for a DKIM `b=` or `bh=`, PERMFAIL (key syntax error) for a key record's `p=`, or PERMERROR for a DKIM2 field.
- A value holds at least one symbol, so an empty value is a syntax error. An empty `p=` means that the key has been revoked (RFC 6376 §3.6.1, dns-03 §4.3), and the check reads that case before it decodes.
- This decision covers tag values only. DKIM2 recipe strings (draft-09 §5) follow the same padding rules but allow no FWS, and trace documents write hashes as base64 inside JSON strings (RFC 4648 §4). Their readers decide how they hold these rules when they arrive.
- hallmark's signer writes padding and zero pad bits, so its output meets RFC 2045 and RFC 4648 alike. The trace format writes data as base64url without padding (RFC 4648 §5, TRC-007), a separate form that this decision doesn't cover.

Draft-09 §2.14 wants padding and zero pad bits so that two base64strings can be compared without decoding them. With one canonical form, two values that decode to the same octets are the same string. A DKIM value that lacks its padding or sets a pad bit can't come from an encoder that follows RFC 2045, so rejecting it follows ADR 0010, under which hallmark never repairs input. Option (c) would give one decoder two rules for the same bytes, and option (b) would let two strings stand for one value.

## Consequences

Positive:
- Each value has one encoding. A trace that records a base64 value shows the bytes that were decoded.
- DKIM and DKIM2 share one decoder with one rule set, and the rules can be tested and fuzzed together.

Negative:
- A signature or key that a lenient encoder wrote without padding, or with pad bits set, fails here and may pass at a verifier that accepts such values. The trace names the rule and the offset.

## References

- [RFC 6376 §2.8, §2.10, §3.2, §3.5 and §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-2.10)
- [draft-ietf-dkim-dkim2-spec-09 §2.12, §2.14 and §5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-2.14)
- [draft-ietf-dkim-dkim2-dns-03 §4.3](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-4.3)
- [RFC 2045 §6.8](https://www.rfc-editor.org/rfc/rfc2045.html#section-6.8)
- [RFC 4648 §3.2, §3.3, §3.5 and §4](https://www.rfc-editor.org/rfc/rfc4648.html#section-3.5)
- [ADR 0010](0010-header-parsers-follow-the-abnf-and-never-repair-input.md)
- [requirements.md: Header parsing](../reference/requirements.md#header-parsing)
