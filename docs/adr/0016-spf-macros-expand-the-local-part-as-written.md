# ADR 0016: SPF macros expand the local part as written

- Status: Accepted
- Date: 2026-11-13
- Affects: SPF macro expansion (ADS-023) and the trace

## Context

An SPF record can contain macros that expand parts of the sender. RFC 7208 §7.2 defines `l` as the `local-part of <sender>` and `s` as the sender itself, and §7.3 says the `l` macro "expands to just the local-part". The sender is the MAIL FROM or HELO identity (§4.1).

An SMTP local part can be a quoted string, as in `<"john smith"@void.invalid>`, and RFC 5321 §4.1.2 says that "all quoted forms MUST be treated as equivalent" when local parts are compared. RFC 7208 doesn't say whether a macro expands the local part with its quoting or without it.

RFC 8616 §4 says that U-labels must become A-labels before they are used in SPF, macro expansion included, and that a term in which `%{s}` or `%{l}` expands a non-ASCII local part matches nothing.

## Options considered

- (a) Expand the local part as written in MAIL FROM, quotes and backslashes included.
- (b) Expand the value of the local part, with the quoting removed.

## Decision

Option (a).

- `%{l}` expands the local part exactly as written in MAIL FROM.
- `%{s}` expands that local part, then `@`, then the domain in its lookup form (RFC 8616 §4). `%{o}` expands the domain in its lookup form.
- The trace records each expansion.

Option (b) would rewrite the sender's text, and ADR 0011 rules out rewriting input.

## Consequences

Positive:
- An expansion uses the bytes the sender sent, and the trace can show them.

Negative:
- Two quoted forms of the same mailbox expand to different strings. RFC 8616 §4 notes that these macros are rarely used.

## References

- [RFC 7208 §4.1, §7.2 and §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.2)
- [RFC 5321 §4.1.2](https://www.rfc-editor.org/rfc/rfc5321.html#section-4.1.2)
- [RFC 8616 §4](https://www.rfc-editor.org/rfc/rfc8616.html#section-4)
- [requirements.md: Addresses and domain names](../reference/requirements.md#addresses-and-domain-names)
