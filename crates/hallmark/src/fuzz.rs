//! Entry points for the fuzz targets in `fuzz/` (SAF-003, ADR 0069). The `fuzzing` feature builds
//! this module for those targets alone: it isn't part of the API, and any release can change it.
//! Each entry point runs the parsers of one area on its input and returns whether every result
//! keeps the promises of those parsers. It never panics (ADR 0004). The fuzz target asserts what
//! it returns, and a panic, an overflow or a loop that never ends shows in the target itself.

/// Runs the tag=value list parser under each grammar, and every value decoder, on `input`
/// (TAG-060). Returns whether each error offset and each range lies within `input`, and whether
/// each decoded value is no longer than `input`.
pub fn tags(_input: &[u8]) -> bool {
    false
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
