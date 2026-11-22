//! The form of header field values, and the header fields that hallmark
//! generates. The header section splitter and the milter field reader stay
//! crate-private.

use crate::*;

/// The form of the header field values that [`Check::header_field`] and
/// [`Signing::header_field`] take, and that hallmark generates (STR-008,
/// STR-009).
///
/// [`CheckOptions::field_form`] and [`SigningOptions::field_form`] hold it,
/// and [`AuthenticationResults::generate`] takes one. Its default is
/// [`LineBreak::Crlf`] and [`LeadingSpace::Kept`]. A value never holds the
/// line ending that ends the field. Each variant means:
///
/// | Field | Variant | A value that `header_field` takes | A value that hallmark generates |
/// |---|---|---|---|
/// | `folds` | `Crlf` | Its folds are CRLF followed by WSP | Its folds are CRLF followed by WSP |
/// | `folds` | `Lf` | Its folds are LF followed by WSP. hallmark reads each LF that WSP follows as CRLF | Its folds are LF followed by WSP |
/// | `leading_space` | `Kept` | It is the field body as the message holds it, from the octet after the colon. hallmark adds nothing | It starts with one SP, so the name, a colon and the value give `Name: value` |
/// | `leading_space` | `Removed` | The receiving side removed the first octet of the field body if that octet was SP. hallmark can't tell whether it did, adds one SP after the colon, and records that assumption in the trace (STR-008) | It has no leading SP, for a receiving side that adds one |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FieldForm {
    /// The line break between folded lines (STR-008, STR-009).
    pub folds: LineBreak,
    /// Whether the value starts with the SP after the colon (STR-008,
    /// STR-009).
    pub leading_space: LeadingSpace,
}

/// The line break between the folded lines of a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineBreak {
    /// LF followed by WSP.
    Lf,
    /// CRLF followed by WSP, the default.
    #[default]
    Crlf,
}

/// Whether a value starts with the SP after the colon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LeadingSpace {
    /// The receiving side removed one leading SP, or adds one.
    Removed,
    /// The value keeps its leading SP, the default.
    #[default]
    Kept,
}

/// A header field that hallmark generates: a field name and a whole value
/// (STR-009).
///
/// [`SigningOutcome::Written`] holds the signature fields, and
/// [`AuthenticationResults::generate`] gives one.
/// [`value`](Self::value) gives the value at once, and
/// [`read_value`](Self::read_value) gives it in chunks of the caller's size
/// (STR-002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderFieldOut {
    name: String,
    value: Vec<u8>,
}

impl HeaderFieldOut {
    /// The field name, such as `DKIM-Signature`.
    ///
    /// Requirements: STR-009.
    pub fn name(&self) -> &str {
        unimplemented!()
    }

    /// The whole value in the form of the options that made it, without the
    /// line ending that ends the field.
    ///
    /// Requirements: STR-009.
    pub fn value(&self) -> &[u8] {
        unimplemented!()
    }

    /// Writes the bytes of the value from `offset` into `buffer`, as many as
    /// fit, and gives how many it wrote. It gives 0 when `offset` is the
    /// length of the value.
    ///
    /// # Parameters
    ///
    /// - `offset`: the octet of the value to start from, counted from 0.
    /// - `buffer`: where the bytes go, from the start of the buffer.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-301 | `buffer` | Holds at least 1 byte | [`header_field.buffer_empty`](HeaderFieldError::BufferEmpty) | STR-002 | API-301 |
    /// | VAL-302 | `offset` | At most the length of the value | [`header_field.offset_out_of_range`](HeaderFieldError::OffsetOutOfRange) | STR-002 | API-302 |
    ///
    /// Requirements: STR-002.
    pub fn read_value(&self, offset: usize, buffer: &mut [u8]) -> Result<usize, HeaderFieldError> {
        unimplemented!()
    }
}

/// The error of [`HeaderFieldOut::read_value`] (STR-002).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `header_field.buffer_empty` | [`BufferEmpty`](Self::BufferEmpty) | [`HeaderFieldOut::read_value`] | VAL-301 | STR-002 |
/// | `header_field.offset_out_of_range` | [`OffsetOutOfRange`](Self::OffsetOutOfRange) | [`HeaderFieldOut::read_value`] | VAL-302 | STR-002 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeaderFieldError {
    /// `header_field.buffer_empty`: a buffer of 0 bytes.
    BufferEmpty,
    /// `header_field.offset_out_of_range`: an offset past the end of the
    /// value.
    OffsetOutOfRange,
}

impl HeaderFieldError {
    /// The error's code, such as `header_field.buffer_empty` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for HeaderFieldError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for HeaderFieldError {}
