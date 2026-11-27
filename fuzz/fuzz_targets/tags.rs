//! TAG-060: the tag=value list parser and each value decoder of hallmark, on
//! any input. The macro takes its input in the syntax of a closure, the one
//! exception to the rule of ADR 0064 against closures (ADR 0069).

#![no_main]

libfuzzer_sys::fuzz_target!(|input: &[u8]| {
    assert!(hallmark::fuzz::tags(input));
});
