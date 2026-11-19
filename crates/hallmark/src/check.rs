//! The `Check` aggregate: its input, its requests, its options and its
//! results.

use crate::*;
use std::net::IpAddr;

#[cfg(feature = "unstable-dkim2")]
mod dkim2_signing;
mod options;
mod results;
mod signing_options;

#[cfg(feature = "unstable-dkim2")]
pub use dkim2_signing::{
    BodyRevision, Dkim2Flags, Dkim2Signer, Dkim2Signing, EarlierLines, FieldRevision, Revision,
};
pub use options::{DkimSource, DmarcMode, Kind, Options, OptionsError, SpfSource};
pub use results::{HeaderFieldOut, Results, ResultsError, Signing, SigningError};
pub use signing_options::{CanonAlgorithm, DkimCanonicalization, DkimSigning};

/// One run of any set of checks on one message: the one type that a caller
/// creates and feeds
/// ([ADR 0047](https://github.com/photom/hallmark/blob/main/docs/adr/0047-one-check-type-runs-any-set-of-checks-and-shares-their-dns-answers.md)).
///
/// The caller creates a check with [`Check::new`], passes it the envelope,
/// the header fields, the body and the end of the message, and answers the
/// requests that [`Check::take_requests`] hands out. When the input is
/// complete and every request has its answer, the check is Finished, and
/// [`Check::take_results`] gives the results.
/// [Lifecycle](https://github.com/photom/hallmark/blob/main/docs/reference/design/lifecycle.md#check)
/// draws the states: Created, Envelope, Headers, Body, Ended and Finished.
///
/// Every method that changes a check takes it by `&mut self`, so a check takes
/// one call at a time
/// ([Concurrency](https://github.com/photom/hallmark/blob/main/docs/reference/design/concurrency.md#inside-one-check)).
/// No method keeps a reference into a byte slice it takes after it returns
/// (STR-007, INV-004).
///
/// # How a call is checked
///
/// A method first checks that the current state accepts it, and gives
/// [`check.out_of_order`](CheckError::OutOfOrder) if it doesn't. It then
/// checks its rules in the order its validation table lists them, and gives
/// the code of the first one broken. When every rule holds, the check takes
/// the input, and the requests it now has wait for [`Check::take_requests`].
///
/// | Rule | Requirements |
/// |---|---|
/// | A method that returns an error changes nothing in the check, and the caller can go on | CHK-004, INV-003 |
/// | A caller error never becomes a protocol result, and a protocol result never comes back as an error | INV-010 |
/// | A rule whose error code reads "Not an error" never stops the call. The check takes the input and applies the rule named | CHK-004 |
/// | Syntax inside the message, the envelope and DNS records is no caller error. The parsers read it, and the check applies the rule of the requirement that covers it | PAR-001, ADS-020 to ADS-023 |
#[derive(Debug)]
pub struct Check {
    state: (),
}

