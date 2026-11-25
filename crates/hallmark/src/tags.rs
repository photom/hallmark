//! The tag=value list parser that DKIM and DKIM2 share (RFC 6376 §3.2,
//! draft-09 §8, HDR-006). It keeps each tag as byte ranges into the input
//! (PAR-002), and the value decoders read the values.

// DKIM and DKIM2, which read their tag lists here, come in later changes, so
// only the tests use this module for now.
#![cfg_attr(not(test), expect(dead_code, reason = "DKIM and DKIM2 come later"))]

use core::ops::Range;

/// The grammar of a tag list, which its source decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Grammar {
    /// RFC 6376 §3.2 as erratum 5070 corrects it, with ASCII values: a DKIM
    /// key record, or a DKIM-Signature field of a message without
    /// internationalized header fields.
    Dkim,
    /// The grammar of [`Grammar::Dkim`], whose values may also hold
    /// well-formed UTF-8 (RFC 3629 §4): a DKIM-Signature field of a message
    /// with internationalized header fields (RFC 8616 §5).
    DkimUtf8,
    /// draft-09 §8, with ASCII values: Message-Instance and DKIM2-Signature
    /// fields, DKIM2 key records and Domain Policy Records (dns-03 §2.4).
    Dkim2,
}

/// One tag of a list, as byte ranges into the input (PAR-002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Tag {
    /// The tag name, which is never empty.
    pub(crate) name: Range<usize>,
    /// The value as written, folds included, without the white space at
    /// either end. An empty value is an empty range where the value would
    /// start.
    pub(crate) value: Range<usize>,
}

/// A tag list that parsed, with no name twice (RFC 6376 §3.2, draft-09 §8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagList {
    /// The tags in list order.
    tags: Vec<Tag>,
}

/// Why a tag list doesn't parse. Either way the whole list is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TagListError {
    /// The rule that failed, and the offset of the first octet that broke it
    /// (PAR-001).
    Syntax(Rule, usize),
    /// A name that an earlier tag-spec already has: its range where it occurs
    /// again (RFC 6376 §3.2, draft-09 §8).
    Duplicate(Range<usize>),
}

/// A rule of the tag-list grammar, by its name in RFC 6376 §3.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rule {
    /// tag-list: a tag-spec ends with neither a semicolon nor the end of the
    /// list.
    List,
    /// tag-spec: no equals sign after the name, or more folds than the white
    /// space allows there.
    Spec,
    /// tag-name: a tag-spec that doesn't start with a letter.
    Name,
    /// tag-value: an octet that starts no well-formed UTF-8 character, where
    /// values may hold UTF-8.
    Value,
}

impl TagList {
    /// Parses `input`, the value of a header field after its colon, folds
    /// included, or the strings of a DNS record joined.
    pub(crate) fn parse(_input: &[u8], _grammar: Grammar) -> Result<Self, TagListError> {
        Ok(Self { tags: Vec::new() })
    }

    /// The tags in list order.
    pub(crate) fn tags(&self) -> &[Tag] {
        &self.tags
    }
}

#[cfg(test)]
mod tests {
    use super::{Grammar, Rule, TagList, TagListError};

    /// The name and value of each tag of `input`, as text.
    fn pairs(input: &[u8], grammar: Grammar) -> Vec<(&str, &str)> {
        let list = TagList::parse(input, grammar).unwrap();
        let mut pairs = Vec::new();
        for tag in list.tags() {
            let name = core::str::from_utf8(&input[tag.name.clone()]);
            let value = core::str::from_utf8(&input[tag.value.clone()]);
            pairs.push((name.unwrap(), value.unwrap()));
        }
        pairs
    }

    /// The error of `input`, which doesn't parse.
    fn fault(input: &[u8], grammar: Grammar) -> TagListError {
        TagList::parse(input, grammar).unwrap_err()
    }

    #[test]
    fn tag_001_a_list_gives_names_and_values() {
        let tags = pairs(b"v=1; a=rsa-sha256", Grammar::Dkim);
        assert_eq!(tags, [("v", "1"), ("a", "rsa-sha256")]);
    }

