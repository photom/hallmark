//! Entry points for the fuzz targets in `fuzz/` (SAF-003, ADR 0069). The `fuzzing` feature builds
//! this module for those targets alone: it isn't part of the API, and any release can change it.
//! Each entry point runs the parsers of one area on its input and returns whether every result
//! keeps the promises of those parsers. It never panics (ADR 0004). The fuzz target asserts what
//! it returns, and a panic, an overflow or a loop that never ends shows in the target itself.

use core::ops::Range;

use crate::Limits;
use crate::base64::{self, Base64Error};
use crate::tags::canon::{self, Algorithm, CanonError, Canonicalization};
use crate::tags::fields::{self, CopiedField, FieldsError};
use crate::tags::groups::{self, GroupsError, HashValues, SignatureValue};
use crate::tags::number::{self, Digits, Number, NumberError};
use crate::tags::qp::{self, QpError};
use crate::tags::recipe::{self, RecipeError};
use crate::tags::{Grammar, TagList, TagListError};

/// The grammars of a tag list (HDR-006).
const GRAMMARS: [Grammar; 3] = [Grammar::Dkim, Grammar::DkimUtf8, Grammar::Dkim2];

/// The rules of the numeric tags (HDR-013).
const DIGITS: [Digits; 3] = [Digits::DkimTime, Digits::DkimLength, Digits::Dkim2];

/// Runs the tag=value list parser under each grammar, and every value decoder, on `input`
/// (TAG-060). Returns whether each error offset and each range lies within `input`, and whether
/// each decoded value is no longer than `input`.
pub fn tags(input: &[u8]) -> bool {
    let mut kept = true;
    for grammar in GRAMMARS {
        kept &= tag_list(input, grammar);
        kept &= quoted_printable(input, grammar);
        kept &= copies(input, grammar);
    }
    for digits in DIGITS {
        kept &= decimal(input, digits);
    }
    kept &= base64_value(input);
    kept &= names(input);
    kept &= canonicalization(input);
    kept &= hashes(input);
    kept &= signatures(input);
    kept &= recipes(input, &Limits::default());
    // Small limits, so that short inputs reach the size and depth errors too.
    let small = Limits {
        recipe_size: 64,
        recipe_depth: 2,
        ..Limits::default()
    };
    kept &= recipes(input, &small);
    kept
}

/// Whether `range` runs forward and ends within `input`.
fn range_within(range: &Range<usize>, input: &[u8]) -> bool {
    range.start <= range.end && range.end <= input.len()
}

/// The tag-list parser (HDR-006).
fn tag_list(input: &[u8], grammar: Grammar) -> bool {
    match TagList::parse(input, grammar) {
        Ok(list) => {
            for tag in list.tags() {
                if !(range_within(&tag.name, input) && range_within(&tag.value, input)) {
                    return false;
                }
            }
            true
        }
        Err(TagListError::Syntax(_rule, offset)) => offset <= input.len(),
        Err(TagListError::Duplicate(name)) => range_within(&name, input),
    }
}

/// The offset of a base64 error.
fn base64_offset(error: Base64Error) -> usize {
    match error {
        Base64Error::Syntax(at) | Base64Error::Padding(at) | Base64Error::PadBits(at) => at,
    }
}

/// The offset of a dkim-quoted-printable error.
fn qp_offset(error: QpError) -> usize {
    match error {
        QpError::HexOctet(at) | QpError::Char(at) => at,
    }
}

/// The offset of an `h=` or `z=` error.
fn fields_offset(error: FieldsError) -> usize {
    match error {
        FieldsError::Name(at) | FieldsError::Separator(at) | FieldsError::Colon(at) => at,
        FieldsError::Value(error) => qp_offset(error),
    }
}

/// The base64 decoder (ADR 0066).
fn base64_value(input: &[u8]) -> bool {
    match base64::decode(input) {
        Ok(octets) => octets.len() <= input.len(),
        Err(error) => base64_offset(error) <= input.len(),
    }
}

/// The dkim-quoted-printable decoder (RFC 6376 §2.11).
fn quoted_printable(input: &[u8], grammar: Grammar) -> bool {
    match qp::decode(input, grammar) {
        Ok(value) => value.len() <= input.len(),
        Err(error) => qp_offset(error) <= input.len(),
    }
}

/// The names of DKIM `h=` (RFC 6376 §3.5).
fn names(input: &[u8]) -> bool {
    match fields::names(input) {
        Ok(names) => {
            for name in &names {
                if !range_within(name, input) {
                    return false;
                }
            }
            true
        }
        Err(error) => fields_offset(error) <= input.len(),
    }
}

