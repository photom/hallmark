//! The DKIM2 signing input (draft-09 §12.8 and §14.6): the Message-Instance fields, then the
//! earlier DKIM2-Signature fields, then the signature's own field with each signature value in
//! its `s=` tag emptied, each field with its name lowercased, its folds removed and all its WSP
//! deleted (HCN-007).

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "verification and signing come later")
)]

use crate::canonical::header::{HeaderMode, write_field};
use crate::header::section::{HeaderLine, LineKind, split};
use crate::tags::Tag;
use crate::tags::groups;

/// The DKIM2-Signature field whose signing input is built. Its raw bytes run from the field
/// name to the CRLF that ends it, if one does; the form adds a missing CRLF (HCN-008).
#[derive(Debug, Clone, Copy)]
pub(crate) enum OwnField<'a> {
    /// A field of the header section that a verifier checks, whose signature values the input
    /// empties (§14.6), with its `s=` tag as ranges into the field value, from the tag-list
    /// parser (HDR-006). A field whose `s=` doesn't decode is invalid (§14.2) and has no input.
    Checked { raw: &'a [u8], s: &'a Tag },
    /// The field that a signer is creating, whose signature values are still null (§12.8). A
    /// null value is no base64string (§10.9), so the input takes the field as it stands.
    Created(&'a [u8]),
}

/// Appends the signing input to `output` and gives `true`. `instances` are the lines of the
/// Message-Instance fields with `m=` up to that of the own field, in ascending `m=` order, and
/// `signatures` the lines of the DKIM2-Signature fields with `i=` below that of the own field,
/// in ascending `i=` order (§14.6). The caller picks these lines, so the step checks only that
/// each is a header field. `lines` come from splitting `input`. Gives `false` and appends
/// nothing when a line isn't a header field or lies outside `input`, or when the `s=` tag of a
/// checked field doesn't fit it or doesn't decode.
pub(crate) fn signing_input(
    input: &[u8],
    lines: &[HeaderLine],
    instances: &[usize],
    signatures: &[usize],
    own: &OwnField<'_>,
    output: &mut Vec<u8>,
) -> bool {
    // A failure cuts `output` back to this length, so it appends nothing.
    let start = output.len();
    for index in instances.iter().chain(signatures) {
        let written = match lines.get(*index) {
            Some(line) => write_field(input, line, HeaderMode::Dkim2Signing, output),
            None => false,
        };
        if !written {
            output.truncate(start);
            return false;
        }
    }
    let written = match *own {
        OwnField::Checked { raw, s } => match emptied_field(raw, s) {
            Some(emptied) => write_own(&emptied, output),
            None => false,
        },
        OwnField::Created(raw) => write_own(raw, output),
    };
    if !written {
        output.truncate(start);
    }
    written
}

/// Appends the field `raw` in the form of the signing input, which ends in CRLF (§12.8).
fn write_own(raw: &[u8], output: &mut Vec<u8>) -> bool {
    match split(raw).lines.first() {
        Some(line) => write_field(raw, line, HeaderMode::Dkim2Signing, output),
        None => false,
    }
}

/// The raw bytes of the checked field `raw` with each message-sig of its `s=` tag deleted. The
/// selectors, the algorithm names and the `:`, `,` and `|` between them stay (§12.8, §14.6).
/// The WSP around a deleted value goes with the other WSP when the field is put in form.
fn emptied_field(raw: &[u8], s: &Tag) -> Option<Vec<u8>> {
    let lines = split(raw).lines;
    let LineKind::Field { value, .. } = &lines.first()?.kind else {
        return None;
    };
    let field_value = raw.get(value.clone())?;
    // The ranges come from parsing this value, folds included, so the name reads `s`.
    if field_value.get(s.name.clone())? != b"s" {
        return None;
    }
    let sets = groups::signatures(field_value.get(s.value.clone())?).ok()?;
    let offset = value.start + s.value.start;
    let mut emptied = Vec::with_capacity(raw.len());
    let mut kept = 0;
    for group in &sets {
        for set in group {
            emptied.extend_from_slice(raw.get(kept..offset + set.written.start)?);
            kept = offset + set.written.end;
        }
    }
    emptied.extend_from_slice(raw.get(kept..)?);
    Some(emptied)
}

#[cfg(test)]
mod tests {
    use super::{OwnField, signing_input};
    use crate::header::section::{LineKind, split};
    use crate::tags::{Grammar, Tag, TagList};

    /// The header section of the section on the DKIM2 signing input, top to bottom.
    const SECTION: &str = concat!(
        "DKIM2-Signature: i=2; m=2; d=void.test;\r\n s=sel2:ed25519-sha256:DDDD\r\n",
        "Message-Instance: m=2; h=sha256:EE\r\n EE:FFFF\r\n",
        "DKIM2-Signature: i=1; m=1; d=void.test; s=sel1:ed25519-sha256:CCCC\r\n",
        "Message-Instance: m=1; h=sha256:AAAA:BBBB\r\n",
        "From: a@void.test\r\n",
    );