impl Check {
    /// Creates a check in Created.
    ///
    /// `Check::new` checks every rule of the options at once, in the order of
    /// the table below, and gives the code of the first one broken. Rules for
    /// a DMARC source apply only when the kinds hold DMARC. Signing rules that
    /// depend on the message, such as an `exploded` flag on the first
    /// DKIM2-Signature (draft-09 §12.5) or a revision that misses a body
    /// change (SIGN-022), stop signing with a [`SigningError`] in the results.
    ///
    /// # Parameters
    ///
    /// - `options`: the options, which the check keeps unchanged (INV-015).
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-210 | `kinds` | Holds at least one kind | [`options.no_kind`](OptionsError::NoKind) | GEN-001 | API-210 |
    /// | VAL-211 | `dmarc_spf` | `SameCheck` only when the kinds hold SPF MAIL FROM | [`options.source_kind_missing`](OptionsError::SourceKindMissing) | DMC-005 | API-211 |
    /// | VAL-212 | `dmarc_dkim` | `SameCheck` only when the kinds hold DKIM | [`options.source_kind_missing`](OptionsError::SourceKindMissing) | DMC-005 | API-212 |
    /// | VAL-213 | `dmarc_spf` | An `Earlier` result is for the MAIL FROM identity | [`options.earlier_result_invalid`](OptionsError::EarlierResultInvalid) | DMC-005 | API-213 |
    /// | VAL-214 | `message_id` | 1 to 256 octets, when given | [`options.message_id_invalid`](OptionsError::MessageIdInvalid) | TRC-014, RFC 7493 §2.1 | API-214 |
    /// | VAL-215 | `receiving_host` | A domain name with a lookup form | [`options.receiving_host_invalid`](OptionsError::ReceivingHostInvalid) | REC-014, ADS-002 | API-215 |
    /// | VAL-216 | `kinds`, `dkim_signing` | The kinds hold `DkimSign` exactly when the list isn't empty | [`options.signing_mismatch`](OptionsError::SigningMismatch) | GEN-001, DKS-002 | API-216 |
    /// | VAL-217 | `kinds`, `dkim2_signing` | The kinds hold `Dkim2Sign` exactly when `dkim2_signing` is `Some` | [`options.signing_mismatch`](OptionsError::SigningMismatch) | GEN-001, DK2-003 | API-217 |
    /// | VAL-220 | `DkimSigning.domain` | A domain name with a lookup form | [`options.signing_domain_invalid`](OptionsError::SigningDomainInvalid) | ADS-002, RFC 6376 §3.5 | API-220 |
    /// | VAL-221 | `DkimSigning.selector` | Matches `selector` of RFC 6376 §3.1, and the key name `<selector>._domainkey.<domain>` fits ADS-004 | [`options.selector_invalid`](OptionsError::SelectorInvalid) | DNS-008, RFC 6376 §3.1 | API-221 |
    /// | VAL-222 | `DkimSigning.statement` | The algorithm is `RsaSha256` or `Ed25519Sha256` | [`options.algorithm_not_allowed`](OptionsError::AlgorithmNotAllowed) | DKS-001 | API-222 |
    /// | VAL-223 | `DkimSigning.statement` | The public key decodes for its algorithm: an RSAPublicKey or a SubjectPublicKeyInfo that holds one, or 32 octets of Ed25519 | [`options.public_key_invalid`](OptionsError::PublicKeyInvalid) | SGN-003, RFC 8463 §4.2 | API-223 |
    /// | VAL-224 | `DkimSigning.statement` | An RSA modulus has at least 1,024 bits | [`options.rsa_key_too_small`](OptionsError::RsaKeyTooSmall) | SGN-003, RFC 8301 §3.2 | SIGN-004 |
    /// | VAL-225 | `DkimSigning.statement` | An RSA modulus has at most `rsa_key_bits` bits | [`options.rsa_key_too_large`](OptionsError::RsaKeyTooLarge) | SAF-002 | API-225 |
    /// | VAL-226 | `DkimSigning.headers` | Each name matches `field-name` of RFC 5322 §3.6.8 | [`options.header_name_invalid`](OptionsError::HeaderNameInvalid) | DKS-003 | API-226 |
    /// | VAL-227 | `DkimSigning.headers` | Holds From, in any case | [`options.from_not_signed`](OptionsError::FromNotSigned) | DKS-003, RFC 6376 §5.4 | API-227 |
    /// | VAL-228 | `DkimSigning.expiration` | At least 1, and `time` plus it at most 999,999,999,999 | [`options.expiration_invalid`](OptionsError::ExpirationInvalid) | DKS-004, RFC 6376 §3.5 | API-228 |
    /// | VAL-230 | `Dkim2Signing.domain` | A domain name with a lookup form | [`options.signing_domain_invalid`](OptionsError::SigningDomainInvalid) | ADS-002, draft-09 §10.8 | API-230 |
    /// | VAL-231 | `Dkim2Signing.groups` | At least one group, and at least one signer in each | [`options.no_signer`](OptionsError::NoSigner) | draft-09 §10.9 | API-231 |
    /// | VAL-232 | `Dkim2Signing.groups` | Each selector matches `selector` of RFC 6376 §3.1 and fits ADS-004 in its key name | [`options.selector_invalid`](OptionsError::SelectorInvalid) | dns-03 §3 | API-232 |
    /// | VAL-233 | `Dkim2Signing.groups` | No selector appears twice | [`options.selector_repeated`](OptionsError::SelectorRepeated) | draft-09 §10.9 | API-233 |
    /// | VAL-234 | `Dkim2Signing.groups` | Each public key decodes for its algorithm: RSA as in VAL-223, 32 octets of Ed25519, or 1,312 octets of ML-DSA-44 | [`options.public_key_invalid`](OptionsError::PublicKeyInvalid) | DK2-011, dns-03 §4 | API-234 |
    /// | VAL-235 | `Dkim2Signing.groups` | An RSA modulus has from 1,024 to `rsa_key_bits` bits | [`options.rsa_key_too_small`](OptionsError::RsaKeyTooSmall) or [`options.rsa_key_too_large`](OptionsError::RsaKeyTooLarge) | SGN-003, draft-09 §3.2 | API-235 |
    /// | VAL-236 | `Dkim2Signing.groups` | An RSA public exponent is 65537 | [`options.rsa_exponent_not_65537`](OptionsError::RsaExponentNot65537) | SGN-003, draft-09 §3.2 | SIGN-005 |
    /// | VAL-237 | `Dkim2Signing` | Either `next_domain` with no `mail_from` and no `rcpt_to`, or `mail_from` and at least one `rcpt_to` with no `next_domain` | [`options.dkim2_envelope_invalid`](OptionsError::Dkim2EnvelopeInvalid) | draft-09 §10.7 | API-237 |
    /// | VAL-238 | `Dkim2Signing.mail_from`, `rcpt_to` | `mail_from` is a Reverse-path and each `rcpt_to` a Forward-path, with angle brackets and without parameters | [`options.path_invalid`](OptionsError::PathInvalid) | ADS-020, ADS-021, draft-09 §10.5, §10.6 | API-238 |
    /// | VAL-239 | `Dkim2Signing.rcpt_to` | At most `rcpt_to` paths | [`options.rcpt_to_limit`](OptionsError::RcptToLimit) | SAF-004 | API-239 |
    /// | VAL-240 | `Dkim2Signing.next_domain` | A domain name with a lookup form | [`options.signing_domain_invalid`](OptionsError::SigningDomainInvalid) | ADS-002, draft-09 §10.7 | API-240 |
    /// | VAL-241 | `Dkim2Signing.nonce` | At most 64 characters of printable ASCII other than `;` | [`options.nonce_invalid`](OptionsError::NonceInvalid) | draft-09 §10.3 | API-241 |
    /// | VAL-242 | `Dkim2Signing.revision` | The `Unchanged` ranges have `first` at most `last`, start at line 1 or later, and go in ascending order without overlap | [`options.revision_invalid`](OptionsError::RevisionInvalid) | DK2-014, draft-09 §5.2 | API-242 |
    ///
    /// Requirements: GEN-001, PAR-004.
    pub fn new(options: Options) -> Result<Check, OptionsError> {
        unimplemented!()
    }

