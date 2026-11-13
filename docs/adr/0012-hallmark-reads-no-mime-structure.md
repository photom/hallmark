# ADR 0012: hallmark reads no MIME structure

- Status: Accepted
- Date: 2026-11-13
- Affects: the scope of `crates/hallmark`, the header parsers, DKIM2 support, and the trace

## Context

The checks hallmark performs don't look inside MIME:
- RFC 6376 §3.7 says that apart from canonicalization, DKIM "treats the body of messages as simply a string of octets", with "no special treatment" for MIME content. §5.3 puts conversion to 7-bit MIME "outside the scope of DKIM", as the job of a mail client or submission server.
- Draft-09 §6.1 says the same for DKIM2.
- SPF and DMARC don't read the body.
- RFC 2047 §5 says an encoded word "MUST NOT appear in any portion of an 'addr-spec'". Encoded words can appear only in display names and comments, so no check needs them decoded to find a domain.

One DKIM2 feature does need MIME. Draft-09 §16.4 says a system that receives a DKIM2-signed delivery status notification SHOULD check that the returned message wasn't altered. The returned message sits in a `message/rfc822` or `text/rfc822-headers` part of a `multipart/report` (draft-09 §16.1, RFC 6522). Finding it means reading the boundary parameter of the top-level `Content-Type` (RFC 2045 §5.1) and splitting the body at the boundaries (RFC 2046 §5.1.1).

RFC 8601 §2.2 borrows the `value` rule of RFC 2045 §5.1, a token or a quoted string, for Authentication-Results. That is a lexical rule and needs no MIME parsing.

## Options considered

- (a) A general MIME parser.
- (b) A part splitter that only finds the returned message in a DKIM2 delivery status notification.
- (c) No MIME structure at all, and DKIM2 delivery status notifications out of scope for now.

## Decision

Option (c).

- hallmark treats every body as octets.
- Display names and comments keep their encoded words as they appear in the message. The demo may decode them for display.
- Checking DKIM2 delivery status notifications (draft-09 §16.4) is out of scope for now. If it is added, a new ADR will add a part splitter for `multipart/report` only, feeding the returned header fields to the existing header parser.
- The Authentication-Results parser uses the token and quoted-string rules of RFC 2045 §5.1 as lexical rules.

## Consequences

Positive:
- Less code and less attack surface. Body streaming stays as it is.
- The draft's delivery status notification rules can settle before hallmark implements them.

Negative:
- hallmark-milter can't check the returned message in a DKIM2 delivery status notification until a later version.
- The trace shows encoded display names in their encoded form.

## References

- [RFC 6376 §3.7 and §5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.7)
- [draft-ietf-dkim-dkim2-spec-09 §6.1, §16.1 and §16.4](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-16.4)
- [RFC 2047 §5](https://www.rfc-editor.org/rfc/rfc2047.html#section-5)
- [RFC 2045 §5.1](https://www.rfc-editor.org/rfc/rfc2045.html#section-5.1), [RFC 2046 §5.1.1](https://www.rfc-editor.org/rfc/rfc2046.html#section-5.1.1) and [RFC 6522](https://www.rfc-editor.org/rfc/rfc6522.html)
- [RFC 8601 §2.2](https://www.rfc-editor.org/rfc/rfc8601.html#section-2.2)
- [requirements.md: Header parsing](../reference/requirements.md#header-parsing)
