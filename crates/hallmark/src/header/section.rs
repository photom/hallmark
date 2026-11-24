//! The header section splitter: the lines of a raw header section, each a
//! header field or a malformed line, as byte ranges into the raw bytes
//! (HDR-001, PAR-002).

// The check and the signing, which split their header sections here, come in
// later changes, so only the tests use this module for now.
#![cfg_attr(
    not(test),
    expect(dead_code, reason = "the check and the signing come later")
)]

use core::ops::Range;

/// A raw header section, split into lines (HDR-001).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeaderSection {
    /// The header fields and malformed lines, in message order.
    pub(crate) lines: Vec<HeaderLine>,
    /// The offset of the body, after the empty line that ends the header
    /// section. `None` when the input holds no empty line.
    pub(crate) body: Option<usize>,
    /// The offsets of the CR and LF octets that no CRLF holds (HDR-012).
    pub(crate) bare_cr_lf: Vec<usize>,
}

/// One line of a header section: a header field with its folded lines, or a
/// malformed line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeaderLine {
    /// What the line is.
    pub(crate) kind: LineKind,
    /// The raw bytes, from the first octet to the CRLF that ends the line,
    /// folds included (HDR-001).
    pub(crate) raw: Range<usize>,
    /// Whether a CRLF ends the line. Only a last line can end without one
    /// (HCN-008).
    pub(crate) line_ending: bool,
}

/// What a line of the header section is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LineKind {
    /// A header field: its name, and its value from the octet after the colon
    /// to the CRLF that ends the field (RFC 5322 §2.2).
    Field {
        name: Range<usize>,
        value: Range<usize>,
    },
    /// A line that isn't a header field, with the rule it breaks (HDR-004).
    Malformed(Malformed),
}

/// The rule that a malformed line breaks (HDR-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Malformed {
    /// The line starts with no field name: an octet outside `ftext`, which is
    /// printable US-ASCII except the colon (RFC 5322 §3.6.8).
    FieldName,
    /// A field name with no colon after it (RFC 5322 §3.6.8, §4.5).
    Colon,
    /// A line that starts with WSP when no field is open to fold
    /// (RFC 5322 §2.2.3).
    NoField,
}

impl HeaderLine {
    /// Whether the line is a field named `name`, compared without regard to
    /// ASCII case (HDR-002).
    pub(crate) fn has_name(&self, input: &[u8], name: &str) -> bool {
        unimplemented!()
    }

    /// The value of a field, unfolded: without the CRLF of each fold
    /// (RFC 5322 §2.2.3, HDR-003). `None` for a malformed line.
    pub(crate) fn unfolded_value(&self, input: &[u8]) -> Option<Vec<u8>> {
        unimplemented!()
    }
}

