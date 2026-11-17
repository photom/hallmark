# ADR 0032: The trace records places in the body, and a separate API streams the canonical body

- Status: Superseded by ADR 0041
- Date: 2026-11-15
- Affects: the trace, body canonicalization, the public API of `crates/hallmark`, and the demo

## Context

The trace records the body as hashes, byte counts and line counts. Until now, TRC-010 also let a caller ask for the body content itself, which streamed out through the trace as the body passed.

Body content has no upper size: it grows with the message, while every other part of the trace stays within the limits of TRC-005. It is also the part of a message that most needs privacy (TRC-006), and hallmark-milter, which checks other people's mail all day, has no reason to put it in a log.

The callers that want to see the body are the ones that look into a body hash that doesn't match, such as the demo. Those callers already hold the whole message. They need hallmark to tell them where the body was cut or changed, and to give them the exact bytes that a check hashed.

## Options considered

- (a) Keep body content in the trace, when the caller asks for it.
- (b) Put body content in the trace up to a size limit.
- (c) Keep body content out of the trace. The trace records places in the body, and a separate API streams the canonical body to callers that want to see it.

## Decision

Option (c).

- The trace records the body only as hashes, byte counts, line counts and places. Body content never enters the trace, at any level.
- The places are the point where an `l=` tag cut the body (RFC 6376 §3.5), the point where the empty lines at the end start, which canonicalization drops (RFC 6376 §3.4.3 and §3.4.4, draft-09 §6.1), whether a CRLF was added at the end, and the places where canonicalization changed bytes.
- The places where canonicalization changed bytes are recorded for each kind of change: a run of WSP turned into one SP, WSP removed at the end of a line (RFC 6376 §3.4.4), and a bare LF turned into CRLF in LF mode (STR-011). For each kind, the trace gives the count and the first places, up to a limit that the trace limits of TRC-005 include, and says so when the list is cut short.
- A place gives the byte offset and the line number in the body that the caller passed. If the hash depends on canonical bytes, as the `l=` cut does, it also gives the offset in the canonical body.
- The library offers a body canonicalizer as an API of its own. It takes body chunks in the line-ending mode the caller states (STR-011) and streams out the canonical body: `simple` or `relaxed` (RFC 6376 §3.4.3 and §3.4.4), or the DKIM2 body hash input (draft-09 §6.1). It can cut the body at an `l=` length, and it can stream an earlier DKIM2 version that the recipes of given Message-Instance header fields rebuild (draft-09 §5.2).
- The canonicalizer runs the same code as the checks, so its output is the bytes they hash. Its memory doesn't grow with the body (STR-001), and its output doesn't depend on where the chunks are cut (STR-005).

Option (a) gives the trace no upper size, and a single run in full mode copies the whole body into wherever the caller keeps traces. Option (b) bounds the size but keeps the start of the body, while the change behind a failed body hash can be at the end: some mailing lists append a few lines to each message (RFC 6377 §3.3).

## Consequences

Positive:
- The trace never holds the body, so a full trace exposes less (TRC-006), and the milter has nothing to switch off.
- The whole trace stays within the limits of TRC-005.
- The demo shows the exact canonical body that a check hashed, and the trace points at the places that matter in it.

Negative:
- A caller that wants to see the body has to keep the message and pass it through the canonicalizer as well.
- The trace alone doesn't show what a changed line said, only where it is.

## References

- [requirements.md: Trace and reasoning, Streaming](../reference/requirements.md#trace-and-reasoning)
- [RFC 6376 §3.4.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.3), [§3.4.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.4) and [§3.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.5)
- [draft-ietf-dkim-dkim2-spec-09 §5.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-5.2) and [§6.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-6.1)
- [RFC 6377 §3.3](https://www.rfc-editor.org/rfc/rfc6377.html#section-3.3)
- [ADR 0031](0031-the-code-that-decides-writes-the-trace-as-events-with-tree-positions.md)
