//! The reader and the writer of the documents that traces, snapshots and demo
//! cases share (TRC-007): I-JSON texts (RFC 7493), written in one fixed form.
//! The reader also reads the JSON of DKIM2 recipes (draft-09 §2, §5).

// The trace recorder, which writes these documents, comes in a later change,
// and only the tests read the offsets of a JsonError, so parts of this module
// are unused outside the tests for now.
#![cfg_attr(
    not(test),
    expect(dead_code, reason = "its callers come in later changes")
)]

use std::collections::BTreeSet;

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

/// Whether `character` is a noncharacter of Unicode: U+FDD0 to U+FDEF, or
/// one of the last two code points of a plane, such as U+FFFE and U+FFFF.
/// I-JSON strings hold none (RFC 7493 §2.1).
pub(crate) fn is_noncharacter(character: char) -> bool {
    let value = u32::from(character);
    (0xFDD0..=0xFDEF).contains(&value) || (value & 0xFFFE) == 0xFFFE
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
    /// An object, its members in document order, no two with the same name
    /// (RFC 7493 §2.3, draft-09 §5).
    Object(Vec<(String, Json)>),
}

/// The JSON that [`read_as`] accepts. Both read UTF-8 text with no lone
/// surrogate and no name twice in an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dialect {
    /// I-JSON (RFC 7493), whose strings hold no noncharacter: traces,
    /// snapshots and demo cases (TRC-007).
    IJson,
    /// RFC 8259, as DKIM2 recipes use it (draft-09 §2), with the unique
    /// names and UTF-8 strings of draft-09 §5.
    Rfc8259,
}

/// Why [`read`] refused a document. Each offset counts bytes from the start
/// of the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsonError {
    /// The document isn't one JSON text of its dialect: the byte at this
    /// offset breaks RFC 8259, RFC 7493 §2 or draft-09 §5.
    Malformed { offset: usize },
    /// The array or object that opens at this offset is nested deeper than
    /// the limit (PAR-005).
    TooDeep { offset: usize },
}

/// Reads one I-JSON text (RFC 7493) from `bytes`, with arrays and objects
/// nested at most `depth_limit` levels, as [`read_as`] does.
pub(crate) fn read(bytes: &[u8], depth_limit: u32) -> Result<Json, JsonError> {
    read_as(bytes, depth_limit, Dialect::IJson)
}

/// Reads one JSON text of `dialect` from `bytes`, with arrays and objects
/// nested at most `depth_limit` levels. It reads left to right with a stack
/// of the open arrays and objects, and never recurses (PAR-005).
pub(crate) fn read_as(bytes: &[u8], depth_limit: u32, dialect: Dialect) -> Result<Json, JsonError> {
    // RFC 7493 §2.1 and RFC 8259 §8.1: the text is UTF-8.
    let text = match core::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            let offset = error.valid_up_to();
            return Err(JsonError::Malformed { offset });
        }
    };
    let mut reader = Reader {
        text,
        at: 0,
        open: Vec::new(),
        depth_limit,
        dialect,
    };
    loop {
        if let Some(value) = reader.value()?
            && let Some(document) = reader.close(value)?
        {
            return Ok(document);
        }
    }
}

/// An array or an object that is still open, with what it holds so far.
enum Open {
    Array(Vec<Json>),
    /// An object, the names of its members, and the name of the member whose
    /// value comes next.
    Object {
        members: Vec<(String, Json)>,
        names: BTreeSet<String>,
        name: String,
    },
}

/// The state of [`read`]: the text, the offset of the next byte, and the
/// open arrays and objects, innermost last.
struct Reader<'a> {
    text: &'a str,
    at: usize,
    open: Vec<Open>,
    depth_limit: u32,
    dialect: Dialect,
}

