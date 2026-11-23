//! The writer of the fixed form that traces, snapshots and demo cases share
//! (TRC-007). The reader, with its size and depth limits (PAR-005), comes
//! with the first API that reads one of those documents.

// The trace recorder, which writes events, comes in a later change, so only
// the tests use this module for now.
#![cfg_attr(
    not(test),
    expect(dead_code, reason = "the trace recorder comes later")
)]

/// Appends `text` to `out` as a JSON string in the fixed form: escaped as
/// RFC 8785 §3.2.2.2 does, with every other code point as it is in UTF-8.
pub(crate) fn write_string(text: &str, out: &mut Vec<u8>) {
    out.push(b'"');
    for character in text.chars() {
        match character {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\u{8}' => out.extend_from_slice(b"\\b"),
            '\t' => out.extend_from_slice(b"\\t"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\u{c}' => out.extend_from_slice(b"\\f"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\u{0}'..='\u{1f}' => write_control(character, out),
            _ => {
                let mut buffer = [0; 4];
                let encoded = character.encode_utf8(&mut buffer);
                out.extend_from_slice(encoded.as_bytes());
            }
        }
    }
    out.push(b'"');
}

/// Appends a control character from U+0000 to U+001F as `\u00` and two
/// lowercase hex digits (RFC 8785 §3.2.2.2).
fn write_control(character: char, out: &mut Vec<u8>) {
    let value = u32::from(character);
    out.extend_from_slice(b"\\u00");
    out.push(hex_digit(value >> 4));
    out.push(hex_digit(value & 0x0F));
}

/// The lowercase hex digit of a value from 0 to 15.
fn hex_digit(nibble: u32) -> u8 {
    match nibble {
        0 => b'0',
        1 => b'1',
        2 => b'2',
        3 => b'3',
        4 => b'4',
        5 => b'5',
        6 => b'6',
        7 => b'7',
        8 => b'8',
        9 => b'9',
        10 => b'a',
        11 => b'b',
        12 => b'c',
        13 => b'd',
        14 => b'e',
        // The callers pass four bits, so this arm is 15.
        _ => b'f',
    }
}
