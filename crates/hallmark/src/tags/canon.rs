//! The DKIM `c=` value: the header and body canonicalization algorithms
//! (RFC 6376 §3.5, HDR-007).

#![cfg_attr(not(test), expect(dead_code, reason = "DKIM comes later"))]

use core::ops::Range;

/// A canonicalization algorithm that RFC 6376 §3.4 defines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Algorithm {
    /// `simple`.
    Simple,
    /// `relaxed`.
    Relaxed,
}

/// The algorithms that a `c=` value names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Canonicalization {
    /// The header canonicalization.
    pub(crate) header: Algorithm,
    /// The body canonicalization.
    pub(crate) body: Algorithm,
}

/// Why a `c=` value gives no pair of algorithms. Offsets count from the
/// start of the value as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CanonError {
    /// sig-c-tag: the value isn't one or two hyphenated-words joined by `/`.
    /// The offset is that of the first octet that breaks the rule.
    Syntax(usize),
    /// A name other than `simple` and `relaxed`, compared as written
    /// (ADR 0067), as its range: the header name if it is unrecognized, else
    /// the body name. The value has the right syntax, and RFC 6376 §3.4 has
    /// the verifier ignore the signature.
    Unknown(Range<usize>),
}

/// Decodes `value`, the `c=` value. Not written yet.
pub(crate) fn canonicalization(value: Option<&[u8]>) -> Result<Canonicalization, CanonError> {
    let _ = value;
    Err(CanonError::Syntax(0))
}

#[cfg(test)]
mod tests {
    use super::{Algorithm, CanonError, Canonicalization, canonicalization};

    /// The pair with `header` and `body`.
    fn pair(header: Algorithm, body: Algorithm) -> Canonicalization {
        Canonicalization { header, body }
    }

    #[test]
    fn tag_026_one_name_leaves_the_body_simple() {
        let expected = pair(Algorithm::Relaxed, Algorithm::Simple);
        assert_eq!(canonicalization(Some(b"relaxed")), Ok(expected));
    }

    #[test]
    fn tag_027_no_c_means_simple_simple() {
        let expected = pair(Algorithm::Simple, Algorithm::Simple);
        assert_eq!(canonicalization(None), Ok(expected));
    }

    #[test]
    fn tag_039_names_match_only_as_written() {
        let both = pair(Algorithm::Relaxed, Algorithm::Relaxed);
        assert_eq!(canonicalization(Some(b"relaxed/relaxed")), Ok(both));
        let mixed = pair(Algorithm::Simple, Algorithm::Relaxed);
        assert_eq!(canonicalization(Some(b"simple/relaxed")), Ok(mixed));
        let error = CanonError::Unknown(0..7);
        assert_eq!(canonicalization(Some(b"Relaxed")), Err(error));
        let error = CanonError::Unknown(7..12);
        assert_eq!(canonicalization(Some(b"simple/x-new")), Err(error));
        let error = CanonError::Unknown(0..3);
        assert_eq!(canonicalization(Some(b"Foo/Bar")), Err(error));
    }

    #[test]
    fn tag_048_a_c_value_is_one_or_two_words() {
        let cases: [(&[u8], usize); 9] = [
            (b"", 0),
            (b"relaxed/", 8),
            (b"/simple", 0),
            (b"relaxed /simple", 7),
            (b"relaxed/simple/simple", 14),
            (b"relaxed-/simple", 8),
            (b"x-", 2),
            (b"1relaxed", 0),
            (b"x/", 2),
        ];
        for (input, offset) in cases {
            let error = CanonError::Syntax(offset);
            assert_eq!(canonicalization(Some(input)), Err(error));
        }
    }
}
