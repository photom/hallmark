//! Snapshots of DNS answers, their resolver, and pasted records.

use crate::*;

/// The DNS questions of a check, each with its answer, and whether it answers
/// as complete zone data (DNS-004, DNS-010).
///
/// A snapshot comes from a full-level trace ([`Snapshot::from_trace`]), from
/// snapshot bytes ([`Snapshot::parse`]) or from pasted records
/// ([`PastedRecords::parse`]). [`SnapshotResolver`] answers from it, and
/// [`Snapshot::to_bytes`] writes it out.
/// [Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md#snapshots)
/// defines a snapshot's bytes
/// ([ADR 0035](https://github.com/photom/hallmark/blob/main/docs/adr/0035-dns-snapshots-and-demo-cases-are-documents-in-the-trace-formats-conventions.md)).
/// Demo cases get their API with the demo, which settles their options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    answers: (),
}

impl Snapshot {
    /// Reads one snapshot document.
    ///
    /// # Parameters
    ///
    /// - `bytes`: one snapshot document in the form of
    ///   [Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md#snapshots).
    /// - `limits`: the size and depth limits of
    ///   [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check-or-a-signing).
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-901 | `bytes` | At most `document_size` octets | [`document.too_large`](DocumentError::TooLarge) | SAF-004 | API-901 |
    /// | VAL-902 | `bytes` | One I-JSON text (RFC 7493) holding one object | [`document.syntax`](DocumentError::Syntax), with the offset | DNS-004, PAR-001 | API-902 |
    /// | VAL-903 | `bytes` | Nested at most `document_depth` levels | [`document.too_deep`](DocumentError::TooDeep) | PAR-005 | API-903 |
    /// | VAL-904 | `v` | Names major version 1 | [`document.version_unknown`](DocumentError::VersionUnknown) | TRC-007 | RES-027 |
    /// | VAL-905 | `type` | `snapshot` | [`document.type_mismatch`](DocumentError::TypeMismatch) | DNS-004 | API-905 |
    /// | VAL-906 | Each answer and suffix answer | Has the members and shapes of [Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md#snapshots), each TXT string at most 255 octets, and a `labels` from 1 to the labels of its name | [`document.member_invalid`](DocumentError::MemberInvalid), with the member | DNS-004, DNS-006, DMC-011 | API-906 |
    /// | VAL-907 | `answers`, `suffixes` | In ascending order of the name in ASCII lowercase, then of the type, with no type and name twice | [`document.order`](DocumentError::Order) | DNS-004 | API-907 |
    /// | VAL-908 | Each name | A domain name in lookup form | [`document.member_invalid`](DocumentError::MemberInvalid), with the member | DNS-008 | API-908 |
    ///
    /// Requirements: DNS-004.
    pub fn parse(bytes: &[u8], limits: &DocumentLimits) -> Result<Snapshot, DocumentError> {
        unimplemented!()
    }

    /// Makes the snapshot of the answers that a trace records.
    ///
    /// # Parameters
    ///
    /// - `trace`: the bytes of one whole trace at the full level, its events
    ///   sorted or not.
    /// - `limits`: the size and depth limits of
    ///   [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check-or-a-signing).
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-910 | `trace` | The rules of [`sort_trace`], VAL-501 to VAL-506 | [`document.trace_invalid`](DocumentError::TraceInvalid), which carries the [`TraceError`] | TRC-007 | API-910 |
    /// | VAL-911 | `trace` | The root event says the level is `full` and the trace wasn't cut short | [`document.trace_incomplete`](DocumentError::TraceIncomplete) | TRC-008, TRC-005 | API-911 |
    /// | VAL-912 | `trace` | Every step that read the same question records the same answer | [`document.trace_inconsistent`](DocumentError::TraceInconsistent) | DNS-007, TRC-008 | RES-024 |
    ///
    /// Requirements: TRC-008.
    pub fn from_trace(trace: &[u8], limits: &DocumentLimits) -> Result<Snapshot, DocumentError> {
        unimplemented!()
    }

