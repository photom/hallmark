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
pub struct CheckResults {
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
    /// when [`CheckOptions::several_author_domains`] is on. Empty otherwise
    /// (DMC-002).
    pub dmarc: Vec<DmarcResult>,
    /// Whether time checks were on (TIM-005).
    pub time_checks: bool,
}
