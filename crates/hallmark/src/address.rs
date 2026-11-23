//! Domain names as results name them. The header and envelope address
//! parsers stay crate-private.

use crate::Data;

/// A domain name as a result names it (ADS-002).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Domain {
    /// The name as the input wrote it, with the parties that control it
    /// (ADS-002, TRC-013).
    pub written: Data,
    /// The lookup form, in ASCII with A-labels, or `None` when the conversion
    /// failed or the labels don't fit (ADS-002 to ADS-004).
    pub lookup: Option<String>,
}
