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
    /// A line that isn't a header field, with the rule it breaks and the
    /// offset of the octet where it breaks it (HDR-004, PAR-001).
    Malformed(Malformed, usize),
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
        let LineKind::Field { name: range, .. } = &self.kind else {
            return false;
        };
        match input.get(range.clone()) {
            Some(bytes) => bytes.eq_ignore_ascii_case(name.as_bytes()),
            None => false,
        }
    }

    /// The value of a field, unfolded: without the CRLF of each fold
    /// (RFC 5322 §2.2.3, HDR-003). `None` for a malformed line.
    pub(crate) fn unfolded_value(&self, input: &[u8]) -> Option<Vec<u8>> {
        let LineKind::Field { value, .. } = &self.kind else {
            return None;
        };
        // A field goes on only over lines that start with WSP, so every CRLF
        // inside its value is a fold.
        let mut rest = input.get(value.clone())?;
        let mut unfolded = Vec::with_capacity(rest.len());
        while let Some((&byte, after)) = rest.split_first() {
            if byte == b'\r'
                && let Some(folded) = after.strip_prefix(b"\n")
            {
                rest = folded;
                continue;
            }
            unfolded.push(byte);
            rest = after;
        }
        Some(unfolded)
    }
}

/// Splits `input` into header fields and malformed lines, up to the first
/// empty line or the end of the input (HDR-001). A line ends at CRLF and
/// nowhere else, so a bare CR or LF is an octet of its line (HDR-012).
pub(crate) fn split(input: &[u8]) -> HeaderSection {
    let mut section = HeaderSection {
        lines: Vec::new(),
        body: None,
        bare_cr_lf: Vec::new(),
    };
    let mut start = 0;
    let mut rest = input;
    while let Some(&first) = rest.first() {
        if rest.starts_with(b"\r\n") {
            section.body = Some(start + 2);
            break;
        }
        let length = line_length(rest, start, &mut section.bare_cr_lf);
        let Some((bytes, after)) = rest.split_at_checked(length) else {
            break;
        };
        let (content, line_ending) = match bytes.strip_suffix(b"\r\n") {
            Some(content) => (content, true),
            None => (bytes, false),
        };
        let end = start + bytes.len();
        let content_end = start + content.len();
        let wsp = matches!(first, b' ' | b'\t');
        let folded = wsp && fold(&mut section.lines, content_end, end, line_ending);
        if !folded {
            let kind = classify(content, start);
            let line = HeaderLine {
                kind,
                raw: start..end,
                line_ending,
            };
            section.lines.push(line);
        }
        start = end;
        rest = after;
    }
    section
}

/// The length of the line that starts `bytes`, its CRLF included, or of all
/// of `bytes` when no CRLF ends the line. Adds the offset of each CR and LF
/// that no CRLF holds to `bare`, the line starting at `start` (HDR-012).
fn line_length(bytes: &[u8], start: usize, bare: &mut Vec<usize>) -> usize {
    let mut pending_cr = None;
    for (index, byte) in bytes.iter().enumerate() {
        let at = start + index;
        if *byte == b'\n' {
            if pending_cr.is_some() {
                return index + 1;
            }
            bare.push(at);
            continue;
        }
        if let Some(cr) = pending_cr {
            bare.push(cr);
        }
        pending_cr = if *byte == b'\r' { Some(at) } else { None };
    }
    if let Some(cr) = pending_cr {
        bare.push(cr);
    }
    bytes.len()
}

/// Adds a line that starts with WSP to the field before it, and gives whether
/// a field was there to fold (RFC 5322 §2.2.3). The line's content ends at
/// `content_end`, and the line, its CRLF included, at `end`.
fn fold(lines: &mut [HeaderLine], content_end: usize, end: usize, line_ending: bool) -> bool {
    let Some(last) = lines.last_mut() else {
        return false;
    };
    let LineKind::Field { value, .. } = &mut last.kind else {
        return false;
    };
    value.end = content_end;
    last.raw.end = end;
    last.line_ending = line_ending;
    true
}

