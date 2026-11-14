# ADR 0017: DKIM2 compares local parts as written and ignores source routes

- Status: Accepted
- Date: 2026-11-14
- Affects: the DKIM2 chain-of-custody check (ADS-024) and the trace

## Context

A DKIM2 verifier compares the `mf=` and `rt=` values of the most recent DKIM2-Signature with the MAIL FROM and RCPT TO of the SMTP transaction. Draft-09 §12.2 asks for "an exact match (including the local parts of the email addresses)", and says the matching "will be case sensitive for the local part (before the @) and case insensitive for the domain (after the @)". §14.4 says "the local parts need to match exactly", and that the decoded values "include the angle brackets around the email addresses". §18 maps U-labels to A-labels before domains are compared. §10.5 and §10.6 record the RFC 5321 reverse-path and forward-path, and say nothing about quoting or source routes.

RFC 5321 §4.1.2 says two things that the draft doesn't address:
- For comparing local parts, "all quoted forms MUST be treated as equivalent", so `<"a"@void.invalid>` and `<a@void.invalid>` name one mailbox.
- A source route, as in `<@relay.invalid:a@void.invalid>`, "MUST BE accepted, SHOULD NOT be generated, and SHOULD be ignored".

The draft's "exact" can't mean byte for byte, because domains compare without regard to case. It doesn't say whether two quoted forms of one local part match. [hallmark#4](https://github.com/photom/hallmark/issues/4) tracks the question until a revision of the draft settles it.

## Options considered

- (a) Compare local parts as written, quoting included, and ignore source routes.
- (b) Compare local parts by their value with the quoting removed, as RFC 5321 §4.1.2 asks, and ignore source routes.
- (c) Compare whole paths as written, source routes included, apart from the domain rules of draft-09 §12.2 and §18.

## Decision

Option (a), until a revision of the draft settles the question.

- The chain-of-custody check compares each local part as written, quotes and backslashes included, so `<"a"@void.invalid>` doesn't match `<a@void.invalid>`.
- Domains compare in their lookup form without regard to case (draft-09 §12.2, §14.4 and §18).
- The comparison ignores a source route on either side (RFC 5321 §4.1.2).
- If two local parts differ only in quoting, the check fails, and the trace adds a hint that RFC 5321 §4.1.2 treats them as one mailbox (TRC-003).
- hallmark#4 stays open. If a later revision of the draft settles the question another way, a new ADR supersedes this one.

Draft-09 §12.2 puts the burden on the signer: "A Signer MUST ensure that this Chain of Custody check will succeed." The signer writes `mf=` and `rt=` and sends MAIL FROM and RCPT TO itself, so it can give both the same bytes. Option (b) reads the draft's "match exactly" more loosely than its text. Option (c) fails a path that RFC 5321 §4.1.2 tells receivers to read as the same, over a part of the path that the draft never mentions.

## Consequences

Positive:
- The check follows the draft's text, in which local parts "need to match exactly".
- The trace tells the operator why a quoting difference failed the check.

Negative:
- A message whose signer quoted a local part in `mf=` or `rt=` differently from the SMTP command fails, although RFC 5321 treats the two as one mailbox.
- A later revision of the draft may change the reading, and then a new ADR and new scenarios follow.

## References

- [draft-ietf-dkim-dkim2-spec-09 §10.5, §10.6, §12.2, §14.4 and §18](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.4)
- [RFC 5321 §4.1.2](https://www.rfc-editor.org/rfc/rfc5321.html#section-4.1.2)
- [hallmark#4](https://github.com/photom/hallmark/issues/4)
- [requirements.md: Addresses and domain names](../reference/requirements.md#addresses-and-domain-names)
