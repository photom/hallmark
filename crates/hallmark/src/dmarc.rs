//! The result of DMARC. The record parser, policy discovery, alignment and
//! evaluation stay crate-private.

use crate::{CheckOptions, CheckResults, Data, Domain, Limit, Position, QuestionType};

/// The result of DMARC for one author domain (DMC-002, DMC-006).
///
/// [`CheckResults::dmarc`] holds one, or one for each author domain when
/// [`CheckOptions::several_author_domains`] is on. hallmark computes no policy
/// from these values: the verifying caller decides what to do with the message
/// ([ADR 0020](https://github.com/photom/hallmark/blob/main/docs/adr/0020-test-modes-are-reported-and-the-caller-decides-what-they-mean.md)).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DmarcResult {
    /// The author domain, absent when DMARC wasn't applied for lack of one
    /// (DMC-002).
    pub author_domain: Option<Domain>,
    /// The DMARC result (DMC-002, DMC-006).
    pub outcome: DmarcOutcome,
    /// Why DMARC gave that result (GEN-004, AR-002).
    pub reason: DmarcReason,
    /// The policy tag that applied (DMC-006, REC-032).
    pub policy_tag: PolicyTag,
    /// Every policy tag of the record, as published (REC-032).
    pub published: PublishedPolicy,
    /// The tag that set test mode: `t=y`, or `pct=0` in RFC 7489 mode
    /// (DMC-007).
    pub test_mode: Option<TestModeTag>,
    /// A pass rests only on DKIM keys with `t=y` (DMC-008).
    pub rests_on_testing_keys: bool,
    /// Each identifier checked, in the order checked (DMC-005).
    pub alignments: Vec<Alignment>,
    /// The result's place in the trace (DMC-005).
    pub position: Position,
}

/// The DMARC result, under RFC 9989 or RFC 7489 (DMC-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DmarcOutcome {
    /// `pass`.
    Pass,
    /// `fail`.
    Fail,
    /// `none`.
    None,
    /// `temperror`.
    Temperror,
    /// `permerror`.
    Permerror,
    /// DMARC wasn't applied. [`DmarcReason::NotApplied`] says why
    /// (DMC-002).
    NotApplied,
}

/// Why DMARC gave its result (GEN-004, AR-002).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DmarcReason {
    /// An identifier aligned with the author domain and passed, for `Pass`
    /// (RFC 9989 §5.3.5, DMC-005).
    AlignedPass {
        /// The SPF or DKIM result of the first such identifier, in the order
        /// checked.
        identifier: Position,
    },
    /// A record applies, and no identifier aligned with a pass, for `Fail`
    /// (RFC 9989 §5.3.5).
    NoAlignedPass,
    /// No DMARC Policy Record, for `None` (DMC-006).
    NoRecord,
    /// A DNS question that the result needs failed, for `Temperror` (GEN-005,
    /// DNS-009, REC-032).
    TemporaryFailure {
        /// The type of the question that failed.
        record_type: QuestionType,
        /// The name of the question that failed.
        name: Vec<u8>,
        /// The resolver's description.
        description: Option<Vec<u8>>,
    },
    /// The verifying caller has no public suffix for `domain`, and the result
    /// depends on its organizational domain, for `Temperror` (DMC-011).
    SuffixUnavailable {
        /// The domain whose public suffix is unavailable.
        domain: Domain,
        /// The verifying caller's description.
        description: Option<Vec<u8>>,
    },
    /// The check reached a limit, such as `dns_questions`, for `Permerror`
    /// (SAF-002).
    Limit(Limit),
    /// Why DMARC wasn't applied, for `NotApplied` (DMC-002).
    NotApplied(NotApplied),
}

/// Why DMARC wasn't applied (DMC-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NotApplied {
    /// The message has no From field (DMC-002).
    NoFrom,
    /// A From field doesn't parse (DMC-002, FLD-055).
    FromUnparsable,
    /// The From field holds a group with no mailbox (DMC-002, RFC 6854).
    NoMailbox,
    /// The From address has a domain literal (ADS-005,
    /// [ADR 0013](https://github.com/photom/hallmark/blob/main/docs/adr/0013-a-from-domain-literal-gives-dmarc-no-author-domain.md)).
    DomainLiteral,
    /// The author domain has no lookup form (ADS-002).
    NoLookupForm,
    /// More than one author domain, and `several_author_domains` is off
    /// (DMC-002).
    SeveralAuthorDomains,
    /// More author domains than `author_domains` (DMC-002, SAF-002).
    AuthorDomainLimit,
    /// More than one DMARC Policy Record remains at a name, in RFC 7489 mode
    /// (REC-030, RFC 7489 §6.6.3).
    SeveralRecords,
    /// The policy tag is missing or invalid, and `rua` holds no valid URI
    /// (REC-032,
    /// [ADR 0023](https://github.com/photom/hallmark/blob/main/docs/adr/0023-dmarc-follows-rfc-9989-section-4-10-1-for-invalid-policy-tags-and-section-4-7-for-nonexistent-subdomains.md)).
    NoValidPolicy,
}

/// The policy tag that applied (DMC-006, REC-032).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyTag {
    /// The tag that applied, with its value as published (DMC-006).
    Applied {
        /// `p`, `sp` or `np`.
        tag: PolicyTagName,
        /// The tag's value as published.
        value: Data,
    },
    /// The question about the author domain failed, so the tag couldn't be
    /// determined (REC-032).
    Undetermined,
    /// No DMARC Policy Record applied, as for `None` and `NotApplied`
    /// (DMC-006).
    NoPolicy,
}

/// The name of a policy tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyTagName {
    /// `p`.
    P,
    /// `sp`.
    Sp,
    /// `np`, in RFC 9989 mode.
    Np,
}

/// Each policy tag of the record, as published (REC-032).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PublishedPolicy {
    /// The value of `p`, or `None` when absent.
    pub p: Option<Data>,
    /// The value of `sp`, or `None` when absent.
    pub sp: Option<Data>,
    /// The value of `np`, or `None` when absent.
    pub np: Option<Data>,
}

/// The tag that set test mode (DMC-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestModeTag {
    /// `t=y`, in RFC 9989 mode.
    T,
    /// `pct=0`, in RFC 7489 mode.
    Pct,
}

/// One identifier that DMARC checked for alignment (DMC-005).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Alignment {
    /// The SPF or DKIM result the identifier came from (DMC-005).
    pub identifier: Position,
    /// The identifier's domain (DMC-005).
    pub domain: Domain,
    /// The mode from `aspf` or `adkim` (DMC-005).
    pub mode: AlignmentMode,
    /// Whether the domain aligned with the author domain (DMC-005).
    pub aligned: bool,
}

/// The alignment mode, from `aspf` or `adkim`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlignmentMode {
    /// Relaxed.
    Relaxed,
    /// Strict.
    Strict,
}