    /// Takes the SMTP envelope.
    ///
    /// Accepted in Created. A check whose kinds need no part of the envelope,
    /// such as DKIM alone, takes `Envelope::default()`. A check whose kinds
    /// are all SPF is in Ended once it takes the envelope, so it refuses
    /// message input with [`check.out_of_order`](CheckError::OutOfOrder).
    ///
    /// # Parameters
    ///
    /// - `envelope`: the SMTP envelope.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-101 | The call | The check is in Created | [`check.out_of_order`](CheckError::OutOfOrder) | CHK-004 | API-101 |
    /// | VAL-102 | `client_ip` | `Some` when the kinds hold `SpfHelo` or `SpfMailFrom` | [`check.envelope_incomplete`](CheckError::EnvelopeIncomplete) | GEN-002, RFC 7208 §4.1 | API-102 |
    /// | VAL-103 | `helo` | `Some` when the kinds hold `SpfHelo`, or hold `SpfMailFrom` and `mail_from` is `<>` | [`check.envelope_incomplete`](CheckError::EnvelopeIncomplete) | ADS-023, RFC 7208 §2.4 | API-103 |
    /// | VAL-104 | `mail_from` | `Some` when the kinds hold `SpfMailFrom` or `Dkim2` | [`check.envelope_incomplete`](CheckError::EnvelopeIncomplete) | GEN-002, draft-09 §14.4 | API-104 |
    /// | VAL-105 | `rcpt_to` | At least one path when the kinds hold `Dkim2` | [`check.envelope_incomplete`](CheckError::EnvelopeIncomplete) | GEN-002, draft-09 §14.4 | API-105 |
    /// | VAL-106 | `rcpt_to` | At most `rcpt_to` paths ([Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-of-a-check)) | [`check.rcpt_to_limit`](CheckError::RcptToLimit) | SAF-004 | API-106 |
    ///
    /// The syntax of `helo`, `mail_from` and each `rcpt_to` path isn't checked
    /// here. A malformed one gets the rule of ADS-020 to ADS-023, and the
    /// trace records the parse error.
    ///
    /// Requirements: CHK-001, GEN-002.
    pub fn envelope(&mut self, envelope: Envelope) -> Result<(), CheckError> {
        unimplemented!()
    }