/// The copies of DKIM `z=` (RFC 6376 §3.5).
fn copies(input: &[u8], grammar: Grammar) -> bool {
    match fields::copies(input, grammar) {
        Ok(copies) => {
            for CopiedField { name, value } in copies {
                if !range_within(&name, input) || value.len() > input.len() {
                    return false;
                }
            }
            true
        }
        Err(error) => fields_offset(error) <= input.len(),
    }
}

/// The DKIM `c=` decoder, which gives `simple/simple` with no value (RFC 6376 §3.5).
fn canonicalization(input: &[u8]) -> bool {
    let default = Canonicalization {
        header: Algorithm::Simple,
        body: Algorithm::Simple,
    };
    if canon::canonicalization(None) != Ok(default) {
        return false;
    }
    match canon::canonicalization(Some(input)) {
        Ok(Canonicalization {
            header: _header,
            body: _body,
        }) => true,
        Err(CanonError::Syntax(at)) => at <= input.len(),
        Err(CanonError::Unrecognized(name)) => range_within(&name, input),
    }
}

/// The decimal decoder under the rule `digits` (HDR-013).
fn decimal(input: &[u8], digits: Digits) -> bool {
    match number::decimal(input, digits) {
        Ok(Number::Finite(_value)) => true,
        Ok(Number::Infinite) => true,
        Err(NumberError(at)) => at <= input.len(),
    }
}

/// Whether a hash set of DKIM2 `h=` lies within `input`.
fn hash_set_within(set: HashValues, input: &[u8]) -> bool {
    let HashValues {
        algorithm,
        header,
        body,
    } = set;
    range_within(&algorithm, input) && header.len() <= input.len() && body.len() <= input.len()
}

/// Whether a signature set of DKIM2 `s=` lies within `input`.
fn signature_set_within(set: SignatureValue, input: &[u8]) -> bool {
    let SignatureValue {
        selector,
        algorithm,
        signature,
    } = set;
    let ranges = range_within(&selector, input) && range_within(&algorithm, input);
    ranges && signature.len() <= input.len()
}

/// Whether a DKIM2 group error lies within `input`.
fn groups_error_within(error: GroupsError, input: &[u8]) -> bool {
    match error {
        GroupsError::Algorithm(at)
        | GroupsError::Selector(at)
        | GroupsError::Colon(at)
        | GroupsError::Separator(at) => at <= input.len(),
        GroupsError::Value(error) => base64_offset(error) <= input.len(),
        GroupsError::Repeat(name) => range_within(&name, input),
    }
}

/// The groups of DKIM2 `h=` (draft-09 §9.3).
fn hashes(input: &[u8]) -> bool {
    match groups::hashes(input) {
        Ok(groups) => {
            for set in groups.into_iter().flatten() {
                if !hash_set_within(set, input) {
                    return false;
                }
            }
            true
        }
        Err(error) => groups_error_within(error, input),
    }
}

/// The groups of DKIM2 `s=` (draft-09 §10.9).
fn signatures(input: &[u8]) -> bool {
    match groups::signatures(input) {
        Ok(groups) => {
            for set in groups.into_iter().flatten() {
                if !signature_set_within(set, input) {
                    return false;
                }
            }
            true
        }
        Err(error) => groups_error_within(error, input),
    }
}

/// The recipes of DKIM2 `r=` within `limits` (draft-09 §9.2). The decoded JSON is shorter than
/// `input`, so each offset into it and its size must lie within the length of `input`.
fn recipes(input: &[u8], limits: &Limits) -> bool {
    match recipe::recipes(input, limits) {
        Ok(members) => members.len() <= input.len(),
        Err(RecipeError::Value(error)) => base64_offset(error) <= input.len(),
        Err(RecipeError::Size(size)) => size <= input.len(),
        Err(RecipeError::Json(at) | RecipeError::Depth(at) | RecipeError::Object(at)) => {
            at <= input.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::tags;

    #[test]
    fn tag_060_every_decoder_keeps_its_results_within_the_input() {
        let cases: [&[u8]; 8] = [
            b"",
            b"v=1; a=rsa-sha256",
            b"a=\r\n \r\n ;",
            b"sha256:AAAA:BBBB|x:CCCC:DDDD",
            b"one:rsa-sha256:AAAA,two:x:BBBB",
            b"eyJiIjpbW11dfQ==",
            b"Subject:a=2\r\n 0|=",
            b"\xff\xfe=\r\n\r\n;;||::",
        ];
        for input in cases {
            assert!(tags(input));
        }
    }
}