/// Splits `input` into header fields and malformed lines, up to the first
/// empty line or the end of the input (HDR-001). A line ends at CRLF and
/// nowhere else, so a bare CR or LF is an octet of its line (HDR-012).
pub(crate) fn split(input: &[u8]) -> HeaderSection {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::{HeaderLine, LineKind, Malformed, split};

    /// The raw bytes of `line`.
    fn raw<'a>(input: &'a [u8], line: &HeaderLine) -> &'a [u8] {
        &input[line.raw.clone()]
    }

    /// The name of the field `line`.
    fn name<'a>(input: &'a [u8], line: &HeaderLine) -> &'a [u8] {
        match &line.kind {
            LineKind::Field { name, .. } => &input[name.clone()],
            LineKind::Malformed(rule) => panic!("a malformed line: {rule:?}"),
        }
    }

    /// The unfolded value of the field `line`.
    fn value(input: &[u8], line: &HeaderLine) -> Vec<u8> {
        line.unfolded_value(input).unwrap()
    }

    #[test]
    fn fld_001_a_folded_field_unfolds() {
        let input = b"Subject: This\r\n is a test\r\n\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 1);
        let line = &section.lines[0];
        assert_eq!(name(input, line), b"Subject");
        assert_eq!(value(input, line), b" This is a test");
        assert_eq!(raw(input, line), b"Subject: This\r\n is a test\r\n");
        assert_eq!(section.body, Some(input.len()));
    }

    #[test]
    fn fld_002_the_first_empty_line_ends_the_header_section() {
        let input = b"A: 1\r\n\r\nB: 2\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 1);
        assert_eq!(name(input, &section.lines[0]), b"A");
        assert_eq!(section.body, Some(8));
    }

    #[test]
    fn fld_003_names_compare_without_regard_to_case() {
        let input = b"SUBJect: AbC\r\n\r\n";
        let line = &split(input).lines[0];
        assert!(line.has_name(input, "subject"));
        assert_eq!(name(input, line), b"SUBJect");
    }

    #[test]
    fn fld_004_fields_keep_their_order() {
        let input = b"Received: <A>\r\nReceived: <B>\r\nReceived: <C>\r\n\r\n";
        let section = split(input);
        let expected: [&[u8]; 3] = [b" <A>", b" <B>", b" <C>"];
        assert_eq!(section.lines.len(), 3);
        for (line, expected) in section.lines.iter().zip(expected) {
            assert!(line.has_name(input, "received"));
            assert_eq!(value(input, line), expected);
        }
    }

    #[test]
    fn fld_005_a_header_section_can_end_with_the_input() {
        let input = b"A: 1\r\nB: 2\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 2);
        assert_eq!(name(input, &section.lines[1]), b"B");
        assert!(section.lines[1].line_ending);
        assert_eq!(section.body, None);
    }

    #[test]
    fn fld_006_a_last_line_can_end_without_crlf() {
        let input = b"A: 1\r\nB: 2";
        let section = split(input);
        assert_eq!(section.lines.len(), 2);
        assert_eq!(raw(input, &section.lines[1]), b"B: 2");
        assert!(section.lines[0].line_ending);
        assert!(!section.lines[1].line_ending);
    }

    #[test]
    fn fld_020_white_space_may_come_before_the_colon() {
        let input = b"From  : a@void.test\r\n\r\n";
        let line = &split(input).lines[0];
        assert_eq!(name(input, line), b"From");
        assert_eq!(value(input, line), b" a@void.test");
        assert_eq!(raw(input, line), b"From  : a@void.test\r\n");
    }

    #[test]
    fn fld_021_a_line_of_white_space_folds_too() {
        let input = b"To: a@void.test,\r\n  \r\n b@void.test\r\n\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 1);
        let expected = b" a@void.test,   b@void.test";
        assert_eq!(value(input, &section.lines[0]), expected);
    }

    #[test]
    fn fld_040_a_value_can_hold_utf_8() {
        let input = b"Subject: caf\xc3\xa9\r\n\r\n";
        let line = &split(input).lines[0];
        assert_eq!(value(input, line), b" caf\xc3\xa9");
    }

    #[test]
    fn fld_041_a_name_is_printable_us_ascii() {
        let input = b"Subj\xc3\xa9ct: x\r\n\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 1);
        let line = &section.lines[0];
        assert_eq!(line.kind, LineKind::Malformed(Malformed::FieldName));
        assert_eq!(raw(input, line), b"Subj\xc3\xa9ct: x\r\n");
    }

    #[test]
    fn fld_042_a_line_with_no_colon_is_malformed_and_folds_nothing() {
        let input = b"A: 1\r\nnot a field\r\nB: 2\r\n\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 3);
        assert_eq!(raw(input, &section.lines[0]), b"A: 1\r\n");
        assert_eq!(section.lines[1].kind, LineKind::Malformed(Malformed::Colon));
        assert_eq!(raw(input, &section.lines[1]), b"not a field\r\n");
        assert_eq!(name(input, &section.lines[2]), b"B");
    }

    #[test]
    fn fld_043_white_space_before_any_field_folds_nothing() {
        let input = b" A: 1\r\nB: 2\r\n\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 2);
        let first = &section.lines[0];
        assert_eq!(first.kind, LineKind::Malformed(Malformed::NoField));
        assert_eq!(raw(input, first), b" A: 1\r\n");
        assert_eq!(name(input, &section.lines[1]), b"B");
    }

    #[test]
    fn fld_052_a_bare_lf_in_a_value_is_an_octet() {
        let input = b"A: 1\nB: 2\r\n\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 1);
        assert_eq!(value(input, &section.lines[0]), b" 1\nB: 2");
        assert_eq!(section.bare_cr_lf, vec![4]);
    }

    #[test]
    fn fld_053_a_line_that_starts_with_a_bare_lf_is_malformed() {
        let input = b"A: 1\r\n\nB: 2\r\n\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 2);
        let second = &section.lines[1];
        assert_eq!(second.kind, LineKind::Malformed(Malformed::FieldName));
        assert_eq!(raw(input, second), b"\nB: 2\r\n");
        assert_eq!(section.bare_cr_lf, vec![6]);
    }

    #[test]
    fn fld_054_a_bare_cr_in_a_name_makes_the_line_malformed() {
        let input = b"Fr\rom: a@void.test\r\n\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 1);
        let line = &section.lines[0];
        assert_eq!(line.kind, LineKind::Malformed(Malformed::FieldName));
        assert!(!line.has_name(input, "from"));
        assert_eq!(section.bare_cr_lf, vec![2]);
    }

    #[test]
    fn fld_080_the_raw_lines_join_to_the_header_section() {
        // A xorshift generator gives the same inputs on every run, drawn from
        // the octets that change how a line splits.
        let alphabet = b"A: \t\r\n\xc3x";
        let mut state: u64 = 0x2545_f491_4f6c_dd1d;
        for round in 0..2_000_usize {
            let length = if round % 200 == 0 { 65_536 } else { round % 300 };
            let mut input = Vec::with_capacity(length);
            for _ in 0..length {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                input.push(alphabet[(state % 8) as usize]);
            }
            let section = split(&input);
            let end = match section.body {
                Some(body) => body - 2,
                None => input.len(),
            };
            let mut joined = Vec::with_capacity(end);
            for line in &section.lines {
                joined.extend_from_slice(raw(&input, line));
            }
            assert_eq!(joined, &input[..end]);
        }
    }
}