    /// Takes one header field as a milter receives it: a name and a value.
    ///
    /// Accepted in Envelope and Headers. The check rebuilds the raw bytes from
    /// `name`, a colon, `value` and [`Options::field_form`], and reads them
    /// with the parser of a raw header section (HDR-005, FLD-060). A `name`
    /// that isn't a field name gives a malformed line, as it would in a raw
    /// header section (HDR-004). A bare CR or LF in `value`, outside the folds
    /// that the field form states, stays an octet of the field (HDR-012). A
    /// field over `header_field_size` reaches that limit.
    ///
    /// # Parameters
    ///
    /// - `name`: one field name as a milter receives it: the octets before the
    ///   colon, without the colon.
    /// - `value`: that field's value in the form that
    ///   [`Options::field_form`] states, without the line ending that ends
    ///   the field.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-107 | The call | The check is in Envelope or Headers | [`check.out_of_order`](CheckError::OutOfOrder) | CHK-004 | API-107 |
    ///
    /// Requirements: STR-008, HDR-005.
    pub fn header_field(&mut self, name: &[u8], value: &[u8]) -> Result<(), CheckError> {
        unimplemented!()
    }

    /// Ends the header fields that [`Check::header_field`] passed.
    ///
    /// Accepted in Envelope and Headers. The check moves to Body.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-108 | The call | The check is in Envelope or Headers | [`check.out_of_order`](CheckError::OutOfOrder) | CHK-004 | API-108 |
    ///
    /// Requirements: CHK-001.
    pub fn end_of_header_fields(&mut self) -> Result<(), CheckError> {
        unimplemented!()
    }

    /// Takes the raw header section in one piece.
    ///
    /// Accepted in Envelope. The check moves to Body.
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
    /// | VAL-109 | The call | The check is in Envelope | [`check.out_of_order`](CheckError::OutOfOrder) | CHK-004 | API-109 |
    /// | VAL-110 | `raw` | No empty line before its last line. The empty line that ends the header section may be its last line, or may be left out | [`check.body_in_header_section`](CheckError::BodyInHeaderSection) | HDR-001 | API-110 |
    ///
    /// Requirements: HDR-001.
    pub fn header_section(&mut self, raw: &[u8]) -> Result<(), CheckError> {
        unimplemented!()
    }