    /// The `s=` tag of the DKIM2-Signature field `own`, as ranges into its field value.
    fn s_tag(own: &[u8]) -> Tag {
        let own_lines = split(own).lines;
        let LineKind::Field { value, .. } = &own_lines[0].kind else {
            panic!("the own field is malformed");
        };
        let field = &own[value.clone()];
        let list = TagList::parse(field, Grammar::Dkim2).unwrap();
        for tag in list.tags() {
            if let b"s" = &field[tag.name.clone()] {
                return tag.clone();
            }
        }
        panic!("the own field has no s= tag");
    }

    /// Builds the signing input of `own` after the `picked` lines of [`SECTION`], the
    /// Message-Instance lines and then the DKIM2-Signature lines, into `output`.
    fn build(picked: [&[usize]; 2], own: &OwnField<'_>, output: &mut Vec<u8>) -> bool {
        let input = SECTION.as_bytes();
        let lines = split(input).lines;
        let [instances, signatures] = picked;
        signing_input(input, &lines, instances, signatures, own, output)
    }

    /// The signing input of the checked field `own` after the given lines of [`SECTION`], as
    /// text.
    fn text(instances: &[usize], signatures: &[usize], own: &[u8]) -> String {
        let s = s_tag(own);
        let own = OwnField::Checked { raw: own, s: &s };
        let mut output = Vec::new();
        assert!(build([instances, signatures], &own, &mut output));
        String::from_utf8(output).unwrap()
    }

    /// The raw bytes of line `index` of `section`.
    fn line(section: &str, index: usize) -> &[u8] {
        let lines = split(section.as_bytes()).lines;
        &section.as_bytes()[lines[index].raw.clone()]
    }

    /// Draft-09 §12.8 and §14.6.
    #[test]
    fn can_240_the_input_of_i_1_leaves_out_later_fields() {
        let expected = concat!(
            "message-instance:m=1;h=sha256:AAAA:BBBB\r\n",
            "dkim2-signature:i=1;m=1;d=void.test;s=sel1:ed25519-sha256:\r\n",
        );
        assert_eq!(text(&[3], &[], line(SECTION, 2)), expected);
    }

    /// Draft-09 §12.8 and §14.6.
    #[test]
    fn can_241_the_input_of_i_2_orders_the_fields_and_deletes_the_wsp() {
        let expected = concat!(
            "message-instance:m=1;h=sha256:AAAA:BBBB\r\n",
            "message-instance:m=2;h=sha256:EEEE:FFFF\r\n",
            "dkim2-signature:i=1;m=1;d=void.test;s=sel1:ed25519-sha256:CCCC\r\n",
            "dkim2-signature:i=2;m=2;d=void.test;s=sel2:ed25519-sha256:\r\n",
        );
        assert_eq!(text(&[3, 1], &[2], line(SECTION, 0)), expected);
    }

    /// Draft-09 §10.9 and §14.6.
    #[test]
    fn can_242_selectors_algorithms_and_punctuation_stay() {
        let own = concat!(
            "DKIM2-Signature: i=1; m=1; d=void.test;\r\n",
            " s=a:ed25519-sha256:AAAA, b:mldsa44:BBBB |\r\n c:rsa-sha256:CCCC\r\n",
        );
        let input = text(&[3], &[], own.as_bytes());
        let end = "s=a:ed25519-sha256:,b:mldsa44:|c:rsa-sha256:\r\n";
        assert!(input.ends_with(end), "{input}");
    }

    /// Draft-09 §12.8 and §10.9, with HCN-008.
    #[test]
    fn can_246_a_created_field_goes_in_as_it_stands() {
        let created = "DKIM2-Signature: i=3; m=2; d=void.test;\r\n s=sel3:ed25519-sha256:";
        let own = OwnField::Created(created.as_bytes());
        let mut output = Vec::new();
        assert!(build([&[3, 1], &[2, 0]], &own, &mut output));
        let expected = concat!(
            "message-instance:m=1;h=sha256:AAAA:BBBB\r\n",
            "message-instance:m=2;h=sha256:EEEE:FFFF\r\n",
            "dkim2-signature:i=1;m=1;d=void.test;s=sel1:ed25519-sha256:CCCC\r\n",
            "dkim2-signature:i=2;m=2;d=void.test;s=sel2:ed25519-sha256:DDDD\r\n",
            "dkim2-signature:i=3;m=2;d=void.test;s=sel3:ed25519-sha256:\r\n",
        );
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }

    /// Draft-09 §12.8 and ADR 0004. A line past the end gives `false`, and the output keeps
    /// what it held.
    #[test]
    fn can_247_a_line_past_the_end_gives_nothing() {
        let raw = line(SECTION, 2);
        let s = s_tag(raw);
        let own = OwnField::Checked { raw, s: &s };
        let mut output = b"kept".to_vec();
        assert!(!build([&[3, 9], &[]], &own, &mut output));
        assert_eq!(output, b"kept");
    }

    /// Draft-09 §10.9 and §14.2, and ADR 0004. A checked field whose `s=` doesn't decode gives
    /// `false`, and the output keeps what it held.
    #[test]
    fn can_248_an_s_value_that_does_not_decode_gives_nothing() {
        let raw = b"DKIM2-Signature: i=1; m=1; d=void.test; s=sel1:ed25519-sha256:C!CC\r\n";
        let s = s_tag(raw);
        let own = OwnField::Checked { raw, s: &s };
        let mut output = b"kept".to_vec();
        assert!(!build([&[3], &[]], &own, &mut output));
        assert_eq!(output, b"kept");
    }
}
