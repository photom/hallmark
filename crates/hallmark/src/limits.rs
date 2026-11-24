//! The limits of a check or a signing, and of the APIs outside them.

use core::ops::RangeInclusive;

// The doc comments link to items of other modules.
#[cfg(doc)]
use crate::{
    AuthenticationResults, Canonicalizer, CheckOptions, Event, PastedRecords, SigningOptions,
    Snapshot, drive, sort_trace,
};

/// The limits of one check or one signing (SAF-002, PAR-004).
///
/// [`CheckOptions::limits`] and [`SigningOptions::limits`] hold one, and
/// [`Canonicalizer::new`], [`AuthenticationResults::parse`] and
/// [`AuthenticationResults::carries_authserv_id`] take the limits that apply
/// to them. A signing or a verifying caller starts from
/// [`Limits::default`], which sets every limit to its default, and sets the
/// fields it wants to change, in any order. `Limits` checks nothing while the
/// caller sets them: each API that takes one checks every rule at once, as
/// [`Limits::validate`] does, before it reads any input.
/// [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-of-a-check-or-a-signing) gives each limit's default,
/// range and effect. Every value fits in `u32`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Limits {
    /// The most header fields and malformed lines in the header section
    /// (SAF-002).
    pub header_fields: u32,
    /// The most octets of one header field, its name and folding included
    /// (SAF-002).
    pub header_field_size: u32,
    /// The most octets of the raw header section (SAF-002).
    pub header_section_size: u32,
    /// The most levels of nested comments (HDR-011).
    pub comment_depth: u32,
    /// The most mailboxes in one address list (SAF-002).
    pub mailboxes: u32,
    /// The most octets of one address or path (ADS-007).
    pub address_size: u32,
    /// The most RCPT TO paths, in the envelope and in DKIM2 signing (SAF-004).
    pub rcpt_to: u32,
    /// The most entries in one DKIM2 `rt=` tag (SAF-002).
    pub rt_entries: u32,
    /// The most DKIM-Signature fields verified, in header order (DKV-004).
    pub dkim_signatures: u32,
    /// The most key records at one DKIM selector (REC-052).
    pub key_records: u32,
    /// The most bits of an RSA modulus (DKV-003).
    pub rsa_key_bits: u32,
    /// The most bits of an RSA public exponent in a DKIM key (REC-051).
    pub rsa_exponent_bits: u32,
    /// The most author domains evaluated when the verifying caller asks for
    /// several (DMC-002).
    pub author_domains: u32,
    /// The most distinct DNS questions in one check (SAF-002).
    pub dns_questions: u32,
    /// The most octets of record data and description in one DNS answer, and of
    /// the description in one public suffix answer (SAF-004).
    pub answer_size: u32,
    /// The most DKIM2-Signature fields (SAF-002).
    pub dkim2_signatures: u32,
    /// The most Message-Instance fields (SAF-002).
    pub message_instances: u32,
    /// The most steps in one recipe (DK2-007).
    pub recipe_steps: u32,
    /// The most octets of one recipe's decoded JSON (SAF-002).
    pub recipe_size: u32,
    /// The most levels of nesting in recipe JSON (PAR-005).
    pub recipe_depth: u32,
    /// The most void lookups in one SPF evaluation (SPF-006).
    pub spf_void_lookups: u32,
    /// The most octets of trace events written, read or not (TRC-005).
    pub trace_size: u32,
    /// The most places listed for each kind of body change, and for bare CR and
    /// LF in the body and the header section (TRC-010).
    pub trace_places: u32,
}

impl Default for Limits {
    /// Every limit of a check or a signing at the default that
    /// [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-of-a-check-or-a-signing) gives (PAR-004).
    fn default() -> Limits {
        Limits {
            header_fields: Limit::HeaderFields.default_value(),
            header_field_size: Limit::HeaderFieldSize.default_value(),
            header_section_size: Limit::HeaderSectionSize.default_value(),
            comment_depth: Limit::CommentDepth.default_value(),
            mailboxes: Limit::Mailboxes.default_value(),
            address_size: Limit::AddressSize.default_value(),
            rcpt_to: Limit::RcptTo.default_value(),
            rt_entries: Limit::RtEntries.default_value(),
            dkim_signatures: Limit::DkimSignatures.default_value(),
            key_records: Limit::KeyRecords.default_value(),
            rsa_key_bits: Limit::RsaKeyBits.default_value(),
            rsa_exponent_bits: Limit::RsaExponentBits.default_value(),
            author_domains: Limit::AuthorDomains.default_value(),
            dns_questions: Limit::DnsQuestions.default_value(),
            answer_size: Limit::AnswerSize.default_value(),
            dkim2_signatures: Limit::Dkim2Signatures.default_value(),
            message_instances: Limit::MessageInstances.default_value(),
            recipe_steps: Limit::RecipeSteps.default_value(),
            recipe_size: Limit::RecipeSize.default_value(),
            recipe_depth: Limit::RecipeDepth.default_value(),
            spf_void_lookups: Limit::SpfVoidLookups.default_value(),
            trace_size: Limit::TraceSize.default_value(),
            trace_places: Limit::TracePlaces.default_value(),
        }
    }
}

