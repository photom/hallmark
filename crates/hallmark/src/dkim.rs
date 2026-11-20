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
    /// Why the signature got that outcome (GEN-004). When the selector has
    /// several key records and none verifies the signature, this is the
    /// reason of the first record tried that passed its key checks, or of the
    /// first record tried if none did (REC-052).
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
    /// A check of the DKIM-Signature field failed, for `Permfail` (RFC 6376
    /// §6.1.1, TIM-002).
    Field {
        /// The check that failed.
        check: DkimFieldCheck,
        /// The trace step that failed.
        step: Position,
    },
    /// A check of the key record failed, or the selector has no key record,
    /// for `Permfail` (RFC 6376 §6.1.2, DKV-003).
    Key {
        /// The check that failed.
        check: DkimKeyCheck,
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

/// The check of a DKIM-Signature field that failed (GEN-004, AR-002).
///
/// [Field checks](https://github.com/photom/hallmark/blob/main/docs/reference/design/dkim.md#field-checks)
/// lists them in the order hallmark runs them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DkimFieldCheck {
    /// The tag list or a tag value doesn't parse, or a tag appears twice:
    /// "signature syntax error" (HDR-006, RFC 6376 §6.1.1).
    Syntax,
    /// `v`, `a`, `b`, `bh`, `d`, `h` or `s` is missing: "signature missing
    /// required tag" (RFC 6376 §6.1.1).
    MissingTag,
    /// `v=` isn't `1`: "incompatible version" (RFC 6376 §6.1.1).
    Version,
    /// `a=` is neither `rsa-sha256` nor `ed25519-sha256` (DKV-002, RFC 8301
    /// §3.1).
    Algorithm,
    /// The domain of `i=` is neither `d=` nor a subdomain of it: "domain
    /// mismatch" (ADS-013, RFC 6376 §6.1.1).
    DomainMismatch,
    /// `h=` doesn't name From: "From field not signed" (RFC 6376 §6.1.1).
    FromNotSigned,
    /// `x=` isn't later than `t=` (TIM-003, RFC 6376 §3.5).
    TimeOrder,
    /// `d=`, `s=` or the key name breaks the name rules (ADS-004, ADS-008,
    /// DNS-008).
    Name,
    /// `x=` has passed: "signature expired" (TIM-002, RFC 6376 §6.1.1).
    Expired,
    /// `t=` is in the future (TIM-002, RFC 6376 §3.5).
    FutureTimestamp,
}

/// The check of a DKIM key record that failed (GEN-004, AR-002).
///
/// [Key checks](https://github.com/photom/hallmark/blob/main/docs/reference/design/dkim.md#key-checks)
/// lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DkimKeyCheck {
    /// The selector has no key record that applies: "no key for signature"
    /// (REC-050, RFC 6376 §6.1.2).
    NoKey,
    /// The tag list or the key data doesn't parse: "key syntax error"
    /// (REC-050, REC-051).
    Syntax,
    /// `p=` is empty: "key revoked" (REC-051).
    Revoked,
    /// `k=` doesn't fit `a=`: "inappropriate key algorithm" (REC-051).
    KeyAlgorithm,
    /// `h=` doesn't list the hash of `a=`: "inappropriate hash algorithm"
    /// (REC-051).
    HashAlgorithm,
    /// The key has `t=s`, and the domain of `i=` isn't `d=`: "domain
    /// mismatch" (REC-050,
    /// [ADR 0044](https://github.com/photom/hallmark/blob/main/docs/adr/0044-a-dkim-signature-that-breaks-the-s-flag-of-its-key-gets-permfail-domain-mismatch.md)).
    DomainMismatch,
    /// The RSA modulus has fewer than 1,024 bits or more than `rsa_key_bits`
    /// (DKV-003, RFC 8301 §3.2).
    KeySize,
    /// The RSA public exponent has more bits than `rsa_exponent_bits`
    /// (REC-051,
    /// [ADR 0043](https://github.com/photom/hallmark/blob/main/docs/adr/0043-rsa-public-exponents-have-a-size-limit-in-dkim-and-must-be-65537-in-dkim2.md)).
    ExponentSize,
}
