//! The groups of DKIM2 `h=` and `s=` (HDR-007): the hash sets of a Message-Instance field
//! (draft-09 §9.3) and the signatures of a DKIM2-Signature field (§10.9). Sets joined by `,`
//! form a group, and `|` separates two groups.

#![cfg_attr(not(test), expect(dead_code, reason = "DKIM2 comes later"))]

use core::ops::Range;

use crate::base64::{self, Base64Error};

/// A textstring's own FWS (draft-09 §2.14) plus the FWS of the rule around it.
const NAME_FOLDS: usize = 2;

/// One hash-set of `h=`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HashValues {
    /// The hash-name, a range of the value, matched as written (draft-09 §2.14).
    pub(crate) algorithm: Range<usize>,
    /// The header hash, decoded.
    pub(crate) header: Vec<u8>,
    /// The body hash, decoded.
    pub(crate) body: Vec<u8>,
}

/// One sig-set of `s=`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SignatureValue {
    /// The selector, a range of the value that holds an RFC 5321 Domain.
    pub(crate) selector: Range<usize>,
    /// The sig-name, a range of the value, matched as written (draft-09 §2.14).
    pub(crate) algorithm: Range<usize>,
    /// The signature, decoded.
    pub(crate) signature: Vec<u8>,
}

/// Why an `h=` or `s=` value doesn't decode. Offsets and ranges count from the value's start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GroupsError {
    /// hash-name or sig-name: no textstring where an algorithm name must start.
    Algorithm(usize),
    /// selector: an octet that breaks the Domain rule of RFC 5321 §4.1.2.
    Selector(usize),
    /// hash-set or sig-set: no colon where one must stand.
    Colon(usize),
    /// hash-any or sig-any: a set that neither `,`, `|` nor the end of the value follows.
    Separator(usize),
    /// A hash or a signature breaks a rule of the base64 decoder.
    Value(Base64Error),
    /// An algorithm in `h=` or a selector in `s=` that an earlier set names (§9.3, §10.9).
    Repeat(Range<usize>),
}

/// A set of a group value, and how it is read.
trait Set: Sized {
    /// The FWS that may stand before the first set.
    const FIRST: usize;
    /// The FWS that may stand after a `,` or `|`.
    const NEXT: usize;
    /// Reads the set at `at`, and returns it with its end.
    fn read(input: &[u8], at: usize) -> Result<(Self, usize), GroupsError>;
    /// The part that no two sets of a value may share.
    fn key(&self) -> Range<usize>;
}

impl Set for HashValues {
    // A hash-name opens a set, and its textstring's FWS adds to that of `[FWS] "|" [FWS]`.
    const FIRST: usize = 1;
    const NEXT: usize = NAME_FOLDS;

    /// `hash-set = hash-name [FWS] ":" header-hash ":" body-hash`.
    fn read(input: &[u8], at: usize) -> Result<(Self, usize), GroupsError> {
        let algorithm = text(input, at)?;
        let header_at = colon(input, fws(input, algorithm.end, NAME_FOLDS))?;
        let (header, header_end) = value(input, header_at)?;
        let (body, end) = value(input, colon(input, header_end)?)?;
        let set = Self {
            algorithm,
            header,
            body,
        };
        Ok((set, end))
    }

    fn key(&self) -> Range<usize> {
        self.algorithm.clone()
    }
}

impl Set for SignatureValue {
    // A selector opens a set, with no FWS of its own.
    const FIRST: usize = 0;
    const NEXT: usize = 1;

    /// `sig-set = selector [FWS] ":" [FWS] sig-name [FWS] ":" message-sig`.
    fn read(input: &[u8], at: usize) -> Result<(Self, usize), GroupsError> {
        let selector = selector(input, at)?;
        let after = colon(input, fws(input, selector.end, 1))?;
        let algorithm = text(input, fws(input, after, NAME_FOLDS))?;
        let signature_at = colon(input, fws(input, algorithm.end, NAME_FOLDS))?;
        let (signature, end) = value(input, signature_at)?;
        let set = Self {
            selector,
            algorithm,
            signature,
        };
        Ok((set, end))
    }

