# ADR 0013: hallmark converts domain names to A-labels with its own RFC 5891 code

- Status: Accepted
- Date: 2026-11-13
- Affects: domain names in addresses and DNS questions (ADS-002), the trace, and the data files in the repository

## Context

Every DNS question and every domain comparison uses the lookup form of a domain name, in which each U-label becomes an A-label (ADS-002). RFC 8616 §3 says this conversion follows RFC 5891.

The lookup protocol of RFC 5891 §5 leaves any mapping of characters to the application (§5.2), then checks and converts each label:
- §5.4 rejects labels that aren't in NFC, labels with "--" in the third and fourth positions, labels that begin with a combining mark, labels with DISALLOWED or unassigned code points, labels that break a CONTEXTJ rule, and labels with a CONTEXTO code point that has no rule. It adds that the bidirectional rule of RFC 5893 SHOULD be checked.
- §5.5 converts the label with Punycode (RFC 3492) and adds the prefix "xn--".

The code point categories come from RFC 5892 §2 and §3, which compute them from Unicode properties for a given Unicode version. IANA publishes the result for each version (RFC 5892 §5.1), and RFC 8753 says how new Unicode versions are reviewed.

Common Rust crates for this follow UTS #46, a Unicode Consortium standard that maps characters before checking them. ADR 0009 says hallmark's behavior comes from the RFCs and IETF drafts.

## Options considered

- (a) Use a crate that implements UTS #46.
- (b) Implement the lookup protocol of RFC 5891 §5 in hallmark, with tables computed for one Unicode version.
- (c) Accept only ASCII domain names.

## Decision

Option (b).

- hallmark implements RFC 5891 §5.3 to §5.5 and Punycode (RFC 3492) itself, the bidirectional rule of RFC 5893 included.
- A generator in the repository computes the code point categories by the rules of RFC 5892 §2 and §3, as RFC 8753 updates them, from the Unicode Character Database files of one Unicode version. It also extracts the data that the NFC check and the CONTEXTJ and bidirectional rules need. The repository holds the generated tables, and a test checks the categories against the IANA registry for that version.
- The Unicode version is fixed. The docs name it, and the trace of every conversion records it. Moving to a new version is a change of its own.
- hallmark maps nothing. A label that isn't already a valid U-label fails the conversion, and the trace names the rule it broke (ADR 0011).

Option (a) would bring UTS #46 mappings into hallmark's behavior. Option (c) can't meet RFC 8616, RFC 9989 §5.3.1 or draft-09 §18.

## Consequences

Positive:
- The conversion follows the RFCs alone and gives the same result on every platform, WebAssembly included.
- The trace can say which rule of RFC 5891 §5.4 a label broke.

Negative:
- hallmark writes and maintains Punycode, an NFC check, the contextual rules and a table generator.
- Supporting a newer Unicode version takes a deliberate update.
- Input that UTS #46 would map, such as an uppercase non-ASCII letter, fails the conversion.
- The Unicode data files and the tables made from them come under the Unicode Consortium's license, which the license check of STB-002 has to accept.

## References

- [RFC 5891 §5](https://www.rfc-editor.org/rfc/rfc5891.html#section-5)
- [RFC 5892 §2, §3 and §5.1](https://www.rfc-editor.org/rfc/rfc5892.html#section-5.1), updated by [RFC 8753](https://www.rfc-editor.org/rfc/rfc8753.html)
- [RFC 5893](https://www.rfc-editor.org/rfc/rfc5893.html) and [RFC 3492](https://www.rfc-editor.org/rfc/rfc3492.html)
- [RFC 8616 §3](https://www.rfc-editor.org/rfc/rfc8616.html#section-3)
- [requirements.md: Addresses and domain names](../reference/requirements.md#addresses-and-domain-names)
- [ADR 0009](0009-requirements-and-tests-come-only-from-the-specifications.md)
