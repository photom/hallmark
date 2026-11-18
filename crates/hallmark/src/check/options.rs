//! The options of a check.

use crate::*;

/// Everything a caller fixes when it creates a check. Nothing changes the
/// options later (INV-015).
///
/// The caller builds options with [`Options::new`], which takes the kinds and
/// the time and sets every other field to its default, and then sets the
/// other public fields. `Options` checks nothing while the caller builds it:
/// [`Check::new`] checks every rule at once, VAL-210 to VAL-242. [`Limits`]
/// checks its own rules in its setters.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Options {
    /// The kinds of check to run. A kind listed twice counts once (GEN-001).
    pub kinds: Vec<Kind>,
    /// The time the message was received (TIM-001).
    pub time: VerificationTime,
    /// Whether signature times are compared with `time`. Default `true`
    /// (TIM-002).
    pub time_checks: bool,
    /// Off, summary or full. Default [`Level::Summary`] (TRC-004).
    pub trace_level: Level,
    /// The caller's ID for the message, which every event carries, such as the
    /// MTA's queue ID. Default `None` (TRC-014).
    pub message_id: Option<String>,
    /// The limits of the check. Default `Limits::default()` (PAR-004).
    pub limits: Limits,
    /// The form of the values that [`Check::header_field`] takes and that
    /// generated fields come out in. Default `FieldForm::default()`: CRLF,
    /// leading space kept (STR-008, STR-009).
    pub field_form: FieldForm,
    /// The name of the receiving host for SPF's `%{r}`, as UTF-8 text.
    /// Default `None`, so `%{r}` expands to `unknown` (REC-014).
    pub receiving_host: Option<String>,
    /// The DMARC specification to evaluate under. Default
    /// [`DmarcMode::Rfc9989`] (DMC-001).
    pub dmarc_mode: DmarcMode,
    /// The source of DMARC's SPF identifier. Default [`SpfSource::SameCheck`]
    /// (DMC-005).
    pub dmarc_spf: SpfSource,
    /// The source of DMARC's DKIM identifiers. Default
    /// [`DkimSource::SameCheck`] (DMC-005).
    pub dmarc_dkim: DkimSource,
    /// Whether DMARC evaluates a message with several author domains. Default
    /// `false` (DMC-002).
    pub several_author_domains: bool,
    /// The DKIM signatures to add. Default empty (DKS-002).
    pub dkim_signing: Vec<DkimSigning>,
    /// The DKIM2 signature to add. Default `None` (DK2-003). Available with
    /// the `unstable-dkim2` feature.
    #[cfg(feature = "unstable-dkim2")]
    pub dkim2_signing: Option<Dkim2Signing>,
    /// Whether each signature from the signer is verified before its field is
    /// written. Default `false` (SGN-006).
    pub verify_signatures: bool,
}

impl Options {
    /// Makes options with the kinds and the time, and every other field at
    /// its default.
    ///
    /// # Parameters
    ///
    /// - `kinds`: the kinds of check to run, in any order. A kind listed twice
    ///   counts once.
    /// - `time`: the time the message was received (TIM-001).
    ///
    /// Building options checks nothing. [`Check::new`] checks them.
    ///
    /// Requirements: GEN-001, TIM-001.
    pub fn new(kinds: Vec<Kind>, time: VerificationTime) -> Options {
        unimplemented!()
    }
}

/// A kind of check.
///
/// `Dkim2` and `Dkim2Sign` exist only with the `unstable-dkim2` feature, so the
/// enum is `#[non_exhaustive]`: a crate that turns the feature on can't break a
/// `match` elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Kind {
    /// SPF for the HELO identity (SPF-002).
    SpfHelo,
    /// SPF for the MAIL FROM identity (SPF-001).
    SpfMailFrom,
    /// DKIM verification (DKV-001).
    Dkim,
    /// DKIM2 verification (DK2-003). Available with the `unstable-dkim2`
    /// feature.
    #[cfg(feature = "unstable-dkim2")]
    Dkim2,
    /// DMARC evaluation (DMC-001).
    Dmarc,
    /// DKIM signing (DKS-001).
    DkimSign,
    /// DKIM2 signing (DK2-003). Available with the `unstable-dkim2` feature.
    #[cfg(feature = "unstable-dkim2")]
    Dkim2Sign,
}

