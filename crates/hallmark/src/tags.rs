//! The tag=value list parser that DKIM and DKIM2 share (RFC 6376 §3.2,
//! draft-09 §8, HDR-006). It keeps each tag as byte ranges into the input
//! (PAR-002), and the value decoders read the values.

// DKIM and DKIM2, which read their tag lists here, come in later changes, so
// only the tests use this module for now.
#![cfg_attr(not(test), expect(dead_code, reason = "DKIM and DKIM2 come later"))]

mod fields;
mod qp;

use core::ops::Range;

/// A run of WSP and FWS with any number of folds.
const ANY_FOLDS: usize = usize::MAX;

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
    pub(crate) fn parse(input: &[u8], grammar: Grammar) -> Result<Self, TagListError> {
        let mut tags = Vec::new();
        let mut names = Vec::new();
        let outcome = read(input, grammar, &mut tags, &mut names);
        // RFC 6376 §3.2, draft-09 §8: a repeated name makes the whole list
        // invalid. A repeat among the complete tag-specs comes before the
        // fault that stopped the reading, if any (PAR-005).
        if let Some(name) = first_repeat(&mut names) {
            return Err(TagListError::Duplicate(name));
        }
        outcome?;
        Ok(Self { tags })
    }

    /// The tags in list order.
    pub(crate) fn tags(&self) -> &[Tag] {
        &self.tags
    }
}

/// Reads the tag-specs of `input` into `tags`, and each name with its
/// offset into `names`, until the list ends or a rule fails.
fn read<'a>(
    input: &'a [u8],
    grammar: Grammar,
    tags: &mut Vec<Tag>,
    names: &mut Vec<(&'a [u8], usize)>,
) -> Result<(), TagListError> {
    let mut at = 0;
    loop {
        let start = white_space(input, at, 1)?;
        let name = tag_name(input, start)?;
        let name_end = start + name.len();
        let equals = white_space(input, name_end, 1)?;
        if input.get(equals) != Some(&b'=') {
            return Err(TagListError::Syntax(Rule::Spec, equals));
        }
        let (value, end) = tag_value(input, equals + 1, grammar)?;
        tags.push(Tag {
            name: start..name_end,
            value,
        });
        names.push((name, start));
        match input.get(end..) {
            Some([] | [b';']) => return Ok(()),
            Some([b';', ..]) => at = end + 1,
            _ => return Err(TagListError::Syntax(Rule::List, end)),
        }
    }
}

/// The range of the first name, in list order, that an earlier tag-spec
/// already has. Sorting keeps the search at O(n log n) for n tags, where a
/// search of the earlier names for each tag would be quadratic.
fn first_repeat(names: &mut [(&[u8], usize)]) -> Option<Range<usize>> {
    names.sort_unstable();
    let mut repeat: Option<Range<usize>> = None;
    for pair in names.windows(2) {
        let [(earlier, _), (name, start)] = pair else {
            continue;
        };
        let first = match &repeat {
            Some(found) => *start < found.start,
            None => true,
        };
        if earlier == name && first {
            repeat = Some(*start..start + name.len());
        }
    }
    repeat
}

/// Skips WSP and folds from `at` and returns the end of the run. A CRLF
/// folds only before WSP (RFC 6376 §2.8). A fold past `folds` breaks the
/// tag-spec rule at its CR.
fn white_space(input: &[u8], mut at: usize, folds: usize) -> Result<usize, TagListError> {
    let mut seen = 0;
    loop {
        match input.get(at..) {
            Some([b' ' | b'\t', ..]) => at += 1,
            Some([b'\r', b'\n', b' ' | b'\t', ..]) if seen < folds => {
                seen += 1;
                at += 2;
            }
            Some([b'\r', b'\n', b' ' | b'\t', ..]) => {
                return Err(TagListError::Syntax(Rule::Spec, at));
            }
            _ => return Ok(at),
        }
    }
}

