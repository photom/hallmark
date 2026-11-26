//! The header field names of DKIM `h=` and the copied header fields of DKIM
//! `z=` (RFC 6376 §3.5, HDR-007).

#![cfg_attr(not(test), expect(dead_code, reason = "DKIM comes later"))]

use core::ops::Range;

use super::Grammar;
use super::qp::{QpError, token};

/// A header field copied into `z=`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CopiedField {
    /// The field name, a range of the `z=` value, which is never empty.
    pub(crate) name: Range<usize>,
    /// The field value, decoded.
    pub(crate) value: Vec<u8>,
}

/// Why an `h=` or `z=` value doesn't decode. Offsets count from the start of
/// the value as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldsError {
    /// hdr-name: no field name where one must start, at an octet outside
    /// ftext, a `|` in `z=`, or the end of the value.
    Name(usize),
    /// sig-h-tag: a name that neither a colon nor the end of the value
    /// follows.
    Separator(usize),
    /// sig-z-tag-copy: a copy whose name no colon follows.
    Colon(usize),
    /// The value of a copy, a qp-hdr-value, breaks hex-octet or
    /// dkim-quoted-printable.
    Value(QpError),
}

/// What a `|` is where a field name is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bar {
    /// ftext, as in `h=` (RFC 5322 §3.6.8).
    Ftext,
    /// A meta-character that ends the name, as in `z=` (RFC 6376 §3.5).
    Meta,
}

impl From<QpError> for FieldsError {
    fn from(error: QpError) -> Self {
        Self::Value(error)
    }
}

/// The names of an `h=` value, in order:
/// `hdr-name *( [FWS] ":" [FWS] hdr-name )`.
pub(crate) fn names(input: &[u8]) -> Result<Vec<Range<usize>>, FieldsError> {
    let mut names = Vec::new();
    let mut at = 0;
    loop {
        let name = field_name(input, at, Bar::Ftext)?;
        let end = name.end;
        names.push(name);
        if end == input.len() {
            return Ok(names);
        }
        at = fws(input, end);
        if input.get(at) != Some(&b':') {
            return Err(FieldsError::Separator(at));
        }
        at = fws(input, at + 1);
    }
}

/// The copies of a `z=` value, in order:
/// `sig-z-tag-copy *( "|" [FWS] sig-z-tag-copy )`, where
/// `sig-z-tag-copy = hdr-name [FWS] ":" qp-hdr-value`. RFC 6376 §3.5 makes
/// each `|` in `z=` a meta-character, so a `|` ends a copy, and no name
/// holds one, although ftext allows it.
pub(crate) fn copies(input: &[u8], grammar: Grammar) -> Result<Vec<CopiedField>, FieldsError> {
    let mut copies = Vec::new();
    let mut at = 0;
    loop {
        let name = field_name(input, at, Bar::Meta)?;
        let colon = fws(input, name.end);
        if input.get(colon) != Some(&b':') {
            return Err(FieldsError::Colon(colon));
        }
        let (value, end) = hdr_value(input, colon + 1, grammar)?;
        copies.push(CopiedField { name, value });
        if end == input.len() {
            return Ok(copies);
        }
        at = fws(input, end + 1);
    }
}

/// Reads the field-name at `at`: one or more ftext octets, which are
/// printable US-ASCII except the colon (RFC 5322 §3.6.8), up to a `|` that
/// `bar` makes a meta-character.
fn field_name(input: &[u8], at: usize, bar: Bar) -> Result<Range<usize>, FieldsError> {
    let mut end = at;
    loop {
        match input.get(end).copied() {
            Some(b'|') if bar == Bar::Meta => break,
            Some(0x21..=0x39 | 0x3B..=0x7E) => end += 1,
            _ => break,
        }
    }
    if end == at {
        return Err(FieldsError::Name(at));
    }
    Ok(at..end)
}

/// Reads the qp-hdr-value at `at` up to the `|` or the end that follows it.
/// Returns the decoded value and the offset of that `|` or end.
fn hdr_value(
    input: &[u8],
    mut at: usize,
    grammar: Grammar,
) -> Result<(Vec<u8>, usize), FieldsError> {
    let mut value = Vec::new();
    loop {
        match input.get(at) {
            None | Some(b'|') => return Ok((value, at)),
            Some(_) => at = token(input, at, grammar, &mut value)?,
        }
    }
}

