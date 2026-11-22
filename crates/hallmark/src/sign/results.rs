//! The results of a signing.

use crate::*;

/// The results of a signing: one outcome for DKIM and one for DKIM2.
///
/// [`Signing::take_results`] gives them. Results are output, and reading a
/// field takes no input, so they have no rule to check (ARH-008).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SigningResults {
    /// Present when [`SigningOptions::dkim`] isn't empty (DKS-005).
    pub dkim: Option<SigningOutcome>,
    /// Present when [`SigningOptions::dkim2`] is `Some` (DK2-014). Available
    /// with the `unstable-dkim2` feature.
    #[cfg(feature = "unstable-dkim2")]
    pub dkim2: Option<SigningOutcome>,
}

/// The outcome of DKIM signing or of DKIM2 signing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SigningOutcome {
    /// The header fields to add, in the order to add them (DKS-005, STR-009).
    Written(Vec<HeaderFieldOut>),
    /// The error that stopped signing (SGN-004).
    Stopped(SignatureError),
}

/// The error that stopped DKIM or DKIM2 signing, held in
/// [`SigningOutcome::Stopped`] (SGN-004).
///
/// The other kind of signing still finishes, and no header field of the
/// stopped kind comes out. A signer is named by its domain and selector.
///
/// | Code | Variant | Raised at | Rules | Requirements | Scenario |
/// |---|---|---|---|---|---|
/// | `signature.signer_failed` | [`SignerFailed`](Self::SignerFailed) | [`Signing::answer`] | | SGN-004 | SIGN-008 |
/// | `signature.wrong_size` | [`WrongSize`](Self::WrongSize) | [`Signing::answer`] | VAL-142 | SGN-004 | SIGN-007 |
/// | `signature.not_verified` | [`NotVerified`](Self::NotVerified) | [`Signing::answer`], with `verify` on in the signing options | VAL-143 | SGN-006 | SIGN-016 |
/// | `signature.header_limit` | [`HeaderLimit`](Self::HeaderLimit) | The end of the header section | | SAF-002 | SIGN-027 |
/// | `signature.exploded_first_signature` | [`ExplodedFirstSignature`](Self::ExplodedFirstSignature) | The end of the header section | | draft-09 §12.5 | SIGN-026 |
/// | `signature.revision_mismatch` | [`RevisionMismatch`](Self::RevisionMismatch) | The end of the message | | DK2-014 | SIGN-022, SIGN-023 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SignatureError {
    /// `signature.signer_failed`: the signer answered with a
    /// [`SignerFailure`]. Keeps its description.
    SignerFailed {
        /// The signer's domain.
        domain: String,
        /// The signer's selector.
        selector: Vec<u8>,
        /// The description from the [`SignerFailure`].
        description: String,
    },
    /// `signature.wrong_size`: the signature doesn't have the size its
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
    /// `signature.not_verified`: the signature doesn't verify with the stated
    /// key.
    NotVerified {
        /// The signer's domain.
        domain: String,
        /// The signer's selector.
        selector: Vec<u8>,
    },
    /// `signature.header_limit`: the header section reached a header limit.
    HeaderLimit(Limit),
    /// `signature.exploded_first_signature`: the `exploded` flag on a message
    /// with no DKIM2-Signature yet.
    ExplodedFirstSignature,
    /// `signature.revision_mismatch`: the earlier version rebuilt from the
    /// revision has hashes other than those in the highest Message-Instance.
    RevisionMismatch,
}

impl SignatureError {
    /// The error's code, such as `signature.signer_failed` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for SignatureError {}
