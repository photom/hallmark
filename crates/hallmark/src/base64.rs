//! base64 and base64url (RFC 4648): the decoder of DKIM and DKIM2 values,
//! with their folding white space rules (HDR-007), and the base64url encoder
//! of the trace format.

// The trace recorder, which writes data values, and the DKIM and DKIM2
// checks, which decode them, come in later changes, so only the tests use
// this module for now.
#![cfg_attr(
    not(test),
    expect(dead_code, reason = "the trace recorder and the checks come later")
)]

/// Why a base64string didn't decode (ADR 0066), with the offset of the first
/// octet that broke the rule (PAR-001). Offsets count from the start of the
/// base64string as written, so a caller that decodes a tag value adds the
/// value's start to name the octet in the field or record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Base64Error {
    /// An octet that the base64string rule doesn't allow there (RFC 6376
    /// §2.10, draft-09 §2.14), or the end of a value with no symbol.
    Syntax(usize),
    /// A surplus `=`, a last group of one symbol, which holds less than an
    /// octet, or a missing `=`, which the error places at the end of the value
    /// (RFC 4648 §3.2, §4).
    Padding(usize),
    /// A pad bit of the last symbol that isn't zero (RFC 4648 §3.5).
    PadBits(usize),
}

/// Decodes the base64string `input` (RFC 6376 §2.10, draft-09 §2.14), which
/// is a tag value as written, folds included. One FWS between two symbols
/// and one before each `=` are ignored. The value must be canonical: padded
/// to whole groups of four symbols, with pad bits of zero (ADR 0066). The
/// decoder checks the symbols and the FWS, then the padding, then the pad
/// bits, and the first check that fails gives the error.
pub(crate) fn decode(input: &[u8]) -> Result<Vec<u8>, Base64Error> {
    if symbol_at(input, 0).is_none() {
        return Err(Base64Error::Syntax(0));
    }
    let mut octets = Vec::with_capacity(input.len() / 4 * 3 + 2);
    // The symbols of the group being read, and their bits.
    let mut filled = 0;
    let mut bits: u32 = 0;
    // The offset of the last symbol, which a padding or pad-bit error names.
    let mut last = 0;
    // The end of the last symbol or `=`, before the FWS that may follow it.
    let mut end = 0;
    let mut at = 0;
    while let Some(value) = symbol_at(input, at) {
        bits = (bits << 6) | u32::from(value);
        filled += 1;
        if filled == 4 {
            let [_, first, second, third] = bits.to_be_bytes();
            octets.extend_from_slice(&[first, second, third]);
            filled = 0;
            bits = 0;
        }
        last = at;
        end = at + 1;
        at = fws(input, end)?;
    }
    // RFC 4648 §4: a last group of two symbols ends with two `=`, one of
    // three symbols with one `=`, and a whole group with none. No padding
    // completes a group of one symbol.
    let pads = match filled {
        0 => Some(0),
        2 => Some(2),
        3 => Some(1),
        _ => None,
    };
    // Up to two `=`, with an FWS before each (RFC 6376 §2.10).
    let mut seen = 0;
    let mut surplus = None;
    while let Some(b'=') = input.get(at).copied() {
        if pads == Some(seen) {
            surplus = Some(at);
        }
        seen += 1;
        end = at + 1;
        if seen == 2 {
            at = end;
            break;
        }
        at = fws(input, end)?;
    }
    if at < input.len() {
        return Err(Base64Error::Syntax(at));
    }
    if at != end {
        // FWS after the last symbol or `=`.
        return Err(Base64Error::Syntax(end));
    }
    let Some(pads) = pads else {
        return Err(Base64Error::Padding(last));
    };
    if let Some(offset) = surplus {
        return Err(Base64Error::Padding(offset));
    }
    if seen < pads {
        return Err(Base64Error::Padding(at));
    }
    // RFC 4648 §3.5: the bits after the last octet are zero.
    match pads {
        2 if bits & 0x0F == 0 => {
            let [_, _, _, octet] = (bits >> 4).to_be_bytes();
            octets.push(octet);
        }
        1 if bits & 0x03 == 0 => {
            let [_, _, first, second] = (bits >> 2).to_be_bytes();
            octets.extend_from_slice(&[first, second]);
        }
        0 => {}
        _ => return Err(Base64Error::PadBits(last)),
    }
    Ok(octets)
}

