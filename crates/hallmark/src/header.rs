//! The form of header field values. The header section splitter and the
//! milter field reader stay crate-private.

use crate::*;

/// The form of the header field values that [`Check::header_field`] takes and
/// that hallmark generates (STR-008, STR-009).
///
/// [`Options::field_form`] holds it, and [`AuthenticationResults::generate`]
/// takes one. Its default is [`LineBreak::Crlf`] and [`LeadingSpace::Kept`].
/// A value never holds the line ending that ends the field. Each variant
/// means:
///
/// | Field | Variant | A value that [`Check::header_field`] takes | A value that hallmark generates |
/// |---|---|---|---|
/// | `folds` | `Crlf` | Its folds are CRLF followed by WSP | Its folds are CRLF followed by WSP |
/// | `folds` | `Lf` | Its folds are LF followed by WSP. The check reads each LF that WSP follows as CRLF | Its folds are LF followed by WSP |
/// | `leading_space` | `Kept` | It is the field body as the message holds it, from the octet after the colon. The check adds nothing | It starts with one SP, so the name, a colon and the value give `Name: value` |
/// | `leading_space` | `Removed` | The receiving side removed the first octet of the field body if that octet was SP. The check can't tell whether it did, adds one SP after the colon, and records that assumption in the trace (STR-008) | It has no leading SP, for a receiving side that adds one |
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