    /// Writes the snapshot in the fixed form of bytes.
    ///
    /// The method takes no input, so it has no rule to check (RES-021).
    ///
    /// Requirements: DNS-004, TRC-007.
    pub fn to_bytes(&self) -> Vec<u8> {
        unimplemented!()
    }
}

/// A resolver that answers from one snapshot, read only after creation
/// (DNS-004).
///
/// It answers through its own methods, for a verifying caller's own loop, or
/// through [`Resolver`] and [`SuffixSource`] for [`drive`]. Every question gets
/// an answer, so its methods have no rule to check (RES-022, RES-041).
///
/// | Rule | Requirements |
/// |---|---|
/// | The resolver matches a question by type and name, without regard to ASCII case | DNS-004, RFC 4343 §2 |
/// | A question the snapshot lacks gets a temporary failure that says so. A public suffix question it lacks gets [`SuffixAnswer::Unavailable`] with a description that says so | DNS-004, DNS-009 |
/// | A snapshot from [`PastedRecords::parse`] answers as complete zone data. A name with no pasted records and none below it doesn't exist. A name with records of other types, or with records below it, has no records of the asked type | DNS-010, RFC 8020 §2 |
#[derive(Debug)]
pub struct SnapshotResolver {
    snapshot: Snapshot,
}

impl SnapshotResolver {
    /// Makes a resolver that answers from `snapshot`.
    ///
    /// A `Snapshot` is valid once built, so this function has no rule to check
    /// (RES-021).
    ///
    /// # Parameters
    ///
    /// - `snapshot`: the snapshot to answer from.
    ///
    /// Requirements: DNS-004.
    pub fn new(snapshot: Snapshot) -> SnapshotResolver {
        unimplemented!()
    }

    /// The answer the snapshot holds for a DNS question.
    ///
    /// # Parameters
    ///
    /// - `question`: a DNS question from a check.
    ///
    /// Requirements: DNS-004.
    pub fn answer(&self, question: &Question) -> Answer {
        unimplemented!()
    }

    /// The answer the snapshot holds for a public suffix question.
    ///
    /// # Parameters
    ///
    /// - `question`: a public suffix question from a check.
    ///
    /// Requirements: DNS-004.
    pub fn answer_suffix(&self, question: &SuffixQuestion) -> SuffixAnswer {
        unimplemented!()
    }
}

impl Resolver for SnapshotResolver {
    /// Gives the answer of [`SnapshotResolver::answer`] at once.
    async fn resolve(&self, question: Question) -> Answer {
        unimplemented!()
    }
}

impl SuffixSource for SnapshotResolver {
    /// Gives the answer of [`SnapshotResolver::answer_suffix`] at once.
    async fn suffix(&self, question: SuffixQuestion) -> SuffixAnswer {
        unimplemented!()
    }
}

/// Reads DNS records pasted in the presentation format of RFC 1035 §5.1
/// (DNS-010).
///
/// The type has no value: it holds [`PastedRecords::parse`].
#[derive(Debug)]
pub enum PastedRecords {}

impl PastedRecords {
    /// Reads pasted records into a snapshot that answers as complete zone data.
    ///
    /// # Parameters
    ///
    /// - `text`: records, one per line, in the presentation format of RFC 1035
    ///   §5.1.
    /// - `limits`: the size and depth limits of
    ///   [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check-or-a-signing).
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-920 | `text` | At most `pasted_records_size` octets | [`document.too_large`](DocumentError::TooLarge) | DNS-010 | API-920 |
    /// | VAL-921 | Each line | One record in the presentation format of RFC 1035 §5.1: an absolute owner name, an optional TTL and class, the type and the data | [`document.line_invalid`](DocumentError::LineInvalid), with the line and the offset | DNS-010, PAR-001 | RES-034 |
    /// | VAL-922 | Each line | The class, when given, is `IN`, and the type is `A`, `AAAA`, `MX`, `PTR` or `TXT` | [`document.line_invalid`](DocumentError::LineInvalid), with the line and the offset | DNS-001, DNS-010 | API-922 |
    /// | VAL-923 | Each TXT line | Each character-string holds at most 255 octets after its escapes | [`document.line_invalid`](DocumentError::LineInvalid), with the line and the offset | DNS-010, RFC 1035 §5.1 | API-923 |
    ///
    /// Requirements: DNS-010.
    pub fn parse(text: &str, limits: &DocumentLimits) -> Result<Snapshot, DocumentError> {
        unimplemented!()
    }
}

