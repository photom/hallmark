//! The options of a signing.

#[cfg(feature = "unstable-dkim2")]
use crate::Dkim2Signing;
use crate::{CheckOptions, DkimSigning, FieldForm, Level, Limits, Signing, VerificationTime};

/// Everything a signing caller fixes when it creates a signing. Nothing
/// changes the options later (INV-015).
///
/// The signing caller builds options with [`SigningOptions::new`], which takes
/// the time and sets every other field to its default, and then sets the other
/// public fields, at least one of `dkim` and `dkim2`. `SigningOptions` checks
/// nothing while the signing caller builds it: [`Signing::new`] checks every
/// rule at once, VAL-250, VAL-251 and VAL-220 to VAL-242. [`Limits`] checks its
/// own rules in its setters. Verification has options of its own,
/// [`CheckOptions`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SigningOptions {
    /// The signing time, which gives `t=` (DKS-004, TIM-001).
    pub time: VerificationTime,
    /// Off, summary or full. Default [`Level::Summary`] (TRC-004).
    pub trace_level: Level,
    /// The signing caller's ID for the message, which every event carries,
    /// such as the MTA's queue ID. Default `None` (TRC-014).
    pub message_id: Option<String>,
    /// The limits of the signing. Default `Limits::default()` (PAR-004).
    pub limits: Limits,
    /// The form of the values that [`Signing::header_field`] takes and that
    /// the signature fields come out in. Default `FieldForm::default()`:
    /// CRLF, leading space kept (STR-008, STR-009).
    pub field_form: FieldForm,
    /// The DKIM signatures to add. Default empty (DKS-002).
    pub dkim: Vec<DkimSigning>,
    /// The DKIM2 signature to add. Default `None` (DK2-003). Available with
    /// the `unstable-dkim2` feature.
    #[cfg(feature = "unstable-dkim2")]
    pub dkim2: Option<Dkim2Signing>,
}

impl SigningOptions {
    /// Makes options with the time, and every other field at its default.
    ///
    /// # Parameters
    ///
    /// - `time`: the signing time, which gives `t=` (DKS-004).
    ///
    /// Building options checks nothing. [`Signing::new`] checks them.
    ///
    /// Requirements: CHK-006, DKS-004.
    pub fn new(time: VerificationTime) -> SigningOptions {
        unimplemented!()
    }
}