/// The six-bit value of the symbol at `at` in the base64 alphabet (RFC 4648
/// §4, Table 1), if a symbol is there.
fn symbol_at(input: &[u8], at: usize) -> Option<u8> {
    let octet = *input.get(at)?;
    match octet {
        b'A'..=b'Z' => Some(octet - b'A'),
        b'a'..=b'z' => Some(octet - b'a' + 26),
        b'0'..=b'9' => Some(octet - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Skips the FWS at `at`, if one is there, and returns the offset after it.
/// An FWS is `[*WSP CRLF] 1*WSP`, RFC 5322's without obs-FWS (RFC 6376
/// §2.8, draft-09 §2.12), so it holds one CRLF at most, and a CRLF folds
/// only before WSP.
fn fws(input: &[u8], at: usize) -> Result<usize, Base64Error> {
    let end = wsp(input, at);
    let Some([b'\r', b'\n']) = input.get(end..end + 2) else {
        return Ok(end);
    };
    let after = wsp(input, end + 2);
    if after == end + 2 {
        return Err(Base64Error::Syntax(end));
    }
    Ok(after)
}

/// Skips the WSP at `at` and returns the offset after it.
fn wsp(input: &[u8], mut at: usize) -> usize {
    while let Some(b' ' | b'\t') = input.get(at).copied() {
        at += 1;
    }
    at
}

/// Appends `bytes` to `out` in base64url without padding (RFC 4648 §5,
/// §3.2), as the trace format writes data that isn't text (TRC-007).
pub(crate) fn encode_url_unpadded(bytes: &[u8], out: &mut Vec<u8>) {
    let (groups, remainder) = bytes.as_chunks::<3>();
    for &[first, second, third] in groups {
        out.push(url_symbol(first >> 2));
        out.push(url_symbol(((first & 0x03) << 4) | (second >> 4)));
        out.push(url_symbol(((second & 0x0F) << 2) | (third >> 6)));
        out.push(url_symbol(third & 0x3F));
    }
    // RFC 4648 §4: a final group of one or two octets gives two or three
    // symbols, and the padding that would follow them is left out (§3.2).
    match *remainder {
        [first] => {
            out.push(url_symbol(first >> 2));
            out.push(url_symbol((first & 0x03) << 4));
        }
        [first, second] => {
            out.push(url_symbol(first >> 2));
            out.push(url_symbol(((first & 0x03) << 4) | (second >> 4)));
            out.push(url_symbol((second & 0x0F) << 2));
        }
        // as_chunks leaves fewer than three octets, so the remainder here is
        // empty.
        _ => {}
    }
}

/// The symbol of a six-bit value in the base64url alphabet (RFC 4648 §5,
/// Table 2).
fn url_symbol(value: u8) -> u8 {
    match value {
        0..=25 => b'A' + value,
        26..=51 => b'a' + (value - 26),
        52..=61 => b'0' + (value - 52),
        62 => b'-',
        // The callers mask the value to six bits, so this arm is 63.
        _ => b'_',
    }
}

#[cfg(test)]
mod tests {
    use super::{Base64Error, decode, encode_url_unpadded};

    #[test]
    fn tag_031_fws_inside_a_value_is_ignored() {
        let cases: [(&[u8], &[u8]); 5] = [
            (b"QQ==", b"A"),
            (b"QUI=", b"AB"),
            (b"QUJD", b"ABC"),
            (b"QU\r\n\tJD", b"ABC"),
            (b"QQ \r\n =\t=", b"A"),
        ];
        for (input, octets) in cases {
            assert_eq!(decode(input), Ok(octets.to_vec()));
        }
        // base64url writes the same octets with `-` and `_` for `+` and `/`.
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let url = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut written = Vec::new();
        encode_url_unpadded(&decode(alphabet).unwrap(), &mut written);
        assert_eq!(written, url);
    }

    #[test]
    fn tag_032_padding_completes_the_last_group() {
        let cases: [(&[u8], usize); 6] = [
            (b"QQ", 2),
            (b"QUI", 3),
            (b"QQ=", 3),
            (b"QUJDR", 4),
            (b"QUJD=", 4),
            (b"QUI==", 4),
        ];
        for (input, offset) in cases {
            assert_eq!(decode(input), Err(Base64Error::Padding(offset)));
        }
    }

    #[test]
    fn tag_033_pad_bits_are_zero() {
        let cases: [(&[u8], usize); 2] = [(b"QR==", 1), (b"QUJ=", 2)];
        for (input, offset) in cases {
            assert_eq!(decode(input), Err(Base64Error::PadBits(offset)));
        }
    }

    #[test]
    fn tag_034_a_value_follows_the_base64string_rule() {
        let cases: [(&[u8], usize); 10] = [
            (b"", 0),
            (b" QQ==", 0),
            (b"QU-I", 2),
            (b"QQ=Q", 3),
            (b"QUJDR-", 5),
            (b"QUJD===", 6),
            (b"QQ== ", 4),
            (b"QQ== =", 4),
            (b"QQ==\r\n", 4),
            (b"QU\r\n \r\n I=", 5),
        ];
        for (input, offset) in cases {
            assert_eq!(decode(input), Err(Base64Error::Syntax(offset)));
        }
    }
}
