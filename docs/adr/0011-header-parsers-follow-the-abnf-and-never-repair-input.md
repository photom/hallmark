# ADR 0011: Header parsers follow the ABNF and never repair input

- Status: Accepted
- Date: 2026-11-13
- Affects: every parser in `crates/hallmark` that reads header fields, tag lists, addresses, paths, or Authentication-Results, and the trace

## Context

hallmark reads header fields from mail that anyone can send, and the parsers' decisions go into the results: the From field gives DMARC its author domain and DKIM2 its Authentic From check, and a DKIM-Signature field decides which key is fetched and what is hashed.

The specifications ask for two different attitudes:
- RFC 5322 §4 says obsolete forms "MUST NOT be generated", but "they MUST be accepted and parsed by a conformant receiver". It adds that a message matching no grammar is no reason for a program to crash.
- RFC 6376 §6.1.1 says that for DKIM-Signature fields, "any inconsistency or unexpected values MUST cause the header field to be completely ignored", and that "Being 'liberal in what you accept' is definitely a bad strategy in this security context". Draft-09 §14.2 asks DKIM2 verifiers to "meticulously validate" their header fields.

A parser that guesses can help an attacker. RFC 9989 §11.5 describes messages crafted with several From domains to get past DMARC while the mail client shows the spoofed one. When a parser repairs a broken field, it can read a different domain from the one a mail client displays.

RFC 5322 §3.2.2 lets comments nest, so a naive recursive parser can be driven into deep recursion by input.

## Options considered

- (a) Use a general email-parsing crate.
- (b) Build the parsers with a parser-combinator crate.
- (c) Write each parser by hand from the ABNF of the section it implements.
- (d) Parse with regular expressions.

## Decision

Option (c).

- Each parser implements the grammar of the RFC or draft section it cites, including the obsolete forms that the RFCs tell receivers to accept: RFC 5322 §4 and the source routes of RFC 5321 §4.1.2.
- The tag-list parser for DKIM and DKIM2 fields and records follows the strict rules of RFC 6376 §6.1.1 and draft-09 §14.2.
- hallmark never repairs input that matches no grammar. The parser returns an error that names the rule that failed and the byte offset. The check then applies the protocol's rule for a missing or malformed value, such as DMC-002 for a From field or PERMFAIL (signature syntax error) for a DKIM-Signature field, and the trace records the error as a fact.
- A line in the header section that isn't a header field stays a separate malformed line with its raw bytes. The parser never joins it to the field before it.
- Parsers keep the raw bytes and return positions in them along with the decoded values, so `simple` canonicalization (RFC 6376 §3.4.1) can hash exactly what arrived.
- Parsers read left to right without recursion, and count nesting, such as comments in comments, against a limit (SAF-002).

Option (a) would bring MIME decoding that hallmark doesn't need (ADR 0012), and its choices about what to repair would become hallmark's. Option (b) builds parsers out of closures, which ADR 0003 rules out. Option (d) can't follow nesting: comments nest (RFC 5322 §3.2.2), and a regular expression can't fully match nested structures. When a regular expression fails to match, it can't name the ABNF rule that failed, as HDR-010 requires. Long regular expressions are also hard for people to read, so TST-007 allows only short, well-established ones.

## Consequences

Positive:
- Each field has one reading, and the trace can name the reason a field didn't parse.
- The parsers are small, have no dependencies, and can be fuzzed one by one (SAF-003).
- Positions into the raw bytes cost no copies, and canonicalization works on the same bytes the parser read.

Negative:
- hallmark writes and tests every parser itself.
- A message whose From field a mail client displays despite a syntax error gets no DMARC result, and the trace says why.

## References

- [RFC 5322 §3.2.2 and §4](https://www.rfc-editor.org/rfc/rfc5322.html#section-4)
- [RFC 5321 §4.1.2](https://www.rfc-editor.org/rfc/rfc5321.html#section-4.1.2)
- [RFC 6376 §3.4.1 and §6.1.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.1)
- [draft-ietf-dkim-dkim2-spec-09 §14.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.2)
- [RFC 9989 §11.5](https://www.rfc-editor.org/rfc/rfc9989.html#section-11.5)
- [requirements.md: Header parsing](../reference/requirements.md#header-parsing)
- [ADR 0003](0003-rust-code-is-written-in-a-procedural-style.md)
