# ADR 0067: DKIM value keywords match only as written

- Status: Accepted
- Date: 2026-11-26
- Affects: the decoders of DKIM tag values in `crates/hallmark`, starting with `c=`, and every later DKIM tag of a signature or key record whose value is a keyword that RFC 6376 or an RFC that updates it defines, such as `a=` and `k=`

## Context

RFC 6376 gives two rules for the case of a keyword in a tag value, and they don't agree.

- The ABNF writes the keywords as quoted strings, such as `sig-c-tag-alg = "simple" / "relaxed" / x-sig-c-tag-alg` in §3.5. In ABNF, a quoted string matches without regard to case (RFC 5234 §2.3), so `Relaxed` and `SIMPLE` match the grammar.
- §3.2 says: "Values MUST be processed as case sensitive unless the specific tag description of semantics specifies case insensitivity." The description of `c=` in §3.5 says nothing about case, and neither do the descriptions of `a=` in §3.5 and of the key record's `k=` in §3.6.1.

The two readings give different outcomes. Under the ABNF, `c=Relaxed` names the relaxed algorithm. Under §3.2, it names no algorithm that RFC 6376 defines. It then matches only `x-sig-c-tag-alg`, and §3.4 says "Verifiers MUST ignore any signatures that use unrecognized canonicalization algorithms."

[ADR 0024](0024-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md) met the same conflict in DMARC. It followed RFC 6376 §3.2 for tag names, and let DMARC keyword values, such as `p=`, ignore case, because the DMARC ABNF marks its one case-sensitive value with the `%s` of RFC 7405: `dmarc-version = "v" equals %s"DMARC1"`. RFC 6376 marks no value that way, so the DMARC reasoning doesn't carry over to DKIM values.

## Options considered

- (a) Match DKIM value keywords only as written, as §3.2 says.
- (b) Match them without regard to case, as the ABNF quoted strings do.

## Decision

Option (a).

- A keyword in a DKIM tag value matches only when its octets equal the keyword as written in RFC 6376 or in an RFC that updates it, unless the tag's description says that the value is case-insensitive. RFC 8463 §4 adds `ed25519` to `sig-a-tag-k` and `key-k-tag-type` as a plain quoted string, so `a=Ed25519-sha256` names no algorithm either.
- In `c=`, `simple` and `relaxed` are the only names hallmark knows. Any other hyphenated-word, such as `Relaxed` or `x-new`, is an unrecognized algorithm. The decoder reports it with its byte range. §3.4 has the verifier ignore such a signature, and the DKIM check decides how its result reports that.
- A value that breaks the ABNF, such as `relaxed/` or `relaxed /simple`, is a syntax error under either option. The decoder checks the whole syntax before it reads the names, so a syntax error wins over an unrecognized name.
- This decision covers DKIM values, from RFC 6376 and the RFCs that update it. Draft-09 marks its case-sensitive strings with the `%s` of RFC 7405, and DKIM2 values follow its ABNF as written.

§3.2 is the rule that DKIM itself states for values, and it names the one way out: a tag description that says otherwise. The case-insensitive reading of quoted strings is a general convention of the ABNF notation. A signer that follows RFC 6376 writes the keywords in lowercase, as the RFC does.

## Consequences

Positive:
- DKIM tag names (RFC 6376 §3.2) and DKIM value keywords both match as written.
- The trace shows the name as the signature wrote it, and a signature that writes `Relaxed` gets the outcome that the DKIM check gives an unrecognized algorithm (§3.4).

Negative:
- A signature with a keyword in another case, which a verifier that follows the ABNF reading would accept, doesn't verify here. The trace names the keyword and its range.
- Keyword values follow two case rules in the crate: DMARC values ignore case (ADR 0024), and DKIM values match as written. Each follows the text of its own specification.

## References

- [RFC 6376 §3.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.2)
- [RFC 6376 §3.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4)
- [RFC 6376 §3.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.5)
- [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1)
- [RFC 8463 §4](https://www.rfc-editor.org/rfc/rfc8463.html#section-4)
- [RFC 5234 §2.3](https://www.rfc-editor.org/rfc/rfc5234.html#section-2.3)
- [RFC 7405 §2.1](https://www.rfc-editor.org/rfc/rfc7405.html#section-2.1)
- [ADR 0024](0024-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md)
- [requirements.md: Header parsing](../reference/requirements.md#header-parsing)