/// The error of [`Signing::new`] (CHK-006).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `signing_options.no_signature` | [`NoSignature`](Self::NoSignature) | [`Signing::new`] | VAL-250 | CHK-006 |
/// | `signing_options.message_id_invalid` | [`MessageIdInvalid`](Self::MessageIdInvalid) | [`Signing::new`] | VAL-251 | TRC-014 |
/// | `signing_options.signing_domain_invalid` | [`SigningDomainInvalid`](Self::SigningDomainInvalid) | [`Signing::new`] | VAL-220, VAL-230, VAL-240 | ADS-002 |
/// | `signing_options.selector_invalid` | [`SelectorInvalid`](Self::SelectorInvalid) | [`Signing::new`] | VAL-221, VAL-232 | DNS-008 |
/// | `signing_options.selector_repeated` | [`SelectorRepeated`](Self::SelectorRepeated) | [`Signing::new`] | VAL-233 | DK2-009 |
/// | `signing_options.algorithm_not_allowed` | [`AlgorithmNotAllowed`](Self::AlgorithmNotAllowed) | [`Signing::new`] | VAL-222 | DKS-001 |
/// | `signing_options.public_key_invalid` | [`PublicKeyInvalid`](Self::PublicKeyInvalid) | [`Signing::new`] | VAL-223, VAL-234 | SGN-003 |
/// | `signing_options.rsa_key_too_small` | [`RsaKeyTooSmall`](Self::RsaKeyTooSmall) | [`Signing::new`] | VAL-224, VAL-235 | SGN-003 |
/// | `signing_options.rsa_key_too_large` | [`RsaKeyTooLarge`](Self::RsaKeyTooLarge) | [`Signing::new`] | VAL-225, VAL-235 | SAF-002 |
/// | `signing_options.rsa_exponent_not_65537` | [`RsaExponentNot65537`](Self::RsaExponentNot65537) | [`Signing::new`] | VAL-236 | SGN-003 |
/// | `signing_options.header_name_invalid` | [`HeaderNameInvalid`](Self::HeaderNameInvalid) | [`Signing::new`] | VAL-226 | DKS-003 |
/// | `signing_options.from_not_signed` | [`FromNotSigned`](Self::FromNotSigned) | [`Signing::new`] | VAL-227 | DKS-003 |
/// | `signing_options.expiration_invalid` | [`ExpirationInvalid`](Self::ExpirationInvalid) | [`Signing::new`] | VAL-228 | DKS-004 |
/// | `signing_options.no_signer` | [`NoSigner`](Self::NoSigner) | [`Signing::new`] | VAL-231 | DK2-009 |
/// | `signing_options.dkim2_envelope_invalid` | [`Dkim2EnvelopeInvalid`](Self::Dkim2EnvelopeInvalid) | [`Signing::new`] | VAL-237 | DK2-003 |
/// | `signing_options.path_invalid` | [`PathInvalid`](Self::PathInvalid) | [`Signing::new`] | VAL-238 | ADS-020, ADS-021 |
/// | `signing_options.rcpt_to_limit` | [`RcptToLimit`](Self::RcptToLimit) | [`Signing::new`] | VAL-239 | SAF-004 |
/// | `signing_options.nonce_invalid` | [`NonceInvalid`](Self::NonceInvalid) | [`Signing::new`] | VAL-241 | DK2-003 |
/// | `signing_options.revision_invalid` | [`RevisionInvalid`](Self::RevisionInvalid) | [`Signing::new`] | VAL-242 | DK2-014 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SigningOptionsError {
    /// `signing_options.no_signature`: neither a DKIM nor a DKIM2 signature
    /// to add.
    NoSignature,
    /// `signing_options.message_id_invalid`: a message ID of 0 octets or over
    /// 256.
    MessageIdInvalid,
    /// `signing_options.signing_domain_invalid`: a `d=` or `nd=` domain with
    /// no lookup form.
    SigningDomainInvalid,
    /// `signing_options.selector_invalid`: a selector that breaks RFC 6376
    /// §3.1 or makes a key name too long.
    SelectorInvalid,
    /// `signing_options.selector_repeated`: one selector twice in a DKIM2
    /// signature.
    SelectorRepeated,
    /// `signing_options.algorithm_not_allowed`: `Mldsa44` for DKIM.
    AlgorithmNotAllowed,
    /// `signing_options.public_key_invalid`: a stated public key that doesn't
    /// decode for its algorithm.
    PublicKeyInvalid,
    /// `signing_options.rsa_key_too_small`: a stated RSA key under 1,024 bits.
    RsaKeyTooSmall,
    /// `signing_options.rsa_key_too_large`: a stated RSA key over
    /// `rsa_key_bits`.
    RsaKeyTooLarge,
    /// `signing_options.rsa_exponent_not_65537`: a stated DKIM2 RSA key whose
    /// exponent isn't 65537.
    RsaExponentNot65537,
    /// `signing_options.header_name_invalid`: a name to sign that isn't a
    /// field name.
    HeaderNameInvalid,
    /// `signing_options.from_not_signed`: a list of fields to sign without
    /// From.
    FromNotSigned,
    /// `signing_options.expiration_invalid`: an expiration of 0, or one that
    /// makes `x=` longer than twelve digits.
    ExpirationInvalid,
    /// `signing_options.no_signer`: DKIM2 signing with no group, or an empty
    /// group.
    NoSigner,
    /// `signing_options.dkim2_envelope_invalid`: `nd=` together with `mf=` or
    /// `rt=`, or neither.
    Dkim2EnvelopeInvalid,
    /// `signing_options.path_invalid`: an outgoing path that isn't a
    /// Reverse-path or a Forward-path.
    PathInvalid,
    /// `signing_options.rcpt_to_limit`: more outgoing RCPT TO paths than
    /// `rcpt_to`.
    RcptToLimit,
    /// `signing_options.nonce_invalid`: an `n=` over 64 characters, or with a
    /// character outside printable ASCII or a `;`.
    NonceInvalid,
    /// `signing_options.revision_invalid`: a revision whose `Unchanged` ranges
    /// are empty, overlap or go backwards.
    RevisionInvalid,
}

impl SigningOptionsError {
    /// The error's code, such as `signing_options.no_signature` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for SigningOptionsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for SigningOptionsError {}