    /// Takes the whole message in one piece: the header section, the empty
    /// line and the body, then the end of the message.
    ///
    /// Accepted in Envelope. The check splits `message` at the first empty
    /// line (HDR-001). Input with no empty line is a header section with no
    /// body, which still has a body hash (CHK-001).
    ///
    /// # Parameters
    ///
    /// - `message`: the whole message with CRLF line endings.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-111 | The call | The check is in Envelope | [`check.out_of_order`](CheckError::OutOfOrder) | CHK-004 | API-111 |
    ///
    /// Requirements: GEN-003.
    pub fn whole_message(&mut self, message: &[u8]) -> Result<(), CheckError> {
        unimplemented!()
    }

    /// Takes the next body bytes.
    ///
    /// Accepted in Body. The body may come in no chunks at all, as for a
    /// message with an empty or missing body (CHK-001).
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
    /// | VAL-112 | The call | The check is in Body | [`check.out_of_order`](CheckError::OutOfOrder) | CHK-004 | RUN-005 |
    ///
    /// Requirements: STR-001, STR-007.
    pub fn body_chunk(&mut self, chunk: &[u8]) -> Result<(), CheckError> {
        unimplemented!()
    }

    /// Ends the message. Only this call tells the check that the body is
    /// complete.
    ///
    /// Accepted in Body. The check moves to Ended.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-113 | The call | The check is in Body | [`check.out_of_order`](CheckError::OutOfOrder) | CHK-004 | API-113 |
    ///
    /// Requirements: CHK-001.
    pub fn end_of_message(&mut self) -> Result<(), CheckError> {
        unimplemented!()
    }

    /// Gives the requests that appeared since the last call, in the order they
    /// appeared.
    ///
    /// Accepted in every state. Each request it gives moves from Waiting to
    /// Taken
    /// ([Lifecycle](https://github.com/photom/hallmark/blob/main/docs/reference/design/lifecycle.md#requests)).
    /// A call with no new request gives an empty list. The method takes no
    /// input, so it has no rule to check (RUN-001).
    ///
    /// Requirements: CHK-001.
    pub fn take_requests(&mut self) -> Vec<Request> {
        unimplemented!()
    }

    /// Takes the answer to a DNS question.
    ///
    /// Accepted in every state.
    ///
    /// # Parameters
    ///
    /// - `id`: the ID of a DNS question from [`Check::take_requests`].
    /// - `answer`: its answer.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-120 | `id` | Names a DNS question that [`Check::take_requests`] gave | [`check.unknown_question`](CheckError::UnknownQuestion) | CHK-004 | RUN-006 |
    /// | VAL-121 | `id` | The question has no answer yet | [`check.already_answered`](CheckError::AlreadyAnswered) | CHK-004 | RUN-007 |
    /// | VAL-122 | `answer` | Every record has the question's type | [`check.answer_type_mismatch`](CheckError::AnswerTypeMismatch) | DNS-006 | API-122 |
    /// | VAL-123 | `answer` | [`Answer::Records`] holds at least one record | [`check.answer_malformed`](CheckError::AnswerMalformed) | DNS-002 | API-123 |
    /// | VAL-124 | `answer` | Each TXT string holds at most 255 octets | [`check.answer_malformed`](CheckError::AnswerMalformed) | DNS-006, RFC 1035 §3.3 | API-124 |
    /// | VAL-125 | `answer` | The record data and the description hold at most `answer_size` octets in total | Not an error: the check takes the answer as a temporary failure that names the limit | SAF-004, GEN-005 | API-125 |
    ///
    /// Requirements: DNS-002, DNS-007.
    pub fn answer(&mut self, id: QuestionId, answer: Answer) -> Result<(), CheckError> {
        unimplemented!()
    }

