//! The limits of a check and of the APIs outside a check.

use crate::*;

/// The limits of one check (SAF-002, PAR-004).
///
/// [`Options::limits`] holds one, and [`Canonicalizer::new`] and
/// [`AuthenticationResults::parse`] read the limits that apply to them. Each
/// limit has a getter, named after it, and a setter, named `set_` and the
/// limit. A setter takes the new value and checks it, so a `Limits` value is
/// always valid, and an API that takes one checks nothing more about it.
/// [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-of-a-check) gives each limit's default,
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
/// | VAL-202 | `value` | Within the range that [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-of-a-check) gives for the limit | [`limits.out_of_range`](LimitsError::OutOfRange) | PAR-004 | API-202 |
/// | VAL-203 | `value` of [`set_header_field_size`](Self::set_header_field_size) or [`set_header_section_size`](Self::set_header_section_size) | `header_field_size` at most `header_section_size`, after the change | [`limits.conflict`](LimitsError::Conflict) | PAR-004 | API-203 |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    values: [u32; 23],
}

impl Default for Limits {
    /// Every limit of a check at the default that
    /// [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-of-a-check) gives (PAR-004).
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

    /// `author_domains`: the most author domains evaluated when the caller asks for several (DMC-002).
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

    /// `answer_size`: the most octets of record data in one answer (SAF-004).
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

    /// `trace_places`: the most places listed for each kind of body change, and for bare CR and LF (TRC-010).
    pub fn trace_places(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`trace_places`](Self::trace_places) to `value`, under the rules of
    /// [Validation](#validation).
    pub fn set_trace_places(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }
}

/// The limits of the APIs that read documents outside a check: snapshots,
/// traces and pasted records.
///
/// [`sort_trace`], [`Event::parse`], [`Snapshot::parse`], [`Snapshot::from_trace`]
/// and [`PastedRecords::parse`] take one. Each limit has a getter and a setter,
/// as in [`Limits`], and a setter checks its value, so a `DocumentLimits`
/// value is always valid. [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check)
/// gives each limit's default, range and effect.
///
/// [`DocumentLimits::default`] sets every limit to its default.
///
/// # Validation
///
/// | ID | Input | Rule | Error code | Requirement | Scenario |
/// |---|---|---|---|---|---|
/// | VAL-204 | `value` | Within the range that [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check) gives for the limit | [`limits.out_of_range`](LimitsError::OutOfRange) | PAR-004 | API-204 |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentLimits {
    values: [u32; 3],
}

impl Default for DocumentLimits {
    /// Every limit at the default that
    /// [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check) gives (DNS-004,
    /// DNS-010).
    fn default() -> DocumentLimits {
        unimplemented!()
    }
}

impl DocumentLimits {
    /// `pasted_records_size`: the most octets of pasted records (DNS-010).
    pub fn pasted_records_size(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`pasted_records_size`](Self::pasted_records_size) to `value`, under the rule of
    /// [Validation](#validation).
    pub fn set_pasted_records_size(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `document_size`: the most octets of a snapshot or a demo case (DNS-004).
    pub fn document_size(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`document_size`](Self::document_size) to `value`, under the rule of
    /// [Validation](#validation).
    pub fn set_document_size(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
    }

    /// `document_depth`: the most levels of nesting in a snapshot or a demo case (PAR-005).
    pub fn document_depth(&self) -> u32 {
        unimplemented!()
    }

    /// Sets [`document_depth`](Self::document_depth) to `value`, under the rule of
    /// [Validation](#validation).
    pub fn set_document_depth(&mut self, value: u32) -> Result<(), LimitsError> {
        unimplemented!()
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
    /// `author_domains`: author domains evaluated when the caller asks for several.
    AuthorDomains,
    /// `dns_questions`: distinct DNS questions in one check.
    DnsQuestions,
    /// `answer_size`: octets of record data in one answer.
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
    /// `trace_places`: places listed for each kind of body change, and for bare CR and LF.
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
        unimplemented!()
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
        unimplemented!()
    }
}

impl core::fmt::Display for LimitsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for LimitsError {}
