//! The reader and the writer of the documents that traces, snapshots and demo
//! cases share (TRC-007): I-JSON texts (RFC 7493), written in one fixed form.

// sort_trace and the trace recorder, which read and write these documents,
// come in later changes, so only the tests use this module for now.
#![cfg_attr(
    not(test),
    expect(dead_code, reason = "its callers come in later changes")
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

/// A JSON value (RFC 8259 §3) that [`read`] took from a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Json {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number, as the text that matched the grammar of RFC 8259 §6. The
    /// reader of each document converts it to the type its member takes.
    Number(String),
    /// A string, with its escapes decoded.
    String(String),
    /// An array, its values in document order.
    Array(Vec<Json>),
    /// An object, its members in document order.
    Object(Vec<(String, Json)>),
}

/// Why [`read`] refused a document. Each offset counts bytes from the start
/// of the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsonError {
    /// The document isn't one I-JSON text: the byte at this offset breaks
    /// RFC 8259 or RFC 7493 §2.
    Malformed { offset: usize },
    /// The array or object that opens at this offset is nested deeper than
    /// the limit (PAR-005).
    TooDeep { offset: usize },
}

/// Reads one I-JSON text (RFC 7493) from `bytes`, with arrays and objects
/// nested at most `depth_limit` levels. It reads left to right with a stack
/// of the open arrays and objects, and never recurses (PAR-005).
pub(crate) fn read(bytes: &[u8], depth_limit: u32) -> Result<Json, JsonError> {
    let _ = (bytes, depth_limit);
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reads `bytes` with the default `document_depth`, 16.
    fn read_text(bytes: &[u8]) -> Result<Json, JsonError> {
        read(bytes, 16)
    }

    /// What `read` gives for a text that breaks at `offset`.
    fn malformed(offset: usize) -> Result<Json, JsonError> {
        Err(JsonError::Malformed { offset })
    }

    /// Arrays nested `levels` deep.
    fn nested(levels: usize) -> Vec<u8> {
        let mut text = vec![b'['; levels];
        text.resize(levels * 2, b']');
        text
    }

    #[test]
    fn prs_007_values_keep_their_order_and_numbers_their_text() {
        let text = b" {\"a\":[1,-0.5e+3,true,false,null],\"b\":{},\"c\":\"x\"}\r\n";
        let values = vec![
            Json::Number("1".to_owned()),
            Json::Number("-0.5e+3".to_owned()),
            Json::Bool(true),
            Json::Bool(false),
            Json::Null,
        ];
        let members = vec![
            ("a".to_owned(), Json::Array(values)),
            ("b".to_owned(), Json::Object(Vec::new())),
            ("c".to_owned(), Json::String("x".to_owned())),
        ];
        assert_eq!(read_text(text), Ok(Json::Object(members)));
    }

    #[test]
    fn prs_010_text_outside_the_grammar_is_refused() {
        let cases: [(&[u8], usize); 10] = [
            (b"01", 1),
            (b"1.", 2),
            (b"-", 1),
            (b"1e+", 3),
            (b"[1,]", 3),
            (b"tru", 0),
            (br#"{"a" 1}"#, 5),
            (br#"{"a":1} x"#, 8),
            (b"", 0),
            (b"\xef\xbb\xbf{}", 0),
        ];
        for (text, offset) in cases {
            assert_eq!(read_text(text), malformed(offset));
        }
    }

    #[test]
    fn prs_011_nesting_stops_at_the_depth_limit() {
        assert!(read_text(&nested(16)).is_ok());
        let too_deep = Err(JsonError::TooDeep { offset: 16 });
        assert_eq!(read_text(&nested(17)), too_deep);
        assert_eq!(read_text(&nested(100_000)), too_deep);
    }
}