    /// Takes the answer to a public suffix question.
    ///
    /// Accepted in every state.
    ///
    /// # Parameters
    ///
    /// - `id`: the ID of a public suffix question from
    ///   [`Check::take_requests`].
    /// - `answer`: its answer.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-130 | `id` | Names a public suffix question that [`Check::take_requests`] gave | [`check.unknown_question`](CheckError::UnknownQuestion) | CHK-004 | API-130 |
    /// | VAL-131 | `id` | The question has no answer yet | [`check.already_answered`](CheckError::AlreadyAnswered) | CHK-004 | API-131 |
    /// | VAL-132 | `answer` | `labels` is from 1 to the number of labels in the question's domain | [`check.suffix_out_of_range`](CheckError::SuffixOutOfRange) | DMC-011 | DMR-067 |
    /// | VAL-133 | `answer` | `list`, when given, holds 1 to 256 octets | [`check.list_name_invalid`](CheckError::ListNameInvalid) | DMC-010, RFC 7493 §2.1 | API-133 |
    ///
    /// Requirements: DMC-010.
    pub fn answer_suffix(
        &mut self,
        id: QuestionId,
        answer: SuffixAnswer,
    ) -> Result<(), CheckError> {
        unimplemented!()
    }

    /// Takes the signer's answer to a signing request.
    ///
    /// Accepted in every state. An `Err` stops that signing with
    /// [`SigningError::SignerFailed`] in the results (SGN-004, SIGN-008). The
    /// check keeps the failure's description for that error and writes none
    /// of it into the trace.
    ///
    /// # Parameters
    ///
    /// - `id`: the ID of a signing request from [`Check::take_requests`].
    /// - `answer`: `Ok` with the signature the signer made, or `Err` with the
    ///   signer's failure.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-140 | `id` | Names a signing request that [`Check::take_requests`] gave | [`check.unknown_request`](CheckError::UnknownRequest) | CHK-004 | API-140 |
    /// | VAL-141 | `id` | The request has no answer yet | [`check.already_answered`](CheckError::AlreadyAnswered) | CHK-004 | API-141 |
    /// | VAL-142 | `answer` | An `Ok` signature has the size that its algorithm and the stated key give | Not an error: signing stops with [`signing.wrong_size`](SigningError::WrongSize) in the results | SGN-004 | SIGN-007 |
    /// | VAL-143 | `answer` | An `Ok` signature verifies with the stated public key, when `verify_signatures` is on | Not an error: signing stops with [`signing.not_verified`](SigningError::NotVerified) in the results | SGN-006 | SIGN-016 |
    ///
    /// Requirements: SGN-004.
    pub fn answer_signing(
        &mut self,
        id: RequestId,
        answer: Result<Signature, SignerFailure>,
    ) -> Result<(), CheckError> {
        unimplemented!()
    }

    /// Writes the next trace bytes into `buffer`.
    ///
    /// Accepted in every state. It gives how many bytes it wrote and whether
    /// the trace is complete. The trace stays readable after the results are
    /// taken.
    /// [Trace format](https://github.com/photom/hallmark/blob/main/docs/reference/trace-format.md)
    /// defines the bytes.
    ///
    /// # Parameters
    ///
    /// - `buffer`: where the check writes the next trace bytes, from the start
    ///   of the buffer.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-150 | `buffer` | Holds at least 1 byte | [`check.buffer_empty`](CheckError::BufferEmpty) | STR-002 | API-150 |
    ///
    /// Requirements: TRC-004, STR-002.
    pub fn read_trace(&mut self, buffer: &mut [u8]) -> Result<TraceRead, CheckError> {
        unimplemented!()
    }