impl Reader<'_> {
    /// Reads the start of a value: a whole value, or `None` for an array or
    /// an object that holds something, which goes on the stack.
    fn value(&mut self) -> Result<Option<Json>, JsonError> {
        self.skip_space();
        let value = match self.peek() {
            Some(b'[') => return self.open_array(),
            Some(b'{') => return self.open_object(),
            Some(b'"') => Json::String(self.string()?),
            Some(b't') => self.literal("true", Json::Bool(true))?,
            Some(b'f') => self.literal("false", Json::Bool(false))?,
            Some(b'n') => self.literal("null", Json::Null)?,
            Some(b'-' | b'0'..=b'9') => self.number()?,
            _ => return Err(self.malformed()),
        };
        Ok(Some(value))
    }

    /// Hands a finished value to the open arrays and objects, closing each one
    /// that ends after it. Gives the document once nothing is left open.
    fn close(&mut self, mut value: Json) -> Result<Option<Json>, JsonError> {
        while let Some(open) = self.open.pop() {
            self.skip_space();
            let closed = match open {
                Open::Array(values) => self.add_item(values, value)?,
                Open::Object {
                    members,
                    names,
                    name,
                } => self.add_member(members, names, name, value)?,
            };
            match closed {
                Some(closed) => value = closed,
                None => return Ok(None),
            }
        }
        // RFC 8259 §2: white space may follow the text, and nothing else.
        self.skip_space();
        if self.at < self.text.len() {
            return Err(self.malformed());
        }
        Ok(Some(value))
    }

    /// Adds `item` to an open array. A comma keeps the array open, and `]`
    /// gives the finished array.
    fn add_item(&mut self, mut items: Vec<Json>, item: Json) -> Result<Option<Json>, JsonError> {
        items.push(item);
        match self.peek() {
            Some(b',') => {
                self.at += 1;
                self.open.push(Open::Array(items));
                Ok(None)
            }
            Some(b']') => {
                self.at += 1;
                Ok(Some(Json::Array(items)))
            }
            _ => Err(self.malformed()),
        }
    }

    /// Adds the member `name` with `value` to an open object. A comma and the
    /// next name keep the object open, and `}` gives the finished object.
    fn add_member(
        &mut self,
        mut members: Vec<(String, Json)>,
        mut names: BTreeSet<String>,
        name: String,
        value: Json,
    ) -> Result<Option<Json>, JsonError> {
        members.push((name, value));
        match self.peek() {
            Some(b',') => {
                self.at += 1;
                let name = self.name(&mut names)?;
                let open = Open::Object {
                    members,
                    names,
                    name,
                };
                self.open.push(open);
                Ok(None)
            }
            Some(b'}') => {
                self.at += 1;
                Ok(Some(Json::Object(members)))
            }
            _ => Err(self.malformed()),
        }
    }

    /// Opens the array at the cursor. An empty array is a finished value.
    fn open_array(&mut self) -> Result<Option<Json>, JsonError> {
        self.enter()?;
        self.skip_space();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(Some(Json::Array(Vec::new())));
        }
        self.open.push(Open::Array(Vec::new()));
        Ok(None)
    }

    /// Opens the object at the cursor and reads its first name. An empty
    /// object is a finished value.
    fn open_object(&mut self) -> Result<Option<Json>, JsonError> {
        self.enter()?;
        self.skip_space();
        if self.peek() == Some(b'}') {
            self.at += 1;
            return Ok(Some(Json::Object(Vec::new())));
        }
        let mut names = BTreeSet::new();
        let name = self.name(&mut names)?;
        let open = Open::Object {
            members: Vec::new(),
            names,
            name,
        };
        self.open.push(open);
        Ok(None)
    }

    /// Steps over the bracket that opens an array or an object, which nests
    /// one level deeper than the open ones (PAR-005).
    fn enter(&mut self) -> Result<(), JsonError> {
        let depth = self.open.len().saturating_add(1);
        let within = match u32::try_from(depth) {
            Ok(depth) => depth <= self.depth_limit,
            Err(_) => false,
        };
        if !within {
            return Err(JsonError::TooDeep { offset: self.at });
        }
        self.at += 1;
        Ok(())
    }

    /// Reads a member name and the colon after it. The name differs from the
    /// names before it in its object (RFC 7493 §2.3, draft-09 §5).
    fn name(&mut self, names: &mut BTreeSet<String>) -> Result<String, JsonError> {
        self.skip_space();
        let start = self.at;
        if self.peek() != Some(b'"') {
            return Err(self.malformed());
        }
        let name = self.string()?;
        if !names.insert(name.clone()) {
            return Err(JsonError::Malformed { offset: start });
        }
        self.skip_space();
        if self.peek() != Some(b':') {
            return Err(self.malformed());
        }
        self.at += 1;
        Ok(name)
    }

    /// Reads the string whose opening quote is at the cursor (RFC 8259 §7),
    /// which holds no surrogate, nor in I-JSON a noncharacter (RFC 7493
    /// §2.1).
    fn string(&mut self) -> Result<String, JsonError> {
        self.at += 1;
        let mut out = String::new();
        loop {
            let start = self.at;
            let character = match self.peek() {
                Some(b'"') => {
                    self.at += 1;
                    return Ok(out);
                }
                Some(b'\\') => self.escape()?,
                Some(0x20..=0xFF) => self.character()?,
                _ => return Err(self.malformed()),
            };
            if self.dialect == Dialect::IJson && is_noncharacter(character) {
                return Err(JsonError::Malformed { offset: start });
            }
            out.push(character);
        }
    }

    /// Reads the character at the cursor. The text is UTF-8, so a valid
    /// UTF-8 sequence starts there and it holds no surrogate.
    fn character(&mut self) -> Result<char, JsonError> {
        let Some(rest) = self.text.get(self.at..) else {
            return Err(self.malformed());
        };
        let Some(character) = rest.chars().next() else {
            return Err(self.malformed());
        };
        self.at += character.len_utf8();
        Ok(character)
    }

    /// Reads the escape whose backslash is at the cursor (RFC 8259 §7).
    fn escape(&mut self) -> Result<char, JsonError> {
        let start = self.at;
        self.at += 1;
        let character = match self.peek() {
            Some(b'"') => '"',
            Some(b'\\') => '\\',
            Some(b'/') => '/',
            Some(b'b') => '\x08',
            Some(b'f') => '\x0c',
            Some(b'n') => '\n',
            Some(b'r') => '\r',
            Some(b't') => '\t',
            Some(b'u') => return self.unicode_escape(start),
            _ => return Err(self.malformed()),
        };
        self.at += 1;
        Ok(character)
    }

    /// Reads the four hex digits of a six-character escape, and the escape
    /// of the low surrogate after them when they give a high surrogate. A
    /// surrogate that isn't half of such a pair breaks RFC 7493 §2.1 and
    /// draft-09 §5.
    fn unicode_escape(&mut self, start: usize) -> Result<char, JsonError> {
        self.at += 1;
        let first = self.code_unit()?;
        let mut value = first;
        if (0xD800..=0xDBFF).contains(&first) {
            let end = self.at + 2;
            if self.text.as_bytes().get(self.at..end) != Some(b"\x5cu".as_slice()) {
                return Err(JsonError::Malformed { offset: start });
            }
            self.at = end;
            let second = self.code_unit()?;
            if !(0xDC00..=0xDFFF).contains(&second) {
                return Err(JsonError::Malformed { offset: start });
            }
            value = 0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00);
        }
        match char::from_u32(value) {
            Some(character) => Ok(character),
            None => Err(JsonError::Malformed { offset: start }),
        }
    }

    /// Reads four hex digits as one UTF-16 code unit.
    fn code_unit(&mut self) -> Result<u32, JsonError> {
        let mut value = 0;
        for _ in 0..4 {
            let digit = match self.peek() {
                Some(byte @ b'0'..=b'9') => byte - b'0',
                Some(byte @ b'a'..=b'f') => byte - b'a' + 10,
                Some(byte @ b'A'..=b'F') => byte - b'A' + 10,
                _ => return Err(self.malformed()),
            };
            value = (value << 4) | u32::from(digit);
            self.at += 1;
        }
        Ok(value)
    }

    /// Reads the literal `word` (RFC 8259 §3).
    fn literal(&mut self, word: &str, value: Json) -> Result<Json, JsonError> {
        let end = self.at + word.len();
        if self.text.as_bytes().get(self.at..end) != Some(word.as_bytes()) {
            return Err(self.malformed());
        }
        self.at = end;
        Ok(value)
    }

    /// Reads a number (RFC 8259 §6): an optional minus, an integer part with
    /// no leading zero, an optional fraction and an optional exponent.
    fn number(&mut self) -> Result<Json, JsonError> {
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        match self.peek() {
            Some(b'0') => self.at += 1,
            Some(b'1'..=b'9') => self.digits()?,
            _ => return Err(self.malformed()),
        }
        if self.peek() == Some(b'.') {
            self.at += 1;
            self.digits()?;
        }
        if let Some(b'e' | b'E') = self.peek() {
            self.at += 1;
            if let Some(b'+' | b'-') = self.peek() {
                self.at += 1;
            }
            self.digits()?;
        }
        match self.text.get(start..self.at) {
            Some(text) => Ok(Json::Number(text.to_owned())),
            None => Err(JsonError::Malformed { offset: start }),
        }
    }

    /// Reads one or more digits.
    fn digits(&mut self) -> Result<(), JsonError> {
        let start = self.at;
        while let Some(b'0'..=b'9') = self.peek() {
            self.at += 1;
        }
        if self.at == start {
            return Err(self.malformed());
        }
        Ok(())
    }

    /// Steps over white space (RFC 8259 §2).
    fn skip_space(&mut self) {
        while let Some(b' ' | b'\t' | b'\n' | b'\r') = self.peek() {
            self.at += 1;
        }
    }

    /// The byte at the cursor, if any.
    fn peek(&self) -> Option<u8> {
        self.text.as_bytes().get(self.at).copied()
    }

    /// The error for the byte at the cursor.
    fn malformed(&self) -> JsonError {
        JsonError::Malformed { offset: self.at }
    }
}

