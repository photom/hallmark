//! The trace level, the event model and sorting. The recorder stays
//! crate-private, and the English renderer is in the application layer
//! ([ADR 0061](https://github.com/photom/hallmark/blob/main/docs/adr/0061-each-context-owns-its-trace-codes-and-the-english-renderer-sits-in-the-application-layer.md)).

use crate::{DocumentLimits, LimitsError, base64, json};
use std::net::IpAddr;

// The doc comments link to items of other modules.
#[cfg(doc)]
use crate::{Check, CheckOptions, Signing, SigningOptions};

/// How much of the trace a check or a signing records (TRC-004).
///
/// [`CheckOptions::trace_level`] and [`SigningOptions::trace_level`] hold it,
/// and its default there is [`Level::Summary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    /// No trace, and nothing allocated for one.
    Off,
    /// The paths that decided each result, and every caution, once the check
    /// or the signing finishes.
    Summary,
    /// Every event, as each step finishes.
    Full,
}

/// The byte before each record of a trace (RFC 7464 §2.2).
const RS: u8 = 0x1E;

/// The byte after each record of a trace (RFC 7464 §2.2).
const LF: u8 = 0x0A;

/// The largest count of the trace format, 2^53−1, the largest integer that
/// I-JSON keeps exact (RFC 7493 §2.2).
const MAX_COUNT: u64 = (1 << 53) - 1;

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
///   [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check-or-a-signing).
///
/// # Validation
///
/// | ID | Input | Rule | Error code | Requirement | Scenario |
/// |---|---|---|---|---|---|
/// | VAL-500 | `limits` | Each limit within its range, as [`DocumentLimits::validate`](crate::DocumentLimits::validate) checks them | [`trace.limits_invalid`](TraceError::LimitsInvalid), which carries the [`LimitsError`](crate::LimitsError) | PAR-004 | API-500 |
/// | VAL-501 | `trace` | At most `document_size` octets | [`trace.too_large`](TraceError::TooLarge) | SAF-004 | API-501 |
/// | VAL-502 | `trace` | A sequence of records, each RS, bytes that hold no RS or LF, and LF. No record of the format holds an LF before its last byte ([Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md#records)), so this rule narrows the parser of RFC 7464 §2.1, which can ignore extra RS bytes and goes on after a record that fails to parse | [`trace.record_malformed`](TraceError::RecordMalformed), with the offset | TRC-007, RFC 7464 §2.1 and §2.2 | API-502, PRS-012 |
/// | VAL-503 | Each record | Between RS and LF, one I-JSON text (RFC 7493) that holds an object, nested at most `document_depth` levels | [`trace.event_malformed`](TraceError::EventMalformed) or [`trace.too_deep`](TraceError::TooDeep) | TRC-007, PAR-005 | API-503 |
/// | VAL-504 | Each event | `v` names major version 1 | [`trace.version_unknown`](TraceError::VersionUnknown), with the major version | TRC-007 | STEP-057 |
/// | VAL-505 | Each event | `pos` is a list of counts, each at least 1 | [`trace.event_malformed`](TraceError::EventMalformed) | TRC-012 | API-505 |
/// | VAL-506 | The events | No two hold the same position | [`trace.position_repeated`](TraceError::PositionRepeated) | TRC-012 | API-506 |
///
/// Requirements: TRC-012.
pub fn sort_trace(trace: &[u8], limits: &DocumentLimits) -> Result<Vec<u8>, TraceError> {
    if let Err(error) = limits.validate() {
        return Err(TraceError::LimitsInvalid(error));
    }
    let within = match u32::try_from(trace.len()) {
        Ok(size) => size <= limits.document_size,
        Err(_) => false,
    };
    if !within {
        return Err(TraceError::TooLarge);
    }
    // VAL-502 covers the whole trace, so every record is split before any
    // event is read.
    let framed = split_records(trace)?;
    let mut records = Vec::with_capacity(framed.len());
    for (offset, bytes) in framed {
        let position = event_position(bytes, offset, limits.document_depth)?;
        records.push(Record { position, bytes });
    }
    records.sort_unstable();
    let mut sorted = Vec::with_capacity(trace.len());
    let mut previous: Option<&Position> = None;
    for record in &records {
        if previous == Some(&record.position) {
            let position = record.position.clone();
            return Err(TraceError::PositionRepeated { position });
        }
        previous = Some(&record.position);
        sorted.extend_from_slice(record.bytes);
    }
    Ok(sorted)
}