    /// Gives the input the check still expects and the IDs of its open
    /// requests.
    ///
    /// Accepted in every state. The method takes no input, so it has no rule
    /// to check (RUN-003).
    ///
    /// Requirements: CHK-002.
    pub fn waiting(&self) -> Waiting {
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
    /// | VAL-160 | The call | The check is Finished | [`check.not_finished`](CheckError::NotFinished), which carries [`Waiting`] | CHK-002 | RUN-003 |
    /// | VAL-161 | The call | The results haven't been taken | [`check.results_taken`](CheckError::ResultsTaken) | CHK-002 | API-161 |
    ///
    /// Requirements: CHK-002.
    pub fn take_results(&mut self) -> Result<Results, CheckError> {
        unimplemented!()
    }

    /// Gives the options the check was created with.
    ///
    /// Accepted in every state. The method takes no input, so it has no rule
    /// to check (SIGN-018).
    ///
    /// Requirements: INV-015.
    pub fn options(&self) -> &Options {
        unimplemented!()
    }
}

/// The SMTP envelope of a message (GEN-002).
///
/// The caller starts from `Envelope::default()`, in which every field is
/// empty, and sets the fields its kinds need. [`Check::envelope`] checks it
/// (VAL-101 to VAL-106).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Envelope {
    /// The SMTP client's address (GEN-002, RFC 7208 §4.1).
    pub client_ip: Option<IpAddr>,
    /// The HELO or EHLO name as the client sent it (ADS-022).
    pub helo: Option<Vec<u8>>,
    /// The reverse-path with its angle brackets and without Mail-parameters,
    /// `<>` included (ADS-020).
    pub mail_from: Option<Vec<u8>>,
    /// Each forward-path with its angle brackets and without Rcpt-parameters
    /// (ADS-021).
    pub rcpt_to: Vec<Vec<u8>>,
}

/// One thing a check needs from the caller.
///
/// [`Check::take_requests`] gives them.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Request {
    /// A DNS question, for [`Check::answer`] (DNS-001).
    Question(Question),
    /// A public suffix question, for [`Check::answer_suffix`] (DMC-010).
    Suffix(SuffixQuestion),
    /// A signature to make, for [`Check::answer_signing`] (SGN-002).
    Signing(SigningRequest),
}

/// What one call of [`Check::read_trace`] wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TraceRead {
    /// The bytes written into `buffer`, from its start. 0 when the check has
    /// no new byte yet (STR-002).
    pub written: usize,
    /// The check is Finished and every trace byte has been read. At the off
    /// level, true as soon as the check is Finished (TRC-004).
    pub complete: bool,
}

/// What a check still waits for (CHK-002).
///
/// [`Check::waiting`] gives it, and [`CheckError::NotFinished`] carries it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Waiting {
    /// The message input the check still expects, or `None` in Ended and
    /// Finished.
    pub input: Option<ExpectedInput>,
    /// The DNS and public suffix questions without an answer, waiting or
    /// taken, in the order they appeared.
    pub questions: Vec<QuestionId>,
    /// The signing requests without an answer, waiting or taken, in the order
    /// they appeared.
    pub signing: Vec<RequestId>,
}

/// The message input a check expects next, by state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpectedInput {
    /// Created: [`Check::envelope`].
    Envelope,
    /// Envelope: [`Check::header_field`], [`Check::end_of_header_fields`],
    /// [`Check::header_section`] or [`Check::whole_message`].
    Header,
    /// Headers: [`Check::header_field`] or [`Check::end_of_header_fields`].
    MoreHeaderFields,
    /// Body: [`Check::body_chunk`] or [`Check::end_of_message`].
    Body,
}