/// Skips one optional FWS at `at` (RFC 6376 §2.8): WSP with at most one
/// CRLF, which WSP follows. Returns the offset after it.
fn fws(input: &[u8], mut at: usize) -> usize {
    let mut folded = false;
    loop {
        match input.get(at..) {
            Some([b' ' | b'\t', ..]) => at += 1,
            Some([b'\r', b'\n', b' ' | b'\t', ..]) if !folded => {
                folded = true;
                at += 3;
            }
            _ => return at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CopiedField, FieldsError, Grammar, QpError, copies, names};

    /// The name and the value of each copy of `input`, as bytes.
    fn pairs(input: &[u8], grammar: Grammar) -> Vec<(&[u8], Vec<u8>)> {
        let mut pairs = Vec::new();
        for CopiedField { name, value } in copies(input, grammar).unwrap() {
            pairs.push((&input[name], value));
        }
        pairs
    }

    #[test]
    fn tag_021_a_copy_has_a_name_and_a_decoded_value() {
        let found = pairs(b"Subject:demo=20run", Grammar::Dkim);
        let expected: [(&[u8], Vec<u8>); 1] = [(b"Subject", b"demo run".to_vec())];
        assert_eq!(found, expected);
    }

    #[test]
    fn tag_022_a_copy_takes_uppercase_hex_digits() {
        let error = FieldsError::Value(QpError::HexOctet(11));
        let input = b"Subject:a=3bb";
        assert_eq!(copies(input, Grammar::Dkim), Err(error));
    }

    #[test]
    fn tag_023_fws_around_a_colon_is_not_part_of_a_name() {
        let input = b"From : To:\r\n Subject";
        let mut found = Vec::new();
        for name in names(input).unwrap() {
            found.push(&input[name]);
        }
        let expected: [&[u8]; 3] = [b"From", b"To", b"Subject"];
        assert_eq!(found, expected);
    }

    #[test]
    fn tag_024_a_copy_may_hold_unencoded_utf8() {
        let input = "Subject:café".as_bytes();
        let found = pairs(input, Grammar::DkimUtf8);
        let expected: [(&[u8], Vec<u8>); 1] = [(b"Subject", "café".into())];
        assert_eq!(found, expected);
        let error = FieldsError::Value(QpError::Char(11));
        assert_eq!(copies(input, Grammar::Dkim), Err(error));
    }

    #[test]
    fn tag_025_a_copy_holds_well_formed_utf8_only() {
        let error = FieldsError::Value(QpError::Char(9));
        let input = b"Subject:a\xFF";
        assert_eq!(copies(input, Grammar::DkimUtf8), Err(error));
        assert_eq!(copies(input, Grammar::Dkim), Err(error));
    }

    #[test]
    fn tag_035_fws_after_a_bar_or_in_a_value_is_not_part_of_a_copy() {
        let input = b"From:a=40void.test|\r\n Subject:demo=20\r\n run";
        let found = pairs(input, Grammar::Dkim);
        let expected: [(&[u8], Vec<u8>); 2] = [
            (b"From", b"a@void.test".to_vec()),
            (b"Subject", b"demo run".to_vec()),
        ];
        assert_eq!(found, expected);
    }

    #[test]
    fn tag_036_a_copy_has_no_fws_inside_a_hex_octet() {
        let error = FieldsError::Value(QpError::HexOctet(11));
        let input = b"Subject:a=2\r\n 0";
        assert_eq!(copies(input, Grammar::Dkim), Err(error));
    }

    #[test]
    fn tag_037_a_name_follows_each_colon_of_h() {
        let cases: [(&[u8], FieldsError); 5] = [
            (b"From::To", FieldsError::Name(5)),
            (b"From:", FieldsError::Name(5)),
            (b":To", FieldsError::Name(0)),
            (b"From To", FieldsError::Separator(5)),
            (b"From\r\n \r\n :To", FieldsError::Separator(7)),
        ];
        for (input, error) in cases {
            assert_eq!(names(input), Err(error));
        }
    }

    #[test]
    fn tag_038_a_copy_has_a_colon_and_follows_each_bar() {
        let cases: [(&[u8], FieldsError); 5] = [
            (b"Subject", FieldsError::Colon(7)),
            (b"Subject:a|", FieldsError::Name(10)),
            (b"Subject demo", FieldsError::Colon(8)),
            (b"From:a||To:b", FieldsError::Name(7)),
            (b"Subject\r\n \r\n :a", FieldsError::Colon(10)),
        ];
        for (input, error) in cases {
            assert_eq!(copies(input, Grammar::Dkim), Err(error));
        }
    }
}