/// One record of a trace: the position of its event, then its bytes from RS
/// to LF. Records order by position, as [`sort_trace`] puts them.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Record<'a> {
    position: Position,
    bytes: &'a [u8],
}

/// Splits `trace` into its records, each with the offset of its RS (VAL-502 of
/// [`sort_trace`]).
fn split_records(trace: &[u8]) -> Result<Vec<(u64, &[u8])>, TraceError> {
    let mut records = Vec::new();
    let mut start = 0;
    let mut rest = trace;
    while !rest.is_empty() {
        let offset = wide(start);
        let Some(length) = record_length(rest) else {
            return Err(TraceError::RecordMalformed { offset });
        };
        let Some((bytes, after)) = rest.split_at_checked(length) else {
            return Err(TraceError::RecordMalformed { offset });
        };
        records.push((offset, bytes));
        start += length;
        rest = after;
    }
    Ok(records)
}

/// The length of the record that starts `bytes`: RS, bytes that are neither
/// RS nor LF, then LF (RFC 7464 §2.2,
/// [Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md#records)).
/// `None` when `bytes` doesn't start with one.
fn record_length(bytes: &[u8]) -> Option<usize> {
    let [RS, text @ ..] = bytes else {
        return None;
    };
    for (index, byte) in text.iter().enumerate() {
        match *byte {
            LF => return Some(index + 2),
            RS => return None,
            _ => {}
        }
    }
    None
}

/// The position of the event in `record`, which runs from RS to LF and starts
/// at `offset` in the trace (VAL-503 to VAL-505 of [`sort_trace`]). Within
/// major version 1, the event's other members can be any that a later minor
/// version adds (TRC-007).
fn event_position(record: &[u8], offset: u64, depth_limit: u32) -> Result<Position, TraceError> {
    // split_records gives each record from RS to LF, so this pattern matches.
    let [RS, text @ .., LF] = record else {
        return Err(TraceError::RecordMalformed { offset });
    };
    let members = match json::read(text, depth_limit) {
        Ok(json::Json::Object(members)) => members,
        Ok(_) | Err(json::JsonError::Malformed { .. }) => {
            return Err(TraceError::EventMalformed { offset });
        }
        Err(json::JsonError::TooDeep { .. }) => return Err(TraceError::TooDeep { offset }),
    };
    let Some(json::Json::Array(version)) = member(&members, "v") else {
        return Err(TraceError::EventMalformed { offset });
    };
    // Trace format, Versions: the major version alone decides whether this
    // version of the library reads the event.
    let Some(first) = version.first() else {
        return Err(TraceError::EventMalformed { offset });
    };
    match count(first) {
        Some(1) => {}
        Some(major) => return Err(TraceError::VersionUnknown { offset, major }),
        None => return Err(TraceError::EventMalformed { offset }),
    }
    let [_, minor] = version.as_slice() else {
        return Err(TraceError::EventMalformed { offset });
    };
    if count(minor).is_none() {
        return Err(TraceError::EventMalformed { offset });
    }
    let Some(path) = counts(member(&members, "pos")) else {
        return Err(TraceError::EventMalformed { offset });
    };
    // Trace format, Positions: child numbers start at 1.
    if path.contains(&0) {
        return Err(TraceError::EventMalformed { offset });
    }
    Ok(Position { path })
}

