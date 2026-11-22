//! The limits of a check or a signing, and of the APIs outside them.

use core::ops::RangeInclusive;

// The doc comments link to items of other modules.
#[cfg(doc)]
use crate::*;

/// The limits of one check or one signing (SAF-002, PAR-004).
///
/// [`CheckOptions::limits`] and [`SigningOptions::limits`] hold one, and
/// [`Canonicalizer::new`] and [`AuthenticationResults::parse`] read the limits
/// that apply to them. Each limit has a getter, named after it, and a setter,
/// named `set_` and the limit. A setter takes the new value and checks it, so
/// a `Limits` value is always valid, and an API that takes one checks nothing
/// more about it.
/// [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-of-a-check-or-a-signing) gives each limit's default,
/// range and effect. Every value fits in `u32`.
///
/// [`Limits::default`] sets every limit to its default.
///
/// # Validation
///
/// Each setter checks these rules in order and gives the code of the first
/// one broken. A setter that returns an error leaves the limits as they were.
///
/// | ID | Input | Rule | Error code | Requirement | Scenario |
/// |---|---|---|---|---|---|
/// | VAL-202 | `value` | Within the range that [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-of-a-check-or-a-signing) gives for the limit | [`limits.out_of_range`](LimitsError::OutOfRange) | PAR-004 | API-202 |
/// | VAL-203 | `value` of [`set_header_field_size`](Self::set_header_field_size) or [`set_header_section_size`](Self::set_header_section_size) | `header_field_size` at most `header_section_size`, after the change | [`limits.conflict`](LimitsError::Conflict) | PAR-004 | API-203 |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    values: [u32; 23],
}

impl Default for Limits {
    /// Every limit of a check or a signing at the default that
    /// [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-of-a-check-or-a-signing) gives (PAR-004).
    fn default() -> Limits {
        unimplemented!()
    }
}

