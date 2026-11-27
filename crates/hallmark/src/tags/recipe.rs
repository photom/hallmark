//! The recipes of a Message-Instance `r=` value (draft-09 §9.2): the base64
//! of a JSON object (§5), read within `recipe_size` and `recipe_depth`
//! (SAF-002). The DKIM2 check reads its keys and steps.

#![cfg_attr(
    not(any(test, feature = "fuzzing")),
    expect(dead_code, reason = "DKIM2 comes later")
)]

use crate::Limits;
use crate::base64::{self, Base64Error};
use crate::json::{self, Dialect, Json, JsonError};

/// Why an `r=` value doesn't decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecipeError {
    /// The value breaks a rule of the base64 decoder, at an offset of the
    /// value as written.
    Value(Base64Error),
    /// The decoded JSON holds this many octets, more than `recipe_size`.
    Size(usize),
    /// The decoded JSON breaks RFC 8259 or draft-09 §5 at this offset of the
    /// decoded octets.
    Json(usize),
    /// An array or object opens at this offset of the decoded octets deeper
    /// than `recipe_depth` (PAR-005).
    Depth(usize),
    /// The JSON text is a value other than an object (§9.2), which starts at
    /// this offset of the decoded octets.
    Object(usize),
}

/// The members of the JSON object that the `r=` value `input` encodes, in
/// order. The size is checked before the JSON is read, so no text over
/// `recipe_size` reaches the reader.
pub(crate) fn recipes(input: &[u8], limits: &Limits) -> Result<Vec<(String, Json)>, RecipeError> {
    let octets = match base64::decode(input) {
        Ok(octets) => octets,
        Err(error) => return Err(RecipeError::Value(error)),
    };
    let within = match u32::try_from(octets.len()) {
        Ok(size) => size <= limits.recipe_size,
        Err(_) => false,
    };
    if !within {
        return Err(RecipeError::Size(octets.len()));
    }
    match json::read_as(&octets, limits.recipe_depth, Dialect::Recipe) {
        Ok(Json::Object(members)) => Ok(members),
        Ok(_) => Err(RecipeError::Object(value_start(&octets))),
        Err(JsonError::Malformed { offset }) => Err(RecipeError::Json(offset)),
        Err(JsonError::TooDeep { offset }) => Err(RecipeError::Depth(offset)),
    }
}

/// The offset of the JSON value in `octets`, after the white space before it
/// (RFC 8259 §2).
fn value_start(octets: &[u8]) -> usize {
    let mut at = 0;
    while let Some(b' ' | b'\t' | b'\r' | b'\n') = octets.get(at).copied() {
        at += 1;
    }
    at
}

#[cfg(test)]
mod tests {
    use super::{Base64Error, Json, Limits, RecipeError, recipes};

    /// The JSON number `text`.
    fn number(text: &str) -> Json {
        Json::Number(text.to_owned())
    }

    #[test]
    fn tag_046_a_recipe_is_a_json_object() {
        let limits = Limits::default();
        let numbers = Json::Array(vec![number("1"), number("2")]);
        let step = Json::Object(vec![("c".to_owned(), numbers)]);
        let expected = vec![("b".to_owned(), Json::Array(vec![step]))];
        let found = recipes(b"eyJiIjpbeyJjIjpbMSwyXX1dfQ==", &limits);
        assert_eq!(found, Ok(expected));
        // U+FFFF in UTF-8.
        let ffff = String::from_utf8(vec![0xEF, 0xBF, 0xBF]).unwrap();
        let expected = vec![("a".to_owned(), Json::String(ffff))];
        let found = recipes(b"eyJhIjoiXHVmZmZmIn0=", &limits);
        assert_eq!(found, Ok(expected));
    }

    #[test]
    fn tag_055_a_recipe_is_valid_json_within_the_limits() {
        let limits = Limits::default();
        let padding = RecipeError::Value(Base64Error::Padding(7));
        assert_eq!(recipes(b"eyJiIjo", &limits), Err(padding));
        assert_eq!(recipes(b"eyJiIjo=", &limits), Err(RecipeError::Json(5)));
        assert_eq!(recipes(b"WzFd", &limits), Err(RecipeError::Object(0)));
        let small = Limits {
            recipe_size: 1,
            ..Limits::default()
        };
        assert_eq!(recipes(b"e30=", &small), Err(RecipeError::Size(2)));
        assert_eq!(recipes(b"eyI=", &small), Err(RecipeError::Size(2)));
        let shallow = Limits {
            recipe_depth: 2,
            ..Limits::default()
        };
        let found = recipes(b"eyJiIjpbW11dfQ==", &shallow);
        assert_eq!(found, Err(RecipeError::Depth(6)));
        let found = recipes(b"eyJiIjpbWw==", &shallow);
        assert_eq!(found, Err(RecipeError::Depth(6)));
    }
}
