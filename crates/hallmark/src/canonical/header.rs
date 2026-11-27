//! The forms of one header field: DKIM `simple` (RFC 6376 §3.4.1), DKIM `relaxed` (§3.4.2 as
//! erratum 5839 corrects it), which the DKIM2 header hash uses too (draft-09 §6.2), and the form
//! of a field in the DKIM2 signing input (draft-09 §12.8). The protocols pick the fields and
//! build the hash inputs.

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "the checks and signings come later")
)]

use crate::header::section::{HeaderLine, LineKind};

/// A header field form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeaderMode {
    /// `simple`: the raw bytes (HCN-001).
    Simple,
    /// `relaxed`, for DKIM and the DKIM2 header hash (HCN-002, HCN-005).
    Relaxed,
    /// The form of the DKIM2 signing input: no WSP at all (HCN-007).
    Dkim2Signing,
}

/// Appends the form `mode` of the field `line` of `input` to `output`, ending in CRLF, and
/// gives `true`. A last line with no line ending gets the CRLF too (HCN-008). Gives `false`
/// and appends nothing for a malformed line, which has no name to select it by (HCN-006).
/// `line` comes from splitting `input`. For a line from another input, whose ranges can fall
/// outside `input`, it gives `false` too.
pub(crate) fn write_field(
    input: &[u8],
    line: &HeaderLine,
    mode: HeaderMode,
    output: &mut Vec<u8>,
) -> bool {
    let LineKind::Field { name, value } = &line.kind else {
        return false;
    };
    let (Some(raw), Some(name), Some(value)) = (
        input.get(line.raw.clone()),
        input.get(name.clone()),
        input.get(value.clone()),
    ) else {
        return false;
    };
    if mode == HeaderMode::Simple {
        output.extend_from_slice(raw);
        if !line.line_ending {
            output.extend_from_slice(b"\r\n");
        }
        return true;
    }
    for byte in name {
        output.push(byte.to_ascii_lowercase());
    }
    output.push(b':');
    write_value(value, mode == HeaderMode::Relaxed, output);
    output.extend_from_slice(b"\r\n");
    true
}