impl Limits {
    /// Checks every rule of the limits at once, in the order of the table
    /// below, and gives the code of the first one broken. The APIs that take
    /// limits run the same checks, so a signing or a verifying caller calls
    /// this only to find a mistake earlier, such as when it reads its
    /// configuration.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-202 | Each limit, in the order of the fields | Within the range that [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-of-a-check-or-a-signing) gives for the limit | [`limits.out_of_range`](LimitsError::OutOfRange), naming the first limit outside its range | PAR-004 | API-202 |
    /// | VAL-203 | `header_field_size`, `header_section_size` | `header_field_size` at most `header_section_size` | [`limits.conflict`](LimitsError::Conflict) | PAR-004 | API-203 |
    ///
    /// Requirements: PAR-004.
    pub fn validate(&self) -> Result<(), LimitsError> {
        let values = [
            (Limit::HeaderFields, self.header_fields),
            (Limit::HeaderFieldSize, self.header_field_size),
            (Limit::HeaderSectionSize, self.header_section_size),
            (Limit::CommentDepth, self.comment_depth),
            (Limit::Mailboxes, self.mailboxes),
            (Limit::AddressSize, self.address_size),
            (Limit::RcptTo, self.rcpt_to),
            (Limit::RtEntries, self.rt_entries),
            (Limit::DkimSignatures, self.dkim_signatures),
            (Limit::KeyRecords, self.key_records),
            (Limit::RsaKeyBits, self.rsa_key_bits),
            (Limit::RsaExponentBits, self.rsa_exponent_bits),
            (Limit::AuthorDomains, self.author_domains),
            (Limit::DnsQuestions, self.dns_questions),
            (Limit::AnswerSize, self.answer_size),
            (Limit::Dkim2Signatures, self.dkim2_signatures),
            (Limit::MessageInstances, self.message_instances),
            (Limit::RecipeSteps, self.recipe_steps),
            (Limit::RecipeSize, self.recipe_size),
            (Limit::RecipeDepth, self.recipe_depth),
            (Limit::SpfVoidLookups, self.spf_void_lookups),
            (Limit::TraceSize, self.trace_size),
            (Limit::TracePlaces, self.trace_places),
        ];
        for (limit, value) in values {
            limit.check(value)?;
        }
        if self.header_field_size > self.header_section_size {
            return Err(LimitsError::Conflict);
        }
        Ok(())
    }
}

/// The limits of the APIs that read documents outside a check or a signing:
/// snapshots, traces and pasted records.
///
/// [`sort_trace`], [`Event::parse`], [`Snapshot::parse`], [`Snapshot::from_trace`]
/// and [`PastedRecords::parse`] take one. A caller starts from
/// [`DocumentLimits::default`], which sets every limit to its default, and sets
/// the fields it wants to change. As with [`Limits`], each API that takes one
/// checks it, as [`DocumentLimits::validate`] does, before it reads any input.
/// [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check-or-a-signing)
/// gives each limit's default, range and effect.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DocumentLimits {
    /// The most octets of pasted records (DNS-010).
    pub pasted_records_size: u32,
    /// The most octets of a trace, a snapshot or a demo case (DNS-004,
    /// SAF-004).
    pub document_size: u32,
    /// The most levels of nesting in a trace event, a snapshot or a demo case
    /// (PAR-005).
    pub document_depth: u32,
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
    /// Checks every limit at once, in the order of the fields, and gives the
    /// code of the first one broken. The APIs that take document limits run
    /// the same check.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-204 | Each limit, in the order of the fields | Within the range that [Limits](https://github.com/photom/hallmark/blob/main/docs/reference/design/limits.md#limits-outside-a-check-or-a-signing) gives for the limit | [`limits.out_of_range`](LimitsError::OutOfRange), naming the first limit outside its range | PAR-004 | API-204 |
    ///
    /// Requirements: PAR-004.
    pub fn validate(&self) -> Result<(), LimitsError> {
        let values = [
            (Limit::PastedRecordsSize, self.pasted_records_size),
            (Limit::DocumentSize, self.document_size),
            (Limit::DocumentDepth, self.document_depth),
        ];
        for (limit, value) in values {
            limit.check(value)?;
        }
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
    /// `document_size`: octets of a trace, a snapshot or a demo case.
    DocumentSize,
    /// `document_depth`: levels of nesting in a trace event, a snapshot or a
    /// demo case.
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

    /// Gives `limits.out_of_range` if the limit doesn't take `value`
    /// (VAL-202, VAL-204).
    fn check(self, value: u32) -> Result<(), LimitsError> {
        let (_, range) = self.default_and_range();
        if range.contains(&value) {
            Ok(())
        } else {
            Err(LimitsError::OutOfRange { limit: self })
        }
    }
}

