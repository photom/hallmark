//! The results of a check.

use crate::*;

/// The results of a check: one result for each kind of check, each with a
/// typed reason (GEN-004) and its position in the trace (DMC-005).
///
/// [`Check::take_results`] gives them.
/// [Domain model](https://github.com/photom/hallmark/blob/main/docs/reference/design/domain-model.md#results)
/// draws the types. Results are output, and reading a field takes no input,
/// so they have no rule to check (ARH-008).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Results {
    /// Present when the kinds hold `SpfHelo` (SPF-002).
    pub spf_helo: Option<SpfResult>,
    /// Present when the kinds hold `SpfMailFrom` (SPF-001).
    pub spf_mail_from: Option<SpfResult>,
    /// Present when the kinds hold `Dkim` (DKV-001).
    pub dkim: Option<DkimResults>,
    /// Present when the kinds hold `Dkim2` (DK2-013). Available with the
    /// `unstable-dkim2` feature.
    #[cfg(feature = "unstable-dkim2")]
    pub dkim2: Option<Dkim2Result>,
    /// One result when the kinds hold `Dmarc`, or one for each author domain
    /// when [`Options::several_author_domains`] is on. Empty otherwise
    /// (DMC-002).
    pub dmarc: Vec<DmarcResult>,
    /// Present when the kinds hold `DkimSign` (DKS-005).
    pub dkim_signing: Option<Signing>,
    /// Present when the kinds hold `Dkim2Sign` (DK2-014). Available with the
    /// `unstable-dkim2` feature.
    #[cfg(feature = "unstable-dkim2")]
    pub dkim2_signing: Option<Signing>,
    /// Whether time checks were on (TIM-005).
    pub time_checks: bool,
}

/// The outcome of one kind of signing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signing {
    /// The header fields to add, in the order to add them (DKS-005, STR-009).
    Written(Vec<HeaderFieldOut>),
    /// The error that stopped signing (SGN-004).
    Stopped(SigningError),
}

/// A header field that hallmark generates: a field name and a whole value
/// (STR-009).
///
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

    /// The whole value in the form that [`Options::field_form`] states,
    /// without the line ending that ends the field.
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
    /// | VAL-301 | `buffer` | Holds at least 1 byte | [`results.buffer_empty`](ResultsError::BufferEmpty) | STR-002 | API-301 |
    /// | VAL-302 | `offset` | At most the length of the value | [`results.offset_out_of_range`](ResultsError::OffsetOutOfRange) | STR-002 | API-302 |
    ///
    /// Requirements: STR-002.
    pub fn read_value(&self, offset: usize, buffer: &mut [u8]) -> Result<usize, ResultsError> {
        unimplemented!()
    }
}

/// The error of [`HeaderFieldOut::read_value`] (STR-002).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `results.buffer_empty` | [`BufferEmpty`](Self::BufferEmpty) | [`HeaderFieldOut::read_value`] | VAL-301 | STR-002 |
/// | `results.offset_out_of_range` | [`OffsetOutOfRange`](Self::OffsetOutOfRange) | [`HeaderFieldOut::read_value`] | VAL-302 | STR-002 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResultsError {
    /// `results.buffer_empty`: a buffer of 0 bytes.
    BufferEmpty,
    /// `results.offset_out_of_range`: an offset past the end of the value.
    OffsetOutOfRange,
}

impl ResultsError {
    /// The error's code, such as `results.buffer_empty` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for ResultsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for ResultsError {}

/// The error that stopped one kind of signing, held in [`Signing::Stopped`]
/// (SGN-004).
///
/// The other kinds still finish, and no header field of the stopped kind
/// comes out. A signer is named by its domain and selector.
///
/// | Code | Variant | Raised at | Rules | Requirements | Scenario |
/// |---|---|---|---|---|---|
/// | `signing.signer_failed` | [`SignerFailed`](Self::SignerFailed) | [`Check::answer_signing`] | | SGN-004 | SIGN-008 |
/// | `signing.wrong_size` | [`WrongSize`](Self::WrongSize) | [`Check::answer_signing`] | VAL-142 | SGN-004 | SIGN-007 |
/// | `signing.not_verified` | [`NotVerified`](Self::NotVerified) | [`Check::answer_signing`], with `verify` on in the signing options | VAL-143 | SGN-006 | SIGN-016 |
/// | `signing.header_limit` | [`HeaderLimit`](Self::HeaderLimit) | The end of the header section | | SAF-002 | SIGN-027 |
/// | `signing.exploded_first_signature` | [`ExplodedFirstSignature`](Self::ExplodedFirstSignature) | The end of the header section | | draft-09 §12.5 | SIGN-026 |
/// | `signing.revision_mismatch` | [`RevisionMismatch`](Self::RevisionMismatch) | The end of the message | | DK2-014 | SIGN-022, SIGN-023 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SigningError {
    /// `signing.signer_failed`: the signer answered with a [`SignerFailure`].
    /// Keeps its description.
    SignerFailed {
        /// The signer's domain.
        domain: String,
        /// The signer's selector.
        selector: Vec<u8>,
        /// The description from the [`SignerFailure`].
        description: String,
    },
    /// `signing.wrong_size`: the signature doesn't have the size its
    /// algorithm and key give.
    WrongSize {
        /// The signer's domain.
        domain: String,
        /// The signer's selector.
        selector: Vec<u8>,
        /// The size the algorithm and key give, in octets.
        expected: usize,
        /// The size of the signature, in octets.
        actual: usize,
    },
    /// `signing.not_verified`: the signature doesn't verify with the stated
    /// key.
    NotVerified {
        /// The signer's domain.
        domain: String,
        /// The signer's selector.
        selector: Vec<u8>,
    },
    /// `signing.header_limit`: the header section reached a header limit.
    HeaderLimit(Limit),
    /// `signing.exploded_first_signature`: the `exploded` flag on a message
    /// with no DKIM2-Signature yet.
    ExplodedFirstSignature,
    /// `signing.revision_mismatch`: the earlier version rebuilt from the
    /// revision has hashes other than those in the highest Message-Instance.
    RevisionMismatch,
}

impl SigningError {
    /// The error's code, such as `signing.signer_failed` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for SigningError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for SigningError {}