/// The error of [`Snapshot::parse`], [`Snapshot::from_trace`] and
/// [`PastedRecords::parse`] (DNS-004, DNS-010).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `document.too_large` | [`TooLarge`](Self::TooLarge) | [`Snapshot::parse`], [`PastedRecords::parse`] | VAL-901, VAL-920 | SAF-004 |
/// | `document.syntax` | [`Syntax`](Self::Syntax) | [`Snapshot::parse`] | VAL-902 | PAR-001 |
/// | `document.too_deep` | [`TooDeep`](Self::TooDeep) | [`Snapshot::parse`] | VAL-903 | PAR-005 |
/// | `document.version_unknown` | [`VersionUnknown`](Self::VersionUnknown) | [`Snapshot::parse`] | VAL-904 | TRC-007 |
/// | `document.type_mismatch` | [`TypeMismatch`](Self::TypeMismatch) | [`Snapshot::parse`] | VAL-905 | DNS-004 |
/// | `document.member_invalid` | [`MemberInvalid`](Self::MemberInvalid) | [`Snapshot::parse`] | VAL-906, VAL-908 | DNS-004 |
/// | `document.order` | [`Order`](Self::Order) | [`Snapshot::parse`] | VAL-907 | DNS-004 |
/// | `document.trace_invalid` | [`TraceInvalid`](Self::TraceInvalid) | [`Snapshot::from_trace`] | VAL-910 | TRC-007 |
/// | `document.trace_incomplete` | [`TraceIncomplete`](Self::TraceIncomplete) | [`Snapshot::from_trace`] | VAL-911 | TRC-008 |
/// | `document.trace_inconsistent` | [`TraceInconsistent`](Self::TraceInconsistent) | [`Snapshot::from_trace`] | VAL-912 | DNS-007 |
/// | `document.line_invalid` | [`LineInvalid`](Self::LineInvalid) | [`PastedRecords::parse`] | VAL-921 to VAL-923 | DNS-010 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DocumentError {
    /// `document.too_large`: a document or pasted text over its limit.
    TooLarge {
        /// The limit the input went over.
        limit: Limit,
    },
    /// `document.syntax`: bytes that aren't one I-JSON object.
    Syntax(ParseError),
    /// `document.too_deep`: a document nested deeper than `document_depth`.
    TooDeep,
    /// `document.version_unknown`: a document of a major version other than
    /// 1.
    VersionUnknown,
    /// `document.type_mismatch`: a `type` other than `snapshot`.
    TypeMismatch,
    /// `document.member_invalid`: a member missing, of the wrong shape or out
    /// of range.
    MemberInvalid {
        /// The member's name.
        member: String,
    },
    /// `document.order`: answers out of order, or one type and name twice.
    Order,
    /// `document.trace_invalid`: a trace that breaks a rule of [`sort_trace`].
    TraceInvalid(TraceError),
    /// `document.trace_incomplete`: a trace below the full level, or cut
    /// short.
    TraceIncomplete,
    /// `document.trace_inconsistent`: two steps record different answers to
    /// one question.
    TraceInconsistent,
    /// `document.line_invalid`: a pasted line that isn't a record of RFC 1035
    /// §5.1 that hallmark reads.
    LineInvalid {
        /// The line, counted from 1.
        line: u64,
        /// The parse error, with the offset in the line.
        error: ParseError,
    },
}

impl DocumentError {
    /// The error's code, such as `document.syntax` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for DocumentError {}