impl Limits {
    /// `header_fields`: the most header fields and malformed lines in the header section (SAF-002).
    pub fn header_fields(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`header_fields`](Self::header_fields) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_header_fields(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `header_field_size`: the most octets of one header field, its name and folding included (SAF-002).
    pub fn header_field_size(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`header_field_size`](Self::header_field_size) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_header_field_size(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `header_section_size`: the most octets of the raw header section (SAF-002).
    pub fn header_section_size(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`header_section_size`](Self::header_section_size) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_header_section_size(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `comment_depth`: the most levels of nested comments (HDR-011).
    pub fn comment_depth(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`comment_depth`](Self::comment_depth) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_comment_depth(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `mailboxes`: the most mailboxes in one address list (SAF-002).
    pub fn mailboxes(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`mailboxes`](Self::mailboxes) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_mailboxes(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `address_size`: the most octets of one address or path (ADS-007).
    pub fn address_size(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`address_size`](Self::address_size) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_address_size(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `rcpt_to`: the most RCPT TO paths, in the envelope and in DKIM2 signing (SAF-004).
    pub fn rcpt_to(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`rcpt_to`](Self::rcpt_to) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_rcpt_to(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `rt_entries`: the most entries in one DKIM2 `rt=` tag (SAF-002).
    pub fn rt_entries(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`rt_entries`](Self::rt_entries) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_rt_entries(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `dkim_signatures`: the most DKIM-Signature fields verified, in header order (DKV-004).
    pub fn dkim_signatures(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`dkim_signatures`](Self::dkim_signatures) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_dkim_signatures(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `key_records`: the most key records at one DKIM selector (REC-052).
    pub fn key_records(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`key_records`](Self::key_records) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_key_records(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `rsa_key_bits`: the most bits of an RSA modulus (DKV-003).
    pub fn rsa_key_bits(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`rsa_key_bits`](Self::rsa_key_bits) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_rsa_key_bits(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `rsa_exponent_bits`: the most bits of an RSA public exponent in a DKIM key (REC-051).
    pub fn rsa_exponent_bits(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`rsa_exponent_bits`](Self::rsa_exponent_bits) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_rsa_exponent_bits(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `author_domains`: the most author domains evaluated when the verifying
    /// caller asks for several (DMC-002).
    pub fn author_domains(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`author_domains`](Self::author_domains) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_author_domains(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `dns_questions`: the most distinct DNS questions in one check (SAF-002).
    pub fn dns_questions(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`dns_questions`](Self::dns_questions) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_dns_questions(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `answer_size`: the most octets of record data and description in one DNS
    /// answer, and of the description in one public suffix answer (SAF-004).
    pub fn answer_size(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`answer_size`](Self::answer_size) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_answer_size(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `dkim2_signatures`: the most DKIM2-Signature fields (SAF-002).
    pub fn dkim2_signatures(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`dkim2_signatures`](Self::dkim2_signatures) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_dkim2_signatures(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `message_instances`: the most Message-Instance fields (SAF-002).
    pub fn message_instances(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`message_instances`](Self::message_instances) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_message_instances(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `recipe_steps`: the most steps in one recipe (DK2-007).
    pub fn recipe_steps(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`recipe_steps`](Self::recipe_steps) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_recipe_steps(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `recipe_size`: the most octets of one recipe's decoded JSON (SAF-002).
    pub fn recipe_size(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`recipe_size`](Self::recipe_size) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_recipe_size(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `recipe_depth`: the most levels of nesting in recipe JSON (PAR-005).
    pub fn recipe_depth(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`recipe_depth`](Self::recipe_depth) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_recipe_depth(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `spf_void_lookups`: the most void lookups in one SPF evaluation (SPF-006).
    pub fn spf_void_lookups(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`spf_void_lookups`](Self::spf_void_lookups) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_spf_void_lookups(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `trace_size`: the most octets of trace events written, read or not (TRC-005).
    pub fn trace_size(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`trace_size`](Self::trace_size) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_trace_size(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `trace_places`: the most places listed for each kind of body change, and
    /// for bare CR and LF in the body and the header section (TRC-010).
    pub fn trace_places(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`trace_places`](Self::trace_places) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_trace_places(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }
}

/// The limits of the APIs that read documents outside a check or a signing:
/// snapshots, traces and pasted records.
///
/// [`sort_trace`], [`Event::parse`], [`Snapshot::parse`], [`Snapshot::from_trace`]
/// and [`PastedRecords::parse`] take one. Each limit has a getter and a setter,
/// as in [`Limits`], and a setter checks its value, so a `DocumentLimits`
/// value is always valid. [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check-or-a-signing)
/// gives each limit's default, range and effect.
///
/// [`DocumentLimits::default`] sets every limit to its default.
///
/// # Validation
///
/// | ID | Input | Rule | Error code | Requirement | Scenario |
/// |---|---|---|---|---|---|
/// | VAL-204 | `value` | Within the range that [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check-or-a-signing) gives for the limit | [`limits.out_of_range`](LimitsError::OutOfRange) | PAR-004 | API-204 |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentLimits {
    pasted_records_size: u32,
    document_size: u32,
    document_depth: u32,
}

impl Default for DocumentLimits {
    /// Every limit at the default that
    /// [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check-or-a-signing) gives (DNS-004,
    /// DNS-010).
    fn default() -> DocumentLimits {
        DocumentLimits {
            pasted_records_size: Limit::PastedRecordsSize.default_value(),
            document_size: Limit::DocumentSize.default_value(),
            document_depth: Limit::DocumentDepth.default_value(),
        }
    }
}

impl DocumentLimits {
    /// `pasted_records_size`: the most octets of pasted records (DNS-010).
    pub fn pasted_records_size(&self) -> u32 {
        self.pasted_records_size
    }

    /// Sets [`pasted_records_size`](Self::pasted_records_size) to `value`, under the rule of
    /// [Validation](#validation).
    pub fn set_pasted_records_size(&mut self, value: u32) -> Result<(), LimitsError> {
        self.pasted_records_size = Limit::PastedRecordsSize.checked(value)?;
        Ok(())
    }

    /// `document_size`: the most octets of a snapshot or a demo case (DNS-004).
    pub fn document_size(&self) -> u32 {
        self.document_size
    }

    /// Sets [`document_size`](Self::document_size) to `value`, under the rule of
    /// [Validation](#validation).
    pub fn set_document_size(&mut self, value: u32) -> Result<(), LimitsError> {
        self.document_size = Limit::DocumentSize.checked(value)?;
        Ok(())
    }

    /// `document_depth`: the most levels of nesting in a snapshot or a demo case (PAR-005).
    pub fn document_depth(&self) -> u32 {
        self.document_depth
    }

    /// Sets [`document_depth`](Self::document_depth) to `value`, under the rule of
    /// [Validation](#validation).
    pub fn set_document_depth(&mut self, value: u32) -> Result<(), LimitsError> {
        self.document_depth = Limit::DocumentDepth.checked(value)?;
        Ok(())
    }
}

/// One limit, as errors and results name it (SAF-002).
///
/// It has one variant for each limit of [`Limits`] and [`DocumentLimits`], and
/// one for `driver_in_flight`, the `in_flight` parameter of [`drive`].
/// [`Limit::name`] gives the name that [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md) writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Limit {
    /// `header_fields`: header fields and malformed lines in the header section.
    HeaderFields,
    /// `header_field_size`: octets of one header field, its name and folding included.
    HeaderFieldSize,
    /// `header_section_size`: octets of the raw header section.
    HeaderSectionSize,
    /// `comment_depth`: levels of nested comments.
    CommentDepth,
    /// `mailboxes`: mailboxes in one address list.
    Mailboxes,
    /// `address_size`: octets of one address or path.
    AddressSize,
    /// `rcpt_to`: RCPT TO paths, in the envelope and in DKIM2 signing.
    RcptTo,
    /// `rt_entries`: entries in one DKIM2 `rt=` tag.
    RtEntries,
    /// `dkim_signatures`: DKIM-Signature fields verified, in header order.
    DkimSignatures,
    /// `key_records`: key records at one DKIM selector.
    KeyRecords,
    /// `rsa_key_bits`: bits of an RSA modulus.
    RsaKeyBits,
    /// `rsa_exponent_bits`: bits of an RSA public exponent in a DKIM key.
    RsaExponentBits,
    /// `author_domains`: author domains evaluated when the verifying caller asks for several.
    AuthorDomains,
    /// `dns_questions`: distinct DNS questions in one check.
    DnsQuestions,
    /// `answer_size`: octets of record data and description in one DNS answer,
    /// and of the description in one public suffix answer.
    AnswerSize,
    /// `dkim2_signatures`: DKIM2-Signature fields.
    Dkim2Signatures,
    /// `message_instances`: Message-Instance fields.
    MessageInstances,
    /// `recipe_steps`: steps in one recipe.
    RecipeSteps,
    /// `recipe_size`: octets of one recipe's decoded JSON.
    RecipeSize,
    /// `recipe_depth`: levels of nesting in recipe JSON.
    RecipeDepth,
    /// `spf_void_lookups`: void lookups in one SPF evaluation.
    SpfVoidLookups,
    /// `trace_size`: octets of trace events written, read or not.
    TraceSize,
    /// `trace_places`: places listed for each kind of body change, and for bare
    /// CR and LF in the body and the header section.
    TracePlaces,
    /// `pasted_records_size`: octets of pasted records.
    PastedRecordsSize,
    /// `document_size`: octets of a snapshot or a demo case.
    DocumentSize,
    /// `document_depth`: levels of nesting in a snapshot or a demo case.
    DocumentDepth,
    /// `driver_in_flight`: futures the driver keeps pending at once.
    DriverInFlight,
}

impl Limit {
    /// The limit's name as [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md) writes it, such as
    /// `header_fields`.
    ///
    /// Requirements: SAF-002.
    pub fn name(self) -> &'static str {
        match self {
            Limit::HeaderFields => "header_fields",
            Limit::HeaderFieldSize => "header_field_size",
            Limit::HeaderSectionSize => "header_section_size",
            Limit::CommentDepth => "comment_depth",
            Limit::Mailboxes => "mailboxes",
            Limit::AddressSize => "address_size",
            Limit::RcptTo => "rcpt_to",
            Limit::RtEntries => "rt_entries",
            Limit::DkimSignatures => "dkim_signatures",
            Limit::KeyRecords => "key_records",
            Limit::RsaKeyBits => "rsa_key_bits",
            Limit::RsaExponentBits => "rsa_exponent_bits",
            Limit::AuthorDomains => "author_domains",
            Limit::DnsQuestions => "dns_questions",
            Limit::AnswerSize => "answer_size",
            Limit::Dkim2Signatures => "dkim2_signatures",
            Limit::MessageInstances => "message_instances",
            Limit::RecipeSteps => "recipe_steps",
            Limit::RecipeSize => "recipe_size",
            Limit::RecipeDepth => "recipe_depth",
            Limit::SpfVoidLookups => "spf_void_lookups",
            Limit::TraceSize => "trace_size",
            Limit::TracePlaces => "trace_places",
            Limit::PastedRecordsSize => "pasted_records_size",
            Limit::DocumentSize => "document_size",
            Limit::DocumentDepth => "document_depth",
            Limit::DriverInFlight => "driver_in_flight",
        }
    }

    /// The default of the limit and the values it takes, as
    /// [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md)
    /// gives them.
    fn default_and_range(self) -> (u32, RangeInclusive<u32>) {
        match self {
            Limit::HeaderFields => (1_000, 1..=100_000),
            Limit::HeaderFieldSize => (65_536, 1..=16_777_216),
            Limit::HeaderSectionSize => (1_048_576, 1..=16_777_216),
            Limit::CommentDepth => (16, 1..=256),
            Limit::Mailboxes => (64, 1..=10_000),
            Limit::AddressSize => (1_024, 1..=65_536),
            Limit::RcptTo => (1_000, 1..=100_000),
            Limit::RtEntries => (1_000, 1..=100_000),
            Limit::DkimSignatures => (16, 1..=1_000),
            Limit::KeyRecords => (8, 1..=100),
            Limit::RsaKeyBits => (4_096, 1_024..=16_384),
            Limit::RsaExponentBits => (32, 2..=16_384),
            Limit::AuthorDomains => (4, 1..=100),
            Limit::DnsQuestions => (512, 1..=100_000),
            Limit::AnswerSize => (65_535, 512..=16_777_216),
            Limit::Dkim2Signatures => (32, 1..=1_000),
            Limit::MessageInstances => (32, 1..=1_000),
            Limit::RecipeSteps => (10_000, 1..=1_000_000),
            Limit::RecipeSize => (65_536, 1..=16_777_216),
            Limit::RecipeDepth => (8, 1..=64),
            Limit::SpfVoidLookups => (2, 0..=10),
            Limit::TraceSize => (1_048_576, 1_024..=268_435_456),
            Limit::TracePlaces => (16, 0..=10_000),
            Limit::PastedRecordsSize => (1_048_576, 1..=16_777_216),
            Limit::DocumentSize => (33_554_432, 1..=268_435_456),
            Limit::DocumentDepth => (16, 1..=64),
            Limit::DriverInFlight => (16, 1..=1_024),
        }
    }

    /// The default of the limit.
    fn default_value(self) -> u32 {
        let (default, _) = self.default_and_range();
        default
    }

    /// Gives `value` back if the limit takes it, and `limits.out_of_range`
    /// otherwise (VAL-202, VAL-204).
    fn checked(self, value: u32) -> Result<u32, LimitsError> {
        let (_, range) = self.default_and_range();
        if range.contains(&value) {
            Ok(value)
        } else {
            Err(LimitsError::OutOfRange { limit: self })
        }
    }
}

/// The error of a setter of [`Limits`] or [`DocumentLimits`].
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `limits.out_of_range` | [`OutOfRange`](Self::OutOfRange) | The setters of [`Limits`] and [`DocumentLimits`] | VAL-202, VAL-204 | PAR-004 |
/// | `limits.conflict` | [`Conflict`](Self::Conflict) | [`Limits::set_header_field_size`], [`Limits::set_header_section_size`] | VAL-203 | PAR-004 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LimitsError {
    /// `limits.out_of_range`: a value outside the limit's range.
    OutOfRange {
        /// The limit whose range the value left.
        limit: Limit,
    },
    /// `limits.conflict`: `header_field_size` over `header_section_size`.
    Conflict,
}

impl LimitsError {
    /// The error's code, such as `limits.out_of_range` (STB-001).
    pub fn code(&self) -> &'static str {
        match self {
            LimitsError::OutOfRange { .. } => "limits.out_of_range",
            LimitsError::Conflict => "limits.conflict",
        }
    }
}

impl core::fmt::Display for LimitsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LimitsError::OutOfRange { limit } => {
                let name = limit.name();
                write!(f, "limits.out_of_range: {name} is outside its range")
            }
            LimitsError::Conflict => {
                let text = "limits.conflict: header_field_size is over header_section_size";
                f.write_str(text)
            }
        }
    }
}

impl std::error::Error for LimitsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_202_comment_depth_takes_1_to_256() {
        let mut limits = Limits::default();
        let out_of_range = LimitsError::OutOfRange {
            limit: Limit::CommentDepth,
        };
        assert_eq!(limits.set_comment_depth(0), Err(out_of_range.clone()));
        assert_eq!(limits.set_comment_depth(257), Err(out_of_range.clone()));
        assert_eq!(limits.comment_depth(), 16);
        assert_eq!(limits.set_comment_depth(256), Ok(()));
        assert_eq!(limits.comment_depth(), 256);
        assert!(out_of_range.to_string().contains("comment_depth"));
    }

    #[test]
    fn api_203_header_field_size_stays_within_header_section_size() {
        let mut limits = Limits::default();
        assert_eq!(limits.header_section_size(), 1_048_576);
        assert_eq!(
            limits.set_header_field_size(2_000_000),
            Err(LimitsError::Conflict)
        );
        assert_eq!(limits.header_field_size(), 65_536);
        assert_eq!(limits.set_header_section_size(2_000_000), Ok(()));
        assert_eq!(limits.set_header_field_size(2_000_000), Ok(()));
        assert_eq!(limits.header_field_size(), 2_000_000);
        assert_eq!(
            limits.set_header_section_size(1_999_999),
            Err(LimitsError::Conflict)
        );
        assert_eq!(limits.header_section_size(), 2_000_000);
        assert_eq!(LimitsError::Conflict.code(), "limits.conflict");
        let text = LimitsError::Conflict.to_string();
        assert!(text.starts_with("limits.conflict"));
    }

    #[test]
    fn prs_006_limits_have_their_documented_defaults() {
        let limits = Limits::default();
        let defaults = [
            (limits.header_fields(), 1_000),
            (limits.header_field_size(), 65_536),
            (limits.header_section_size(), 1_048_576),
            (limits.comment_depth(), 16),
            (limits.mailboxes(), 64),
            (limits.address_size(), 1_024),
            (limits.rcpt_to(), 1_000),
            (limits.rt_entries(), 1_000),
            (limits.dkim_signatures(), 16),
            (limits.key_records(), 8),
            (limits.rsa_key_bits(), 4_096),
            (limits.rsa_exponent_bits(), 32),
            (limits.author_domains(), 4),
            (limits.dns_questions(), 512),
            (limits.answer_size(), 65_535),
            (limits.dkim2_signatures(), 32),
            (limits.message_instances(), 32),
            (limits.recipe_steps(), 10_000),
            (limits.recipe_size(), 65_536),
            (limits.recipe_depth(), 8),
            (limits.spf_void_lookups(), 2),
            (limits.trace_size(), 1_048_576),
            (limits.trace_places(), 16),
        ];
        for (value, default) in defaults {
            assert_eq!(value, default);
        }
    }

    #[test]
    fn api_204_document_depth_takes_1_to_64() {
        let mut limits = DocumentLimits::default();
        let out_of_range = LimitsError::OutOfRange {
            limit: Limit::DocumentDepth,
        };
        assert_eq!(limits.set_document_depth(65), Err(out_of_range.clone()));
        assert_eq!(limits.document_depth(), 16);
        assert_eq!(out_of_range.code(), "limits.out_of_range");
        let text = out_of_range.to_string();
        assert!(text.starts_with("limits.out_of_range"));
        assert!(text.contains("document_depth"));
        assert_eq!(limits.set_document_depth(64), Ok(()));
        assert_eq!(limits.document_depth(), 64);
    }

    #[test]
    fn prs_006_document_limits_have_their_documented_defaults() {
        let documents = DocumentLimits::default();
        let defaults = [
            (documents.pasted_records_size(), 1_048_576),
            (documents.document_size(), 33_554_432),
            (documents.document_depth(), 16),
        ];
        for (value, default) in defaults {
            assert_eq!(value, default);
        }
    }

    #[test]
    fn prs_006_limits_have_their_documented_names() {
        let names = [
            (Limit::HeaderFields, "header_fields"),
            (Limit::HeaderFieldSize, "header_field_size"),
            (Limit::HeaderSectionSize, "header_section_size"),
            (Limit::CommentDepth, "comment_depth"),
            (Limit::Mailboxes, "mailboxes"),
            (Limit::AddressSize, "address_size"),
            (Limit::RcptTo, "rcpt_to"),
            (Limit::RtEntries, "rt_entries"),
            (Limit::DkimSignatures, "dkim_signatures"),
            (Limit::KeyRecords, "key_records"),
            (Limit::RsaKeyBits, "rsa_key_bits"),
            (Limit::RsaExponentBits, "rsa_exponent_bits"),
            (Limit::AuthorDomains, "author_domains"),
            (Limit::DnsQuestions, "dns_questions"),
            (Limit::AnswerSize, "answer_size"),
            (Limit::Dkim2Signatures, "dkim2_signatures"),
            (Limit::MessageInstances, "message_instances"),
            (Limit::RecipeSteps, "recipe_steps"),
            (Limit::RecipeSize, "recipe_size"),
            (Limit::RecipeDepth, "recipe_depth"),
            (Limit::SpfVoidLookups, "spf_void_lookups"),
            (Limit::TraceSize, "trace_size"),
            (Limit::TracePlaces, "trace_places"),
            (Limit::PastedRecordsSize, "pasted_records_size"),
            (Limit::DocumentSize, "document_size"),
            (Limit::DocumentDepth, "document_depth"),
            (Limit::DriverInFlight, "driver_in_flight"),
        ];
        for (limit, name) in names {
            assert_eq!(limit.name(), name);
        }
    }
}
