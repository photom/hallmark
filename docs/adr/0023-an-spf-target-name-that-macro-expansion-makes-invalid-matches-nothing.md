# ADR 0023: An SPF target name that macro expansion makes invalid matches nothing

- Status: Accepted
- Date: 2026-11-14
- Affects: SPF evaluation of `a`, `mx`, `ptr` and `exists`, and the trace

## Context

An SPF term with a `domain-spec` expands its macros into a `<target-name>` and looks that name up (RFC 7208 §4.8). The result can be a name that DNS can't hold: an empty label, as in `foo..void.invalid`, or a label longer than 63 octets (RFC 1035 §2.3.4). §7.3 already cuts a name longer than 253 characters from the left, so only the label rules remain. RFC 7208 leaves the case open. §4.8 says: "The result of evaluating check_host() with a syntactically invalid domain is undefined." It goes on: "Some implementations choose to treat such errors as not-match and therefore ignore such names, while others return a "permerror" exception."

For some terms, other sections settle it:
- `include` runs check_host() on the name (§5.2). check_host() returns `none` for a malformed domain (§4.3), and `include` turns `none` into `permerror` (§5.2).
- `redirect` gives `permerror` "if the `<target-name>` is malformed" (§6.1).
- `exp` goes on "as if no "exp" modifier was given" after any DNS error or syntax error in the explanation (§6.2).

That leaves `a`, `mx`, `ptr` and `exists`. RFC 8616 §4 settles a related case: if the local part has non-ASCII characters, "terms that include %{s} or %{l} do not match anything, because non-ASCII local parts cannot be used as the DNS labels the macros are intended to match."

Senders control several macro values: `%{s}`, `%{l}` and `%{o}` come from MAIL FROM, and `%{h}` from HELO. A sender can pick a local part of 64 characters, or one with two dots in a row, to make a name invalid.

## Options considered

- (a) `a`, `mx`, `ptr` and `exists` match nothing when their target name is invalid. `include`, `redirect` and `exp` keep the results their sections give.
- (b) Every term with an invalid target name gives `permerror`.
- (c) `permerror` if the record text itself holds the invalid name, and no match if a macro produced it.

## Decision

Option (a).

- After expansion and the cut to 253 characters, hallmark checks the target name against the label limits of ADS-004. If the name breaks them, `a`, `mx`, `ptr` and `exists` match nothing, and evaluation goes on with the next term. The trace records the name, the rule it broke, and the macros that produced it.
- The term still counts toward the limit of 10 terms that cause DNS lookups (§4.6.4). hallmark sends no question for it, so it doesn't count as a void lookup.
- `include` and `redirect` give `permerror`, and `exp` goes on as if the record had no `exp=`.

## Consequences

Positive:
- A sender can't turn a domain's `-all` into `permerror` by choosing a local part. Evaluation reaches the terms the domain owner wrote after the broken one.
- The rule matches what RFC 8616 §4 gives for a non-ASCII local part: a term whose name can't be a DNS name matches nothing.

Negative:
- A typo such as `a:mail..void.invalid` in a record gives no `permerror`. The term matches nothing, and only the trace shows why.
- A verifier that returns `permerror` here gives a different result for the same message.

## References

- [RFC 7208 §4.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.3), [§4.6.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.4), [§4.8](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.8), [§5.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.2), [§6.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.1), [§6.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.2) and [§7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3)
- [RFC 8616 §4](https://www.rfc-editor.org/rfc/rfc8616.html#section-4)
- [RFC 1035 §2.3.4](https://www.rfc-editor.org/rfc/rfc1035.html#section-2.3.4)
- [requirements.md: DNS records](../reference/requirements.md#dns-records)
- [ADR 0016](0016-spf-macros-expand-the-local-part-as-written.md), which expands the local part as written