/// What a line that folds no field is: a header field, or a malformed line
/// with the rule it breaks (RFC 5322 §2.2, §3.6.8, §4.5). `content` is the
/// line without its CRLF, and starts at `start`.
fn classify(content: &[u8], start: usize) -> LineKind {
    if let Some(b' ' | b'\t') = content.first() {
        return LineKind::Malformed(Malformed::NoField, start);
    }
    let mut name = 0;
    let mut before_colon = false;
    for (index, byte) in content.iter().enumerate() {
        match *byte {
            b':' if name > 0 => {
                let value = start + index + 1..start + content.len();
                return LineKind::Field {
                    name: start..start + name,
                    value,
                };
            }
            33..=57 | 59..=126 if !before_colon => name += 1,
            b' ' | b'\t' if name > 0 => before_colon = true,
            _ if before_colon => return LineKind::Malformed(Malformed::Colon, start + index),
            _ => return LineKind::Malformed(Malformed::FieldName, start + index),
        }
    }
    LineKind::Malformed(Malformed::Colon, start + content.len())
}

#[cfg(test)]
mod tests {
    use super::Malformed::{Colon, FieldName, NoField};
    use super::{HeaderLine, LineKind, Malformed, split};

    /// The raw bytes of `line`.
    fn raw<'a>(input: &'a [u8], line: &HeaderLine) -> &'a [u8] {
        &input[line.raw.clone()]
    }

    /// The name of the field `line`.
    fn name<'a>(input: &'a [u8], line: &HeaderLine) -> &'a [u8] {
        match &line.kind {
            LineKind::Field { name, .. } => &input[name.clone()],
            LineKind::Malformed(rule, at) => panic!("a malformed line: {rule:?} at {at}"),
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
    fn fld_005_and_006_a_header_section_can_end_with_the_input() {
        let section = split(b"A: 1\r\nB: 2\r\n");
        assert_eq!(section.lines.len(), 2);
        assert!(section.lines[1].line_ending);
        assert_eq!(section.body, None);
        // FLD-006: the last line ends with no CRLF.
        let input = b"A: 1\r\nB: 2";
        let section = split(input);
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
    fn fld_041_to_043_053_054_lines_that_hold_no_field_are_malformed() {
        // The input, the malformed line, its rule, the offset of the octet that
        // breaks it, and the range of its raw bytes. A malformed line folds into
        // nothing (FLD-042).
        let cases: [(&[u8], usize, Malformed, usize, core::ops::Range<usize>); 5] = [
            (b"Subj\xc3\xa9ct: x\r\n\r\n", 0, FieldName, 4, 0..13),
            (b"A: 1\r\nnot a field\r\nB: 2\r\n\r\n", 1, Colon, 10, 6..19),
            (b" A: 1\r\nB: 2\r\n\r\n", 0, NoField, 0, 0..7),
            (b"A: 1\r\n\nB: 2\r\n\r\n", 1, FieldName, 6, 6..13),
            (b"Fr\rom: a@void.test\r\n\r\n", 0, FieldName, 2, 0..20),
        ];
        for (input, index, rule, at, expected) in cases {
            let section = split(input);
            let line = &section.lines[index];
            assert_eq!(line.kind, LineKind::Malformed(rule, at));
            assert_eq!(line.raw, expected);
        }
    }

    #[test]
    fn fld_052_to_054_a_bare_cr_or_lf_is_an_octet_of_its_line() {
        let input = b"A: 1\nB: 2\r\n\r\n";
        let section = split(input);
        assert_eq!(section.lines.len(), 1);
        assert_eq!(value(input, &section.lines[0]), b" 1\nB: 2");
        let cases: [(&[u8], usize); 3] = [
            (input, 4),
            (b"A: 1\r\n\nB: 2\r\n\r\n", 6),
            (b"Fr\rom: a@void.test\r\n\r\n", 2),
        ];
        for (input, offset) in cases {
            assert_eq!(split(input).bare_cr_lf, [offset]);
        }
    }

    #[test]
    fn fld_080_the_raw_lines_join_to_the_header_section() {
        // A xorshift generator gives the same inputs on every run. Odd rounds
        // draw from every octet, even rounds from the octets that change how a
        // line splits.
        let alphabet = b"A: \t\r\n\xc3x";
        let mut state: u64 = 0x2545_f491_4f6c_dd1d;
        for round in 0..2_000_usize {
            let long = round % 200 == 0;
            let length = if long { 65_536 } else { round % 300 };
            let every_octet = round % 2 == 1;
            let mut input = Vec::with_capacity(length);
            for _ in 0..length {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let octet = state.to_le_bytes()[0];
                let drawn = alphabet[usize::from(octet % 8)];
                input.push(if every_octet { octet } else { drawn });
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
