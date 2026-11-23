//! Parse errors. The cursor, byte ranges and number readers stay
//! crate-private.

use crate::{AuthResError, CanonicalizerError, DocumentError, Limit, SpfReason};

/// A parse error: the grammar rule that failed, the byte offset and the input
/// that the parser read (PAR-001).
///
/// It sits inside [`AuthResError::Syntax`], [`DocumentError::Syntax`],
/// [`DocumentError::LineInvalid`] and [`CanonicalizerError::InstanceInvalid`],
/// and in [`SpfReason::Syntax`] in the results. Inside a check, parse errors of
/// the message, the envelope and DNS records go into the trace and the
/// results, never into a returned error (HDR-010).
///
/// hallmark builds it, and the caller reads its fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParseError {
    /// The name of the grammar rule that failed, as its specification writes
    /// it, such as `authres-payload` (PAR-001).
    pub rule: String,
    /// The offset of the first octet that broke the rule, from the start of
    /// the input (PAR-001).
    pub offset: u64,
    /// The input the parser read, by its role, such as
    /// `Authentication-Results field` (PAR-001).
    pub input: String,
    /// The limit the input reached, when a limit stopped the parser
    /// (SAF-002).
    pub limit: Option<Limit>,
}
