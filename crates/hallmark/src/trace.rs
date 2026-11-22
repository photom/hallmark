//! The trace level, the event model and sorting. The recorder stays
//! crate-private, and the English renderer is in the application layer
//! ([ADR 0061](https://github.com/photom/hallmark/blob/main/docs/adr/0061-each-context-owns-its-trace-codes-and-the-english-renderer-sits-in-the-application-layer.md)).

use crate::*;
use std::net::IpAddr;

/// How much of the trace a check records (TRC-004).
///
/// [`Options::trace_level`] holds it, and its default there is
/// [`Level::Summary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    /// No trace, and nothing allocated for one.
    Off,
    /// The paths that decided each result, and every caution, once the check
    /// finishes.
    Summary,
    /// Every event, as each step finishes.
    Full,
}

/// Sorts one trace into its canonical form: the same events, ordered by
/// position, each in the same bytes (TRC-012).
///
/// [Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md)
/// defines the bytes, and
/// [Trace codes](https://github.com/photom/hallmark/blob/main/docs/reference/trace-codes.md)
/// the codes. `sort_trace` sorts the events of one trace. A store that mixes
/// traces separates them by `msg` first (TRC-014). The sorted trace keeps the
/// members, codes, outcomes and parameters that this version of the library
/// doesn't know (TRC-007).
///
/// # Parameters
///
/// - `trace`: the bytes of one whole trace, its events in any order.
/// - `limits`: the size and depth limits of
///   [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check).
///
/// # Validation
///
/// | ID | Input | Rule | Error code | Requirement | Scenario |
/// |---|---|---|---|---|---|
/// | VAL-501 | `trace` | At most `document_size` octets | [`trace.too_large`](TraceError::TooLarge) | SAF-004 | API-501 |
/// | VAL-502 | `trace` | A sequence of records, each RS, one JSON text and LF | [`trace.record_malformed`](TraceError::RecordMalformed), with the offset | TRC-007, RFC 7464 §2.2 | API-502 |
/// | VAL-503 | Each record | An I-JSON object (RFC 7493) nested at most `document_depth` levels | [`trace.event_malformed`](TraceError::EventMalformed) or [`trace.too_deep`](TraceError::TooDeep) | TRC-007, PAR-005 | API-503 |
/// | VAL-504 | Each event | `v` names major version 1 | [`trace.version_unknown`](TraceError::VersionUnknown) | TRC-007 | STEP-057 |
/// | VAL-505 | Each event | `pos` is a list of counts, each at least 1 | [`trace.event_malformed`](TraceError::EventMalformed) | TRC-012 | API-505 |
/// | VAL-506 | The events | No two hold the same position | [`trace.position_repeated`](TraceError::PositionRepeated) | TRC-012 | API-506 |
///
/// Requirements: TRC-012.
pub fn sort_trace(trace: &[u8], limits: &DocumentLimits) -> Result<Vec<u8>, TraceError> {
    unimplemented!()
}

/// One trace event: one step of a check, as one record of the trace (TRC-002).
///
/// A caller that stores the bytes of [`Check::read_trace`] can sort them with
/// [`sort_trace`], split them at each RS, and read each record with
/// [`Event::parse`], which leaves out the members it doesn't know (TRC-007).
///
/// A trace holds addresses and header contents, and never a private key or
/// body content (TRC-006, TRC-010). Values that a sender or a DNS publisher
/// controls come out unescaped, and the reader escapes them for its output
/// (TRC-013).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Event {
    /// The format version, from the JSON member `v` (TRC-007).
    pub version: Version,
    /// The caller's ID for the message, from `msg` (TRC-014).
    pub message_id: Option<String>,
    /// The step's place in the tree, from `pos` (TRC-002, TRC-012).
    pub position: Position,
    /// The kind of step, from `kind` (TRC-003).
    pub kind: EventKind,
    /// The step's code, from `code` (TRC-011).
    pub code: String,
    /// The step's outcome, from `outcome`, for a code that has outcomes
    /// (TRC-002).
    pub outcome: Option<String>,
    /// The parameters, from `params`, in the order of the JSON object
    /// (TRC-011).
    pub params: Vec<Param>,
    /// The sections the step applies, from `cite` (TRC-002).
    pub cite: Vec<String>,
}

impl Event {
    /// Reads one event from one record of a trace.
    ///
    /// # Parameters
    ///
    /// - `record`: one record: one JSON text, with or without the RS before it
    ///   and the LF after it.
    /// - `limits`: the size and depth limits of
    ///   [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check).
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-510 | `record` | One JSON text, with an RS before it and an LF after it, or neither | [`trace.record_malformed`](TraceError::RecordMalformed) | TRC-007 | API-510 |
    /// | VAL-511 | `record` | The rules of VAL-503 to VAL-505 of [`sort_trace`] | As VAL-503 to VAL-505 | TRC-007 | API-511 |
    /// | VAL-512 | `record` | `kind` is `fact`, `caution` or `hint`, and `code` is a keyword | [`trace.event_malformed`](TraceError::EventMalformed) | TRC-003, TRC-011 | API-512 |
    ///
    /// Requirements: TRC-007.
    pub fn parse(record: &[u8], limits: &DocumentLimits) -> Result<Event, TraceError> {
        unimplemented!()
    }
}

