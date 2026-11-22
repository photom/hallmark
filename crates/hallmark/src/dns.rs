//! DNS questions and answers.

use crate::*;
use std::net::{Ipv4Addr, Ipv6Addr};

/// A DNS question that a check hands the verifying caller (DNS-001).
///
/// [`Check::take_requests`] gives it inside [`Request::Question`], and the
/// verifying caller answers it with [`Check::answer`]. A check holds each type
/// and name once (INV-001), so the verifying caller never gets the same
/// question twice. The check builds it, and the verifying caller reads its
/// fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Question {
    /// Unique within the check. It depends on when the question was asked, so
    /// the trace never shows it (DNS-007, TRC-008).
    pub id: QuestionId,
    /// The type of record asked for (DNS-001).
    pub record_type: QuestionType,
    /// The name in lookup form, built by the check, as ASCII without a final
    /// dot (DNS-008, ADS-002).
    pub name: Vec<u8>,
}

/// The ID of a DNS question or a public suffix question, unique within its
/// check (DNS-007, INV-002).
///
/// The two kinds of question share one ID space. A check numbers its requests
/// in the order they appear, so an ID depends on when a question was asked, and
/// the trace never shows it (TRC-008). The verifying caller gets IDs only from
/// requests: the type has no public field or constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuestionId(u64);

/// The type of record a question asks for (DNS-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestionType {
    /// TXT records.
    Txt,
    /// A records.
    A,
    /// AAAA records.
    Aaaa,
    /// MX records.
    Mx,
    /// PTR records.
    Ptr,
}

/// The answer to a DNS question, which the verifying caller builds (DNS-002).
///
/// [`Check::answer`] checks it when the verifying caller hands it back (VAL-120
/// to VAL-125). [`drive`] hands a resolver's answers to the check in the same
/// way (VAL-804), and [`Snapshot::parse`] checks the answers that a snapshot
/// holds (VAL-901 to VAL-908).
///
/// The resolver needs to know no protocol rule: the check joins TXT strings
/// itself (DNS-006), and a name in a record that doesn't parse gets the rule
/// of the protocol that read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// Records found, in the order the resolver gave them (DNS-002, DNS-006).
    Records(Vec<Record>),
    /// The name doesn't exist, NXDOMAIN (DNS-002).
    NameNotFound,
    /// The name exists with no records of this type, NODATA (DNS-002).
    NoRecords,
    /// The resolver couldn't get an answer, such as on a timeout or SERVFAIL.
    /// It carries the resolver's description, which the trace records as given
    /// (DNS-009).
    TemporaryFailure(Option<Vec<u8>>),
}

/// One record of an answer (DNS-006).
///
/// A name in a record is the bytes of its labels joined by dots, without a
/// final dot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Record {
    /// A TXT record: its character-strings, in order and not joined (RFC 1035
    /// §3.3.14).
    Txt(Vec<Vec<u8>>),
    /// An A record.
    A(Ipv4Addr),
    /// An AAAA record (RFC 3596 §2.2).
    Aaaa(Ipv6Addr),
    /// An MX record (RFC 1035 §3.3.9).
    Mx {
        /// The preference.
        preference: u16,
        /// The exchange name.
        exchange: Vec<u8>,
    },
    /// A PTR record: a name (RFC 1035 §3.3.12).
    Ptr(Vec<u8>),
}
