//! The `Signing` aggregate: its input, its signing requests, its options and
//! its results.

use crate::{Check, ExpectedInput, RequestId, Signature, SignerFailure, SigningRequest, TraceRead};

#[cfg(feature = "unstable-dkim2")]
mod dkim2_signing;
mod dkim_signing;
mod options;
mod results;

pub use dkim_signing::{CanonAlgorithm, DkimCanonicalization, DkimSigning};
#[cfg(feature = "unstable-dkim2")]
pub use dkim2_signing::{
    BodyRevision, Dkim2Flags, Dkim2Signer, Dkim2Signing, EarlierLines, FieldRevision, Revision,
};
pub use options::{SigningOptions, SigningOptionsError};
pub use results::{SignatureError, SigningOutcome, SigningResults};

/// One run of DKIM signing, DKIM2 signing or both on one message: the type
/// that a signing caller creates and feeds
/// ([ADR 0062](https://github.com/photom/hallmark/blob/main/docs/adr/0062-verification-and-signing-have-separate-entry-types.md)).
///
/// The signing caller creates a signing with [`Signing::new`], passes it the
/// header fields, the body and the end of the message, and answers the
/// signing requests that [`Signing::take_requests`] hands out. A signing takes
/// no envelope and asks no DNS question. When the input is complete and every
/// request has its answer, the signing is Finished, and
/// [`Signing::take_results`] gives the header fields to add.
/// [Lifecycle](https://github.com/photom/hallmark/blob/main/docs/reference/design/lifecycle.md#signing)
/// draws the states: Created, Headers, Body, Ended and Finished.
///
/// Every method that changes a signing takes it by `&mut self`, so a signing
/// takes one call at a time. No method keeps a reference into a byte slice it
/// takes after it returns (STR-007, INV-004).
///
/// # How a call is checked
///
/// A method first checks that the current state accepts it, and gives
/// [`signing.out_of_order`](SigningError::OutOfOrder) if it doesn't. It then
/// checks its rules in the order its validation table lists them, and gives
/// the code of the first one broken. When every rule holds, the signing takes
/// the input, and the requests it now has wait for [`Signing::take_requests`].
///
/// | Rule | Requirements |
/// |---|---|
/// | A method that returns an error changes nothing in the signing, and the signing caller can go on | CHK-004, INV-003 |
/// | An error of the signing caller never becomes a signing outcome, and an outcome never comes back as an error | INV-010 |
/// | A rule whose error code reads "Not an error" never stops the call. The signing takes the input and applies the rule named | CHK-004 |
#[derive(Debug)]
pub struct Signing {
    state: (),
}

