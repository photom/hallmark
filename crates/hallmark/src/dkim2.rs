//! The result of DKIM2 verification. The fields, recipes, key and policy
//! records, verification, signing and revisions stay crate-private. The whole
//! module exists only with the `unstable-dkim2` feature.

use crate::*;

/// The result of DKIM2 verification (DK2-013). Available with the
/// `unstable-dkim2` feature.
///
/// [`Results::dkim2`] holds it. hallmark builds it, and the caller reads its
/// fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Dkim2Result {
    /// The output state of draft-09 §14.1 (DK2-013).
    pub state: Dkim2State,
    /// The step that decided the state (GEN-004).
    pub reason: Dkim2Reason,
    /// The human-readable string of draft-09 §14 with its placeholders filled
    /// in, for a state other than PASS. `None` where the draft has no string
    /// (DK2-013).
    pub draft_text: Option<String>,
    /// The `header.d` property of authres-02 §3.2 (AR-004).
    pub header_d: Option<Data>,
    /// The `header.i` property of authres-02 §3.2 (AR-004).
    pub header_i: Option<Data>,
    /// The result's place in the trace (DMC-005).
    pub position: Position,
}

/// The output state of DKIM2, draft-09 §14.1 (DK2-013). Available with the
/// `unstable-dkim2` feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dkim2State {
    /// NONE.
    None,
    /// PASS.
    Pass,
    /// FAIL.
    Fail,
    /// PERMERROR.
    Permerror,
    /// TEMPERROR.
    Temperror,
}

/// The step that decided the DKIM2 state (GEN-004). Available with the
/// `unstable-dkim2` feature.
///
/// Each step variant names one step of
/// [DKIM2](https://github.com/photom/hallmark/blob/main/docs/reference/design/dkim2.md#steps)
/// and carries `step`, the trace step that decided the state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Dkim2Reason {
    /// The message has no DKIM2 fields, for `None` (DK2-013, AR-004).
    NoFields,
    /// Every check passed, for `Pass` (DK2-013).
    Verified,
    /// The DKIM2 header fields aren't valid, for `Permerror` (draft-09
    /// §14.2).
    Fields {
        /// The trace step that decided the state.
        step: Position,
    },
    /// A timestamp check failed, for `Fail` (draft-09 §14.3).
    Timestamps {
        /// The trace step that decided the state.
        step: Position,
    },
    /// The chain of custody check failed, for `Fail` (draft-09 §14.4).
    ChainOfCustody {
        /// The trace step that decided the state.
        step: Position,
    },
    /// A public key couldn't be fetched, or its key record can't be used,
    /// for `Temperror` or `Permerror` (draft-09 §14.5, REC-053).
    Keys {
        /// The trace step that decided the state.
        step: Position,
    },
    /// The signature verification failed, for `Fail` or `Permerror`
    /// (draft-09 §14.6).
    Signatures {
        /// The trace step that decided the state.
        step: Position,
    },
    /// The body or header hashes didn't match, for `Fail` (draft-09 §14.7).
    Hashes {
        /// The trace step that decided the state.
        step: Position,
    },
    /// A From header field isn't Authentic, for `Fail` or `Permerror`
    /// (draft-09 §14.8).
    AuthenticFrom {
        /// The trace step that decided the state.
        step: Position,
    },
    /// The message reached a limit, for `Permerror` (SAF-002).
    Limit(Limit),
}
