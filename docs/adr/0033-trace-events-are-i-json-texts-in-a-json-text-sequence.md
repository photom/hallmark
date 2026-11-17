# ADR 0033: Trace events are I-JSON texts in a JSON text sequence

- Status: Accepted
- Date: 2026-11-15
- Affects: the trace format, hallmark-milter, the demo, outside tools, and the tests

## Context

Three kinds of reader take the trace: the demo, in TypeScript in a browser; hallmark-milter, in Rust; and outside tools (TRC-007). Events go out one by one as their steps finish ([ADR 0031](0031-the-code-that-decides-writes-the-trace-as-events-with-tree-positions.md)), and a reader may read them as they arrive or after the check.

The format has to meet these requirements:

- The same inputs give the same events, and the same bytes once sorted (TRC-012). So the format can't leave the writer a choice of bytes.
- Values that come from the message or from DNS can hold any bytes. Header field values, malformed lines, TXT strings and DKIM2 recipe lines are examples.
- Each value that a sender or a DNS publisher controls carries a mark (TRC-013), and a reader must find the mark even under a step code it doesn't know.
- The format has a version of its own, which changes separately from the crate version (STB-001).

JSON is a text of one value (RFC 8259 §2), so a reader of a single JSON document needs all of it, or a streaming parser. RFC 7464 defines JSON text sequences for that case: "JSON text sequences can be parsed (and produced) incrementally without having to have a streaming parser" (§1). JSON text must be UTF-8 (RFC 8259 §8.1), and I-JSON forbids surrogates and noncharacters in strings (RFC 7493 §2.1). The order of an object's members carries no meaning (RFC 8259 §4, RFC 7493 §2.3).

## Options considered

- (a) Each event is an I-JSON text, and the trace is a JSON text sequence of them.
- (b) The trace is one JSON document.
- (c) CBOR (RFC 8949).
- (d) Protocol Buffers.
- (e) YAML.

For the schema: JSON Schema, CDDL (RFC 8610, updated by RFC 9682), or a prose description only.

## Decision

Option (a), with a prose reference page and a CDDL schema.

- Each event is an I-JSON text (RFC 7493) holding one JSON object. hallmark writes each event as one record of a JSON text sequence (RFC 7464 §2.2): the byte RS (0x1E), the JSON text, then LF. The media type is `application/json-seq` (RFC 7464 §4).
- hallmark writes each event in one fixed form of bytes. It puts no white space between tokens, and it writes members in the order that the format reference lists them. It leaves out a member that has no value, and never writes `null`. It escapes strings as RFC 8785 §3.2.2.2 does, and writes integers in decimal, with no sign, leading zero, fraction or exponent. Sorted by position, the same inputs then give the same bytes.
- A value that comes from the message, the envelope or DNS, or is made from such bytes, is an object with two members. The first is `text`, if the bytes are UTF-8 with no noncharacters (RFC 3629, RFC 7493 §2.1), or `base64url` otherwise: base64url (RFC 4648 §5), as RFC 7493 §4.4 recommends for binary data, without the padding that RFC 4648 §3.2 lets a specification leave out. The second, `from`, lists who controls the bytes: `dns`, `sender`, or both. That is the mark of TRC-013. A header field in canonical form and a domain name in lookup form are such values too.
- Counts, offsets and line numbers are JSON integers, which the trace limits keep below 2^53 (RFC 7493 §2.2). A number taken from the message that can be larger, such as a DKIM `l=` of up to 76 digits (HDR-013), is always a string of decimal digits.
- The reference page `docs/reference/trace-format.md` describes the format. A CDDL schema (RFC 8610, RFC 9682) comes with the first code that writes events, and the tests check every trace in the test suite against it. The schema check runs only in tests, and its dependency gets a license check (STB-002).
- Every event carries the format version `v`, a major and a minor number. This ADR defines version 1.0. A minor version may add codes, outcomes, parameters, members and slots under the root of the tree. Any other change needs a new major version. Within its major version, a reader ignores the members, codes and outcomes it doesn't know, as RFC 7493 §4.2 recommends, and it refuses events of a major version it doesn't know.
- hallmark hands the records to the caller in chunks of bounded size (STR-002), and a chunk may end inside a record. A reader splits the stream at each RS.

Option (b) can't send an event before the check finishes. Option (c) needs a library in the demo, and its bytes can't be read in a log without a tool. Option (d) needs generated code in both Rust and TypeScript, and a runtime in the demo, and its bytes can't be read without the schema. YAML parsers disagree about implicit types, and a machine-written trace needs none of what YAML adds. JSON Schema has no RFC, and its Internet-Drafts have expired. CDDL is an IETF standard and describes JSON directly.

## Consequences

Positive:
- The demo reads each record with `JSON.parse`, with no npm dependency (REP-003).
- A log keeps one event per line, so line tools such as `grep` work on it, and a trace cut short still gives every complete record (RFC 7464 §2.3).
- Every event can be read alone: it carries its version, its position and its marks.

Negative:
- JSON is larger than a binary format, and each event repeats its version and the caller's ID for the message.
- Bytes that aren't valid text appear as base64url, which a person can't read without decoding.
- hallmark has to write JSON in its own fixed form, which a general JSON serializer doesn't promise.

## References

- [RFC 8259 §2](https://www.rfc-editor.org/rfc/rfc8259.html#section-2), [§4](https://www.rfc-editor.org/rfc/rfc8259.html#section-4) and [§8.1](https://www.rfc-editor.org/rfc/rfc8259.html#section-8.1)
- [RFC 7493 §2.1](https://www.rfc-editor.org/rfc/rfc7493.html#section-2.1), [§2.2](https://www.rfc-editor.org/rfc/rfc7493.html#section-2.2), [§2.3](https://www.rfc-editor.org/rfc/rfc7493.html#section-2.3), [§4.2](https://www.rfc-editor.org/rfc/rfc7493.html#section-4.2) and [§4.4](https://www.rfc-editor.org/rfc/rfc7493.html#section-4.4)
- [RFC 7464 §1](https://www.rfc-editor.org/rfc/rfc7464.html#section-1), [§2.2](https://www.rfc-editor.org/rfc/rfc7464.html#section-2.2), [§2.3](https://www.rfc-editor.org/rfc/rfc7464.html#section-2.3) and [§4](https://www.rfc-editor.org/rfc/rfc7464.html#section-4)
- [RFC 8785 §3.2.2.2](https://www.rfc-editor.org/rfc/rfc8785.html#section-3.2.2.2)
- [RFC 4648 §3.2](https://www.rfc-editor.org/rfc/rfc4648.html#section-3.2) and [§5](https://www.rfc-editor.org/rfc/rfc4648.html#section-5)
- [RFC 8610](https://www.rfc-editor.org/rfc/rfc8610.html) and [RFC 9682](https://www.rfc-editor.org/rfc/rfc9682.html)
- [Trace format](../reference/trace-format.md)
- [requirements.md: Trace and reasoning](../reference/requirements.md#trace-and-reasoning)