impl Signing {
    /// Creates a signing in Created.
    ///
    /// `Signing::new` checks every rule of the options at once, in the order
    /// of the table below, and gives the code of the first one broken. Rules
    /// that depend on the message, such as an `exploded` flag on the first
    /// DKIM2-Signature (draft-09 §12.5) or a revision that misses a body
    /// change (SIGN-022), stop that kind of signing with a [`SignatureError`]
    /// in the results.
    ///
    /// # Parameters
    ///
    /// - `options`: the options, which the signing keeps unchanged (INV-015).
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-249 | `limits` | Each limit within its range, and `header_field_size` at most `header_section_size`, as [`Limits::validate`](crate::Limits::validate) checks them | [`signing_options.limits_invalid`](SigningOptionsError::LimitsInvalid), which carries the [`LimitsError`](crate::LimitsError) | PAR-004 | API-249 |
    /// | VAL-250 | `dkim`, `dkim2` | `dkim` holds at least one entry, or `dkim2` is `Some` | [`signing_options.no_signature`](SigningOptionsError::NoSignature) | CHK-006 | API-250 |
    /// | VAL-251 | `message_id` | 1 to 256 octets, when given | [`signing_options.message_id_invalid`](SigningOptionsError::MessageIdInvalid) | TRC-014, RFC 7493 §2.1 | API-251 |
    /// | VAL-220 | `DkimSigning.domain` | A domain name with a lookup form | [`signing_options.signing_domain_invalid`](SigningOptionsError::SigningDomainInvalid) | ADS-002, RFC 6376 §3.5 | API-220 |
    /// | VAL-221 | `DkimSigning.selector` | Matches `selector` of RFC 6376 §3.1, and the key name `<selector>._domainkey.<domain>` fits ADS-004 | [`signing_options.selector_invalid`](SigningOptionsError::SelectorInvalid) | DNS-008, RFC 6376 §3.1 | API-221 |
    /// | VAL-222 | `DkimSigning.statement` | The algorithm is `RsaSha256` or `Ed25519Sha256` | [`signing_options.algorithm_not_allowed`](SigningOptionsError::AlgorithmNotAllowed) | DKS-001 | API-222 |
    /// | VAL-223 | `DkimSigning.statement` | The public key decodes for its algorithm: an RSAPublicKey or a SubjectPublicKeyInfo that holds one, or 32 octets of Ed25519 | [`signing_options.public_key_invalid`](SigningOptionsError::PublicKeyInvalid) | SGN-003, RFC 8463 §4.2 | API-223 |
    /// | VAL-224 | `DkimSigning.statement` | An RSA modulus has at least 1,024 bits | [`signing_options.rsa_key_too_small`](SigningOptionsError::RsaKeyTooSmall) | SGN-003, RFC 8301 §3.2 | SIGN-004 |
    /// | VAL-225 | `DkimSigning.statement` | An RSA modulus has at most `rsa_key_bits` bits | [`signing_options.rsa_key_too_large`](SigningOptionsError::RsaKeyTooLarge) | SAF-002 | API-225 |
    /// | VAL-226 | `DkimSigning.headers` | Each name matches `field-name` of RFC 5322 §3.6.8 | [`signing_options.header_name_invalid`](SigningOptionsError::HeaderNameInvalid) | DKS-003 | API-226 |
    /// | VAL-227 | `DkimSigning.headers` | Holds From, in any case | [`signing_options.from_not_signed`](SigningOptionsError::FromNotSigned) | DKS-003, RFC 6376 §5.4 | API-227 |
    /// | VAL-228 | `DkimSigning.expiration` | At least 1, and `time` plus it at most 999,999,999,999 | [`signing_options.expiration_invalid`](SigningOptionsError::ExpirationInvalid) | DKS-004, RFC 6376 §3.5 | API-228 |
    /// | VAL-230 | `Dkim2Signing.domain` | A domain name with a lookup form | [`signing_options.signing_domain_invalid`](SigningOptionsError::SigningDomainInvalid) | ADS-002, draft-09 §10.8 | API-230 |
    /// | VAL-231 | `Dkim2Signing.groups` | At least one group, and at least one signer in each | [`signing_options.no_signer`](SigningOptionsError::NoSigner) | draft-09 §10.9 | API-231 |
    /// | VAL-232 | `Dkim2Signing.groups` | Each selector matches `selector` of RFC 6376 §3.1 and fits ADS-004 in its key name | [`signing_options.selector_invalid`](SigningOptionsError::SelectorInvalid) | dns-03 §3 | API-232 |
    /// | VAL-233 | `Dkim2Signing.groups` | No selector appears twice | [`signing_options.selector_repeated`](SigningOptionsError::SelectorRepeated) | draft-09 §10.9 | API-233 |
    /// | VAL-234 | `Dkim2Signing.groups` | Each public key decodes for its algorithm: RSA as in VAL-223, 32 octets of Ed25519, or 1,312 octets of ML-DSA-44 | [`signing_options.public_key_invalid`](SigningOptionsError::PublicKeyInvalid) | DK2-011, dns-03 §4 | API-234 |
    /// | VAL-235 | `Dkim2Signing.groups` | An RSA modulus has from 1,024 to `rsa_key_bits` bits | [`signing_options.rsa_key_too_small`](SigningOptionsError::RsaKeyTooSmall) or [`signing_options.rsa_key_too_large`](SigningOptionsError::RsaKeyTooLarge) | SGN-003, draft-09 §3.2 | API-235 |
    /// | VAL-236 | `Dkim2Signing.groups` | An RSA public exponent is 65537 | [`signing_options.rsa_exponent_not_65537`](SigningOptionsError::RsaExponentNot65537) | SGN-003, draft-09 §3.2 | SIGN-005 |
    /// | VAL-237 | `Dkim2Signing` | Either `next_domain` with no `mail_from` and no `rcpt_to`, or `mail_from` and at least one `rcpt_to` with no `next_domain` | [`signing_options.dkim2_envelope_invalid`](SigningOptionsError::Dkim2EnvelopeInvalid) | draft-09 §10.7 | API-237 |
    /// | VAL-238 | `Dkim2Signing.mail_from`, `rcpt_to` | `mail_from` is a Reverse-path and each `rcpt_to` a Forward-path, with angle brackets and without parameters | [`signing_options.path_invalid`](SigningOptionsError::PathInvalid) | ADS-020, ADS-021, draft-09 §10.5, §10.6 | API-238 |
    /// | VAL-239 | `Dkim2Signing.rcpt_to` | At most `rcpt_to` paths | [`signing_options.rcpt_to_limit`](SigningOptionsError::RcptToLimit) | SAF-004 | API-239 |
    /// | VAL-240 | `Dkim2Signing.next_domain` | A domain name with a lookup form | [`signing_options.signing_domain_invalid`](SigningOptionsError::SigningDomainInvalid) | ADS-002, draft-09 §10.7 | API-240 |
    /// | VAL-241 | `Dkim2Signing.nonce` | At most 64 characters of printable ASCII other than `;` | [`signing_options.nonce_invalid`](SigningOptionsError::NonceInvalid) | draft-09 §10.3 | API-241 |
    /// | VAL-242 | `Dkim2Signing.revision` | The `Unchanged` ranges have `first` at most `last`, start at line 1 or later, and go in ascending order without overlap | [`signing_options.revision_invalid`](SigningOptionsError::RevisionInvalid) | DK2-014, draft-09 §5.2 | API-242 |
    ///
    /// Requirements: CHK-006, PAR-004.
    pub fn new(options: SigningOptions) -> Result<Signing, SigningOptionsError> {
        unimplemented!()
    }