/// The value of the member `name` of an object.
fn member<'a>(members: &'a [(String, json::Json)], name: &str) -> Option<&'a json::Json> {
    for (key, value) in members {
        if key == name {
            return Some(value);
        }
    }
    None
}

/// The counts of a JSON list of counts, such as `pos`.
fn counts(value: Option<&json::Json>) -> Option<Vec<u64>> {
    let Some(json::Json::Array(items)) = value else {
        return None;
    };
    let mut counts = Vec::with_capacity(items.len());
    for item in items {
        counts.push(count(item)?);
    }
    Some(counts)
}

/// A count: a JSON integer from 0 to 2^53−1, written in decimal digits with no
/// sign, fraction or exponent
/// ([Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md#values)).
fn count(value: &json::Json) -> Option<u64> {
    let json::Json::Number(text) = value else {
        return None;
    };
    for byte in text.bytes() {
        if !byte.is_ascii_digit() {
            return None;
        }
    }
    let number = text.parse::<u64>().ok()?;
    if number > MAX_COUNT {
        return None;
    }
    Some(number)
}

/// An offset in the trace as the errors carry it. A `usize` is at most 64
/// bits wide on every target that Rust supports, so the cast keeps the value.
fn wide(offset: usize) -> u64 {
    offset as u64
}

/// One trace event: one step of a check or a signing, as one record of the
/// trace (TRC-002).
///
/// A caller that stores the bytes of [`Check::read_trace`] or
/// [`Signing::read_trace`] can sort them with [`sort_trace`], split them at
/// each RS, and read each record with [`Event::parse`], which leaves out the
/// members it doesn't know (TRC-007).
///
/// A trace holds addresses and header contents, and never a private key or
/// body content (TRC-006, TRC-010). Values that a sender or a DNS publisher
/// controls come out unescaped, and the reader escapes them for its output
/// (TRC-013).
#[cfg(doc)]
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

#[cfg(doc)]
impl Event {
    /// Reads one event from one record of a trace.
    ///
    /// # Parameters
    ///
    /// - `record`: one record: one JSON text, with or without the RS before it
    ///   and the LF after it.
    /// - `limits`: the size and depth limits of
    ///   [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check-or-a-signing).
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-509 | `limits` | Each limit within its range, as [`DocumentLimits::validate`](crate::DocumentLimits::validate) checks them | [`trace.limits_invalid`](TraceError::LimitsInvalid), which carries the [`LimitsError`](crate::LimitsError) | PAR-004 | API-509 |
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
    /// What the check or the signing found.
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

impl Data {
    /// Appends the value to `out` in the fixed form of
    /// [Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md#values):
    /// `text` when the bytes are text that I-JSON can hold, `base64url`
    /// otherwise, then `from` (TRC-007, TRC-013).
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the trace recorder comes later")
    )]
    pub(crate) fn write_json(&self, out: &mut Vec<u8>) {
        match json_text(&self.bytes) {
            Some(text) => {
                out.extend_from_slice(b"{\"text\":");
                json::write_string(text, out);
            }
            None => {
                out.extend_from_slice(b"{\"base64url\":\"");
                base64::encode_url_unpadded(&self.bytes, out);
                out.push(b'"');
            }
        }
        out.extend_from_slice(b",\"from\":");
        self.from.write_json(out);
        out.push(b'}');
    }
}