    fn key(&self) -> Range<usize> {
        self.selector.clone()
    }
}

/// The groups of a Message-Instance `h=` value, `hash-all *( [FWS] "|" [FWS] hash-all )`, in
/// order. An algorithm named twice makes the value invalid (draft-09 §9.3).
pub(crate) fn hashes(input: &[u8]) -> Result<Vec<Vec<HashValues>>, GroupsError> {
    groups(input)
}

/// The groups of a DKIM2-Signature `s=` value, `sig-all *( [FWS] "|" [FWS] sig-all )`, in
/// order. A selector named twice, as written, makes the value invalid (draft-09 §8, §10.9).
pub(crate) fn signatures(input: &[u8]) -> Result<Vec<Vec<SignatureValue>>, GroupsError> {
    groups(input)
}

/// Reads the groups of `input`. A repeat among complete sets comes before a later fault (PAR-005).
fn groups<S: Set>(input: &[u8]) -> Result<Vec<Vec<S>>, GroupsError> {
    let mut groups = Vec::new();
    let mut names = Vec::new();
    let outcome = read(input, &mut groups, &mut names);
    if let Some(name) = super::first_repeat(&mut names) {
        return Err(GroupsError::Repeat(name));
    }
    outcome?;
    Ok(groups)
}

/// Reads the sets of `input` into `groups`, and each key into `names`, until the end or a fault.
fn read<'a, S: Set>(
    input: &'a [u8],
    groups: &mut Vec<Vec<S>>,
    names: &mut Vec<(&'a [u8], usize)>,
) -> Result<(), GroupsError> {
    let mut group = Vec::new();
    let mut at = fws(input, 0, S::FIRST);
    loop {
        let (set, end) = S::read(input, at)?;
        let key = set.key();
        if let Some(name) = input.get(key.clone()) {
            names.push((name, key.start));
        }
        group.push(set);
        let after = fws(input, end, 1);
        match input.get(after).copied() {
            // hash-any and sig-any end with a set, never with FWS.
            None if after == end => {
                groups.push(group);
                return Ok(());
            }
            Some(b',') => {}
            Some(b'|') => groups.push(core::mem::take(&mut group)),
            _ => return Err(GroupsError::Separator(after)),
        }
        at = fws(input, after + 1, S::NEXT);
    }
}

/// Reads the colon at `at`, and returns the offset after it.
fn colon(input: &[u8], at: usize) -> Result<usize, GroupsError> {
    match input.get(at).copied() {
        Some(b':') => Ok(at + 1),
        _ => Err(GroupsError::Colon(at)),
    }
}

/// Reads the textstring at `start`, without its FWS: letters, digits, `-` and `_` (draft-09 §2.14).
fn text(input: &[u8], start: usize) -> Result<Range<usize>, GroupsError> {
    let mut end = start;
    while let Some(octet) = input.get(end).copied() {
        if !octet.is_ascii_alphanumeric() && octet != b'-' && octet != b'_' {
            break;
        }
        end += 1;
    }
    if end == start {
        return Err(GroupsError::Algorithm(start));
    }
    Ok(start..end)
}

/// Reads the selector at `start`, an RFC 5321 Domain (draft-09 §3.6): sub-domains joined by dots,
/// each of letters, digits and hyphens that starts and ends with a letter or digit (§4.1.2).
/// The lengths of labels and names are the DKIM2 check's rule (ADS-004).
fn selector(input: &[u8], start: usize) -> Result<Range<usize>, GroupsError> {
    let mut at = start;
    loop {
        let label = at;
        while let Some(b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-') = input.get(at).copied() {
            at += 1;
        }
        if at == label || input.get(label) == Some(&b'-') {
            return Err(GroupsError::Selector(label));
        }
        if input.get(at - 1) == Some(&b'-') {
            return Err(GroupsError::Selector(at));
        }
        if input.get(at) != Some(&b'.') {
            return Ok(start..at);
        }
        at += 1;
    }
}

