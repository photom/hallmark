# Snapshots and pasted records

The APIs that keep the DNS answers of a check and answer from them later: snapshots, their resolver, and records pasted in the presentation format (DNS-004, DNS-010, [ADR 0035](../../adr/0035-dns-snapshots-and-demo-cases-are-documents-in-the-trace-formats-conventions.md)). [Trace format](../trace-format.md#snapshots) defines a snapshot's bytes. Demo cases get their API with the demo, which settles their options ([open item](../requirements.md#open-items)).

## Contents

- [APIs](#apis)
- [Validation](#validation)
  - [`Snapshot::parse`](#snapshotparse)
  - [`Snapshot::from_trace`](#snapshotfrom_trace)
  - [`Snapshot::to_bytes`](#snapshotto_bytes)
  - [`SnapshotResolver::new`](#snapshotresolvernew)
  - [`SnapshotResolver::answer` and `answer_suffix`](#snapshotresolveranswer-and-answer_suffix)
  - [`PastedRecords::parse`](#pastedrecordsparse)

## APIs

```mermaid
flowchart LR
    trace["Full-level trace"] --> from["Snapshot::from_trace"]
    bytes["Snapshot bytes"] --> parse["Snapshot::parse"]
    text["Pasted records"] --> pasted["PastedRecords::parse"]
    from --> snap["Snapshot"]
    parse --> snap
    pasted --> snap
    snap --> out["Snapshot::to_bytes"]
    snap --> res["SnapshotResolver"]
    res -->|"Resolver and SuffixSource"| drive["drive"]
    res -->|"answer and answer_suffix"| loop["Caller's own loop"]
```

| API | Takes | Gives | Requirements |
|---|---|---|---|
| `Snapshot::parse` | Bytes and `DocumentLimits` | `Snapshot`, or `DocumentError` | DNS-004 |
| `Snapshot::from_trace` | A full-level trace, its events sorted or not, and `DocumentLimits` | `Snapshot`, or `DocumentError` | TRC-008 |
| `Snapshot::to_bytes` | Nothing | The snapshot in the fixed form of bytes | DNS-004, TRC-007 |
| `SnapshotResolver::new` | A `Snapshot` | A resolver that answers from it | DNS-004 |
| `SnapshotResolver::answer` | A `Question` | The `Answer` the snapshot holds, at once | DNS-004, ADR 0048 |
| `SnapshotResolver::answer_suffix` | A `SuffixQuestion` | The `SuffixAnswer` the snapshot holds, at once | DNS-004 |
| `PastedRecords::parse` | Text and `DocumentLimits` | A `Snapshot` that answers as complete zone data, or `DocumentError` | DNS-010 |

| Rule | Requirements |
|---|---|
| The resolver matches a question by type and name, without regard to ASCII case | DNS-004, RFC 4343 §2 |
| A question the snapshot lacks gets a temporary failure that says so. A public suffix question it lacks gets the answer that no suffix is available | DNS-004, DNS-009 |
| A name with no pasted records and none below it doesn't exist. A name with records of other types, or with records below it, has no records of the asked type | DNS-010, RFC 8020 §2 |

## Validation

### `Snapshot::parse`

| ID | Input | Rule | Error code | Requirement | Scenario |
|---|---|---|---|---|---|
| VAL-901 | `bytes` | At most `document_size` octets | `document.too_large` | SAF-004 | API-901 |
| VAL-902 | `bytes` | One I-JSON text (RFC 7493) holding one object | `document.syntax`, with the offset | DNS-004, PAR-001 | API-902 |
| VAL-903 | `bytes` | Nested at most `document_depth` levels | `document.too_deep` | PAR-005 | API-903 |
| VAL-904 | `v` | Names major version 1 | `document.version_unknown` | TRC-007 | RES-027 |
| VAL-905 | `type` | `snapshot` | `document.type_mismatch` | DNS-004 | API-905 |
| VAL-906 | Each answer and suffix answer | Has the members and shapes of [Trace format](../trace-format.md#snapshots), each TXT string at most 255 octets, and a `labels` from 1 to the labels of its name | `document.member_invalid`, with the member | DNS-004, DNS-006, DMC-011 | API-906 |
| VAL-907 | `answers`, `suffixes` | In ascending order of the name in ASCII lowercase, then of the type, with no type and name twice | `document.order` | DNS-004 | API-907 |
| VAL-908 | Each name | A domain name in lookup form | `document.member_invalid`, with the member | DNS-008 | API-908 |

### `Snapshot::from_trace`

| ID | Input | Rule | Error code | Requirement | Scenario |
|---|---|---|---|---|---|
| VAL-910 | `trace` | The rules of [`sort_trace`](trace.md#sort_trace), VAL-501 to VAL-506 | `document.trace_invalid`, which carries the `TraceError` | TRC-007 | API-910 |
| VAL-911 | `trace` | The root event says the level is `full` and the trace wasn't cut short | `document.trace_incomplete` | TRC-008, TRC-005 | API-911 |
| VAL-912 | `trace` | Every step that read the same question records the same answer | `document.trace_inconsistent` | DNS-007, TRC-008 | RES-024 |

### `Snapshot::to_bytes`

| ID | Input | Rule | Error code | Requirement | Scenario |
|---|---|---|---|---|---|
| | None | No rule: the method takes no input | | DNS-004 | RES-021 |

### `SnapshotResolver::new`

| ID | Input | Rule | Error code | Requirement | Scenario |
|---|---|---|---|---|---|
| | `snapshot` | No rule: a `Snapshot` is valid once built | | DNS-004 | RES-021 |

### `SnapshotResolver::answer` and `answer_suffix`

| ID | Input | Rule | Error code | Requirement | Scenario |
|---|---|---|---|---|---|
| | `question` | No rule: every question gets an answer | | DNS-004 | RES-022, RES-041 |

### `PastedRecords::parse`

| ID | Input | Rule | Error code | Requirement | Scenario |
|---|---|---|---|---|---|
| VAL-920 | `text` | At most `pasted_records_size` octets | `document.too_large` | DNS-010 | API-920 |
| VAL-921 | Each line | One record in the presentation format of RFC 1035 §5.1: an absolute owner name, an optional TTL and class, the type and the data | `document.line_invalid`, with the line and the offset | DNS-010, PAR-001 | RES-034 |
| VAL-922 | Each line | The class, when given, is `IN`, and the type is `A`, `AAAA`, `MX`, `PTR` or `TXT` | `document.line_invalid`, with the line and the offset | DNS-001, DNS-010 | API-922 |
| VAL-923 | Each TXT line | Each character-string holds at most 255 octets after its escapes | `document.line_invalid`, with the line and the offset | DNS-010, RFC 1035 §5.1 | API-923 |