    /// Takes one header field as a milter receives it: a name and a value.
    ///
    /// Accepted in Created and Headers. The signing rebuilds the raw bytes
    /// from `name`, a colon, `value` and [`SigningOptions::field_form`], as
    /// [`Check::header_field`] does (HDR-005, FLD-060).
    ///
    /// # Parameters
    ///
    /// - `name`: one field name as a milter receives it: the octets before the
    ///   colon, without the colon.
    /// - `value`: that field's value in the form that
    ///   [`SigningOptions::field_form`] states, without the line ending that
    ///   ends the field.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-170 | The call | The signing is in Created or Headers | [`signing.out_of_order`](SigningError::OutOfOrder) | CHK-004 | API-170 |
    ///
    /// Requirements: STR-008, HDR-005.
    pub fn header_field(&mut self, name: &[u8], value: &[u8]) -> Result<(), SigningError> {
        unimplemented!()
    }

    /// Ends the header fields that [`Signing::header_field`] passed.
    ///
    /// Accepted in Created and Headers. The signing moves to Body.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-171 | The call | The signing is in Created or Headers | [`signing.out_of_order`](SigningError::OutOfOrder) | CHK-004 | API-171 |
    ///
    /// Requirements: CHK-006.
    pub fn end_of_header_fields(&mut self) -> Result<(), SigningError> {
        unimplemented!()
    }

    /// Takes the raw header section in one piece.
    ///
    /// Accepted in Created. The signing moves to Body.
    ///
    /// # Parameters
    ///
    /// - `raw`: the raw header section with CRLF line endings: every header
    ///   field, then the empty line that ends the section, which may be left
    ///   out.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-172 | The call | The signing is in Created | [`signing.out_of_order`](SigningError::OutOfOrder) | CHK-004 | API-172 |
    /// | VAL-173 | `raw` | No empty line before its last line. The empty line that ends the header section may be its last line, or may be left out | [`signing.body_in_header_section`](SigningError::BodyInHeaderSection) | HDR-001 | API-173 |
    ///
    /// Requirements: HDR-001.
    pub fn header_section(&mut self, raw: &[u8]) -> Result<(), SigningError> {
        unimplemented!()
    }

