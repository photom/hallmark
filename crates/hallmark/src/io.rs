//! Adapters that run a check or a signing with a standard blocking reader and
//! writer, for callers that don't need async (STR-004).
//!
//! A stream error never becomes a protocol result or a signing outcome
//! (STR-006), and the check or the signing stays as the last successful call
//! left it.

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
/// - `buffer`: the verifying caller's buffer for each read.
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

/// Passes a whole message from `reader` to `signing`: the header section in
/// one call, then the body in chunks the size of `buffer`, then the end of the
/// message.
///
/// It returns once the signing has taken the end of the message.
///
/// # Parameters
///
/// - `signing`: a signing in Created.
/// - `reader`: gives the whole message, with CRLF line endings.
/// - `buffer`: the signing caller's buffer for each read.
///
/// # Validation
///
/// | ID | Input | Rule | Error code | Requirement | Scenario |
/// |---|---|---|---|---|---|
/// | VAL-814 | `buffer` | Holds at least 1 byte | [`stream.buffer_empty`](StreamError::BufferEmpty) | STR-004 | API-814 |
/// | VAL-815 | `signing` | In Created (VAL-172 of [`Signing::header_section`]) | [`stream.signing_refused`](StreamError::SigningRefused), which carries the [`SigningError`] | CHK-004 | API-815 |
/// | VAL-816 | `reader` | Reads without error | [`stream.source_failed`](StreamError::SourceFailed), which names the reader's error kind | STR-006 | API-816 |
/// | VAL-817 | The header section | Ends within `header_section_size` octets | Not an error: the adapter passes the first `header_section_size` + 1 octets as the header section, which reaches the limit, then the end of the message, and reads no more | SAF-002, SAF-004 | API-817 |
///
/// Requirements: STR-004, STR-006, CHK-006.
pub fn read_message_to_sign<R: std::io::Read>(
    signing: &mut Signing,
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
/// - `buffer`: the verifying caller's buffer for each write.
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

/// Writes the trace bytes that `signing` has now to `writer`, and returns when
/// none is left.
///
/// # Parameters
///
/// - `signing`: any signing.
/// - `writer`: takes the trace bytes.
/// - `buffer`: the signing caller's buffer for each write.
///
/// # Validation
///
/// | ID | Input | Rule | Error code | Requirement | Scenario |
/// |---|---|---|---|---|---|
/// | VAL-822 | `buffer` | Holds at least 1 byte | [`stream.buffer_empty`](StreamError::BufferEmpty) | STR-004 | API-822 |
/// | VAL-823 | `writer` | Writes without error | [`stream.sink_failed`](StreamError::SinkFailed), which names the writer's error kind | STR-006 | API-823 |
///
/// Requirements: STR-004, STR-006.
pub fn write_signing_trace<W: std::io::Write>(
    signing: &mut Signing,
    writer: &mut W,
    buffer: &mut [u8],
) -> Result<(), StreamError> {
    unimplemented!()
}

/// The error of the adapters of this module (STR-006).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `stream.buffer_empty` | [`BufferEmpty`](Self::BufferEmpty) | Every adapter | VAL-810, VAL-814, VAL-820, VAL-822 | STR-004 |
/// | `stream.check_refused` | [`CheckRefused`](Self::CheckRefused) | [`read_message`] | VAL-811 | CHK-004 |
/// | `stream.signing_refused` | [`SigningRefused`](Self::SigningRefused) | [`read_message_to_sign`] | VAL-815 | CHK-004 |
/// | `stream.source_failed` | [`SourceFailed`](Self::SourceFailed) | [`read_message`], [`read_message_to_sign`] | VAL-812, VAL-816 | STR-006 |
/// | `stream.sink_failed` | [`SinkFailed`](Self::SinkFailed) | [`write_trace`], [`write_signing_trace`] | VAL-821, VAL-823 | STR-006 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StreamError {
    /// `stream.buffer_empty`: a buffer of 0 bytes.
    BufferEmpty,
    /// `stream.check_refused`: the check refused the message input.
    CheckRefused(CheckError),
    /// `stream.signing_refused`: the signing refused the message input.
    SigningRefused(SigningError),
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