#[cfg(test)]
mod tests {
    use super::{Dialect, Json, JsonError, read, read_as};

    /// Reads `bytes` as I-JSON with the default `document_depth`, 16.
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
    fn prs_008_strings_decode_escapes_and_refuse_bad_code_points() {
        // A backslash in these byte strings is written as \x5c.
        let text = b"\"\x5c\"\x5c\x5c\x5c/\x5cb\x5cf\x5cn\x5cr\x5ct\x5cu00e9\x5cuD83D\x5cuDE00\"";
        let expected = "\"\x5c/\x08\x0c\n\r\té😀".to_owned();
        assert_eq!(read_text(text), Ok(Json::String(expected)));
        let cases: [(&[u8], usize); 9] = [
            (b"\"\x5cudead\"", 1),
            (b"\"\x5cud83dx\"", 1),
            (b"\"\x5cud83d\x5cu0041\"", 1),
            (b"\"\x5cuffff\"", 1),
            (b"\"\xef\xbf\xbf\"", 1),
            (b"\"\x5ca\"", 2),
            (b"\"\x5cu12\"", 5),
            (b"\"a\x1fb\"", 2),
            (b"\"\xff\"", 1),
        ];
        for (text, offset) in cases {
            assert_eq!(read_text(text), malformed(offset));
        }
    }