    /// Takes the whole message in one piece: the header section, the empty
    /// line and the body, then the end of the message.
    ///
    /// Accepted in Created. The signing splits `message` at the first empty
    /// line (HDR-001).
    ///
    /// # Parameters
    ///
    /// - `message`: the whole message with CRLF line endings.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-174 | The call | The signing is in Created | [`signing.out_of_order`](SigningError::OutOfOrder) | CHK-004 | API-174 |
    ///
    /// Requirements: GEN-003.
    pub fn whole_message(&mut self, message: &[u8]) -> Result<(), SigningError> {
        unimplemented!()
    }

    /// Takes the next body bytes.
    ///
    /// Accepted in Body. The body may come in no chunks at all, as for a
    /// message with an empty or missing body.
    ///
    /// # Parameters
    ///
    /// - `chunk`: the next body bytes, any number from 0, cut anywhere. The
    ///   body has CRLF line endings, and a CRLF may span two chunks (STR-011).
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-175 | The call | The signing is in Body | [`signing.out_of_order`](SigningError::OutOfOrder) | CHK-004 | API-175 |
    ///
    /// Requirements: STR-001, STR-007.
    pub fn body_chunk(&mut self, chunk: &[u8]) -> Result<(), SigningError> {
        unimplemented!()
    }

    /// Ends the message. Only this call tells the signing that the body is
    /// complete.
    ///
    /// Accepted in Body. The signing moves to Ended.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-176 | The call | The signing is in Body | [`signing.out_of_order`](SigningError::OutOfOrder) | CHK-004 | API-176 |
    ///
    /// Requirements: CHK-006.
    pub fn end_of_message(&mut self) -> Result<(), SigningError> {
        unimplemented!()
    }

    /// Gives the signing requests that appeared since the last call, in the
    /// order they appeared.
    ///
    /// Accepted in every state. Each request it gives moves from Waiting to
    /// Taken
    /// ([Lifecycle](https://github.com/photom/hallmark/blob/main/docs/reference/design/lifecycle.md#requests)).
    /// A call with no new request gives an empty list. The method takes no
    /// input, so it has no rule to check (SIGN-001).
    ///
    /// Requirements: SGN-002, CHK-006.
    pub fn take_requests(&mut self) -> Vec<SigningRequest> {
        unimplemented!()
    }

    /// Takes the signer's answer to a signing request.
    ///
    /// Accepted in every state. An `Err` stops that kind of signing with
    /// [`SignatureError::SignerFailed`] in the results (SGN-004, SIGN-008).
    /// The signing keeps the failure's description for that error and writes
    /// none of it into the trace.
    ///
    /// # Parameters
    ///
    /// - `id`: the ID of a signing request from [`Signing::take_requests`].
    /// - `answer`: `Ok` with the signature the signer made, or `Err` with the
    ///   signer's failure.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-140 | `id` | Names a signing request that [`Signing::take_requests`] gave | [`signing.unknown_request`](SigningError::UnknownRequest) | CHK-004 | API-140 |
    /// | VAL-141 | `id` | The request has no answer yet | [`signing.already_answered`](SigningError::AlreadyAnswered) | CHK-004 | API-141 |
    /// | VAL-142 | `answer` | An `Ok` signature has the size that its algorithm and the stated key give | Not an error: signing stops with [`signature.wrong_size`](SignatureError::WrongSize) in the results | SGN-004 | SIGN-007 |
    /// | VAL-143 | `answer` | An `Ok` signature verifies with the stated public key, when `verify` is on in its [`DkimSigning`] or `Dkim2Signing` | Not an error: signing stops with [`signature.not_verified`](SignatureError::NotVerified) in the results | SGN-006 | SIGN-016 |
    ///
    /// Requirements: SGN-004.
    pub fn answer(
        &mut self,
        id: RequestId,
        answer: Result<Signature, SignerFailure>,
    ) -> Result<(), SigningError> {
        unimplemented!()
    }

    /// Writes the next trace bytes into `buffer`.
    ///
    /// Accepted in every state. It gives how many bytes it wrote and whether
    /// the trace is complete. The trace stays readable after the results are
    /// taken.
    ///
    /// # Parameters
    ///
    /// - `buffer`: where the signing writes the next trace bytes, from the
    ///   start of the buffer.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-177 | `buffer` | Holds at least 1 byte | [`signing.buffer_empty`](SigningError::BufferEmpty) | STR-002 | API-177 |
    ///
    /// Requirements: TRC-004, STR-002.
    pub fn read_trace(&mut self, buffer: &mut [u8]) -> Result<TraceRead, SigningError> {
        unimplemented!()
    }

