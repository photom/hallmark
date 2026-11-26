//! The dkim-quoted-printable decoder (RFC 6376 §2.11, HDR-007), with the
//! unencoded UTF-8 that RFC 8616 §5 allows in a message with
//! internationalized header fields.

#![cfg_attr(not(test), expect(dead_code, reason = "DKIM comes later"))]

use super::Grammar;

/// Why a dkim-quoted-printable value doesn't decode. Offsets count from the
/// start of the input as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QpError {
    /// hex-octet: an `=` without two uppercase hex digits after it, at the
    /// first octet that isn't one, or at the end of the input when it ends
    /// first.
    HexOctet(usize),
    /// dkim-quoted-printable: an octet that is neither FWS, nor `=`, nor a
    /// dkim-safe-char, or that starts no well-formed UTF-8 character.
    Char(usize),
}

/// Decodes `input`, a whole dkim-quoted-printable value. Not written yet.
pub(crate) fn decode(input: &[u8], grammar: Grammar) -> Result<Vec<u8>, QpError> {
    token(input, 0, grammar, &mut Vec::new())?;
    Ok(Vec::new())
}

/// Reads the token at `at`. Not written yet.
pub(super) fn token(
    input: &[u8],
    at: usize,
    grammar: Grammar,
    value: &mut Vec<u8>,
) -> Result<usize, QpError> {
    let _ = (input, grammar, value);
    Err(QpError::Char(at))
}

#[cfg(test)]
mod tests {
    use super::{Grammar, QpError, decode};

    #[test]
    fn tag_024_and_025_the_decoder_keeps_well_formed_utf8() {
        let value = decode(b"caf\xC3\xA9", Grammar::DkimUtf8);
        assert_eq!(value, Ok(b"caf\xC3\xA9".to_vec()));
        let error = QpError::Char(2);
        assert_eq!(decode(b"\xC3\xA9\xFF", Grammar::DkimUtf8), Err(error));
    }

    #[test]
    fn tag_036_fws_stands_between_tokens_only() {
        let error = QpError::HexOctet(3);
        let input = b"a=2\r\n 0";
        assert_eq!(decode(input, Grammar::Dkim), Err(error));
        // A CRLF that no WSP follows is no FWS.
        let error = QpError::Char(1);
        assert_eq!(decode(b"a\r\nb", Grammar::Dkim), Err(error));
    }
}
