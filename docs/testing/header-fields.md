# Test scenarios: Header fields

Scenarios for the header section parser: splitting the header section into fields, unfolding, the obsolete forms, raw bytes, and the two input forms of STR-008. Conventions are in [README.md](README.md).

Unless a scenario says otherwise, the parser gets a raw header section with CRLF line endings, and a field's value is its unfolded text after the colon.

## Splitting and unfolding

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-001 | unit | The header section `Subject: This\r\n is a test\r\n\r\n`, the folding example of RFC 5322 §2.2.3 | the header section is parsed | one field, named `Subject`, with the value ` This is a test` and the raw bytes `Subject: This\r\n is a test\r\n` | [HDR-001](../reference/requirements.md#header-parsing), [HDR-003](../reference/requirements.md#header-parsing), [RFC 5322 §2.2.3](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2.3) |
| FLD-002 | unit | The header section `A: 1\r\n\r\nB: 2\r\n` | the header section is parsed | one field, `A`, and the body starts at `B: 2\r\n`: the header section ends at the first empty line | [HDR-001](../reference/requirements.md#header-parsing), [RFC 5322 §2.1](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.1) |
| FLD-003 | unit | The header section `SUBJect: AbC\r\n\r\n`, the field of RFC 6376's relaxed header example | the parser looks for the field `subject` | it finds the field, and the raw bytes still say `SUBJect` | [HDR-002](../reference/requirements.md#header-parsing), [RFC 5234 §2.3](https://www.rfc-editor.org/rfc/rfc5234.html#section-2.3), [RFC 6376 §3.4.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.2) |
| FLD-004 | unit | The header section `Received: <A>\r\nReceived: <B>\r\nReceived: <C>\r\n\r\n`, the example of RFC 6376 §5.4.2 | the header section is parsed | three `Received` fields in message order, each with its position, so that DKIM can take `<C>` first and work upward | [HDR-001](../reference/requirements.md#header-parsing), [RFC 6376 §5.4.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.4.2) |
| FLD-005 | unit | The header section `A: 1\r\nB: 2\r\n`, with no empty line and no body | the header section is parsed | the fields `A` and `B`, and no body: the header section ends at the end of the input | [HDR-001](../reference/requirements.md#header-parsing), [RFC 5322 §3.5](https://www.rfc-editor.org/rfc/rfc5322.html#section-3.5) |

## Obsolete forms

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-020 | unit | The header section `From  : a@void.invalid\r\n\r\n`, with white space before the colon as in RFC 5322 Appendix A.6.3 | the header section is parsed | one field named `From`, with the value ` a@void.invalid`, and the raw bytes unchanged | [HDR-003](../reference/requirements.md#header-parsing), [RFC 5322 §4.5](https://www.rfc-editor.org/rfc/rfc5322.html#section-4.5) |
| FLD-021 | unit | The header section `To: a@void.invalid,\r\n  \r\n b@void.invalid\r\n\r\n`, whose second line holds only white space | the header section is parsed | one field, `To`, with the value ` a@void.invalid,   b@void.invalid`: both line breaks unfold | [HDR-003](../reference/requirements.md#header-parsing), [RFC 5322 §4.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-4.2) |

## Characters and malformed lines

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-040 | unit | The header section `Subject: café\r\n\r\n`, with `é` in UTF-8 | the header section is parsed | one field, `Subject`, whose value holds the UTF-8 bytes of ` café` | [HDR-004](../reference/requirements.md#header-parsing), [RFC 6532 §3.2](https://www.rfc-editor.org/rfc/rfc6532.html#section-3.2) |
| FLD-041 | unit | The header section `Subjéct: x\r\n\r\n`, with `é` in UTF-8 in the field name | the header section is parsed | no field, but one malformed line with its raw bytes, and an error that names the field-name rule: field names are printable US-ASCII | [HDR-004](../reference/requirements.md#header-parsing), [HDR-010](../reference/requirements.md#header-parsing), [RFC 5322 §2.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2), [RFC 6532 §3.2](https://www.rfc-editor.org/rfc/rfc6532.html#section-3.2) |
| FLD-042 | unit | The header section `A: 1\r\nnot a field\r\nB: 2\r\n\r\n` | the header section is parsed | the fields `A` and `B`, and between them the malformed line `not a field\r\n` with its raw bytes, not joined to `A` | [HDR-004](../reference/requirements.md#header-parsing), [HDR-010](../reference/requirements.md#header-parsing), [ADR 0011](../adr/0011-header-parsers-follow-the-abnf-and-never-repair-input.md), [RFC 5322 §2.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2) |
| FLD-043 | unit | The header section ` A: 1\r\nB: 2\r\n\r\n`, whose first line starts with a space | the header section is parsed | a malformed line ` A: 1\r\n`, which continues no field, and then the field `B` | [HDR-004](../reference/requirements.md#header-parsing), [HDR-010](../reference/requirements.md#header-parsing), [ADR 0011](../adr/0011-header-parsers-follow-the-abnf-and-never-repair-input.md), [RFC 5322 §2.2.3](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2.3) |

## Line endings

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-050 | unit | The header section `Subject: This\n is a test\n\n`, declared as LF line endings | the header section is parsed | one field, `Subject`, with the value ` This is a test`, and canonicalization gets the bytes `Subject: This\r\n is a test\r\n` | [HDR-012](../reference/requirements.md#header-parsing), [STR-011](../reference/requirements.md#streaming), [RFC 6376 §5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.3) |
| FLD-051 | unit | The header section `A: 1\r\nB: 2\n\n`, declared as LF line endings | the header section is parsed | the fields `A` and `B`: the CRLF after `A: 1` stays one line ending | [HDR-012](../reference/requirements.md#header-parsing), [STR-011](../reference/requirements.md#streaming) |

## Fields from a milter

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-060 | unit | One field from a milter: the name `Subject` and the value `This\n is a test`, declared as LF between folded lines with one leading space removed | the field is parsed | the same field as FLD-001, with the raw bytes rebuilt as `Subject: This\r\n is a test\r\n`, and the trace records that one leading space was assumed after the colon | [HDR-005](../reference/requirements.md#header-parsing), [STR-008](../reference/requirements.md#streaming) |
| FLD-061 | unit | One field from a milter that negotiated `SMFIP_HDR_LEADSPC`: the name `From` and the value `x@void.invalid`, declared as unmodified | the field is parsed | the raw bytes `From:x@void.invalid\r\n`, and the trace records no assumption | [HDR-005](../reference/requirements.md#header-parsing), [STR-008](../reference/requirements.md#streaming) |

## Properties

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-080 | property | Any byte string of up to 64 KiB, used as a header section | the header section is parsed | the parser returns without panicking, and the raw bytes of its fields and malformed lines, joined in order, equal the input up to the end of the header section | [HDR-001](../reference/requirements.md#header-parsing), [HDR-010](../reference/requirements.md#header-parsing), [SAF-001](../reference/requirements.md#hostile-input) |
| FLD-081 | fuzz | The header section parser | it is fuzzed | it never panics, and the property of FLD-080 holds | [SAF-001](../reference/requirements.md#hostile-input), [SAF-003](../reference/requirements.md#hostile-input) |