/// Decodes the base64string at `start`, which runs to the next `:`, `,`, `|` or the end, less the
/// white space before it. Returns the octets and the end of the base64string.
fn value(input: &[u8], start: usize) -> Result<(Vec<u8>, usize), GroupsError> {
    let mut end = start;
    while let Some(octet) = input.get(end).copied() {
        if let b':' | b',' | b'|' = octet {
            break;
        }
        end += 1;
    }
    while end > start {
        let Some(b' ' | b'\t' | b'\r' | b'\n') = input.get(end - 1).copied() else {
            break;
        };
        end -= 1;
    }
    let Some(written) = input.get(start..end) else {
        return Err(GroupsError::Value(Base64Error::Syntax(start)));
    };
    match base64::decode(written) {
        Ok(octets) => Ok((octets, end)),
        Err(error) => Err(GroupsError::Value(shifted(error, start))),
    }
}

/// `error`, with its offset counted from the start of the group value.
fn shifted(error: Base64Error, start: usize) -> Base64Error {
    match error {
        Base64Error::Syntax(at) => Base64Error::Syntax(start + at),
        Base64Error::Padding(at) => Base64Error::Padding(start + at),
        Base64Error::PadBits(at) => Base64Error::PadBits(start + at),
    }
}

/// Skips `count` optional FWS at `at`: none, or WSP with up to `count` folds. A CRLF folds only
/// before WSP (draft-09 §2.12).
fn fws(input: &[u8], mut at: usize, count: usize) -> usize {
    if count == 0 {
        return at;
    }
    let mut seen = 0;
    loop {
        match input.get(at..) {
            Some([b' ' | b'\t', ..]) => at += 1,
            Some([b'\r', b'\n', b' ' | b'\t', ..]) if seen < count => {
                seen += 1;
                at += 3;
            }
            _ => return at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Base64Error::{PadBits, Padding, Syntax};
    use super::GroupsError::{Algorithm, Colon, Repeat, Selector, Separator, Value};
    use super::{GroupsError, hashes, signatures};

    /// `octets` in base64url, which writes these tests' 3-octet values as base64 does.
    fn encoded(octets: &[u8]) -> String {
        let mut text = Vec::new();
        crate::base64::encode_url_unpadded(octets, &mut text);
        String::from_utf8(text).unwrap()
    }

    /// An `h=` value's groups as text: each set's name and hashes, joined by `:`.
    fn hash_text(input: &[u8]) -> String {
        let mut groups = Vec::new();
        for group in hashes(input).unwrap() {
            let mut sets = Vec::new();
            for set in group {
                let name = String::from_utf8_lossy(&input[set.algorithm]);
                let (header, body) = (encoded(&set.header), encoded(&set.body));
                sets.push(format!("{name}:{header}:{body}"));
            }
            groups.push(sets.join(","));
        }
        groups.join("|")
    }

    /// The groups of an `s=` value as text, as [`hash_text`] writes them.
    fn signature_text(input: &[u8]) -> String {
        let mut groups = Vec::new();
        for group in signatures(input).unwrap() {
            let mut sets = Vec::new();
            for set in group {
                let selector = String::from_utf8_lossy(&input[set.selector]);
                let name = String::from_utf8_lossy(&input[set.algorithm]);
                let signature = encoded(&set.signature);
                sets.push(format!("{selector}:{name}:{signature}"));
            }
            groups.push(sets.join(","));
        }
        groups.join("|")
    }

    #[test]
    fn tag_040_a_comma_binds_more_tightly_than_a_bar_in_h() {
        let input = b"sha256:AAAA:BBBB,sha512:CCCC:DDDD|x-new:EEEE:FFFF";
        let expected = "sha256:AAAA:BBBB,sha512:CCCC:DDDD|x-new:EEEE:FFFF";
        assert_eq!(hash_text(input), expected);
    }

    #[test]
    fn tag_042_a_comma_binds_more_tightly_than_a_bar_in_s() {
        let input = b"one:rsa-sha256:AAAA,two:mldsa44:BBBB|three:ed25519-sha256:CCCC";
        let expected = "one:rsa-sha256:AAAA,two:mldsa44:BBBB|three:ed25519-sha256:CCCC";
        assert_eq!(signature_text(input), expected);
    }

    #[test]
    fn tag_051_an_algorithm_or_selector_appears_once() {
        let input = b"sha256:AAAA:BBBB|sha256:CCCC:DDDD";
        assert_eq!(hashes(input), Err(Repeat(17..23)));
        let input = b"sha256:AAAA:BBBB|sha256:CCCC:DDDD|x";
        assert_eq!(hashes(input), Err(Repeat(17..23)));
        let input = b"sha256:AAAA:BBBB,sha256:CCCC:DDDD";
        assert_eq!(hashes(input), Err(Repeat(17..23)));
        let input = b"sha256:AAAA:BBBB|SHA256:CCCC:DDDD";
        assert_eq!(hash_text(input), "sha256:AAAA:BBBB|SHA256:CCCC:DDDD");
        let input = b"one:rsa-sha256:AAAA|one:ed25519-sha256:BBBB";
        assert_eq!(signatures(input), Err(Repeat(20..23)));
        let input = b"one:rsa-sha256:AAAA|ONE:rsa-sha256:BBBB";
        let expected = "one:rsa-sha256:AAAA|ONE:rsa-sha256:BBBB";
        assert_eq!(signature_text(input), expected);
        let input = b"one:rsa-sha256:AAAA,two:rsa-sha256:BBBB";
        let expected = "one:rsa-sha256:AAAA,two:rsa-sha256:BBBB";
        assert_eq!(signature_text(input), expected);
    }

    #[test]
    fn tag_052_fws_around_names_and_inside_values() {
        let input = b"sha256\r\n \r\n :AAAA:BBBB|\r\n \r\n x-new:CCCC:DDDD";
        assert_eq!(hash_text(input), "sha256:AAAA:BBBB|x-new:CCCC:DDDD");
        let input = b"sha256:AA\r\n AA:BB BB";
        assert_eq!(hash_text(input), "sha256:AAAA:BBBB");
        let input = b"one\r\n :\r\n \r\n rsa-sha256\r\n \r\n :AAAA";
        assert_eq!(signature_text(input), "one:rsa-sha256:AAAA");
        let input = b"one:x:AAAA|\r\n two:x:BBBB";
        assert_eq!(signature_text(input), "one:x:AAAA|two:x:BBBB");
    }

    #[test]
    fn tag_053_an_h_value_follows_hash_any() {
        let cases: [(&[u8], GroupsError); 12] = [
            (b"", Algorithm(0)),
            (b"sha256AAAA:BBBB", Colon(15)),
            (b"sha256:AAAA", Colon(11)),
            (b"sha256:AAAA :BBBB", Colon(11)),
            (b"sha256\r\n \r\n \r\n :AAAA:BBBB", Colon(12)),
            (b"sha256:AAAA:BBBB,", Algorithm(17)),
            (b"sha256:AAAA:BBBB||x:CCCC:DDDD", Algorithm(17)),
            (b"sha256:AAAA:BBBB:CCCC", Separator(16)),
            (b"sha256: AAAA:BBBB", Value(Syntax(7))),
            (b"sha256:AAAA:BBB", Value(Padding(15))),
            (b"sha256:AAAA:AB==", Value(PadBits(13))),
            (b"sha256:AAAA:BBBB\r\n ", Separator(19)),
        ];
        for (input, error) in cases {
            assert_eq!(hashes(input), Err(error));
        }
    }

    #[test]
    fn tag_054_an_s_value_follows_sig_any() {
        let cases: [(&[u8], GroupsError); 11] = [
            (b"", Selector(0)),
            (b" one:x:AAAA", Selector(0)),
            (b"-one:x:AAAA", Selector(0)),
            (b"one-:x:AAAA", Selector(4)),
            (b"a..b:x:AAAA", Selector(2)),
            (b"one.:x:AAAA", Selector(4)),
            (b"one:rsa sha256:AAAA", Colon(8)),
            (b"one:x:AAAA|\r\n \r\n two:x:BBBB", Selector(14)),
            (b"one:x:AAAA\r\n \r\n |two:x:BBBB", Separator(13)),
            (b"one:rsa-sha256: AAAA", Value(Syntax(15))),
            (b"one:rsa-sha256:AAA", Value(Padding(18))),
        ];
        for (input, error) in cases {
            assert_eq!(signatures(input), Err(error));
        }
    }
}
