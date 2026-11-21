# Trace format

The format of the trace that hallmark writes, version 1.0, and of the two documents built on its rules: DNS snapshots and demo cases. [ADR 0031](../adr/0031-the-code-that-decides-writes-the-trace-as-events-with-tree-positions.md), [ADR 0033](../adr/0033-trace-events-are-i-json-texts-in-a-json-text-sequence.md), [ADR 0035](../adr/0035-dns-snapshots-and-demo-cases-are-documents-in-the-trace-formats-conventions.md) and [ADR 0041](../adr/0041-the-trace-records-places-in-the-body-and-a-separate-api-streams-the-canonical-body.md) record the decisions behind it, and [Trace codes](trace-codes.md) lists the codes that events carry. The requirements are TRC-001 to TRC-014 in [requirements.md](requirements.md#trace-and-reasoning).

## Contents

- [Records](#records)
- [Events](#events)
- [Positions](#positions)
- [Values](#values)
- [Bytes](#bytes)
- [Levels and order](#levels-and-order)
- [The root event](#the-root-event)
- [The body](#the-body)
- [Versions](#versions)
- [Snapshots](#snapshots)
- [Demo cases](#demo-cases)

## Records

A trace is a JSON text sequence (RFC 7464). Each record is the byte RS (0x1E), one I-JSON text (RFC 7493) that holds one event as a JSON object, and the byte LF (0x0A). The media type is `application/json-seq` (RFC 7464 §4).

hallmark hands the records to the caller in chunks of bounded size (STR-002), and a chunk can end inside a record. A reader splits the stream at each RS. No record holds an LF before its last byte, because strings escape control characters and no white space separates tokens, so a reader can also split at each LF.

## Events

Each event is one step of a check. Its members appear in this order:

| Member | Type | Present | Meaning |
|---|---|---|---|
| `v` | list of two counts | always | The format version, major then minor. This page describes `[1,0]` |
| `msg` | string | when the caller gave one | The caller's ID for the message, such as the MTA's queue ID (TRC-014). It isn't the Message-ID header field |
| `pos` | position | always | The step's place in the tree |
| `kind` | keyword | always | `fact`, `caution` or `hint` (TRC-003) |
| `code` | keyword | always | The step's code, from [Trace codes](trace-codes.md) |
| `outcome` | keyword | when the code has outcomes | One of the outcomes that the code lists |
| `params` | object | when the step has parameters | The code's parameters, in the order its entry lists them |
| `cite` | list of strings | when the step cites a section | The sections the step applies, each a document and a section, such as `RFC 6376 §3.7`, `draft-ietf-dkim-dkim2-spec-09 §6.2` or `RFC 6376 erratum 5839` |

## Positions

A position is a list of counts: the child numbers on the path from the root of the tree. `[]` is the root, `[3]` is the third slot under the root, and `[3,1]` is the first child of `[3]`. Child numbers start at 1.

The slots under the root are fixed, one for each kind of check and one for the header section ([Trace codes](trace-codes.md#slots-under-the-root)). Below them, a step's number comes from the order that the specifications fix and from the order of items in the input, such as header fields, terms in a record and records in an answer. It never depends on when a DNS answer arrived, where the body chunks were cut, or how many events went out before it (TRC-012). A step that didn't happen leaves its number unused, and the numbers after it don't move.

To sort events, compare their positions element by element as integers. The first element that differs decides, and the smaller sorts first. A position that is a prefix of another sorts before it. So `[]`, `[3]`, `[3,1]`, `[3,1,2]`, `[3,2]` and `[10]` are in order. This puts each parent before its children, and siblings in their order. The trace in its canonical form is its events sorted this way.

## Values

| Type | JSON form | Used for |
|---|---|---|
| count | integer from 0 to 2^53−1 | Counts, sizes, byte offsets and line numbers |
| number | string of decimal digits | Numbers taken from the message that can exceed a count, such as DKIM `l=`, `t=` and `x=`, and DKIM2 `m=`, `i=` and `t=` (HDR-013) |
| keyword | string from a fixed list | Kinds, codes, outcomes, and the parameters that a code lists as keywords |
| boolean | `true` or `false` | Flags |
| position | list of counts | A step in the same trace, or the step in the same slot of another call's trace, such as the SPF or DKIM result that a DMARC alignment step used (DMC-005) |
| hash | string in base64 (RFC 4648 §4) | Hashes that hallmark computed |
| address | string | An IPv4 address as a dotted quad, or an IPv6 address in the text form of RFC 5952 §4 |
| place | object | A place in the body or the header section ([The body](#the-body)) |
| data | object | Bytes from the message, the envelope, DNS or the caller, and values made from them |
| list | list of values of one type | Several values |

A data value has these members, in this order:

| Member | Present | Meaning |
|---|---|---|
| `text` | when the bytes are UTF-8 with no noncharacters (RFC 3629, RFC 7493 §2.1) | The bytes, as a string |
| `base64url` | otherwise | The bytes in base64url (RFC 4648 §5), without padding (RFC 4648 §3.2) |
| `from` | always | The parties that control the bytes (TRC-013): one or more of `caller`, `dns` and `sender`, in that order, such as `["sender"]` or `["dns","sender"]`. `caller` covers the values that the caller of hallmark gave, such as the signing domain and selector, `dns` covers whoever publishes the records, and `sender` covers the message and the envelope that the SMTP client gave |

Header field values, header fields in canonical form, addresses, domain names in both forms, DNS record contents, SPF macro expansions and recipe content are data values. A reader escapes `text` for wherever it shows it (TRC-013), and shows the SPF explanation as text from a third party (RFC 7208 §6.2).

## Bytes

hallmark writes every event, snapshot and case in one fixed form, so the same content always gives the same bytes:

- No white space between tokens.
- Members in the order that this page lists them, and parameters in the order that their code's entry lists them.
- A member that has no value is left out, and `null` never appears.
- Strings escaped as RFC 8785 §3.2.2.2 does: `"` and `\` as `\"` and `\\`, U+0008, U+0009, U+000A, U+000C and U+000D as `\b`, `\t`, `\n`, `\f` and `\r`, the other code points from U+0000 to U+001F as `\u` and four lowercase hex digits, and every other code point as it is, in UTF-8.
- Integers in decimal digits, with no sign, leading zero, fraction or exponent.
- No byte order mark (RFC 8259 §8.1).

## Levels and order

| Level | Events that go out | Timing |
|---|---|---|
| off | None | |
| summary | The root event, then, for each check, the events on the path from its top step to the steps that decided its result, and every caution (TRC-004) | Once the check finishes, in position order |
| full | Every event | Each event when its step finishes, and the root event last |

At the full level, a step finishes after the steps it ran, so a parent's event usually goes out after its children's. The order of events can differ between runs with the same inputs, because DNS answers arrive in any order. Sorted by position, the events are the same, in the same bytes (TRC-012). The library offers the sort.

## The root event

Every trace has one root event, at the position `[]`, with the code `trace` and the kind `fact`. Its parameters are:

| Parameter | Type | Meaning |
|---|---|---|
| `level` | keyword | `summary` or `full` |
| `events` | count | The number of other events that went out |
| `cut_short` | boolean | Whether a size limit stopped the trace (TRC-005) |

A reader that holds the root event and as many other events as `events` gives holds the whole trace.

## The body

The trace records the body only as hashes, byte counts, line counts and places, never as content (TRC-010, [ADR 0041](../adr/0041-the-trace-records-places-in-the-body-and-a-separate-api-streams-the-canonical-body.md)). A place has these members, in this order:

| Member | Type | Present | Meaning |
|---|---|---|---|
| `offset` | count | always | The byte offset in the body that the caller passed, from 0 |
| `line` | count | always | The line number in that body, from 1 |
| `canonical` | count | when the hash depends on it | The byte offset in the canonical body, from 0, as for the `l=` cut |

A place in a header step, such as the bare CR and LF of the header section, counts its offset and line in the raw header section instead, and has no `canonical` (HDR-012).

The steps of the body record the `l=` cut (RFC 6376 §3.5), where the empty lines at the end start (RFC 6376 §3.4.3 and §3.4.4, draft-09 §6.1), whether a CRLF was added at the end, and the places where canonicalization changed bytes. Those changes come in two kinds: a run of WSP turned into one SP, and WSP removed at the end of a line (RFC 6376 §3.4.4). The steps also record, under a caution, the places of bare CR and LF, which canonicalization keeps as octets (STR-011). For each kind of change, and for bare CR and LF, the trace gives the count and the first places, up to a limit (TRC-005), and says when the list is cut short. [Trace codes](trace-codes.md) gives the parameters.

## Versions

`v` gives the major and the minor version, and this page describes `[1,0]`. The version changes separately from the crate version (STB-001), and it covers traces, snapshots and cases.

- A minor version may add codes, outcomes, parameters, members and slots under the root.
- Any other change needs a new major version: removing or renaming anything, moving a step to another position, changing a type, or changing a rule of [Bytes](#bytes).
- Within its major version, a reader ignores the members, codes, outcomes and parameters it doesn't know, as RFC 7493 §4.2 recommends, and can show an unknown step by its code and parameters.
- A reader refuses an event, a snapshot or a case of a major version it doesn't know.

A CDDL schema (RFC 8610, RFC 9682) of version 1.0 comes with the first code that writes events, and the tests check every trace in the test suite against it.

## Snapshots

A snapshot holds the DNS answers of a check, and in RFC 7489 mode its public suffix answers, for the snapshot resolver (DNS-004). It is one I-JSON text holding one object, not a sequence, and follows [Values](#values) and [Bytes](#bytes) ([ADR 0035](../adr/0035-dns-snapshots-and-demo-cases-are-documents-in-the-trace-formats-conventions.md)).

| Member | Type | Present | Meaning |
|---|---|---|---|
| `v` | list of two counts | always | The format version |
| `type` | keyword | always | `snapshot` |
| `answers` | list of answers | when the check asked a DNS question | One answer for each question type and name, in ascending order of the name in ASCII lowercase, then of the type |
| `suffixes` | list of suffix answers | when the check asked a public suffix question | One public suffix answer for each domain that the check asked about in RFC 7489 mode (DMC-010), in ascending order of the domain in ASCII lowercase |

Each answer has these members, in this order:

| Member | Type | Present | Meaning |
|---|---|---|---|
| `type` | keyword | always | `A`, `AAAA`, `MX`, `PTR` or `TXT` |
| `name` | data | always | The name that the check asked for |
| `result` | keyword | always | `records`, `nxdomain` for a name that doesn't exist, `nodata` for no records of that type, or `tempfail` (DNS-002) |
| `records` | list | when `result` is `records` | The records, in the order the resolver gave them |
| `description` | data | when `result` is `tempfail` and the resolver described the failure | The resolver's description (DNS-009) |

A record has the shape of its type (DNS-006):

| Type | Record |
|---|---|
| `TXT` | A list of data values: the record's strings, in order and not joined |
| `A`, `AAAA` | An address |
| `MX` | An object with `preference`, a count, and `exchange`, a data value |
| `PTR` | A data value: the name |

Each suffix answer has these members, in this order:

| Member | Type | Present | Meaning |
|---|---|---|---|
| `name` | data | always | The domain that the check asked about |
| `result` | keyword | always | `suffix`, or `unavailable` when the caller had no suffix (DMC-011) |
| `labels` | count | when `result` is `suffix` | The number of labels in the public suffix, the x of RFC 7489 §3.2 step 3 |
| `list` | string | when the caller named the list | The name of the list, such as its date or a hash |

The library extracts a snapshot from a trace, its events sorted or not (TRC-008).

## Demo cases

A demo case holds everything that a replay of a check needs (DEM-004). It is one I-JSON text holding one object, and follows [Values](#values) and [Bytes](#bytes). A case never holds a private key, not even one that the user signed with in the page (SEC-002).

| Member | Type | Present | Meaning |
|---|---|---|---|
| `v` | list of two counts | always | The format version |
| `type` | keyword | always | `case` |
| `message` | data | always | The message, as the user gave it |
| `line_endings` | keyword | always | `crlf` if the core got the message as it is, or `lf` if the demo converted each LF without a CR before it to CRLF before the check (STR-011, DEM-008) |
| `client_ip` | address | when the user gave one | The client IP address |
| `helo` | data | when the user gave one | The HELO or EHLO name |
| `mail_from` | data | when the user gave one | The MAIL FROM path, without its parameters (ADS-020) |
| `rcpt_to` | list of data | when the user gave any | The RCPT TO paths |
| `time` | count | always | The verification time, in seconds since 1970-01-01T00:00:00Z (TIM-001) |
| `options` | object | open | The options of the call. The demo settles which options a case holds, and under which names ([open item](requirements.md#open-items)) |
| `snapshot` | snapshot | always | The DNS answers |
