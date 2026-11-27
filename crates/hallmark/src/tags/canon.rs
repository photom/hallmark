//! The DKIM `c=` value: the header and body canonicalization algorithms
//! (RFC 6376 §3.5, HDR-007).

#![cfg_attr(
    not(any(test, feature = "fuzzing")),
    expect(dead_code, reason = "DKIM comes later")
)]

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
    /// the body name. This is no syntax error: RFC 6376 §3.4 has the verifier
    /// ignore the signature.
    Unrecognized(Range<usize>),
}

/// Decodes `value`, the `c=` value, which is `None` when the signature has no
/// `c=` tag: `sig-c-tag-alg ["/" sig-c-tag-alg]`. The decoder checks the
/// syntax before it reads the names.
pub(crate) fn canonicalization(value: Option<&[u8]>) -> Result<Canonicalization, CanonError> {
    let Some(input) = value else {
        // RFC 6376 §3.5: c= defaults to simple/simple.
        return Ok(Canonicalization {
            header: Algorithm::Simple,
            body: Algorithm::Simple,
        });
    };
    let header_end = word(input, 0)?;
    let body_word = match input.get(header_end..) {
        Some([]) => None,
        Some([b'/', ..]) => Some(last_word(input, header_end + 1)?),
        _ => return Err(CanonError::Syntax(header_end)),
    };
    let header = algorithm(input, 0, header_end)?;
    let body = match body_word {
        // RFC 6376 §3.5: one name leaves the body simple.
        None => Algorithm::Simple,
        Some((start, end)) => algorithm(input, start, end)?,
    };
    Ok(Canonicalization { header, body })
}

/// Reads the hyphenated-word at `start`, which must end the value, and
/// returns its range.
fn last_word(input: &[u8], start: usize) -> Result<(usize, usize), CanonError> {
    let end = word(input, start)?;
    if end != input.len() {
        return Err(CanonError::Syntax(end));
    }
    Ok((start, end))
}

/// Reads the hyphenated-word at `start` and returns its end:
/// `ALPHA [ *(ALPHA / DIGIT / "-") (ALPHA / DIGIT) ]` (RFC 6376 §2.10).
fn word(input: &[u8], start: usize) -> Result<usize, CanonError> {
    let Some(b'a'..=b'z' | b'A'..=b'Z') = input.get(start).copied() else {
        return Err(CanonError::Syntax(start));
    };
    let mut end = start + 1;
    let mut word_end = end;
    loop {
        match input.get(end).copied() {
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9') => {
                end += 1;
                word_end = end;
            }
            Some(b'-') => end += 1,
            _ => break,
        }
    }
    // A word ends with a letter or a digit, so one must stand where the
    // trailing hyphens stop.
    if word_end != end {
        return Err(CanonError::Syntax(end));
    }
    Ok(end)
}

/// The algorithm that the name from `start` to `end` gives, compared as
/// written (ADR 0067).
fn algorithm(input: &[u8], start: usize, end: usize) -> Result<Algorithm, CanonError> {
    match input.get(start..end) {
        Some(b"simple") => Ok(Algorithm::Simple),
        Some(b"relaxed") => Ok(Algorithm::Relaxed),
        _ => Err(CanonError::Unrecognized(start..end)),
    }
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
        let error = CanonError::Unrecognized(0..7);
        assert_eq!(canonicalization(Some(b"Relaxed")), Err(error));
        let error = CanonError::Unrecognized(7..12);
        assert_eq!(canonicalization(Some(b"simple/x-new")), Err(error));
        let error = CanonError::Unrecognized(0..3);
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