/// The bytes as a string, when I-JSON can hold them as one: UTF-8
/// (RFC 3629) with no noncharacter (RFC 7493 §2.1).
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the trace recorder comes later")
)]
fn json_text(bytes: &[u8]) -> Option<&str> {
    let text = core::str::from_utf8(bytes).ok()?;
    for character in text.chars() {
        if json::is_noncharacter(character) {
            return None;
        }
    }
    Some(text)
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

impl Parties {
    /// Appends the parties to `out` as a list, in the order `caller`, `dns`,
    /// `sender` (TRC-013).
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the trace recorder comes later")
    )]
    fn write_json(self, out: &mut Vec<u8>) {
        let parties = [
            (self.caller, "caller"),
            (self.dns, "dns"),
            (self.sender, "sender"),
        ];
        out.push(b'[');
        let mut first = true;
        for (controls, name) in parties {
            if !controls {
                continue;
            }
            if !first {
                out.push(b',');
            }
            first = false;
            json::write_string(name, out);
        }
        out.push(b']');
    }
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
/// | `trace.limits_invalid` | [`LimitsInvalid`](Self::LimitsInvalid) | [`sort_trace`], [`Event::parse`] | VAL-500, VAL-509 | PAR-004 |
/// | `trace.too_large` | [`TooLarge`](Self::TooLarge) | [`sort_trace`] | VAL-501 | SAF-004 |
/// | `trace.record_malformed` | [`RecordMalformed`](Self::RecordMalformed) | [`sort_trace`], [`Event::parse`] | VAL-502, VAL-510 | TRC-007 |
/// | `trace.event_malformed` | [`EventMalformed`](Self::EventMalformed) | [`sort_trace`], [`Event::parse`] | VAL-503, VAL-505, VAL-511, VAL-512 | TRC-007 |
/// | `trace.too_deep` | [`TooDeep`](Self::TooDeep) | [`sort_trace`], [`Event::parse`] | VAL-503, VAL-511 | PAR-005 |
/// | `trace.version_unknown` | [`VersionUnknown`](Self::VersionUnknown) | [`sort_trace`], [`Event::parse`] | VAL-504, VAL-511 | TRC-007 |
/// | `trace.position_repeated` | [`PositionRepeated`](Self::PositionRepeated) | [`sort_trace`] | VAL-506 | TRC-012 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TraceError {
    /// `trace.limits_invalid`: limits that break a rule of their `validate` method.
    LimitsInvalid(LimitsError),
    /// `trace.too_large`: a trace over `document_size`.
    TooLarge,
    /// `trace.record_malformed`: bytes that don't split into records of RS, a
    /// text with no RS or LF, and LF.
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
        /// The major version that the event's `v` names.
        major: u64,
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
        match self {
            TraceError::LimitsInvalid(_) => "trace.limits_invalid",
            TraceError::TooLarge => "trace.too_large",
            TraceError::RecordMalformed { .. } => "trace.record_malformed",
            TraceError::EventMalformed { .. } => "trace.event_malformed",
            TraceError::TooDeep { .. } => "trace.too_deep",
            TraceError::VersionUnknown { .. } => "trace.version_unknown",
            TraceError::PositionRepeated { .. } => "trace.position_repeated",
        }
    }
}

impl core::fmt::Display for TraceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let code = self.code();
        match self {
            TraceError::LimitsInvalid(error) => write!(f, "{code}: {error}"),
            TraceError::TooLarge => write!(f, "{code}: the trace is over document_size"),
            TraceError::RecordMalformed { offset } => {
                let text = "isn't RS, a text with no RS or LF, and LF";
                write!(f, "{code}: the record at offset {offset} {text}")
            }
            TraceError::EventMalformed { offset } => {
                let text = "isn't an event of the trace format";
                write!(f, "{code}: the record at offset {offset} {text}")
            }
            TraceError::TooDeep { offset } => {
                let text = "is nested deeper than document_depth";
                write!(f, "{code}: the event at offset {offset} {text}")
            }
            TraceError::VersionUnknown { offset, major } => {
                write!(f, "{code}: the event at offset {offset} ")?;
                write!(f, "has major version {major}; this library reads 1")
            }
            TraceError::PositionRepeated { position } => {
                write!(f, "{code}: two events hold the position ")?;
                write_path(f, &position.path)
            }
        }
    }
}

/// Writes a position as the trace writes it, such as `[3,1]`.
fn write_path(f: &mut core::fmt::Formatter<'_>, path: &[u64]) -> core::fmt::Result {
    f.write_str("[")?;
    let mut separator = "";
    for number in path {
        write!(f, "{separator}{number}")?;
        separator = ",";
    }
    f.write_str("]")
}

