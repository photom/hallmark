//! The verification time.

use crate::*;

/// The time a message was received, in seconds since 1970-01-01T00:00:00Z.
///
/// The core reads no clock, so the caller passes this time (TIM-001). RFC 6376
/// §3.5 asks for the time the message was first received at the verifier's
/// administrative domain, and draft-09 §14.3 measures DKIM2 timestamps against
/// the time the message was accepted over SMTP. A caller that checks a stored
/// message later passes the time it was received, not the current time.
///
/// The value lies from 0 to 999,999,999,999, the largest `t=` of twelve
/// digits (VAL-201).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VerificationTime {
    seconds: u64,
}

impl VerificationTime {
    /// Makes a verification time from seconds since 1970-01-01T00:00:00Z.
    ///
    /// # Parameters
    ///
    /// - `seconds`: seconds since 1970-01-01T00:00:00Z.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-201 | `seconds` | At most 999,999,999,999, the largest `t=` of twelve digits | [`options.time_out_of_range`](OptionsError::TimeOutOfRange) | TIM-001, DKS-004, RFC 6376 §3.5 | API-201 |
    ///
    /// Requirements: TIM-001.
    pub fn from_unix_seconds(seconds: u64) -> Result<VerificationTime, OptionsError> {
        unimplemented!()
    }

    /// The seconds the time was made from.
    ///
    /// Requirements: TIM-001.
    pub fn unix_seconds(self) -> u64 {
        unimplemented!()
    }
}
