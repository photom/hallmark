//! The DKIM header hash input (RFC 6376 §3.7, as erratum 5252 corrects it): the fields that
//! the names in `h=` select, from the bottom of the header section up (§5.4), then the
//! DKIM-Signature field itself with its `b=` value deleted and no CRLF (HCN-003, HCN-004).

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "verification and signing come later")
)]

use crate::canonical::header::HeaderMode;
use crate::header::section::HeaderLine;
use crate::tags::Tag;

/// The DKIM-Signature field whose header hash input is built.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Signature<'a> {
    /// Its index among the lines of the header section.
    pub(crate) line: usize,
    /// The names of its `h=` tag, in order.
    pub(crate) names: &'a [&'a [u8]],
    /// Its `b=` tag, as ranges into the field value, from the tag-list parser (HDR-006).
    pub(crate) b: &'a Tag,
    /// The header canonicalization of its `c=` tag.
    pub(crate) mode: HeaderMode,
}

/// What a name in `h=` selected (DKV-011).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pick {
    /// A field, by its index among the lines of the header section.
    Field(usize),
    /// The signature's own field, which counts as an instance like any other (HCN-004).
    OwnField,
    /// No field of that name is left, so the name adds nothing (RFC 6376 §5.4).
    NoneLeft,
}

/// Appends the header hash input of `signature` to `output`, and gives what each name in
/// `h=` selected, in `h=` order. `lines` come from splitting `input`. Gives `None` and appends
/// nothing when the signature's line is no header field or its `b=` tag lies outside it.
pub(crate) fn hash_input(
    input: &[u8],
    lines: &[HeaderLine],
    signature: &Signature<'_>,
    output: &mut Vec<u8>,
) -> Option<Vec<Pick>> {
    let _ = (input, lines, signature, output);
    None
}

#[cfg(test)]
mod tests {
    use super::Pick::{Field, NoneLeft, OwnField};
    use super::{Pick, Signature, hash_input};
    use crate::canonical::header::HeaderMode::{Relaxed, Simple};
    use crate::header::section::{LineKind, split};
    use crate::tags::canon::{Algorithm, canonicalization};
    use crate::tags::fields::names;
    use crate::tags::{Grammar, TagList};

    /// The fields below S in the DKIM header hash section.
    const REST: &str = "From: a@void.test\r\nTo: b@void.test\r\nSubject: Hi\r\n";
    const BH: &str = "frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY=";
    const S_HEAD: &str = "DKIM-Signature: v=1; a=ed25519-sha256; c=";
    const S_TAGS: &str = "; d=void.test; s=s;\r\n h=";
    const S_END: &str = ";\r\n b=AAAA\r\n";
    const SIG_HEAD: &str = "DKIM-Signature: v=1; a=ed25519-sha256; c=relaxed; d=void.test; s=";
    const SIG_H: &str = ";\r\n h=";
    const SIG_BH: &str = "; bh=frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY=; b=";
    const RELAXED_HEAD: &str = "dkim-signature:v=1; a=ed25519-sha256; c=relaxed; d=void.test; s=";
    const FROM: &str = "from:a@void.test\r\n";

    /// S of the DKIM header hash section, with `c=`, `h=`, and `z` before `bh=`, then `after`.
    fn s(c: &str, h: &str, z: &str, after: &str) -> Vec<u8> {
        let parts = [S_HEAD, c, S_TAGS, h, "; ", z, "bh=", BH, S_END, after];
        parts.concat().into_bytes()
    }

    /// A signature of the section on DKIM-Signature fields in `h=`.
    fn sig(selector: &str, h: &str, b: &str) -> String {
        let parts = [SIG_HEAD, selector, SIG_H, h, SIG_BH, b, "\r\n"];
        parts.concat()
    }

    /// The `relaxed` form of [`sig`] with an empty `b=` and no CRLF.
    fn relaxed(selector: &str, h: &str) -> String {
        let parts = [RELAXED_HEAD, selector, "; h=", h, SIG_BH];
        parts.concat()
    }

    /// S2, S1 and From, top to bottom, with `h` in S2.
    fn s2_s1_from(h: &str) -> Vec<u8> {
        let s2 = sig("new", h, "BBBB");
        let s1 = sig("old", "From", "AAAA");
        let parts = [s2.as_str(), &s1, "From: a@void.test\r\n"];
        parts.concat().into_bytes()
    }

    /// The header hash input of the signature on line `own` of `input` as text, and the picks.
    fn text(input: &[u8], own: usize) -> (String, Vec<Pick>) {
        let lines = split(input).lines;
        let LineKind::Field { value, .. } = &lines[own].kind else {
            panic!("line {own} is malformed");
        };
        let field = &input[value.clone()];
        let list = TagList::parse(field, Grammar::Dkim).unwrap();
        let (mut h, mut b, mut c) = (None, None, None);
        for tag in list.tags() {
            match &field[tag.name.clone()] {
                b"h" => h = Some(&field[tag.value.clone()]),
                b"b" => b = Some(tag),
                b"c" => c = Some(&field[tag.value.clone()]),
                _ => {}
            }
        }
        let h = h.unwrap();
        let mut selected = Vec::new();
        for range in names(h).unwrap() {
            selected.push(&h[range]);
        }
        let mode = match canonicalization(c).unwrap().header {
            Algorithm::Simple => Simple,
            Algorithm::Relaxed => Relaxed,
        };
        let signature = Signature {
            line: own,
            names: &selected,
            b: b.unwrap(),
            mode,
        };
        let mut output = Vec::new();
        let picks = hash_input(input, &lines, &signature, &mut output).unwrap();
        (String::from_utf8(output).unwrap(), picks)
    }