/// The error of every method of [`Check`] except [`Check::new`] (CHK-004).
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `check.out_of_order` | [`OutOfOrder`](Self::OutOfOrder) | [`Check::envelope`], [`Check::header_field`], [`Check::end_of_header_fields`], [`Check::header_section`], [`Check::whole_message`], [`Check::body_chunk`], [`Check::end_of_message`] | VAL-101, VAL-107 to VAL-109, VAL-111 to VAL-113 | CHK-004 |
/// | `check.envelope_incomplete` | [`EnvelopeIncomplete`](Self::EnvelopeIncomplete) | [`Check::envelope`] | VAL-102 to VAL-105 | GEN-002 |
/// | `check.rcpt_to_limit` | [`RcptToLimit`](Self::RcptToLimit) | [`Check::envelope`] | VAL-106 | SAF-004 |
/// | `check.body_in_header_section` | [`BodyInHeaderSection`](Self::BodyInHeaderSection) | [`Check::header_section`] | VAL-110 | HDR-001 |
/// | `check.unknown_question` | [`UnknownQuestion`](Self::UnknownQuestion) | [`Check::answer`], [`Check::answer_suffix`] | VAL-120, VAL-130 | CHK-004 |
/// | `check.unknown_request` | [`UnknownRequest`](Self::UnknownRequest) | [`Check::answer_signing`] | VAL-140 | CHK-004 |
/// | `check.already_answered` | [`AlreadyAnswered`](Self::AlreadyAnswered) | [`Check::answer`], [`Check::answer_suffix`], [`Check::answer_signing`] | VAL-121, VAL-131, VAL-141 | CHK-004 |
/// | `check.answer_type_mismatch` | [`AnswerTypeMismatch`](Self::AnswerTypeMismatch) | [`Check::answer`] | VAL-122 | DNS-006 |
/// | `check.answer_malformed` | [`AnswerMalformed`](Self::AnswerMalformed) | [`Check::answer`] | VAL-123, VAL-124 | DNS-002, DNS-006 |
/// | `check.suffix_out_of_range` | [`SuffixOutOfRange`](Self::SuffixOutOfRange) | [`Check::answer_suffix`] | VAL-132 | DMC-011 |
/// | `check.list_name_invalid` | [`ListNameInvalid`](Self::ListNameInvalid) | [`Check::answer_suffix`] | VAL-133 | DMC-010 |
/// | `check.buffer_empty` | [`BufferEmpty`](Self::BufferEmpty) | [`Check::read_trace`] | VAL-150 | STR-002 |
/// | `check.not_finished` | [`NotFinished`](Self::NotFinished) | [`Check::take_results`] | VAL-160 | CHK-002 |
/// | `check.results_taken` | [`ResultsTaken`](Self::ResultsTaken) | [`Check::take_results`] | VAL-161 | CHK-002 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckError {
    /// `check.out_of_order`: message input in a state that doesn't accept it.
    OutOfOrder,
    /// `check.envelope_incomplete`: the envelope lacks a part that a kind
    /// needs.
    EnvelopeIncomplete {
        /// The part the envelope lacks.
        field: EnvelopeField,
    },
    /// `check.rcpt_to_limit`: more RCPT TO paths than `rcpt_to`.
    RcptToLimit,
    /// `check.body_in_header_section`: an empty line inside the raw header
    /// section.
    BodyInHeaderSection,
    /// `check.unknown_question`: no question with that ID was handed out.
    UnknownQuestion,
    /// `check.unknown_request`: no signing request with that ID was handed
    /// out.
    UnknownRequest,
    /// `check.already_answered`: the request has its answer.
    AlreadyAnswered,
    /// `check.answer_type_mismatch`: a record of another type than the
    /// question's.
    AnswerTypeMismatch,
    /// `check.answer_malformed`: an empty list of records, or a TXT string
    /// over 255 octets.
    AnswerMalformed,
    /// `check.suffix_out_of_range`: a suffix of 0 labels, or of more labels
    /// than the domain.
    SuffixOutOfRange,
    /// `check.list_name_invalid`: a list name of 0 octets or over 256.
    ListNameInvalid,
    /// `check.buffer_empty`: a buffer of 0 bytes.
    BufferEmpty,
    /// `check.not_finished`: the check isn't Finished. Carries what it waits
    /// for.
    NotFinished(Waiting),
    /// `check.results_taken`: the results were taken before.
    ResultsTaken,
}

impl CheckError {
    /// The error's code, such as `check.out_of_order` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for CheckError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for CheckError {}

/// The part of the envelope that [`CheckError::EnvelopeIncomplete`] names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EnvelopeField {
    /// [`Envelope::client_ip`].
    ClientIp,
    /// [`Envelope::helo`].
    Helo,
    /// [`Envelope::mail_from`].
    MailFrom,
    /// [`Envelope::rcpt_to`].
    RcptTo,
}
