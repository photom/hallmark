//! The DKIM2 header hash input (draft-09 §6.2): every header field except Message-Instance,
//! DKIM2-Signature and the fields of §4.1, each in the `relaxed` form, in ascending order of
//! their lowercased names, and fields with the same name from the bottom up (HCN-005). A line
//! that isn't a header field stays out too (HCN-006).

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "verification and signing come later")
)]

use std::collections::BTreeMap;

use crate::canonical::header::{HeaderMode, write_field};
use crate::header::section::{HeaderLine, LineKind};

/// The fields that draft-09 §4.1 lists as not signed, lowercased, apart from the two prefixes.
const UNSIGNED: [&[u8]; 14] = [
    b"apparently-to",
    b"arc-authentication-results",
    b"arc-message-signature",
    b"arc-seal",
    b"authentication-results",
    b"delivered-to",
    b"dkim-signature",
    b"dl-expansion-history",
    b"original-recipient",
    b"received",
    b"return-path",
    b"sio-label-history",
    b"x400-received",
    b"x400-trace",
];

/// `Received-*` and `X-*` of draft-09 §4.1: every name that begins with one of these.
const UNSIGNED_PREFIXES: [&[u8]; 2] = [b"received-", b"x-"];

/// Why a line of the header section stays out of the DKIM2 header hash (HCN-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rule {
    /// A field that draft-09 §4.1 lists as not signed.
    Unsigned,
    /// A Message-Instance or DKIM2-Signature field (draft-09 §6.2).
    Dkim2Field,
    /// A line that isn't a header field (HCN-006, draft-09 §2.7).
    NotAField,
}

/// A line of the header section that the DKIM2 header hash leaves out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LeftOut {
    /// Its index among the lines of the header section.
    pub(crate) line: usize,
    /// The rule that leaves it out.
    pub(crate) rule: Rule,
}

/// Which lines a DKIM2 header hash input took and which it left out, for the trace (HCN-009).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Selection {
    /// The indices of the fields it took, in the order they enter the hash: by lowercased name,
    /// and fields of one name from the bottom up.
    pub(crate) taken: Vec<usize>,
    /// The lines it left out, in header order.
    pub(crate) left_out: Vec<LeftOut>,
}

/// Appends the DKIM2 header hash input of the header section `input` to `output`, and gives
/// which lines it took and left out. `lines` come from splitting `input`. Gives `None` and
/// appends nothing when a line lies outside `input`.
pub(crate) fn hash_input(
    input: &[u8],
    lines: &[HeaderLine],
    output: &mut Vec<u8>,
) -> Option<Selection> {
    let mut left_out = Vec::new();
    let fields = fields_by_name(input, lines, &mut left_out)?;
    // §6.2: ascending octets of the lowercased names, and fields of one name bottom up (§5.1).
    let mut taken = Vec::new();
    for indices in fields.values() {
        for index in indices.iter().rev() {
            taken.push(*index);
        }
    }
    // A failure cuts `output` back to this length, so it appends nothing.
    let start = output.len();
    for index in &taken {
        let written = match lines.get(*index) {
            Some(line) => write_field(input, line, HeaderMode::Relaxed, output),
            None => false,
        };
        if !written {
            output.truncate(start);
            return None;
        }
    }
    Some(Selection { taken, left_out })
}

/// The lines of each lowercased field name that the hash takes, in header order. Pushes the
/// lines it leaves out to `left_out`, in header order. `None` when a name lies outside `input`.
fn fields_by_name(
    input: &[u8],
    lines: &[HeaderLine],
    left_out: &mut Vec<LeftOut>,
) -> Option<BTreeMap<Vec<u8>, Vec<usize>>> {
    let mut fields: BTreeMap<Vec<u8>, Vec<usize>> = BTreeMap::new();
    for (index, line) in lines.iter().enumerate() {
        let LineKind::Field { name, .. } = &line.kind else {
            let rule = Rule::NotAField;
            left_out.push(LeftOut { line: index, rule });
            continue;
        };
        let key = input.get(name.clone())?.to_ascii_lowercase();
        if let Some(rule) = rule_for(&key) {
            left_out.push(LeftOut { line: index, rule });
            continue;
        }
        match fields.get_mut(&key) {
            Some(indices) => indices.push(index),
            None => {
                fields.insert(key, vec![index]);
            }
        }
    }
    Some(fields)
}

