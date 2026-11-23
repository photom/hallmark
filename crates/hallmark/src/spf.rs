//! The result of SPF. The record parser, macro expansion and `check_host()`
//! stay crate-private.

use crate::{CheckResults, Data, Domain, Limit, ParseError, Position, QuestionType};

/// The result of SPF for one identity (SPF-001, SPF-002).
///
/// [`CheckResults::spf_helo`] and [`CheckResults::spf_mail_from`] hold one.
/// hallmark builds it, and the verifying caller reads its fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SpfResult {
    /// The identity checked (SPF-001, SPF-002).
    pub identity: SpfIdentity,
    /// The `<domain>` of `check_host()` (ADS-023).
    pub domain: Domain,
    /// The result of RFC 7208 §2.6 (SPF-003).
    pub outcome: SpfOutcome,
    /// Why the evaluation gave that outcome (GEN-004).
    pub reason: SpfReason,
    /// The expanded `exp=` text for `fail`, marked as from a third party
    /// (REC-016, TRC-013).
    pub explanation: Option<Data>,
    /// The result's place in the trace (DMC-005).
    pub position: Position,
}

/// The identity that SPF checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpfIdentity {
    /// The HELO identity (SPF-002, RFC 7208 §2.3).
    Helo,
    /// The MAIL FROM identity (SPF-001, RFC 7208 §2.4).
    MailFrom,
}

/// The result of SPF, RFC 7208 §2.6 (SPF-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpfOutcome {
    /// `none`.
    None,
    /// `neutral`.
    Neutral,
    /// `pass`.
    Pass,
    /// `fail`.
    Fail,
    /// `softfail`.
    Softfail,
    /// `temperror`.
    Temperror,
    /// `permerror`.
    Permerror,
}

/// Why SPF gave its outcome (GEN-004).
///
/// A [`Position`] in a variant points at the trace step that the reason rests
/// on.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SpfReason {
    /// The mechanism at `term` matched, for `Pass`, `Fail`, `Softfail` or
    /// `Neutral` (SPF-003).
    Matched {
        /// The mechanism that matched.
        term: Position,
    },
    /// No mechanism matched, and no `redirect` applied, for `Neutral` (RFC 7208
    /// §4.7).
    Default,
    /// The domain isn't a valid multi-label domain name, for `None` (RFC 7208
    /// §4.3).
    NoValidDomain,
    /// The domain publishes no SPF record, for `None` (RFC 7208 §4.5).
    NoRecord,
    /// The domain publishes more than one SPF record, for `Permerror`
    /// (SPF-007).
    SeveralRecords,
    /// A DNS question failed, for `Temperror` (GEN-005, DNS-009).
    TemporaryFailure {
        /// The type of the question that failed.
        record_type: QuestionType,
        /// The name of the question that failed.
        name: Vec<u8>,
        /// The resolver's description.
        description: Option<Vec<u8>>,
    },
    /// The record doesn't parse, for `Permerror` (REC-010).
    Syntax(ParseError),
    /// The term at `term` was the eleventh that causes a DNS lookup, for
    /// `Permerror` (SPF-006).
    LookupLimit {
        /// The term that went over the limit.
        term: Position,
    },
    /// The term at `term` went over `spf_void_lookups`, for `Permerror`
    /// (SPF-006).
    VoidLookupLimit {
        /// The term that went over the limit.
        term: Position,
    },
    /// The `mx` at `term` got more than 10 MX records, for `Permerror`
    /// (SPF-006).
    MxLimit {
        /// The `mx` term.
        term: Position,
    },
    /// The term at `term` reached a limit of the check, such as
    /// `dns_questions`, for `Permerror` (SAF-002).
    CheckLimit {
        /// The limit reached.
        limit: Limit,
        /// The term that reached it.
        term: Position,
    },
    /// The `include` or `redirect` at `term` gave this outcome, for
    /// `Temperror` or `Permerror` (RFC 7208 §5.2, §6.1).
    Nested {
        /// The `include` or `redirect` term.
        term: Position,
    },
}