    /// Gives the input the signing still expects and the IDs of its open
    /// signing requests.
    ///
    /// Accepted in every state. The method takes no input, so it has no rule
    /// to check (API-178).
    ///
    /// Requirements: CHK-002.
    pub fn waiting(&self) -> SigningWaiting {
        unimplemented!()
    }

    /// Gives the results.
    ///
    /// Accepted in every state, and gives the results only in Finished.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-178 | The call | The signing is Finished | [`signing.not_finished`](SigningError::NotFinished), which carries [`SigningWaiting`] | CHK-002 | API-178 |
    /// | VAL-179 | The call | The results haven't been taken | [`signing.results_taken`](SigningError::ResultsTaken) | CHK-002 | API-179 |
    ///
    /// Requirements: CHK-002.
    pub fn take_results(&mut self) -> Result<SigningResults, SigningError> {
        unimplemented!()
    }

    /// Gives the options the signing was created with.
    ///
    /// Accepted in every state. The method takes no input, so it has no rule
    /// to check (SIGN-018).
    ///
    /// Requirements: INV-015.
    pub fn options(&self) -> &SigningOptions {
        unimplemented!()
    }
}

/// What a signing still waits for (CHK-002).
///
/// [`Signing::waiting`] gives it, and [`SigningError::NotFinished`] carries
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SigningWaiting {
    /// The message input the signing still expects, or `None` in Ended and
    /// Finished. It is never [`ExpectedInput::Envelope`].
    pub input: Option<ExpectedInput>,
    /// The signing requests without an answer, waiting or taken, in the order
    /// they appeared.
    pub requests: Vec<RequestId>,
}

/// The error of every method of [`Signing`] except [`Signing::new`]
/// (CHK-004).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `signing.out_of_order` | [`OutOfOrder`](Self::OutOfOrder) | [`Signing::header_field`], [`Signing::end_of_header_fields`], [`Signing::header_section`], [`Signing::whole_message`], [`Signing::body_chunk`], [`Signing::end_of_message`] | VAL-170 to VAL-172, VAL-174 to VAL-176 | CHK-004 |
/// | `signing.body_in_header_section` | [`BodyInHeaderSection`](Self::BodyInHeaderSection) | [`Signing::header_section`] | VAL-173 | HDR-001 |
/// | `signing.unknown_request` | [`UnknownRequest`](Self::UnknownRequest) | [`Signing::answer`] | VAL-140 | CHK-004 |
/// | `signing.already_answered` | [`AlreadyAnswered`](Self::AlreadyAnswered) | [`Signing::answer`] | VAL-141 | CHK-004 |
/// | `signing.buffer_empty` | [`BufferEmpty`](Self::BufferEmpty) | [`Signing::read_trace`] | VAL-177 | STR-002 |
/// | `signing.not_finished` | [`NotFinished`](Self::NotFinished) | [`Signing::take_results`] | VAL-178 | CHK-002 |
/// | `signing.results_taken` | [`ResultsTaken`](Self::ResultsTaken) | [`Signing::take_results`] | VAL-179 | CHK-002 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SigningError {
    /// `signing.out_of_order`: message input in a state that doesn't accept
    /// it.
    OutOfOrder,
    /// `signing.body_in_header_section`: an empty line inside the raw header
    /// section.
    BodyInHeaderSection,
    /// `signing.unknown_request`: no signing request with that ID was handed
    /// out.
    UnknownRequest,
    /// `signing.already_answered`: the request has its answer.
    AlreadyAnswered,
    /// `signing.buffer_empty`: a buffer of 0 bytes.
    BufferEmpty,
    /// `signing.not_finished`: the signing isn't Finished. Carries what it
    /// waits for.
    NotFinished(SigningWaiting),
    /// `signing.results_taken`: the results were taken before.
    ResultsTaken,
}

impl SigningError {
    /// The error's code, such as `signing.out_of_order` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for SigningError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for SigningError {}
