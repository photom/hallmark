# ADR 0035: DNS snapshots and demo cases are documents in the trace format's conventions

- Status: Accepted
- Date: 2026-11-15
- Affects: the snapshot resolver, the trace, the demo, the tests, and bug reports

## Context

The snapshot resolver answers from the questions of a check, each with the answer its resolver gave (DNS-004). Tests, bug reports and the demo's replay of a shared case use it, and a caller saves a snapshot from a trace (TRC-008). The demo also saves a case, which is the message, the DNS snapshot, the time and the options, so that anyone can replay it offline (DEM-004). An open item said that snapshots use the trace's format, without saying how.

A trace holds much more than the answers, and at the full level its events come in any order ([ADR 0031](0031-the-code-that-decides-writes-the-trace-as-events-with-tree-positions.md)). DNS also answers a name without regard to ASCII case (RFC 4343 §2), and a check asks each name and type once (DNS-007).

## Options considered

- (a) A snapshot is a trace, and the snapshot resolver reads the DNS steps out of it.
- (b) A snapshot is a document of its own, written in the conventions of the trace format, and the library extracts one from a trace.
- (c) A snapshot has a format of its own, unrelated to the trace.

## Decision

Option (b).

- A snapshot is one I-JSON text (RFC 7493) holding one object, written in the fixed form of bytes and with the value forms of [ADR 0033](0033-trace-events-are-i-json-texts-in-a-json-text-sequence.md). It carries the trace format's version `v` and the member `type` with the value `snapshot`.
- It holds one answer for each question type and name. Each answer gives the type, the name, and the outcome: records found, the name doesn't exist, no records of that type, or a temporary failure, which keeps the resolver's description if it gave one (DNS-002, DNS-009). Records keep the order and the shapes in which the resolver gave them: each TXT record as its strings (DNS-006), A and AAAA records as addresses, MX records as a preference and an exchange name, and PTR records as a name.
- The answers go in ascending order of the name in ASCII lowercase, then of the type, so the same answers always give the same bytes.
- The library extracts a snapshot from a trace, its events sorted or not (TRC-008). The snapshot resolver reads a snapshot, and refuses one of a major version it doesn't know.
- A demo case is one more such document, with the `type` value `case`: the message, its line endings (STR-011), the envelope, the time, the options of the call, and a snapshot.
- Records pasted into the demo (DNS-010) keep the presentation format of RFC 1035 §5.1, because a person types them.

Option (a) makes a snapshot as large as a full trace, ties replay to the trace level that was chosen, and gives a replay no fixed order to read. Option (c) adds a second set of rules for bytes, values and versions.

## Consequences

Positive:
- A snapshot holds only what a replay needs, in a fixed order, so the same answers always give the same file.
- One set of rules covers traces, snapshots and cases, and one version number covers all three.
- A bug report can carry a snapshot without the trace, so it leaves out the steps and the header content of the trace.

Negative:
- The library has to write and read one more document type.
- A snapshot taken from a summary-level trace holds only the answers on the path that decided each result, so a replay may lack an answer that the check asks for. The snapshot resolver reports that as a temporary failure (DNS-004).

## References

- [Trace format](../reference/trace-format.md)
- [requirements.md: DNS, Browser demo](../reference/requirements.md#dns)
- [RFC 4343 §2](https://www.rfc-editor.org/rfc/rfc4343.html#section-2)
- [RFC 1035 §5.1](https://www.rfc-editor.org/rfc/rfc1035.html#section-5.1)
