# ADR 0037: The core takes CRLF line endings only, and a bare CR or LF in the body is an octet

- Status: Accepted
- Date: 2026-11-15
- Affects: STR-011, STR-012, HDR-012, TRC-010, the browser demo (DEM-008), and the scenarios for canonicalization, header fields, the trace and the demo

## Context

The requirements let the caller state the line endings of a message: CRLF, as a milter receives it, or LF, as in many `.eml` files and in a browser text area. In LF mode the core turned each LF into CRLF. Two questions stayed open: what a verifier does with a bare CR or a bare LF in a body with CRLF line endings, and what happens to a CR at the very end of the body.

The specifications say this about bare CR and LF:

- RFC 5322 §2.3: in a body, CR and LF "MUST only occur together as CRLF". §4 and §4.1 add bare CR and LF to the obsolete body syntax, which a receiver must accept and parse. §4.1 notes that messages use them as line separators, against the rules, or as plain control characters.
- RFC 5321 §2.3.8: CRLF ends a line, and implementations "MUST NOT recognize or generate any other character or character sequence as a line terminator".
- RFC 6376 §5.3 and draft-09 §17 tell signers to convert bare CR or LF to CRLF before signing. Neither tells verifiers to convert anything.
- RFC 6376 §3.4.3 says `simple` "makes no other changes to the message body", and draft-09 §6.1 says "No other changes are made to the body". RFC 6376 §2.8 defines whitespace as SP, HTAB and CRLF.

LF mode served two inputs. A browser gives the contents of a text area to scripts with every line break as LF, on any operating system, so a pasted message reaches the demo with LF line endings whatever it had before. Files saved with LF line endings are the other case.

## Options considered

For the line endings that the core takes:

- (a) Keep LF mode in the core.
- (b) Take CRLF only. Callers convert LF line endings before the check, and the demo does so for pasted text and, when the user chooses, for a loaded file.

For a bare CR or LF in the body:

- (c) An octet of the body, never a line ending.
- (d) Convert it to CRLF when verifying, as RFC 6376 §5.3 tells signers to.
- (e) Convert only a CR at the very end of the body.

## Decision

Options (b) and (c).

- The core takes messages with CRLF line endings and recognizes no other line ending (RFC 5321 §2.3.8). Messages with LF line endings are out of scope for the first release, and LF mode can return if a caller asks for it.
- A bare CR or LF in the body is an octet of the body. Canonicalization hashes it as it is (RFC 6376 §3.4.3 and §3.4.4, draft-09 §6.1). `relaxed` keeps the WSP before a bare LF, because the bare LF ends no line, and a body that ends with a bare CR or LF has no trailing CRLF and gets one: the body `A\r` becomes `A\r\r\n`.
- The trace records a caution (TRC-003) with the count and the first places of bare CR and LF in the body, citing RFC 5322 §2.3 and RFC 5321 §2.3.8, in verification and in signing. The caution changes no result.
- The demo hands the core CRLF line endings (DEM-008). It converts each LF of a pasted message to CRLF and tells the user, before the analysis, that pasting doesn't keep the original line endings. It passes the bytes of a loaded file unchanged. If the file holds an LF without a CR before it, the demo warns before the analysis and lets the user convert each such LF to CRLF or analyze the bytes as they are.
- A bare CR or LF inside the header section is a separate open item.

Option (a) keeps a conversion in the core that only LF files need, and pasted text has lost its original line endings before any conversion runs. Option (d) hashes bytes that the message doesn't hold, and no specification asks verifiers to convert. Option (e) adds a rule for one position that no specification text supports.

## Consequences

Positive:
- One rule settles every bare CR and LF in the body, wherever the chunks are cut (STR-005).
- The core has one path for line endings, which the milter, the demo and the tests share.
- If a bare CR or LF breaks a signature, the trace shows where it sits.

Negative:
- A caller that holds LF files converts them before the check.
- A pasted message can't be checked exactly if its original held a bare CR or LF, because pasting loses them. The demo says so before the analysis.
- ADR 0032 lists a bare LF turned into CRLF in LF mode among the changes the trace records, and has the canonicalizer take the line-ending mode the caller states. With CRLF as the only mode, that change never occurs, and the rest of ADR 0032 holds. The demo case of ADR 0035 keeps its line endings member, which now records whether the demo converted the input.

## References

- [requirements.md: Streaming](../reference/requirements.md#streaming) (STR-011, STR-012)
- [requirements.md: Header parsing](../reference/requirements.md#header-parsing) (HDR-012)
- [requirements.md: Browser demo](../reference/requirements.md#browser-demo) (DEM-008)
- [RFC 5321 §2.3.8](https://www.rfc-editor.org/rfc/rfc5321.html#section-2.3.8)
- [RFC 5322 §2.3](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.3), [§4](https://www.rfc-editor.org/rfc/rfc5322.html#section-4) and [§4.1](https://www.rfc-editor.org/rfc/rfc5322.html#section-4.1)
- [RFC 6376 §2.8](https://www.rfc-editor.org/rfc/rfc6376.html#section-2.8), [§3.4.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.3), [§3.4.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.4) and [§5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.3)
- [draft-ietf-dkim-dkim2-spec-09 §6.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-6.1) and [§17](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-17)
- [ADR 0032](0032-the-trace-records-places-in-the-body-and-a-separate-api-streams-the-canonical-body.md), [ADR 0035](0035-dns-snapshots-and-demo-cases-are-documents-in-the-trace-formats-conventions.md)
