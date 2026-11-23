//! The verification time, which a signing also takes as its signing time.

/// The largest `t=` of twelve digits (RFC 6376 §3.5, HDR-013).
const LARGEST_SECONDS: u64 = 999_999_999_999;

/// A time in seconds since 1970-01-01T00:00:00Z: for a check, the time the
/// message was received, and for a signing, the signing time.
///
/// The core reads no clock, so the caller passes this time (TIM-001, DKS-004).
/// RFC 6376 §3.5 asks for the time the message was first received at the
/// verifier's administrative domain, and draft-09 §14.3 measures DKIM2
/// timestamps against the time the message was accepted over SMTP. A verifying
/// caller that checks a stored message later passes the time it was received,
/// not the current time.
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
    /// | VAL-201 | `seconds` | At most 999,999,999,999, the largest `t=` of twelve digits | [`time.out_of_range`](TimeError::OutOfRange) | TIM-001, DKS-004, RFC 6376 §3.5 | API-201 |
    ///
    /// Requirements: TIM-001.
    pub fn from_unix_seconds(seconds: u64) -> Result<VerificationTime, TimeError> {
        if seconds > LARGEST_SECONDS {
            return Err(TimeError::OutOfRange);
        }
        Ok(VerificationTime { seconds })
    }

    /// The seconds the time was made from.
    ///
    /// Requirements: TIM-001.
    pub fn unix_seconds(self) -> u64 {
        self.seconds
    }
}

/// The error of [`VerificationTime::from_unix_seconds`] (TIM-001).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `time.out_of_range` | [`OutOfRange`](Self::OutOfRange) | [`VerificationTime::from_unix_seconds`] | VAL-201 | TIM-001 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TimeError {
    /// `time.out_of_range`: a time over 999,999,999,999 seconds.
    OutOfRange,
}

impl TimeError {
    /// The error's code, such as `time.out_of_range` (STB-001).
    pub fn code(&self) -> &'static str {
        match self {
            TimeError::OutOfRange => "time.out_of_range",
        }
    }
}

impl core::fmt::Display for TimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TimeError::OutOfRange => {
                let text = "time.out_of_range: the time is over 999,999,999,999 seconds";
                f.write_str(text)
            }
        }
    }
}

impl std::error::Error for TimeError {}

#[cfg(test)]
mod tests {
    use super::{TimeError, VerificationTime};

    #[test]
    fn api_201_time_takes_at_most_twelve_digits() {
        let largest = VerificationTime::from_unix_seconds(999_999_999_999).unwrap();
        assert_eq!(largest.unix_seconds(), 999_999_999_999);

        let error = VerificationTime::from_unix_seconds(1_000_000_000_000).unwrap_err();
        assert_eq!(error, TimeError::OutOfRange);
        assert_eq!(error.code(), "time.out_of_range");
        assert!(error.to_string().starts_with("time.out_of_range"));
    }
}