/// The rule that leaves out a field with the lowercased name `name`, or `None` when the hash
/// takes it. Because `name` is lowercased, it matches the lists without regard to case
/// (HCN-005).
fn rule_for(name: &[u8]) -> Option<Rule> {
    if name == b"message-instance" || name == b"dkim2-signature" {
        return Some(Rule::Dkim2Field);
    }
    if UNSIGNED.contains(&name) {
        return Some(Rule::Unsigned);
    }
    for prefix in UNSIGNED_PREFIXES {
        if name.starts_with(prefix) {
            return Some(Rule::Unsigned);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::Rule::{Dkim2Field, NotAField, Unsigned};
    use super::{LeftOut, Rule, hash_input};
    use crate::header::section::split;

    /// The header section of CAN-220.
    const CAN_220: &str = concat!(
        "DKIM2-Signature: i=1; m=1; d=void.test; s=sel:ed25519-sha256:CCCC\r\n",
        "Message-Instance: m=1; h=sha256:AAAA:BBBB\r\n",
        "Received: from mx.void.test\r\n",
        "Subject: Hi\r\nFrom: a@void.test\r\nTo: b@void.test\r\n",
    );

    /// The fields of CAN-221, each with its CRLF.
    const CAN_221: &str = concat!(
        "RECEIVED: x\r\nReceived-SPF: pass\r\nReceivedBy: k\r\nX-Mailer: y\r\nx-spam: z\r\n",
        "Return-Path: <a@void.test>\r\nAuthentication-Results: mx.void.test; none\r\n",
        "DKIM-Signature: v=1\r\nARC-Seal: i=1\r\nDelivered-To: b@void.test\r\n",
        "X400-Received: r\r\nX400-Trace: t\r\nX400-Other: k\r\n",
    );

    /// The DKIM2 header hash input of `header` as text, the fields it took, and the lines it
    /// left out.
    fn text(header: &str) -> (String, Vec<usize>, Vec<LeftOut>) {
        let lines = split(header.as_bytes()).lines;
        let mut output = Vec::new();
        let selection = hash_input(header.as_bytes(), &lines, &mut output).unwrap();
        let bytes = String::from_utf8(output).unwrap();
        (bytes, selection.taken, selection.left_out)
    }

    fn left_out(line: usize, rule: Rule) -> LeftOut {
        LeftOut { line, rule }
    }

    /// The SHA-256 and SHA-512 hashes of CAN-220 wait for the hash functions, and its trace
    /// for the trace of HCN-009. This test checks the input and the lines behind the trace.
    #[test]
    fn can_220_dkim2_fields_and_trace_fields_stay_out() {
        let (bytes, taken, left) = text(CAN_220);
        let input = "from:a@void.test\r\nsubject:Hi\r\nto:b@void.test\r\n";
        assert_eq!(bytes, input);
        assert_eq!(taken, [4, 3, 5]);
        let expected = [
            left_out(0, Dkim2Field),
            left_out(1, Dkim2Field),
            left_out(2, Unsigned),
        ];
        assert_eq!(left, expected);
    }

    #[test]
    fn can_221_the_list_of_section_4_1_matches_without_case() {
        let (bytes, taken, left) = text(CAN_221);
        assert_eq!(bytes, "receivedby:k\r\nx400-other:k\r\n");
        assert_eq!(taken, [2, 12]);
        let mut expected = Vec::new();
        for line in [0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11] {
            expected.push(left_out(line, Unsigned));
        }
        assert_eq!(left, expected);
    }

    #[test]
    fn can_222_fields_with_one_name_go_from_the_bottom_up() {
        let (bytes, taken, _) = text("Comments: 1\r\nCOMMENTS: 2\r\n");
        assert_eq!(bytes, "comments:2\r\ncomments:1\r\n");
        assert_eq!(taken, [1, 0]);
    }

    #[test]
    fn can_223_names_compare_octet_by_octet() {
        let header = concat!(
            "Date: d\r\nContent-Type: t\r\nCc: c\r\nContent: x\r\n",
            "Content-Transfer-Encoding: e\r\n",
        );
        let expected = concat!(
            "cc:c\r\ncontent:x\r\ncontent-transfer-encoding:e\r\n",
            "content-type:t\r\ndate:d\r\n",
        );
        assert_eq!(text(header).0, expected);
    }

    #[test]
    fn can_224_each_field_gets_the_relaxed_form() {
        let header = "SUBJect:  A \t B \r\nTo: a@void.test,\r\n\tb@void.test\r\n";
        let expected = "subject:A B\r\nto:a@void.test, b@void.test\r\n";
        assert_eq!(text(header).0, expected);
    }

    /// The SHA-256 hash of CAN-225 waits for the hash functions.
    #[test]
    fn can_225_only_fields_left_out_give_an_empty_input() {
        let (bytes, taken, left) = text("Received: x\r\nX-Spam: y\r\n");
        assert_eq!(bytes, "");
        assert!(taken.is_empty());
        assert_eq!(left, [left_out(0, Unsigned), left_out(1, Unsigned)]);
    }

    /// The caution of CAN-260 waits for the trace. This test checks the rule behind it.
    #[test]
    fn can_260_a_line_that_is_no_field_stays_out() {
        let header = "Subject: Hi\r\nnot a field\r\nFrom: a@void.test\r\n";
        let (bytes, taken, left) = text(header);
        assert_eq!(bytes, "from:a@void.test\r\nsubject:Hi\r\n");
        assert_eq!(taken, [2, 0]);
        assert_eq!(left, [left_out(1, NotAField)]);
    }
}
