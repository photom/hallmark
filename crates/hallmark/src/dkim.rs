//! The results of DKIM verification. DKIM-Signature fields, key records,
//! verification and signing stay crate-private.

use crate::*;

/// The results of DKIM verification (DKV-001).
///
/// [`Results::dkim`] holds them. hallmark builds them, and the caller reads
/// their fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DkimResults {
    /// One result for each DKIM-Signature field, in header order (DKV-004).
    pub signatures: Vec<DkimResult>,
    /// The header limit that stopped DKIM before any signature, which gives
    /// `permerror` for the whole check (SAF-002).
    pub stopped: Option<Limit>,
}

/// The result of one DKIM-Signature field (DKV-004).
///
/// `domain`, `selector` and `algorithm` are optional, because a field that
/// lacks them still gets PERMFAIL (RFC 6376 §6.1.1).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DkimResult {
    /// `d=`, or `None` when the field gives no valid `d=` (DKV-001).
    pub domain: Option<Domain>,
    /// `i=` (RFC 6376 §3.5).
    pub identity: Option<Data>,
    /// `s=`, or `None` when the field gives none (DKV-001).
    pub selector: Option<Data>,
    /// `a=` as written, or `None` when the field gives none (DKV-002).
    pub algorithm: Option<Data>,
    /// The result of RFC 6376 §3.9 (DKV-004).
    pub outcome: DkimOutcome,
    /// Why the signature got that outcome (GEN-004).
    pub reason: DkimReason,
    /// The signature counts as testing: a key record that verified it has
    /// `t=y` (DKV-009, REC-052).
    pub testing: bool,
    /// Each key record at the selector that hallmark tried, with its own
    /// reason, in ascending order of the record's octets, so the order of the
    /// records in the answer changes nothing. Empty when no record was tried:
    /// the selector has no key record, or more records than the limit
    /// `key_records` (REC-052,
    /// [ADR 0025](https://github.com/photom/hallmark/blob/main/docs/adr/0025-a-dkim-selector-with-several-key-records-passes-if-any-of-them-verifies-the-signature.md)).
    pub key_records: Vec<DkimKeyRecord>,
    /// Body octets after the `l=` cut (DKV-006).
    pub unsigned_body: Option<u64>,
    /// The result's place in the trace (DMC-005).
    pub position: Position,
}

/// One key record that DKIM verification tried for a signature (REC-052).
///
/// A selector can publish several records. The signature passes if any of
/// them verifies it, and [`DkimResult::key_records`] lists each one with its
/// own reason.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DkimKeyRecord {
    /// The record as the resolver gave it, its strings joined (DNS-006,
    /// TRC-013).
    pub record: Data,
    /// The record has `t=y` (DKV-009).
    pub testing: bool,
    /// [`DkimReason::Verified`] for a record that verified the signature, and
    /// otherwise the check that failed with this record, such as a key check
    /// or the signature check.
    pub reason: DkimReason,
    /// The record's place in the trace (TRC-008).
    pub position: Position,
}

/// The result of one DKIM signature, RFC 6376 §3.9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DkimOutcome {
    /// SUCCESS.
    Success,
    /// PERMFAIL.
    Permfail,
    /// TEMPFAIL.
    Tempfail,
}

/// Why a DKIM signature got its outcome (GEN-004).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DkimReason {
    /// The signature verified, for `Success` (DKV-001).
    Verified,
    /// A check of the DKIM-Signature field failed at `step`, such as a missing
    /// tag or an expired `x=`, for `Permfail` (RFC 6376 §6.1.1).
    Field {
        /// The trace step that failed.
        step: Position,
    },
    /// A check of the key record failed at `step`, such as a revoked key or a
    /// key too short, for `Permfail` (RFC 6376 §6.1.2, DKV-003).
    Key {
        /// The trace step that failed.
        step: Position,
    },
    /// The body hash didn't match `bh=`, for `Permfail` (RFC 6376 §6.1.3).
    BodyHash {
        /// The trace step that compared the hashes.
        step: Position,
    },
    /// The signature didn't verify over the header hash, for `Permfail`
    /// (RFC 6376 §6.1.3).
    Signature {
        /// The trace step that verified the signature.
        step: Position,
    },
    /// The signature reached a limit, such as `dkim_signatures` or
    /// `key_records`, for `Permfail` (SAF-002).
    Limit(Limit),
    /// The key question failed, for `Tempfail` (GEN-005, DNS-009).
    KeyTemporaryFailure {
        /// The resolver's description.
        description: Option<Vec<u8>>,
    },
}
