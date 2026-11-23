//! base64 and base64url (RFC 4648). The decoder, with the folding white space
//! rules of DKIM and DKIM2 (HDR-007), comes with the parsers that read it.

// The trace recorder, which writes data values, comes in a later change, so
// only the tests use this module for now.
#![cfg_attr(
    not(test),
    expect(dead_code, reason = "the trace recorder comes later")
)]

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
