//! The options of a check.

use crate::{
    Check, DkimResult, FieldForm, Level, Limits, LimitsError, SigningOptions, SpfResult,
    VerificationTime,
};

/// Everything a verifying caller fixes when it creates a check. Nothing
/// changes the options later (INV-015).
///
/// The verifying caller builds options with [`CheckOptions::new`], which takes
/// the kinds and the time and sets every other field to its default, and then
/// sets the other public fields. `CheckOptions` checks nothing while the
/// verifying caller builds it: [`Check::new`] checks every rule at once,
/// VAL-209 to VAL-215, the rules of [`Limits`] included. Signing has options
/// of its own, [`SigningOptions`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CheckOptions {
    /// The kinds of check to run. A kind listed twice counts once (GEN-001).
    pub kinds: Vec<Kind>,
    /// The time the message was received (TIM-001).
    pub time: VerificationTime,
    /// Whether signature times are compared with `time`. Default `true`
    /// (TIM-002).
    pub time_checks: bool,
    /// Off, summary or full. Default [`Level::Summary`] (TRC-004).
    pub trace_level: Level,
    /// The verifying caller's ID for the message, which every event carries,
    /// such as the MTA's queue ID. Default `None` (TRC-014).
    pub message_id: Option<String>,
    /// The limits of the check. Default `Limits::default()` (PAR-004).
    pub limits: Limits,
    /// The form of the values that [`Check::header_field`] takes. Default
    /// `FieldForm::default()`: CRLF, leading space kept (STR-008).
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
}

impl CheckOptions {
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
    pub fn new(kinds: Vec<Kind>, time: VerificationTime) -> CheckOptions {
        unimplemented!()
    }
}

/// A kind of check.
///
/// `Dkim2` exists only with the `unstable-dkim2` feature, so the enum is
/// `#[non_exhaustive]`: a crate that turns the feature on can't break a
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
}

/// The DMARC specification to evaluate under (DMC-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DmarcMode {
    /// DMARC under RFC 9989, the default.
    Rfc9989,
    /// DMARC under RFC 7489, with public suffix questions to the verifying
    /// caller (DMC-010).
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

/// The error of [`Check::new`] (GEN-001, PAR-004).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `check_options.limits_invalid` | [`LimitsInvalid`](Self::LimitsInvalid) | [`Check::new`] | VAL-209 | PAR-004 |
/// | `check_options.no_kind` | [`NoKind`](Self::NoKind) | [`Check::new`] | VAL-210 | GEN-001 |
/// | `check_options.source_kind_missing` | [`SourceKindMissing`](Self::SourceKindMissing) | [`Check::new`] | VAL-211, VAL-212 | DMC-005 |
/// | `check_options.earlier_result_invalid` | [`EarlierResultInvalid`](Self::EarlierResultInvalid) | [`Check::new`] | VAL-213 | DMC-005 |
/// | `check_options.message_id_invalid` | [`MessageIdInvalid`](Self::MessageIdInvalid) | [`Check::new`] | VAL-214 | TRC-014 |
/// | `check_options.receiving_host_invalid` | [`ReceivingHostInvalid`](Self::ReceivingHostInvalid) | [`Check::new`] | VAL-215 | REC-014 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckOptionsError {
    /// `check_options.limits_invalid`: limits that break a rule of their `validate` method.
    LimitsInvalid(LimitsError),
    /// `check_options.no_kind`: no kind of check.
    NoKind,
    /// `check_options.source_kind_missing`: DMARC takes an identifier from a
    /// kind the check doesn't run.
    SourceKindMissing,
    /// `check_options.earlier_result_invalid`: an earlier SPF result for the
    /// HELO identity.
    EarlierResultInvalid,
    /// `check_options.message_id_invalid`: a message ID of 0 octets or over
    /// 256.
    MessageIdInvalid,
    /// `check_options.receiving_host_invalid`: a receiving host with no lookup
    /// form.
    ReceivingHostInvalid,
}

impl CheckOptionsError {
    /// The error's code, such as `check_options.no_kind` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for CheckOptionsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for CheckOptionsError {}