    #[test]
    fn tag_002_white_space_around_a_value_is_not_part_of_it() {
        let tags = pairs(b" a = x y ;  b=z;", Grammar::Dkim);
        assert_eq!(tags, [("a", "x y"), ("b", "z")]);
    }

    #[test]
    fn tag_003_a_repeated_name_invalidates_the_list() {
        let error = TagListError::Duplicate(5..6);
        assert_eq!(fault(b"a=1; a=2", Grammar::Dkim), error);
    }

    #[test]
    fn tag_005_an_empty_value_is_present() {
        let tags = pairs(b"a=; b=1", Grammar::Dkim);
        assert_eq!(tags, [("a", ""), ("b", "1")]);
    }

    #[test]
    fn tag_007_a_name_starts_with_a_letter() {
        let error = TagListError::Syntax(Rule::Name, 0);
        assert_eq!(fault(b"1a=x", Grammar::Dkim), error);
    }

    #[test]
    fn tag_008_dkim_values_may_hold_utf8() {
        let tags = pairs("d=例え.test".as_bytes(), Grammar::DkimUtf8);
        assert_eq!(tags, [("d", "例え.test")]);
    }

    #[test]
    fn tag_010_an_equals_sign_follows_the_name() {
        let error = TagListError::Syntax(Rule::Spec, 1);
        assert_eq!(fault(b"a:1", Grammar::Dkim), error);
    }

    #[test]
    fn tag_011_dkim_allows_one_fws_around_an_empty_value() {
        let error = TagListError::Syntax(Rule::Spec, 5);
        assert_eq!(fault(b"a=\r\n \r\n ;", Grammar::Dkim), error);
    }

    #[test]
    fn tag_012_dkim2_allows_two_fws_around_an_empty_value() {
        let tags = pairs(b"a=\r\n \r\n ;", Grammar::Dkim2);
        assert_eq!(tags, [("a", "")]);
    }

    #[test]
    fn tag_013_folds_between_tvals_stay_in_the_value() {
        let tags = pairs(b"a=x\r\n \r\n y", Grammar::Dkim);
        assert_eq!(tags, [("a", "x\r\n \r\n y")]);
    }

    #[test]
    fn tag_014_a_crlf_folds_only_before_wsp() {
        let error = TagListError::Syntax(Rule::List, 3);
        assert_eq!(fault(b"a=1\r\nb=2", Grammar::Dkim), error);
    }

    #[test]
    fn tag_015_a_tag_spec_follows_each_semicolon_but_the_last() {
        let cases: [(&[u8], usize); 3] = [(b"a=1; ", 5), (b"a=1;;", 4), (b"", 0)];
        for (input, offset) in cases {
            let error = TagListError::Syntax(Rule::Name, offset);
            assert_eq!(fault(input, Grammar::Dkim), error);
        }
    }

    #[test]
    fn tag_016_ascii_grammars_refuse_utf8() {
        let input = "d=例え.test".as_bytes();
        let error = TagListError::Syntax(Rule::List, 2);
        assert_eq!(fault(input, Grammar::Dkim), error);
        assert_eq!(fault(input, Grammar::Dkim2), error);
    }

    #[test]
    fn tag_017_utf8_values_are_well_formed() {
        let error = TagListError::Syntax(Rule::Value, 3);
        assert_eq!(fault(b"d=a\xFF", Grammar::DkimUtf8), error);
    }

    #[test]
    fn tag_018_the_first_repeat_in_list_order_wins() {
        let error = TagListError::Duplicate(5..6);
        assert_eq!(fault(b"a=1; a=2; 1=x", Grammar::Dkim), error);
        let error = TagListError::Duplicate(10..11);
        assert_eq!(fault(b"b=1; a=1; b=2; a=2", Grammar::Dkim), error);
    }

    #[test]
    fn tag_019_names_are_case_sensitive() {
        let tags = pairs(b"v=1; V=1", Grammar::Dkim);
        assert_eq!(tags, [("v", "1"), ("V", "1")]);
    }
}
