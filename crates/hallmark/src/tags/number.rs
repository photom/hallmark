//! Decimal tag values, read with overflow checks (HDR-013).

#![cfg_attr(not(test), expect(dead_code, reason = "DKIM and DKIM2 come later"))]

/// The digits that a numeric tag allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Digits {
    /// DKIM `t=` and `x=`: `1*12DIGIT`, where a longer value counts as
    /// infinite (RFC 6376 §3.5).
    DkimTime,
    /// DKIM `l=`: `1*76DIGIT` (RFC 6376 §3.5).
    DkimLength,
    /// DKIM2 `t=`, `m=` and `i=`: `1*DIGIT` (draft-09 §9.1, §10.1, §10.2,
    /// §10.4).
    Dkim2,
}

/// A decimal tag value. `Infinite` is larger than every `Finite` value, and
/// two `Infinite` values compare equal, so a check that orders two values,
/// such as `x=` after `t=`, decides that case itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Number {
    /// A value that fits in 64 bits.
    Finite(u64),
    /// A value larger than any it is compared with: a DKIM `t=` or `x=` of
    /// more than 12 digits, or a value too large for 64 bits (HDR-013).
    Infinite,
}

/// Why a decimal value doesn't decode: the offset of the first octet that
/// isn't a digit, of the end of an empty value, or of a digit past the 76 of
/// DKIM `l=`. The rule is that of the tag the caller read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NumberError(pub(crate) usize);

/// Reads `input`, the value of a numeric tag. Not written yet.
pub(crate) fn decimal(input: &[u8], digits: Digits) -> Result<Number, NumberError> {
    let _ = (input, digits);
    Err(NumberError(0))
}

#[cfg(test)]
mod tests {
    use super::{Digits, Number, NumberError, decimal};

    #[test]
    fn tag_028_a_13_digit_t_is_infinite() {
        let value = decimal(b"1234567890123", Digits::DkimTime);
        assert_eq!(value, Ok(Number::Infinite));
        assert!(Number::Infinite > Number::Finite(u64::MAX));
        let largest = Number::Finite(999_999_999_999);
        assert_eq!(decimal(b"999999999999", Digits::DkimTime), Ok(largest));
    }

    #[test]
    fn tag_029_a_13_digit_x_is_infinite_whatever_its_value() {
        let value = decimal(b"0000000000001", Digits::DkimTime);
        assert_eq!(value, Ok(Number::Infinite));
    }

    #[test]
    fn tag_030_an_l_of_76_digits_is_larger_than_any_body() {
        let input = [b'9'; 76];
        let value = decimal(&input, Digits::DkimLength);
        assert_eq!(value, Ok(Number::Infinite));
    }

    #[test]
    fn tag_049_a_number_is_digits_only() {
        let cases: [(&[u8], usize); 4] = [(b"", 0), (b"12a", 2), (b"+1", 0), (b"1 2", 1)];
        for (input, offset) in cases {
            let error = Err(NumberError(offset));
            assert_eq!(decimal(input, Digits::DkimTime), error);
            assert_eq!(decimal(input, Digits::DkimLength), error);
            assert_eq!(decimal(input, Digits::Dkim2), error);
        }
        let input = [b'1'; 77];
        let error = Err(NumberError(76));
        assert_eq!(decimal(&input, Digits::DkimLength), error);
    }

    #[test]
    fn tag_050_dkim2_numbers_have_no_digit_limit() {
        let cases: [(&[u8], Number); 3] = [
            (b"1234567890123", Number::Finite(1_234_567_890_123)),
            (b"18446744073709551615", Number::Finite(u64::MAX)),
            (b"18446744073709551616", Number::Infinite),
        ];
        for (input, expected) in cases {
            assert_eq!(decimal(input, Digits::Dkim2), Ok(expected));
        }
    }
}