    #[test]
    fn prs_009_names_in_one_object_differ() {
        assert_eq!(read_text(b"{\"a\":1,\"a\":2}"), malformed(7));
        assert_eq!(read_text(b"{\"a\":1,\"\x5cu0061\":2}"), malformed(7));
        let one = Json::Number("1".to_owned());
        let inner = Json::Object(vec![("a".to_owned(), one)]);
        let outer = Json::Object(vec![("a".to_owned(), inner)]);
        assert_eq!(read_text(b"{\"a\":{\"a\":1}}"), Ok(outer));
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

    #[test]
    fn prs_013_rfc_8259_text_may_hold_noncharacters() {
        // U+FFFF and U+FDD0 in UTF-8.
        let ffff = String::from_utf8(vec![0xEF, 0xBF, 0xBF]).unwrap();
        let fdd0 = String::from_utf8(vec![0xEF, 0xB7, 0x90]).unwrap();
        let cases: [(&[u8], String); 3] = [
            (b"\"\x5cuffff\"", ffff.clone()),
            (b"\"\xef\xbf\xbf\"", ffff),
            (b"\"\x5cufdd0\"", fdd0),
        ];
        for (text, expected) in cases {
            let found = read_as(text, 16, Dialect::Rfc8259);
            assert_eq!(found, Ok(Json::String(expected)));
        }
        let surrogate = read_as(b"\"\x5cudead\"", 16, Dialect::Rfc8259);
        assert_eq!(surrogate, malformed(1));
        let repeat = read_as(b"{\"a\":1,\"a\":2}", 16, Dialect::Rfc8259);
        assert_eq!(repeat, malformed(7));
    }
}