/// Appends `value` unfolded, with no WSP at its start or end. Inside it, each run of WSP
/// becomes one SP when `relaxed` holds, and goes otherwise (RFC 6376 §3.4.2, draft-09 §12.8).
/// Each CRLF in a value is a fold, because a field goes on only over lines that start with
/// WSP. A bare CR or LF is an octet and not WSP (HDR-012).
fn write_value(value: &[u8], relaxed: bool, output: &mut Vec<u8>) {
    let mut rest = value;
    let mut wsp = false;
    let mut content = false;
    while let Some((&byte, after)) = rest.split_first() {
        rest = after;
        if byte == b'\r'
            && let Some(unfolded) = after.strip_prefix(b"\n")
        {
            rest = unfolded;
        } else if matches!(byte, b' ' | b'\t') {
            wsp = true;
        } else {
            if wsp && content && relaxed {
                output.push(b' ');
            }
            wsp = false;
            content = true;
            output.push(byte);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HeaderMode::{Dkim2Signing, Relaxed, Simple};
    use super::{HeaderMode, write_field};
    use crate::header::section::split;

    /// The form `mode` of each field of the header section `input`.
    fn forms(input: &[u8], mode: HeaderMode) -> Vec<Vec<u8>> {
        let mut fields = Vec::new();
        for line in &split(input).lines {
            let mut output = Vec::new();
            assert!(write_field(input, line, mode, &mut output));
            fields.push(output);
        }
        fields
    }

    /// The form `mode` of the one field `input`.
    fn form(input: &[u8], mode: HeaderMode) -> Vec<u8> {
        let mut fields = forms(input, mode);
        assert_eq!(fields.len(), 1);
        fields.remove(0)
    }

    /// The header section of RFC 6376 §3.4.5.
    const EXAMPLE: &[u8] = b"A: X\r\nB : Y\t\r\n\tZ  \r\n\r\n";

    #[test]
    fn can_140_simple_keeps_the_raw_bytes() {
        let expected = [b"A: X\r\n".to_vec(), b"B : Y\t\r\n\tZ  \r\n".to_vec()];
        assert_eq!(forms(EXAMPLE, Simple), expected);
    }

    #[test]
    fn can_141_simple_keeps_the_case_of_the_name() {
        assert_eq!(form(b"SUBJect: AbC\r\n", Simple), b"SUBJect: AbC\r\n");
    }

    #[test]
    fn can_160_relaxed_gives_the_example_of_rfc_6376() {
        let expected = [b"a:X\r\n".to_vec(), b"b:Y Z\r\n".to_vec()];
        assert_eq!(forms(EXAMPLE, Relaxed), expected);
    }

    #[test]
    fn can_161_relaxed_lowercases_the_name_only() {
        assert_eq!(form(b"SUBJect: AbC\r\n", Relaxed), b"subject:AbC\r\n");
    }

    #[test]
    fn can_162_relaxed_empties_a_value_of_white_space() {
        assert_eq!(form(b"Subject:\r\n", Relaxed), b"subject:\r\n");
        assert_eq!(form(b"Subject: \t \r\n", Relaxed), b"subject:\r\n");
    }

    #[test]
    fn can_163_relaxed_unfolds_a_line_of_white_space() {
        let input = b"To: a@void.test,\r\n  \r\n b@void.test\r\n";
        assert_eq!(form(input, Relaxed), b"to:a@void.test, b@void.test\r\n");
    }

    #[test]
    fn can_164_only_sp_and_htab_are_wsp() {
        let input = b"Subject: a\xc2\xa0 b  c \r\n";
        assert_eq!(form(input, Relaxed), b"subject:a\xc2\xa0 b c\r\n");
    }

    #[test]
    fn can_165_relaxed_deletes_the_sp_before_the_final_crlf() {
        assert_eq!(form(b"Subject: x \r\n", Relaxed), b"subject:x\r\n");
    }

    #[test]
    fn can_244_the_signing_input_form_has_no_wsp() {
        let input = b"Message-Instance :\tm=2; h=sha256:EE\r\n EE:FFFF \r\n";
        let expected = b"message-instance:m=2;h=sha256:EEEE:FFFF\r\n";
        assert_eq!(form(input, Dkim2Signing), expected);
    }

    /// The form `mode` of the last field of `input`.
    fn last_form(input: &[u8], mode: HeaderMode) -> Vec<u8> {
        forms(input, mode).pop().unwrap()
    }

    /// The bytes of CAN-263, CAN-265 and CAN-268. The trace parts of CAN-263 come with the header
    /// hash steps.
    #[test]
    fn can_263_265_268_a_last_line_with_no_line_ending_gets_a_crlf() {
        let input = b"From: a@void.test\r\nSubject: Hi";
        assert_eq!(last_form(input, Simple), b"Subject: Hi\r\n");
        assert_eq!(last_form(input, Relaxed), b"subject:Hi\r\n");
        let input = b"Subject: Hi\r\n there";
        assert_eq!(last_form(input, Simple), b"Subject: Hi\r\n there\r\n");
        assert_eq!(last_form(input, Relaxed), b"subject:Hi there\r\n");
        let input = b"From: a@void.test\r\nSubject: Hi\r";
        assert_eq!(last_form(input, Simple), b"Subject: Hi\r\r\n");
        assert_eq!(last_form(input, Relaxed), b"subject:Hi\r\r\n");
    }

    #[test]
    fn can_245_the_signing_input_form_ends_a_last_line_with_crlf() {
        let input = b"Message-Instance: m=1; h=sha256:AAAA:BBBB";
        let expected = b"message-instance:m=1;h=sha256:AAAA:BBBB\r\n";
        assert_eq!(last_form(input, Dkim2Signing), expected);
    }

    #[test]
    fn can_267_a_bare_lf_is_neither_crlf_nor_wsp() {
        assert_eq!(form(b"Subject: A \nB\r\n", Relaxed), b"subject:A \nB\r\n");
    }

    #[test]
    fn can_269_a_malformed_line_has_no_form() {
        let input = b"not a field\r\n";
        let line = &split(input).lines[0];
        for mode in [Simple, Relaxed, Dkim2Signing] {
            let mut output = Vec::new();
            assert!(!write_field(input, line, mode, &mut output));
            assert!(output.is_empty());
        }
    }
}
