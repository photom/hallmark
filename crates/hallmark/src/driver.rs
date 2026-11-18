//! The driver and the traits a caller implements for it.

use crate::*;

/// Resolves the DNS questions of a check for [`drive`] (DNS-001, DNS-003).
///
/// The trait requires `Send` and `Sync`, and its method returns a future that
/// is `Send`, so the driver can hold a shared reference across an await and
/// its own future is `Send`
/// ([ADR 0048](https://github.com/photom/hallmark/blob/main/docs/adr/0048-the-resolver-the-suffix-source-and-the-signer-return-futures-that-are-send.md)).
/// [`SnapshotResolver`] implements it.
pub trait Resolver: Send + Sync {
    /// Resolves one question. The future gives its answer.
    ///
    /// The method takes the question by value, so the future owns what it
    /// needs. The resolver applies its own time limits, such as the one RFC
    /// 7208 §4.6.4 asks for, and reports a timeout as
    /// [`Answer::TemporaryFailure`] (DNS-009).
    fn resolve(&self, question: Question) -> impl Future<Output = Answer> + Send;
}

/// Answers the public suffix questions of a check for [`drive`] (DMC-010).
///
/// A check asks them only in RFC 7489 mode. The trait requires `Send` and
/// `Sync`, and its method returns a future that is `Send` (ADR 0048).
/// [`SnapshotResolver`] implements it, and [`NoSuffixSource`] stands for none.
pub trait SuffixSource: Send + Sync {
    /// Answers one public suffix question. The future gives its answer.
    ///
    /// The method takes the question by value, so the future owns what it
    /// needs.
    fn suffix(&self, question: SuffixQuestion) -> impl Future<Output = SuffixAnswer> + Send;
}

/// Signs the signing requests of a check for [`drive`] (SGN-001).
///
/// The caller supplies the signer, which can keep its key in memory, in a
/// hardware security module or in a remote key service, and can answer at
/// once or later. The trait requires `Send` and `Sync`, and its method returns
/// a future that is `Send` (ADR 0048). [`MemorySigner`] implements it, and
/// [`NoSigner`] stands for none.
pub trait Signer: Send + Sync {
    /// Signs one request. The future gives the signature, or the signer's
    /// failure.
    ///
    /// The method takes the request by value, so the future owns what it
    /// needs.
    fn sign(
        &self,
        request: SigningRequest,
    ) -> impl Future<Output = Result<Signature, SignerFailure>> + Send;
}

/// A [`SuffixSource`] that has no value, for a caller that passes no suffix
/// source to [`drive`] (CHK-005).
///
/// The caller names the type in `None::<&NoSuffixSource>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoSuffixSource {}

impl SuffixSource for NoSuffixSource {
    async fn suffix(&self, question: SuffixQuestion) -> SuffixAnswer {
        match *self {}
    }
}

/// A [`Signer`] that has no value, for a caller that passes no signer to
/// [`drive`] (CHK-005).
///
/// The caller names the type in `None::<&NoSigner>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoSigner {}

impl Signer for NoSigner {
    async fn sign(&self, request: SigningRequest) -> Result<Signature, SignerFailure> {
        match *self {}
    }
}

/// Runs a check with a resolver, and when it needs them, a suffix source and
/// a signer, until the check has no open request (CHK-005, DNS-003).
///
/// The check is then Finished if its input was complete. `drive` depends on no
/// async runtime, and its future is `Send`.
/// [Lifecycle](https://github.com/photom/hallmark/blob/main/docs/reference/design/lifecycle.md#driver)
/// gives the driver's states, and
/// [Concurrency](https://github.com/photom/hallmark/blob/main/docs/reference/design/concurrency.md#the-driver)
/// its rules.
///
/// | Rule | Requirements |
/// |---|---|
/// | `drive` hands each answer to the check as its future completes | CHK-003 |
/// | A signer failure is an answer, and `drive` goes on | SGN-004 |
/// | `drive` reads no clock. The resolver applies its own time limits | DNS-009 |
///
/// # Parameters
///
/// - `check`: the check to run. Its message input may be complete or still to
///   come.
/// - `resolver`: resolves each DNS question.
/// - `suffix_source`: answers each public suffix question. Needed for DMARC in
///   RFC 7489 mode.
/// - `signer`: signs each signing request. Needed for DKIM and DKIM2 signing.
/// - `in_flight`: how many futures the driver keeps pending at once, the
///   `driver_in_flight` of
///   [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check).
///
/// # Validation
///
/// `drive` checks VAL-801 to VAL-803 before it takes any request.
///
/// | ID | Input | Rule | Error code | Requirement | Scenario |
/// |---|---|---|---|---|---|
/// | VAL-801 | `in_flight` | From 1 to 1,024, the range of `driver_in_flight` | [`driver.in_flight_out_of_range`](DriveError::InFlightOutOfRange) | CHK-005 | API-801 |
/// | VAL-802 | `suffix_source` | `Some` when the check runs DMARC in RFC 7489 mode | [`driver.suffix_source_missing`](DriveError::SuffixSourceMissing) | DMC-010, CHK-005 | API-802 |
/// | VAL-803 | `signer` | `Some` when the check runs DKIM or DKIM2 signing | [`driver.signer_missing`](DriveError::SignerMissing) | SGN-001, CHK-005 | API-803 |
/// | VAL-804 | Each answer a future gives | The check takes it: the rules of VAL-120 to VAL-143 of [`Check`] | [`driver.answer_refused`](DriveError::AnswerRefused), which carries the [`CheckError`] | CHK-004 | API-804 |
///
/// Requirements: CHK-005, DNS-003.
pub async fn drive<R: Resolver, S: SuffixSource, G: Signer>(
    check: &mut Check,
    resolver: &R,
    suffix_source: Option<&S>,
    signer: Option<&G>,
    in_flight: u32,
) -> Result<(), DriveError> {
    unimplemented!()
}

/// The error of [`drive`] (CHK-005).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `driver.in_flight_out_of_range` | [`InFlightOutOfRange`](Self::InFlightOutOfRange) | [`drive`] | VAL-801 | CHK-005 |
/// | `driver.suffix_source_missing` | [`SuffixSourceMissing`](Self::SuffixSourceMissing) | [`drive`] | VAL-802 | DMC-010 |
/// | `driver.signer_missing` | [`SignerMissing`](Self::SignerMissing) | [`drive`] | VAL-803 | SGN-001 |
/// | `driver.answer_refused` | [`AnswerRefused`](Self::AnswerRefused) | [`drive`] | VAL-804 | CHK-004 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DriveError {
    /// `driver.in_flight_out_of_range`: a count of pending futures outside 1
    /// to 1,024.
    InFlightOutOfRange,
    /// `driver.suffix_source_missing`: no suffix source for DMARC in RFC 7489
    /// mode.
    SuffixSourceMissing,
    /// `driver.signer_missing`: no signer for a signing kind.
    SignerMissing,
    /// `driver.answer_refused`: the check refused an answer from a future.
    /// [`std::error::Error::source`] gives the [`CheckError`].
    AnswerRefused(CheckError),
}

impl DriveError {
    /// The error's code, such as `driver.signer_missing` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for DriveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for DriveError {}
