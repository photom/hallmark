# ADR 0022: SPF reads `v=spf1` without regard to case, and allows `c`, `r` and `t` only in explanations

- Status: Accepted
- Date: 2026-11-14
- Affects: SPF record selection, the SPF record parser, and the trace

## Context

RFC 7208 leaves two questions about the record grammar open.

The first is the case of the version. §4.5 gives the rule `version = "v=spf1"` and says to "discard records that do not begin with a version section of exactly "v=spf1"", and goes on: "Note that the version section is terminated by either an SP character or the end of the record. As an example, a record with a version section of "v=spf10" does not match and is discarded." In ABNF, a quoted string ignores case (RFC 5234 §2.3), and §4.6.1 applies that rule to the rest of the record: "As per the definition of the ABNF notation in [RFC5234], mechanism and modifier names are case-insensitive." Read alone, "exactly" could also mean that `V=SPF1` is not a version section.

The second is where the macro letters `c`, `r` and `t` may appear. The ABNF of §7.1 lists them among the `macro-letter` values that any `domain-spec` may hold. §7.2 then says: "The following macro letters are allowed only in "exp" text: c, r, t". `c` is the client address written for people, `r` is the name of the host that runs the check, and `t` is the current time. §4.6 says that "if there are any syntax errors anywhere in the record, check_host() returns immediately with the result "permerror", without further interpretation or evaluation."

## Options considered

For the version:
- (a) Compare `v=spf1` without regard to ASCII case, as the ABNF does, and require a space or the end of the record after it.
- (b) Compare the bytes exactly, so that `V=SPF1` is not a record.

For `c`, `r` and `t` outside explanations:
- (a) Treat them as a syntax error, so the record gives `permerror` before evaluation.
- (b) Expand them wherever they appear.
- (c) Let the record parse, and make the term that holds them match nothing.

## Decision

Option (a) for both.

- Record selection compares the first six bytes of each joined TXT record with `v=spf1` without regard to ASCII case. The next byte must be a space, or the record must end there. The trace records the version as published.
- A `c`, `r` or `t` macro in any term of the record is a syntax error, and the record gives `permerror` before any term runs (§4.6). That includes the `domain-spec` of `exp=` itself, which is a term argument like any other. The "exp" text of §7.2 is the explanation string that `exp=` fetches (§6.2), and only that string may use the three letters.

## Consequences

Positive:
- hallmark reads the version with the same ABNF rule as the rest of the record.
- A record that misuses `c`, `r` or `t` fails before hallmark sends any DNS question for it, as §4.6 asks for every syntax error.
- Mechanisms never need the receiving host's name or the time. Only an explanation uses them.

Negative:
- A verifier that compares the version's bytes returns `none` for a `V=SPF1` record, while hallmark evaluates it.
- A verifier that expands `c`, `r` or `t` in a mechanism returns a different result for that record.

## References

- [RFC 7208 §4.5](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.5), [§4.6](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6), [§4.6.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.1), [§6.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.2), [§7.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.1) and [§7.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.2)
- [RFC 5234 §2.3](https://www.rfc-editor.org/rfc/rfc5234.html#section-2.3)
- [requirements.md: SPF, DNS records](../reference/requirements.md#dns-records)
- [ADR 0009](0009-requirements-and-tests-come-only-from-the-specifications.md), which asks for an ADR where a specification leaves room for more than one reading
