//! The groups of DKIM2 `h=` and `s=` (HDR-007): the hash sets of a Message-Instance field
//! (draft-09 §9.3) and the signatures of a DKIM2-Signature field (§10.9). Sets joined by `,`
//! form a group, and `|` separates two groups.

#![cfg_attr(not(test), expect(dead_code, reason = "DKIM2 comes later"))]

use core::ops::Range;

use crate::base64::Base64Error;

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

/// The groups of a Message-Instance `h=` value, `hash-all *( [FWS] "|" [FWS] hash-all )`, in
/// order. An algorithm named twice makes the value invalid (draft-09 §9.3).
pub(crate) fn hashes(_input: &[u8]) -> Result<Vec<Vec<HashValues>>, GroupsError> {
    Err(GroupsError::Algorithm(0))
}

/// The groups of a DKIM2-Signature `s=` value, `sig-all *( [FWS] "|" [FWS] sig-all )`, in
/// order. A selector named twice, as written, makes the value invalid (draft-09 §8, §10.9).
pub(crate) fn signatures(_input: &[u8]) -> Result<Vec<Vec<SignatureValue>>, GroupsError> {
    Err(GroupsError::Algorithm(0))
}

#[cfg(test)]
mod tests {
    use super::Base64Error::{Padding, Syntax};
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
        let input = b"sha256:AAAA:BBBB|SHA256:CCCC:DDDD";
        assert_eq!(hash_text(input), "sha256:AAAA:BBBB|SHA256:CCCC:DDDD");
        let input = b"one:rsa-sha256:AAAA|one:ed25519-sha256:BBBB";
        assert_eq!(signatures(input), Err(Repeat(20..23)));
        let input = b"one:rsa-sha256:AAAA|ONE:rsa-sha256:BBBB";
        let expected = "one:rsa-sha256:AAAA|ONE:rsa-sha256:BBBB";
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
        let cases: [(&[u8], GroupsError); 11] = [
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