impl std::error::Error for TraceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TraceError::LimitsInvalid(error) => Some(error),
            TraceError::TooLarge
            | TraceError::RecordMalformed { .. }
            | TraceError::EventMalformed { .. }
            | TraceError::TooDeep { .. }
            | TraceError::VersionUnknown { .. }
            | TraceError::PositionRepeated { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Data, LF, Parties, Position, RS, TraceError, sort_trace};
    use crate::{DocumentLimits, Limit, LimitsError};

    /// Writes `bytes` from the sender as a data value.
    fn written(bytes: &[u8]) -> String {
        let from = Parties {
            caller: false,
            dns: false,
            sender: true,
        };
        let data = Data {
            bytes: bytes.to_vec(),
            from,
        };
        let mut out = Vec::new();
        data.write_json(&mut out);
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn step_051_text_is_escaped_as_rfc_8785_says() {
        let expected = r#"{"text":"a\"b\\c\u001f\té/","from":["sender"]}"#;
        assert_eq!(written(b"a\"b\\c\x1f\t\xc3\xa9/"), expected);
    }

    #[test]
    fn step_052_bytes_that_are_not_utf_8_are_base64url() {
        let expected = r#"{"base64url":"-_8","from":["sender"]}"#;
        assert_eq!(written(b"\xfb\xff"), expected);
    }

    #[test]
    fn step_053_a_noncharacter_makes_the_value_base64url() {
        let expected = r#"{"base64url":"77-_","from":["sender"]}"#;
        assert_eq!(written(b"\xef\xbf\xbf"), expected);
    }

    /// The JSON texts as the records of one trace (RFC 7464 §2.2).
    fn records(texts: &[&str]) -> Vec<u8> {
        let mut trace = Vec::new();
        for text in texts {
            trace.push(RS);
            trace.extend_from_slice(text.as_bytes());
            trace.push(LF);
        }
        trace
    }

    /// One event at `pos`, written as hallmark writes it.
    fn event(pos: &str) -> String {
        format!(r#"{{"v":[1,0],"pos":{pos},"kind":"fact","code":"trace"}}"#)
    }

    /// A trace of one event at each position, in the order given.
    fn events(positions: &[&str]) -> Vec<u8> {
        let mut trace = Vec::new();
        for pos in positions {
            let text = event(pos);
            trace.extend_from_slice(&records(&[text.as_str()]));
        }
        trace
    }

    /// Sorts `trace` with the default limits.
    fn sorted(trace: &[u8]) -> Result<Vec<u8>, TraceError> {
        sort_trace(trace, &DocumentLimits::default())
    }

    #[test]
    fn step_043_events_sort_by_position_with_parents_first() {
        let given = events(&["[10]", "[3,2]", "[]", "[3,1,2]", "[3]", "[3,1]"]);
        let expected = events(&["[]", "[3]", "[3,1]", "[3,1,2]", "[3,2]", "[10]"]);
        assert_eq!(sorted(&given), Ok(expected));
    }

    #[test]
    fn step_056_an_unknown_member_and_code_stay_in_place() {
        let unknown = r#"{"v":[1,3],"pos":[3],"kind":"fact","code":"later.step","later":true}"#;
        let root = event("[]");
        let given = records(&[unknown, root.as_str()]);
        let expected = records(&[root.as_str(), unknown]);
        assert_eq!(sorted(&given), Ok(expected));
    }

    #[test]
    fn step_057_a_major_version_other_than_1_is_refused() {
        let root = event("[]");
        let later = r#"{"v":[2,0],"pos":[3],"kind":"fact","code":"trace"}"#;
        let given = records(&[root.as_str(), later]);
        let offset = u64::try_from(root.len() + 2).unwrap();
        let error = sorted(&given).unwrap_err();
        assert_eq!(error, TraceError::VersionUnknown { offset, major: 2 });
        assert_eq!(error.code(), "trace.version_unknown");
        assert!(error.to_string().contains("major version 2"));
    }

    #[test]
    fn api_500_limits_are_checked_first() {
        let limits = DocumentLimits {
            document_size: 0,
            ..DocumentLimits::default()
        };
        let invalid = LimitsError::OutOfRange {
            limit: Limit::DocumentSize,
        };
        let result = sort_trace(&events(&["[]"]), &limits);
        assert_eq!(result, Err(TraceError::LimitsInvalid(invalid)));
    }

    #[test]
    fn api_501_a_trace_over_document_size_is_refused() {
        let limits = DocumentLimits {
            document_size: 1_024,
            ..DocumentLimits::default()
        };
        // RS, the text, and LF make 2,000 octets.
        let head = r#"{"v":[1,0],"pos":[],"kind":"fact","code":""#;
        let code = "a".repeat(2_000 - head.len() - 4);
        let text = format!(r#"{head}{code}"}}"#);
        let trace = records(&[text.as_str()]);
        assert_eq!(trace.len(), 2_000);
        assert_eq!(sort_trace(&trace, &limits), Err(TraceError::TooLarge));
        assert_eq!(sorted(&trace), Ok(trace));
    }

    #[test]
    fn api_502_a_record_without_rs_is_refused() {
        let mut trace = events(&["[]"]);
        let offset = u64::try_from(trace.len()).unwrap();
        trace.extend_from_slice(event("[3]").as_bytes());
        trace.push(LF);
        let expected = TraceError::RecordMalformed { offset };
        assert_eq!(sorted(&trace), Err(expected));
    }

    #[test]
    fn api_503_a_record_holds_an_object_within_document_depth() {
        let expected = TraceError::EventMalformed { offset: 0 };
        assert_eq!(sorted(&records(&["[1,2]"])), Err(expected));
        // The event and its params make two levels, and the arrays the rest.
        let head = r#"{"v":[1,0],"pos":[3],"kind":"fact","code":"x","params":{"p":"#;
        let deepest = format!("{head}{}0{}}}}}", "[".repeat(14), "]".repeat(14));
        assert!(sorted(&records(&[deepest.as_str()])).is_ok());
        let too_deep = format!("{head}{}0{}}}}}", "[".repeat(15), "]".repeat(15));
        let expected = TraceError::TooDeep { offset: 0 };
        assert_eq!(sorted(&records(&[too_deep.as_str()])), Err(expected));
    }

    #[test]
    fn api_505_child_numbers_start_at_1() {
        let expected = TraceError::EventMalformed { offset: 0 };
        assert_eq!(sorted(&events(&["[3,0]", "[]"])), Err(expected));
    }

    #[test]
    fn api_506_two_events_at_one_position_are_refused() {
        let position = Position { path: vec![3, 1] };
        let expected = TraceError::PositionRepeated { position };
        let error = sorted(&events(&["[3,1]", "[]", "[3,1]"])).unwrap_err();
        assert_eq!(error, expected);
        assert!(error.to_string().ends_with("position [3,1]"));
    }

    #[test]
    fn prs_012_a_record_is_rs_a_text_without_rs_or_lf_and_lf() {
        let root = event("[]");
        let spread = r#"{"v":[1,0],"pos":[3],"kind":"fact",
"code":"trace"}"#;
        let after_lf = u64::try_from(spread.find('\n').unwrap() + 2).unwrap();
        let mut doubled = vec![RS];
        doubled.extend_from_slice(&records(&[root.as_str()]));
        let mut unended = records(&[root.as_str()]);
        let last = u64::try_from(unended.len()).unwrap();
        unended.push(RS);
        unended.extend_from_slice(event("[3]").as_bytes());
        let cases = [
            (records(&[spread]), after_lf),
            (doubled, 0),
            (unended, last),
        ];
        for (trace, offset) in cases {
            let expected = TraceError::RecordMalformed { offset };
            assert_eq!(sorted(&trace), Err(expected));
        }
    }
}
