//! Adapters that run a check with a standard blocking reader and writer, for
//! callers that don't need async (STR-004).
//!
//! A stream error never becomes a protocol result (STR-006), and the check
//! stays as the last successful call left it.

use crate::*;

/// Passes a whole message from `reader` to `check`: the header section in one
/// call, then the body in chunks the size of `buffer`, then the end of the
/// message.
///
/// It returns once the check has taken the end of the message.
///
/// # Parameters
///
/// - `check`: a check in Envelope.
/// - `reader`: gives the whole message, with CRLF line endings.
/// - `buffer`: the caller's buffer for each read.
///
/// # Validation
///
/// | ID | Input | Rule | Error code | Requirement | Scenario |
/// |---|---|---|---|---|---|
/// | VAL-810 | `buffer` | Holds at least 1 byte | [`stream.buffer_empty`](StreamError::BufferEmpty) | STR-004 | API-810 |
/// | VAL-811 | `check` | In Envelope (VAL-109 of [`Check::header_section`]) | [`stream.check_refused`](StreamError::CheckRefused), which carries the [`CheckError`] | CHK-004 | API-811 |
/// | VAL-812 | `reader` | Reads without error | [`stream.source_failed`](StreamError::SourceFailed), which names the reader's error kind | STR-006 | API-812 |
/// | VAL-813 | The header section | Ends within `header_section_size` octets | Not an error: the adapter passes the first `header_section_size` + 1 octets as the header section, which reaches the limit, then the end of the message, and reads no more | SAF-002, SAF-004 | API-813 |
///
/// Requirements: STR-004, STR-006, CHK-001.
pub fn read_message<R: std::io::Read>(
    check: &mut Check,
    reader: &mut R,
    buffer: &mut [u8],
) -> Result<(), StreamError> {
    unimplemented!()
}

/// Writes the trace bytes that `check` has now to `writer`, and returns when
/// none is left.
///
/// # Parameters
///
/// - `check`: any check.
/// - `writer`: takes the trace bytes.
/// - `buffer`: the caller's buffer for each write.
///
/// # Validation
///
/// | ID | Input | Rule | Error code | Requirement | Scenario |
/// |---|---|---|---|---|---|
/// | VAL-820 | `buffer` | Holds at least 1 byte | [`stream.buffer_empty`](StreamError::BufferEmpty) | STR-004 | API-820 |
/// | VAL-821 | `writer` | Writes without error | [`stream.sink_failed`](StreamError::SinkFailed), which names the writer's error kind | STR-006 | API-821 |
///
/// Requirements: STR-004, STR-006.
pub fn write_trace<W: std::io::Write>(
    check: &mut Check,
    writer: &mut W,
    buffer: &mut [u8],
) -> Result<(), StreamError> {
    unimplemented!()
}

/// The error of [`read_message`] and [`write_trace`] (STR-006).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `stream.buffer_empty` | [`BufferEmpty`](Self::BufferEmpty) | [`read_message`], [`write_trace`] | VAL-810, VAL-820 | STR-004 |
/// | `stream.check_refused` | [`CheckRefused`](Self::CheckRefused) | [`read_message`] | VAL-811 | CHK-004 |
/// | `stream.source_failed` | [`SourceFailed`](Self::SourceFailed) | [`read_message`] | VAL-812 | STR-006 |
/// | `stream.sink_failed` | [`SinkFailed`](Self::SinkFailed) | [`write_trace`] | VAL-821 | STR-006 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StreamError {
    /// `stream.buffer_empty`: a buffer of 0 bytes.
    BufferEmpty,
    /// `stream.check_refused`: the check refused the message input.
    CheckRefused(CheckError),
    /// `stream.source_failed`: the reader gave an error of this kind.
    SourceFailed(std::io::ErrorKind),
    /// `stream.sink_failed`: the writer gave an error of this kind.
    SinkFailed(std::io::ErrorKind),
}

impl StreamError {
    /// The error's code, such as `stream.source_failed` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for StreamError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for StreamError {}