/// Reads the tag-name at `start`: a letter, then letters, digits and
/// underscores.
fn tag_name(input: &[u8], start: usize) -> Result<&[u8], TagListError> {
    let fault = TagListError::Syntax(Rule::Name, start);
    let Some(rest) = input.get(start..) else {
        return Err(fault);
    };
    let Some(b'a'..=b'z' | b'A'..=b'Z') = rest.first().copied() else {
        return Err(fault);
    };
    let mut length = 1;
    while let Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') = rest.get(length).copied() {
        length += 1;
    }
    match rest.get(..length) {
        Some(name) => Ok(name),
        None => Err(fault),
    }
}

/// Reads the tag-value from `start`, just after the equals sign, with the
/// white space around it. Returns the value's range and the offset after
/// the white space that follows the value.
fn tag_value(
    input: &[u8],
    start: usize,
    grammar: Grammar,
) -> Result<(Range<usize>, usize), TagListError> {
    let around_empty = match grammar {
        // RFC 6376 §3.2, erratum 5070: [FWS] "=" [FWS] [tag-value [FWS]]
        Grammar::Dkim | Grammar::DkimUtf8 => 1,
        // draft-09 §8: [FWS] "=" [FWS] tag-value [FWS], tag-value optional
        Grammar::Dkim2 => 2,
    };
    let first = white_space(input, start, around_empty)?;
    let mut end = tval(input, first, grammar)?;
    if end == first {
        return Ok((first..first, first));
    }
    // One FWS before a value.
    white_space(input, start, 1)?;
    loop {
        let gap = white_space(input, end, ANY_FOLDS)?;
        let next = tval(input, gap, grammar)?;
        if next == gap {
            // One FWS after a value.
            let after = white_space(input, end, 1)?;
            return Ok((first..end, after));
        }
        end = next;
    }
}

/// Reads a tval from `at` and returns its end, which is `at` when no tval
/// starts there.
fn tval(input: &[u8], mut at: usize, grammar: Grammar) -> Result<usize, TagListError> {
    loop {
        match input.get(at).copied() {
            Some(0x21..=0x3A | 0x3C..=0x7E) => at += 1,
            Some(0x80..) if grammar == Grammar::DkimUtf8 => at = utf8(input, at)?,
            _ => return Ok(at),
        }
    }
}

/// Reads the run of non-ASCII octets at `at`, which must be well-formed
/// UTF-8 (RFC 3629 §4), and returns its end. An ill-formed sequence breaks
/// the tag-value rule at its first octet.
fn utf8(input: &[u8], at: usize) -> Result<usize, TagListError> {
    let mut end = at;
    while let Some(0x80..) = input.get(end).copied() {
        end += 1;
    }
    let Some(run) = input.get(at..end) else {
        return Err(TagListError::Syntax(Rule::Value, at));
    };
    match core::str::from_utf8(run) {
        Ok(_) => Ok(end),
        Err(error) => {
            let offset = at + error.valid_up_to();
            Err(TagListError::Syntax(Rule::Value, offset))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Grammar, Rule, TagList, TagListError};
    use crate::base64::decode;

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

    #[test]
    fn tag_020_a_fold_inside_base64_is_not_part_of_it() {
        let input = b"bh=frcCV1k9oG9oKj3dpUqdJg1P\r\n xRT2RSN/XKdLCPjaYaY=";
        let tags = pairs(input, Grammar::Dkim);
        let hash = decode(tags[0].1.as_bytes()).unwrap();
        let unfolded = decode(b"frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY=");
        // The SHA-256 hash of CRLF, in two halves.
        let high = b"\x7e\xb7\x02\x57\x59\x3d\xa0\x6f\x68\x2a\x3d\xdd\xa5\x4a\x9d\x26";
        let low = b"\x0d\x4f\xc5\x14\xf6\x45\x23\x7f\x5c\xa7\x4b\x08\xf8\xda\x61\xa6";
        assert_eq!(hash.len(), 32);
        assert_eq!(&hash[..16], high);
        assert_eq!(&hash[16..], low);
        assert_eq!(Ok(hash), unfolded);
    }
}