/// The version of the trace format, `major` and `minor` (TRC-007).
///
/// A reader ignores the members, codes, outcomes and parameters it doesn't
/// know within major version 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Version {
    /// The major version.
    pub major: u64,
    /// The minor version.
    pub minor: u64,
}

/// The kind of a step (TRC-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    /// What the check found.
    Fact,
    /// A fact that a specification warns about, which cites the section with
    /// the warning.
    Caution,
    /// A labeled guess about the cause. A hint never changes a result.
    Hint,
}

/// One parameter of an event (TRC-011).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Param {
    /// The parameter's name, as the code lists it.
    pub name: String,
    /// The parameter's value.
    pub value: Value,
}

/// A step's place in the tree: the child numbers on the path from the root
/// (TRC-002, TRC-012).
///
/// `Position` orders as
/// [Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md#positions)
/// orders positions: element by element, with a prefix before the positions
/// that extend it. Results carry the position of their step, so another check
/// can point at them (DMC-005).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Position {
    /// The child numbers on the path from the root, each at least 1. Empty for
    /// the root.
    pub path: Vec<u64>,
}

/// The value of a parameter, with one variant for each value type of
/// [Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md#values).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// An integer from 0 to 2^53−1, a JSON integer.
    Count(u64),
    /// Decimal digits, a JSON string of decimal digits.
    Number(String),
    /// A keyword from a fixed list, a JSON string.
    Keyword(String),
    /// `true` or `false`.
    Boolean(bool),
    /// A position, a JSON list of counts.
    Position(Position),
    /// A hash, decoded from its base64 JSON string.
    Hash(Vec<u8>),
    /// An IP address, from a JSON string: a dotted quad, or an IPv6 address in
    /// the text form of RFC 5952 §4.
    Address(IpAddr),
    /// A place in the body or the header section, a JSON object.
    Place(Place),
    /// Bytes with the parties that control them, a JSON object.
    Data(Data),
    /// A list of values of one type.
    List(Vec<Value>),
    /// The JSON text of a parameter of a code that this version of the library
    /// doesn't list, whose type it can't tell from the JSON.
    Unknown(Vec<u8>),
}

/// Bytes that a trace or a result shows, with the parties that control them
/// (TRC-013).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Data {
    /// The bytes, decoded from the JSON member `text` or `base64url`.
    pub bytes: Vec<u8>,
    /// The parties that control the bytes.
    pub from: Parties,
}

/// The parties that control a value (TRC-013). At least one is true.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Parties {
    /// The caller of hallmark gave the value, such as the signing domain.
    pub caller: bool,
    /// A DNS publisher controls the value.
    pub dns: bool,
    /// The sender controls the value.
    pub sender: bool,
}

/// A place in the body (TRC-010), or in the raw header section (HDR-012).
///
/// A place in the header section counts its offset and line in the raw header
/// section instead, and has no `canonical`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Place {
    /// The byte offset in the body that the caller passed, from 0, from the
    /// JSON member `offset`.
    pub offset: u64,
    /// The line number in that body, from 1, from `line`.
    pub line: u64,
    /// The byte offset in the canonical body, from 0, from `canonical`, when
    /// the hash depends on this place.
    pub canonical: Option<u64>,
}

/// The error of [`sort_trace`] and [`Event::parse`].
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `trace.too_large` | [`TooLarge`](Self::TooLarge) | [`sort_trace`] | VAL-501 | SAF-004 |
/// | `trace.record_malformed` | [`RecordMalformed`](Self::RecordMalformed) | [`sort_trace`], [`Event::parse`] | VAL-502, VAL-510 | TRC-007 |
/// | `trace.event_malformed` | [`EventMalformed`](Self::EventMalformed) | [`sort_trace`], [`Event::parse`] | VAL-503, VAL-505, VAL-511, VAL-512 | TRC-007 |
/// | `trace.too_deep` | [`TooDeep`](Self::TooDeep) | [`sort_trace`], [`Event::parse`] | VAL-503, VAL-511 | PAR-005 |
/// | `trace.version_unknown` | [`VersionUnknown`](Self::VersionUnknown) | [`sort_trace`], [`Event::parse`] | VAL-504, VAL-511 | TRC-007 |
/// | `trace.position_repeated` | [`PositionRepeated`](Self::PositionRepeated) | [`sort_trace`] | VAL-506 | TRC-012 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TraceError {
    /// `trace.too_large`: a trace over `document_size`.
    TooLarge,
    /// `trace.record_malformed`: bytes that aren't RS, one JSON text and LF.
    RecordMalformed {
        /// The offset of the record in the input.
        offset: u64,
    },
    /// `trace.event_malformed`: a record that isn't an event of the format.
    EventMalformed {
        /// The offset of the record in the input.
        offset: u64,
    },
    /// `trace.too_deep`: an event nested deeper than `document_depth`.
    TooDeep {
        /// The offset of the record in the input.
        offset: u64,
    },
    /// `trace.version_unknown`: an event of a major version other than 1.
    VersionUnknown {
        /// The offset of the record in the input.
        offset: u64,
    },
    /// `trace.position_repeated`: two events at one position.
    PositionRepeated {
        /// The position that two events hold.
        position: Position,
    },
}

impl TraceError {
    /// The error's code, such as `trace.too_large` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for TraceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for TraceError {}