/// The error of [`Limits::validate`] and [`DocumentLimits::validate`]. Each API
/// that takes limits carries it in a `LimitsInvalid` variant of its own error.
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `limits.out_of_range` | [`OutOfRange`](Self::OutOfRange) | [`Limits::validate`], [`DocumentLimits::validate`] | VAL-202, VAL-204 | PAR-004 |
/// | `limits.conflict` | [`Conflict`](Self::Conflict) | [`Limits::validate`] | VAL-203 | PAR-004 |
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
    use super::{DocumentLimits, Limit, Limits, LimitsError};

    #[test]
    fn api_202_comment_depth_takes_1_to_256() {
        let mut limits = Limits::default();
        let out_of_range = LimitsError::OutOfRange {
            limit: Limit::CommentDepth,
        };
        for value in [0, 257] {
            limits.comment_depth = value;
            assert_eq!(limits.validate(), Err(out_of_range.clone()));
        }
        limits.comment_depth = 256;
        assert_eq!(limits.validate(), Ok(()));
        assert!(out_of_range.to_string().contains("comment_depth"));
        let two = Limits {
            header_fields: 0,
            comment_depth: 0,
            ..Limits::default()
        };
        let first = LimitsError::OutOfRange {
            limit: Limit::HeaderFields,
        };
        assert_eq!(two.validate(), Err(first));
    }

    #[test]
    fn api_203_header_field_size_stays_within_header_section_size() {
        let mut limits = Limits::default();
        assert_eq!(limits.header_section_size, 1_048_576);
        limits.header_field_size = 2_000_000;
        assert_eq!(limits.validate(), Err(LimitsError::Conflict));
        limits.header_section_size = 2_000_000;
        assert_eq!(limits.validate(), Ok(()));
        let mut other_order = Limits::default();
        assert_eq!(other_order.validate(), Ok(()));
        other_order.header_section_size = 2_000_000;
        other_order.header_field_size = 2_000_000;
        assert_eq!(other_order.validate(), Ok(()));
        limits.header_field_size = 20_000_000;
        let out_of_range = LimitsError::OutOfRange {
            limit: Limit::HeaderFieldSize,
        };
        assert_eq!(limits.validate(), Err(out_of_range));
        assert_eq!(LimitsError::Conflict.code(), "limits.conflict");
        let text = LimitsError::Conflict.to_string();
        assert!(text.starts_with("limits.conflict"));
    }

    #[test]
    fn prs_006_limits_have_their_documented_defaults() {
        let limits = Limits::default();
        let defaults = [
            (limits.header_fields, 1_000),
            (limits.header_field_size, 65_536),
            (limits.header_section_size, 1_048_576),
            (limits.comment_depth, 16),
            (limits.mailboxes, 64),
            (limits.address_size, 1_024),
            (limits.rcpt_to, 1_000),
            (limits.rt_entries, 1_000),
            (limits.dkim_signatures, 16),
            (limits.key_records, 8),
            (limits.rsa_key_bits, 4_096),
            (limits.rsa_exponent_bits, 32),
            (limits.author_domains, 4),
            (limits.dns_questions, 512),
            (limits.answer_size, 65_535),
            (limits.dkim2_signatures, 32),
            (limits.message_instances, 32),
            (limits.recipe_steps, 10_000),
            (limits.recipe_size, 65_536),
            (limits.recipe_depth, 8),
            (limits.spf_void_lookups, 2),
            (limits.trace_size, 1_048_576),
            (limits.trace_places, 16),
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
        limits.document_depth = 65;
        assert_eq!(limits.validate(), Err(out_of_range.clone()));
        assert_eq!(out_of_range.code(), "limits.out_of_range");
        let text = out_of_range.to_string();
        assert!(text.starts_with("limits.out_of_range"));
        assert!(text.contains("document_depth"));
        limits.document_depth = 64;
        assert_eq!(limits.validate(), Ok(()));
    }

    #[test]
    fn prs_006_document_limits_have_their_documented_defaults() {
        let documents = DocumentLimits::default();
        let defaults = [
            (documents.pasted_records_size, 1_048_576),
            (documents.document_size, 33_554_432),
            (documents.document_depth, 16),
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