    #[test]
    fn can_180_the_named_fields_then_the_signature_with_b_empty() {
        let input = s("relaxed", "From:Subject", "", REST);
        let expected = concat!(
            "from:a@void.test\r\nsubject:Hi\r\ndkim-signature:v=1; a=ed25519-sha256; ",
            "c=relaxed; d=void.test; s=s; h=From:Subject; ",
            "bh=frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY=; b=",
        );
        let (bytes, picks) = text(&input, 0);
        assert_eq!(bytes, expected);
        assert_eq!(picks, [Field(1), Field(3)]);
    }

    #[test]
    fn can_181_simple_deletes_the_b_value_and_the_white_space_around_it() {
        let field = concat!(
            "DKIM-Signature: v=1; a=ed25519-sha256; d=void.test; s=s; h=From;\r\n",
            " bh=frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY=; b= AAAA\r\n BBBB ; t=1\r\n",
        );
        let expected = concat!(
            "From: a@void.test\r\nDKIM-Signature: v=1; a=ed25519-sha256; d=void.test; s=s; ",
            "h=From;\r\n bh=frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY=; b=; t=1",
        );
        let input = [field, REST].concat();
        assert_eq!(text(input.as_bytes(), 0).0, expected);
    }

    #[test]
    fn can_182_a_b_inside_another_value_stays() {
        let input = s("relaxed", "From:Subject", "z=Subject:b=3D1; ", REST);
        let end = ["z=Subject:b=3D1; bh=", BH, "; b="].concat();
        assert!(text(&input, 0).0.ends_with(&end));
    }

    #[test]
    fn can_183_a_name_with_no_field_left_adds_nothing() {
        let h = "From:Subject:Subject:Reply-To";
        let (bytes, picks) = text(&s("relaxed", h, "", REST), 0);
        let start = "from:a@void.test\r\nsubject:Hi\r\ndkim-signature:";
        let end = ["h=", h, "; bh=", BH, "; b="].concat();
        assert!(bytes.starts_with(start));
        assert!(bytes.ends_with(&end));
        assert_eq!(picks, [Field(1), Field(3), NoneLeft, NoneLeft]);
    }

    #[test]
    fn can_184_instances_go_from_the_bottom_up() {
        let received = "Received: <A>\r\nReceived: <B>\r\nReceived: <C>\r\nFrom: a@void.test\r\n";
        let (bytes, picks) = text(&s("relaxed", "From:Received:Received", "", received), 0);
        let start = "from:a@void.test\r\nreceived:<C>\r\nreceived:<B>\r\n";
        assert!(bytes.starts_with(start));
        assert_eq!(picks, [Field(4), Field(3), Field(2)]);
    }

    #[test]
    fn can_185_names_match_without_regard_to_case() {
        let (bytes, picks) = text(&s("simple/simple", "FROM", "", "fROM: a@void.test\r\n"), 0);
        let start = "fROM: a@void.test\r\nDKIM-Signature: v=1;";
        assert!(bytes.starts_with(start));
        assert_eq!(picks, [Field(1)]);
    }

    #[test]
    fn can_200_an_older_signature_is_a_field_like_any_other() {
        let h = "From:DKIM-Signature";
        let (bytes, picks) = text(&s2_s1_from(h), 0);
        let s1 = relaxed("old", "From") + "AAAA\r\n";
        let parts = [FROM, &s1, &relaxed("new", h)];
        assert_eq!(bytes, parts.concat());
        assert_eq!(picks, [Field(2), Field(1)]);
    }

    #[test]
    fn can_201_a_name_beyond_the_older_signatures_selects_the_own_field() {
        let h = "From:DKIM-Signature:DKIM-Signature";
        let (bytes, picks) = text(&s2_s1_from(h), 0);
        let s1 = relaxed("old", "From") + "AAAA\r\n";
        let s2 = relaxed("new", h) + "BBBB\r\n";
        let parts = [FROM, &s1, &s2, &relaxed("new", h)];
        assert_eq!(bytes, parts.concat());
        assert_eq!(picks, [Field(2), Field(1), OwnField]);
    }

    #[test]
    fn can_202_a_moved_signature_leaves_the_own_field_lowest() {
        let h = "From:DKIM-Signature";
        let s1 = sig("old", "From", "AAAA");
        let s2 = sig("new", h, "BBBB");
        let input = [s1.as_str(), &s2, "From: a@void.test\r\n"].concat();
        let (bytes, picks) = text(input.as_bytes(), 1);
        let s2 = relaxed("new", h) + "BBBB\r\n";
        let parts = [FROM, &s2, &relaxed("new", h)];
        assert_eq!(bytes, parts.concat());
        assert_eq!(picks, [Field(2), OwnField]);
    }

    #[test]
    fn can_262_no_name_selects_a_malformed_line() {
        let fields = "Subject: Hi\r\nnot a field\r\nFrom: a@void.test\r\n";
        let (bytes, picks) = text(&s("relaxed", "From:Subject", "", fields), 0);
        let start = "from:a@void.test\r\nsubject:Hi\r\ndkim-signature:";
        assert!(bytes.starts_with(start));
        assert_eq!(picks, [Field(3), Field(1)]);
    }

    #[test]
    fn can_266_a_signature_at_the_end_of_the_input_gets_no_crlf() {
        let field = s("relaxed", "From:Subject", "", "");
        let unterminated = &field[..field.len() - 2];
        let at_end = text(&[REST.as_bytes(), unterminated].concat(), 3);
        let on_top = text(&s("relaxed", "From:Subject", "", REST), 0);
        assert_eq!(at_end.0, on_top.0);
        assert_eq!(at_end.1, [Field(0), Field(2)]);
    }
}