/// The DMARC specification to evaluate under (DMC-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DmarcMode {
    /// DMARC under RFC 9989, the default.
    Rfc9989,
    /// DMARC under RFC 7489, with public suffix questions to the caller
    /// (DMC-010).
    Rfc7489,
}

/// The source of DMARC's SPF identifier.
///
/// An earlier result carries its position in the trace of the check that
/// made it, and DMARC's alignment steps name it by that position (DMC-005).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpfSource {
    /// The SPF MAIL FROM kind of this check gives the identifier (GEN-001).
    SameCheck,
    /// A result from another check, for the MAIL FROM identity (DMC-005).
    Earlier(SpfResult),
    /// No SPF identifier (RFC 9989 §5.3.3).
    Absent,
}

/// The source of DMARC's DKIM identifiers.
///
/// An earlier result carries its position in the trace of the check that
/// made it, and DMARC's alignment steps name it by that position (DMC-005).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DkimSource {
    /// The DKIM kind of this check gives the identifiers (GEN-001).
    SameCheck,
    /// Results from another check (DMC-005).
    Earlier(Vec<DkimResult>),
    /// No DKIM identifier (RFC 9989 §5.3.3).
    Absent,
}

/// The error of [`Check::new`] and [`VerificationTime::from_unix_seconds`]
/// (GEN-001).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `options.time_out_of_range` | [`TimeOutOfRange`](Self::TimeOutOfRange) | [`VerificationTime::from_unix_seconds`] | VAL-201 | TIM-001 |
/// | `options.no_kind` | [`NoKind`](Self::NoKind) | [`Check::new`] | VAL-210 | GEN-001 |
/// | `options.source_kind_missing` | [`SourceKindMissing`](Self::SourceKindMissing) | [`Check::new`] | VAL-211, VAL-212 | DMC-005 |
/// | `options.earlier_result_invalid` | [`EarlierResultInvalid`](Self::EarlierResultInvalid) | [`Check::new`] | VAL-213 | DMC-005 |
/// | `options.message_id_invalid` | [`MessageIdInvalid`](Self::MessageIdInvalid) | [`Check::new`] | VAL-214 | TRC-014 |
/// | `options.receiving_host_invalid` | [`ReceivingHostInvalid`](Self::ReceivingHostInvalid) | [`Check::new`] | VAL-215 | REC-014 |
/// | `options.signing_mismatch` | [`SigningMismatch`](Self::SigningMismatch) | [`Check::new`] | VAL-216, VAL-217 | GEN-001 |
/// | `options.signing_domain_invalid` | [`SigningDomainInvalid`](Self::SigningDomainInvalid) | [`Check::new`] | VAL-220, VAL-230, VAL-240 | ADS-002 |
/// | `options.selector_invalid` | [`SelectorInvalid`](Self::SelectorInvalid) | [`Check::new`] | VAL-221, VAL-232 | DNS-008 |
/// | `options.selector_repeated` | [`SelectorRepeated`](Self::SelectorRepeated) | [`Check::new`] | VAL-233 | DK2-009 |
/// | `options.algorithm_not_allowed` | [`AlgorithmNotAllowed`](Self::AlgorithmNotAllowed) | [`Check::new`] | VAL-222 | DKS-001 |
/// | `options.public_key_invalid` | [`PublicKeyInvalid`](Self::PublicKeyInvalid) | [`Check::new`] | VAL-223, VAL-234 | SGN-003 |
/// | `options.rsa_key_too_small` | [`RsaKeyTooSmall`](Self::RsaKeyTooSmall) | [`Check::new`] | VAL-224, VAL-235 | SGN-003 |
/// | `options.rsa_key_too_large` | [`RsaKeyTooLarge`](Self::RsaKeyTooLarge) | [`Check::new`] | VAL-225, VAL-235 | SAF-002 |
/// | `options.rsa_exponent_not_65537` | [`RsaExponentNot65537`](Self::RsaExponentNot65537) | [`Check::new`] | VAL-236 | SGN-003 |
/// | `options.header_name_invalid` | [`HeaderNameInvalid`](Self::HeaderNameInvalid) | [`Check::new`] | VAL-226 | DKS-003 |
/// | `options.from_not_signed` | [`FromNotSigned`](Self::FromNotSigned) | [`Check::new`] | VAL-227 | DKS-003 |
/// | `options.expiration_invalid` | [`ExpirationInvalid`](Self::ExpirationInvalid) | [`Check::new`] | VAL-228 | DKS-004 |
/// | `options.no_signer` | [`NoSigner`](Self::NoSigner) | [`Check::new`] | VAL-231 | DK2-009 |
/// | `options.dkim2_envelope_invalid` | [`Dkim2EnvelopeInvalid`](Self::Dkim2EnvelopeInvalid) | [`Check::new`] | VAL-237 | DK2-003 |
/// | `options.path_invalid` | [`PathInvalid`](Self::PathInvalid) | [`Check::new`] | VAL-238 | ADS-020, ADS-021 |
/// | `options.rcpt_to_limit` | [`RcptToLimit`](Self::RcptToLimit) | [`Check::new`] | VAL-239 | SAF-004 |
/// | `options.nonce_invalid` | [`NonceInvalid`](Self::NonceInvalid) | [`Check::new`] | VAL-241 | DK2-003 |
/// | `options.revision_invalid` | [`RevisionInvalid`](Self::RevisionInvalid) | [`Check::new`] | VAL-242 | DK2-014 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum OptionsError {
    /// `options.time_out_of_range`: a time over 999,999,999,999 seconds.
    TimeOutOfRange,
    /// `options.no_kind`: no kind of check.
    NoKind,
    /// `options.source_kind_missing`: DMARC takes an identifier from a kind
    /// the check doesn't run.
    SourceKindMissing,
    /// `options.earlier_result_invalid`: an earlier SPF result for the HELO
    /// identity.
    EarlierResultInvalid,
    /// `options.message_id_invalid`: a message ID of 0 octets or over 256.
    MessageIdInvalid,
    /// `options.receiving_host_invalid`: a receiving host with no lookup
    /// form.
    ReceivingHostInvalid,
    /// `options.signing_mismatch`: a signing kind without its signing
    /// options, or the reverse.
    SigningMismatch,
    /// `options.signing_domain_invalid`: a `d=` or `nd=` domain with no
    /// lookup form.
    SigningDomainInvalid,
    /// `options.selector_invalid`: a selector that breaks RFC 6376 §3.1 or
    /// makes a key name too long.
    SelectorInvalid,
    /// `options.selector_repeated`: one selector twice in a DKIM2 signature.
    SelectorRepeated,
    /// `options.algorithm_not_allowed`: `Mldsa44` for DKIM.
    AlgorithmNotAllowed,
    /// `options.public_key_invalid`: a stated public key that doesn't decode
    /// for its algorithm.
    PublicKeyInvalid,
    /// `options.rsa_key_too_small`: a stated RSA key under 1,024 bits.
    RsaKeyTooSmall,
    /// `options.rsa_key_too_large`: a stated RSA key over `rsa_key_bits`.
    RsaKeyTooLarge,
    /// `options.rsa_exponent_not_65537`: a stated DKIM2 RSA key whose
    /// exponent isn't 65537.
    RsaExponentNot65537,
    /// `options.header_name_invalid`: a name to sign that isn't a field name.
    HeaderNameInvalid,
    /// `options.from_not_signed`: a list of fields to sign without From.
    FromNotSigned,
    /// `options.expiration_invalid`: an expiration of 0, or one that makes
    /// `x=` longer than twelve digits.
    ExpirationInvalid,
    /// `options.no_signer`: DKIM2 signing with no group, or an empty group.
    NoSigner,
    /// `options.dkim2_envelope_invalid`: `nd=` together with `mf=` or `rt=`,
    /// or neither.
    Dkim2EnvelopeInvalid,
    /// `options.path_invalid`: an outgoing path that isn't a Reverse-path or
    /// a Forward-path.
    PathInvalid,
    /// `options.rcpt_to_limit`: more outgoing RCPT TO paths than `rcpt_to`.
    RcptToLimit,
    /// `options.nonce_invalid`: an `n=` over 64 characters, or with a
    /// character outside printable ASCII or a `;`.
    NonceInvalid,
    /// `options.revision_invalid`: a revision whose `Unchanged` ranges are
    /// empty, overlap or go backwards.
    RevisionInvalid,
}

impl OptionsError {
    /// The error's code, such as `options.no_kind` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for OptionsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for OptionsError {}
