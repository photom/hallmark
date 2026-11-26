//! The dkim-quoted-printable decoder (RFC 6376 §2.11, HDR-007), with the
//! unencoded UTF-8 that RFC 8616 §5 allows in a message with
//! internationalized header fields.

#![cfg_attr(not(test), expect(dead_code, reason = "DKIM comes later"))]

use super::{Grammar, utf8};

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

/// Decodes `input`, a whole dkim-quoted-printable value. FWS isn't part of
/// the value (RFC 6376 §2.11). A dkim-safe-char is printable US-ASCII except
/// `;` and `=`, and under [`Grammar::DkimUtf8`] also well-formed non-ASCII
/// UTF-8 (RFC 8616 §5, RFC 3629 §4). Only DKIM tags use this encoding, and
/// [`Grammar::Dkim2`] keeps values ASCII as [`Grammar::Dkim`] does.
pub(crate) fn decode(input: &[u8], grammar: Grammar) -> Result<Vec<u8>, QpError> {
    let mut value = Vec::new();
    let mut at = 0;
    while at < input.len() {
        at = token(input, at, grammar, &mut value)?;
    }
    Ok(value)
}

/// Reads the token at `at`, adds the octets it decodes to `value`, and
/// returns the offset after it. A token is WSP or a CRLF and the WSP after
/// it, which add nothing, a hex-octet, a dkim-safe-char, or a run of
/// unencoded UTF-8, which adds the whole run. Runs of these tokens are
/// exactly the runs of FWS that RFC 6376 §2.8 allows.
pub(super) fn token(
    input: &[u8],
    at: usize,
    grammar: Grammar,
    value: &mut Vec<u8>,
) -> Result<usize, QpError> {
    match input.get(at..) {
        Some([b' ' | b'\t', ..]) => Ok(at + 1),
        Some([b'\r', b'\n', b' ' | b'\t', ..]) => Ok(at + 3),
        Some([b'=', ..]) => hex_octet(input, at + 1, value),
        Some([octet @ (0x21..=0x3A | 0x3C | 0x3E..=0x7E), ..]) => {
            value.push(*octet);
            Ok(at + 1)
        }
        Some([0x80..=0xFF, ..]) if grammar == Grammar::DkimUtf8 => unencoded(input, at, value),
        _ => Err(QpError::Char(at)),
    }
}

/// Reads the two uppercase hex digits at `at`, after an `=`, and adds the
/// octet they encode to `value`.
fn hex_octet(input: &[u8], at: usize, value: &mut Vec<u8>) -> Result<usize, QpError> {
    let Some(high) = hex_digit(input, at) else {
        return Err(QpError::HexOctet(at));
    };
    let Some(low) = hex_digit(input, at + 1) else {
        return Err(QpError::HexOctet(at + 1));
    };
    value.push((high << 4) | low);
    Ok(at + 2)
}

/// The value of the hex digit at `at`, from the alphabet `0123456789ABCDEF`
/// (RFC 6376 §2.11, RFC 2045 §6.7).
fn hex_digit(input: &[u8], at: usize) -> Option<u8> {
    match input.get(at).copied() {
        Some(digit @ b'0'..=b'9') => Some(digit - b'0'),
        Some(digit @ b'A'..=b'F') => Some(digit - b'A' + 10),
        _ => None,
    }
}

/// Adds the run of non-ASCII octets at `at` to `value`, if it is well-formed
/// UTF-8, and returns its end.
fn unencoded(input: &[u8], at: usize, value: &mut Vec<u8>) -> Result<usize, QpError> {
    let end = match utf8(input, at) {
        Ok(end) => end,
        Err(offset) => return Err(QpError::Char(offset)),
    };
    let Some(run) = input.get(at..end) else {
        return Err(QpError::Char(at));
    };
    value.extend_from_slice(run);
    Ok(end)
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
        let error = QpError::Char(1);
        assert_eq!(decode(b"a\r\nb", Grammar::Dkim), Err(error));
    }
}
