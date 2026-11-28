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
use crate::tags::groups::signatures;

/// The DKIM2-Signature field whose signing input is built: a field of the header section when
/// a verifier checks it, or the field that a signer is creating (draft-09 §12.8).
#[derive(Debug, Clone, Copy)]
pub(crate) struct OwnField<'a> {
    /// Its raw bytes, from the field name to the CRLF that ends it, if one does.
    pub(crate) raw: &'a [u8],
    /// Its `s=` tag, as ranges into the field value, from the tag-list parser (HDR-006).
    pub(crate) s: &'a Tag,
}

/// Appends the signing input to `output` and gives `true`. `instances` are the lines of the
/// Message-Instance fields with `m=` up to that of the own field, in ascending `m=` order, and
/// `signatures` the lines of the DKIM2-Signature fields with `i=` below that of the own field,
/// in ascending `i=` order (§14.6). `lines` come from splitting `input`. Gives `false` and
/// appends nothing when a line isn't a header field or lies outside `input`, or when the `s=`
/// tag doesn't fit the own field or doesn't decode.
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
    let written = match emptied_field(own) {
        Some(emptied) => write_own(&emptied, output),
        None => false,
    };
    if !written {
        output.truncate(start);
    }
    written
}

/// Appends the form of the signing input of the field `raw`, which ends in CRLF (§12.8).
fn write_own(raw: &[u8], output: &mut Vec<u8>) -> bool {
    match split(raw).lines.first() {
        Some(line) => write_field(raw, line, HeaderMode::Dkim2Signing, output),
        None => false,
    }
}

/// The raw bytes of the own field with each message-sig of its `s=` tag deleted. The
/// selectors, the algorithm names and the `:`, `,` and `|` between them stay (§12.8, §14.6).
/// The WSP around a deleted value goes with the other WSP when the field is put in form.
fn emptied_field(own: &OwnField<'_>) -> Option<Vec<u8>> {
    let lines = split(own.raw).lines;
    let LineKind::Field { value, .. } = &lines.first()?.kind else {
        return None;
    };
    let field_value = own.raw.get(value.clone())?;
    // The ranges come from parsing this value, folds included, so the name reads `s`.
    if field_value.get(own.s.name.clone())? != b"s" {
        return None;
    }
    let groups = signatures(field_value.get(own.s.value.clone())?).ok()?;
    let offset = value.start + own.s.value.start;
    let mut emptied = Vec::with_capacity(own.raw.len());
    let mut kept = 0;
    for group in &groups {
        for set in group {
            emptied.extend_from_slice(own.raw.get(kept..offset + set.written.start)?);
            kept = offset + set.written.end;
        }
    }
    emptied.extend_from_slice(own.raw.get(kept..)?);
    Some(emptied)
}

#[cfg(test)]
mod tests {
    use super::{OwnField, signing_input};
    use crate::header::section::{LineKind, split};
    use crate::tags::{Grammar, TagList};

    /// The header section of the section on the DKIM2 signing input, top to bottom.
    const SECTION: &str = concat!(
        "DKIM2-Signature: i=2; m=2; d=void.test;\r\n s=sel2:ed25519-sha256:DDDD\r\n",
        "Message-Instance: m=2; h=sha256:EE\r\n EE:FFFF\r\n",
        "DKIM2-Signature: i=1; m=1; d=void.test; s=sel1:ed25519-sha256:CCCC\r\n",
        "Message-Instance: m=1; h=sha256:AAAA:BBBB\r\n",
        "From: a@void.test\r\n",
    );

    /// The signing input of `own`, the raw bytes of a DKIM2-Signature field, after the given
    /// lines of `section`, as text.
    fn text(section: &str, instances: &[usize], signatures: &[usize], own: &[u8]) -> String {
        let own_lines = split(own).lines;
        let LineKind::Field { value, .. } = &own_lines[0].kind else {
            panic!("the own field is malformed");
        };
        let field = &own[value.clone()];
        let list = TagList::parse(field, Grammar::Dkim2).unwrap();
        let mut s = None;
        for tag in list.tags() {
            if let b"s" = &field[tag.name.clone()] {
                s = Some(tag);
            }
        }
        let s = s.unwrap();
        let own = OwnField { raw: own, s };
        let input = section.as_bytes();
        let lines = split(input).lines;
        let mut output = Vec::new();
        let built = signing_input(input, &lines, instances, signatures, &own, &mut output);
        assert!(built);
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
        assert_eq!(text(SECTION, &[3], &[], line(SECTION, 2)), expected);
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
        assert_eq!(text(SECTION, &[3, 1], &[2], line(SECTION, 0)), expected);
    }

    /// Draft-09 §10.9 and §14.6.
    #[test]
    fn can_242_selectors_algorithms_and_punctuation_stay() {
        let own = concat!(
            "DKIM2-Signature: i=1; m=1; d=void.test;\r\n",
            " s=a:ed25519-sha256:AAAA, b:mldsa44:BBBB |\r\n c:rsa-sha256:CCCC\r\n",
        );
        let input = text(SECTION, &[3], &[], own.as_bytes());
        let end = "s=a:ed25519-sha256:,b:mldsa44:|c:rsa-sha256:\r\n";
        assert!(input.ends_with(end), "{input}");
    }
}
