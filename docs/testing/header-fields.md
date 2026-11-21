# Test scenarios: Header fields

Scenarios for the header section parser: splitting the header section into fields, unfolding, the obsolete forms, raw bytes, and the two input forms of STR-008. Conventions are in [README.md](README.md).

Unless a scenario says otherwise, the parser gets a raw header section with CRLF line endings, and a field's value is its unfolded text after the colon.

## Contents

- [Splitting and unfolding](#splitting-and-unfolding)
- [Obsolete forms](#obsolete-forms)
- [Characters and malformed lines](#characters-and-malformed-lines)
- [Line endings](#line-endings)
- [Fields from a milter](#fields-from-a-milter)
- [Properties](#properties)

## Splitting and unfolding

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-001 | unit | The header section `Subject: This\r\n is a test\r\n\r\n`, the folding example of RFC 5322 §2.2.3 | the header section is parsed | one field, named `Subject`, with the value ` This is a test` and the raw bytes `Subject: This\r\n is a test\r\n` | [HDR-001](../reference/requirements.md#header-parsing), [HDR-003](../reference/requirements.md#header-parsing), [RFC 5322 §2.2.3](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2.3) |
| FLD-002 | unit | The header section `A: 1\r\n\r\nB: 2\r\n` | the header section is parsed | one field, `A`, and the body starts at `B: 2\r\n`: the header section ends at the first empty line | [HDR-001](../reference/requirements.md#header-parsing), [RFC 5322 §2.1](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.1) |
| FLD-003 | unit | The header section `SUBJect: AbC\r\n\r\n`, the field of RFC 6376's relaxed header example | the parser looks for the field `subject` | it finds the field, and the raw bytes still say `SUBJect` | [HDR-002](../reference/requirements.md#header-parsing), [RFC 5234 §2.3](https://www.rfc-editor.org/rfc/rfc5234.html#section-2.3), [RFC 6376 §3.4.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.2) |
| FLD-004 | unit | The header section `Received: <A>\r\nReceived: <B>\r\nReceived: <C>\r\n\r\n`, the example of RFC 6376 §5.4.2 | the header section is parsed | three `Received` fields in message order, each with its position, so that DKIM can take `<C>` first and work upward | [HDR-001](../reference/requirements.md#header-parsing), [RFC 6376 §5.4.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.4.2) |
| FLD-005 | unit | The header section `A: 1\r\nB: 2\r\n`, with no empty line and no body | the header section is parsed | the fields `A` and `B`, and no body: the header section ends at the end of the input | [HDR-001](../reference/requirements.md#header-parsing), [RFC 5322 §3.5](https://www.rfc-editor.org/rfc/rfc5322.html#section-3.5) |
| FLD-006 | unit | The header section `A: 1\r\nB: 2`, which ends with no line ending and has no body | the header section is parsed | the fields `A` and `B`. `B` keeps the raw bytes `B: 2`, and the parser records as a fact that the input ended `B` with no line ending, citing RFC 5322 §2.2 | [HDR-001](../reference/requirements.md#header-parsing), [HCN-008](../reference/requirements.md#header-canonicalization), [ADR 0030](../adr/0030-a-last-header-line-without-a-line-ending-is-a-header-field-that-canonicalization-ends-with-crlf.md), [RFC 5322 §2.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2) |

## Obsolete forms

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-020 | unit | The header section `From  : a@void.test\r\n\r\n`, with white space before the colon as in RFC 5322 Appendix A.6.3 | the header section is parsed | one field named `From`, with the value ` a@void.test`, and the raw bytes unchanged | [HDR-003](../reference/requirements.md#header-parsing), [RFC 5322 §4.5](https://www.rfc-editor.org/rfc/rfc5322.html#section-4.5) |
| FLD-021 | unit | The header section `To: a@void.test,\r\n  \r\n b@void.test\r\n\r\n`, whose second line holds only white space | the header section is parsed | one field, `To`, with the value ` a@void.test,   b@void.test`: both line breaks unfold | [HDR-003](../reference/requirements.md#header-parsing), [RFC 5322 §4.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-4.2) |

## Characters and malformed lines

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-040 | unit | The header section `Subject: café\r\n\r\n`, with `é` in UTF-8 | the header section is parsed | one field, `Subject`, whose value holds the UTF-8 bytes of ` café` | [HDR-004](../reference/requirements.md#header-parsing), [RFC 6532 §3.2](https://www.rfc-editor.org/rfc/rfc6532.html#section-3.2) |
| FLD-041 | unit | The header section `Subjéct: x\r\n\r\n`, with `é` in UTF-8 in the field name | the header section is parsed | no field, but one malformed line with its raw bytes, and an error that names the field-name rule: field names are printable US-ASCII | [HDR-004](../reference/requirements.md#header-parsing), [HDR-010](../reference/requirements.md#header-parsing), [RFC 5322 §2.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2), [RFC 6532 §3.2](https://www.rfc-editor.org/rfc/rfc6532.html#section-3.2) |
| FLD-042 | unit | The header section `A: 1\r\nnot a field\r\nB: 2\r\n\r\n` | the header section is parsed | the fields `A` and `B`, and between them the malformed line `not a field\r\n` with its raw bytes, not joined to `A` | [HDR-004](../reference/requirements.md#header-parsing), [HDR-010](../reference/requirements.md#header-parsing), [ADR 0010](../adr/0010-header-parsers-follow-the-abnf-and-never-repair-input.md), [RFC 5322 §2.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2) |
| FLD-043 | unit | The header section ` A: 1\r\nB: 2\r\n\r\n`, whose first line starts with a space | the header section is parsed | a malformed line ` A: 1\r\n`, which continues no field, and then the field `B` | [HDR-004](../reference/requirements.md#header-parsing), [HDR-010](../reference/requirements.md#header-parsing), [ADR 0010](../adr/0010-header-parsers-follow-the-abnf-and-never-repair-input.md), [RFC 5322 §2.2.3](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2.3) |

## Line endings

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-050 | unit | The header section `Subject: This\n is a test\n\n`, declared as LF line endings | the header section is parsed | Withdrawn: LF mode is out of scope for the first release, and the core takes CRLF line endings only ([ADR 0037](../adr/0037-the-core-takes-crlf-line-endings-only-and-a-bare-cr-or-lf-in-the-body-is-an-octet.md)) | [HDR-012](../reference/requirements.md#header-parsing), [STR-011](../reference/requirements.md#streaming), [RFC 6376 §5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.3) |
| FLD-051 | unit | The header section `A: 1\r\nB: 2\n\n`, declared as LF line endings | the header section is parsed | Withdrawn: LF mode is out of scope for the first release, and the core takes CRLF line endings only ([ADR 0037](../adr/0037-the-core-takes-crlf-line-endings-only-and-a-bare-cr-or-lf-in-the-body-is-an-octet.md)) | [HDR-012](../reference/requirements.md#header-parsing), [STR-011](../reference/requirements.md#streaming) |
| FLD-052 | unit | The header section `A: 1\nB: 2\r\n\r\n`, with a bare LF after `A: 1` | the header section is parsed | one field `A` whose value is `1\nB: 2`, with the bare LF kept as an octet, and no field `B`. The trace records a caution with one bare LF and its place, citing RFC 5322 §2.2 and RFC 5321 §2.3.8 | [HDR-012](../reference/requirements.md#header-parsing), [ADR 0051](../adr/0051-a-bare-cr-or-lf-in-the-header-section-is-an-octet-of-the-line-that-holds-it.md), [RFC 5321 §2.3.8](https://www.rfc-editor.org/rfc/rfc5321.html#section-2.3.8), [RFC 5322 §4.1](https://www.rfc-editor.org/rfc/rfc5322.html#section-4.1) |
| FLD-053 | unit | The header section `A: 1\r\n\nB: 2\r\n\r\n`, whose second line starts with a bare LF | the header section is parsed | the field `A`, then the malformed line `\nB: 2\r\n`: a bare LF isn't WSP, so the line continues no field | [HDR-012](../reference/requirements.md#header-parsing), [HDR-004](../reference/requirements.md#header-parsing), [ADR 0051](../adr/0051-a-bare-cr-or-lf-in-the-header-section-is-an-octet-of-the-line-that-holds-it.md), [RFC 5322 §2.2.3](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2.3) |
| FLD-054 | unit | The header section `Fr\rom: a@void.test\r\n\r\n`, with a bare CR in the field name | the header section is parsed | the malformed line `Fr\rom: a@void.test\r\n`: the name breaks `field-name`, and the message has no From field | [HDR-012](../reference/requirements.md#header-parsing), [HDR-004](../reference/requirements.md#header-parsing), [ADR 0051](../adr/0051-a-bare-cr-or-lf-in-the-header-section-is-an-octet-of-the-line-that-holds-it.md), [RFC 5322 §2.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-2.2) |
| FLD-055 | unit | The header section `From: a@void.test\n\r\n\r\n`, with a bare LF at the end of the From value | DMARC runs | one From field whose value ends with the bare LF. The address grammar allows no bare LF there, so the field doesn't parse, and DMARC isn't applied for lack of an author domain | [HDR-012](../reference/requirements.md#header-parsing), [DMC-002](../reference/requirements.md#dmarc), [ADR 0051](../adr/0051-a-bare-cr-or-lf-in-the-header-section-is-an-octet-of-the-line-that-holds-it.md), [RFC 5322 §3.6.2](https://www.rfc-editor.org/rfc/rfc5322.html#section-3.6.2) |

## Fields from a milter

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-060 | unit | One field from a milter: the name `Subject` and the value `This\n is a test`, declared as LF between folded lines with one leading space removed | the field is parsed | the same field as FLD-001, with the raw bytes rebuilt as `Subject: This\r\n is a test\r\n`, and the trace records that one leading space was assumed after the colon | [HDR-005](../reference/requirements.md#header-parsing), [STR-008](../reference/requirements.md#streaming) |
| FLD-061 | unit | One field from a milter that negotiated `SMFIP_HDR_LEADSPC`: the name `From` and the value `x@void.test`, declared as unmodified | the field is parsed | the raw bytes `From:x@void.test\r\n`, and the trace records no assumption | [HDR-005](../reference/requirements.md#header-parsing), [STR-008](../reference/requirements.md#streaming) |
| FLD-062 | unit | One field from a milter: the name `Subject` and the value `A\rB`, declared as CRLF between folded lines with no leading space removed | the field is parsed | the raw bytes `Subject:A\rB\r\n`, with the bare CR kept as an octet, and the trace's caution names its place | [HDR-012](../reference/requirements.md#header-parsing), [STR-008](../reference/requirements.md#streaming), [ADR 0051](../adr/0051-a-bare-cr-or-lf-in-the-header-section-is-an-octet-of-the-line-that-holds-it.md) |

## Properties

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| FLD-080 | property | Any byte string of up to 64 KiB, used as a header section | the header section is parsed | the parser returns without panicking, and the raw bytes of its fields and malformed lines, joined in order, equal the input up to the end of the header section | [HDR-001](../reference/requirements.md#header-parsing), [HDR-010](../reference/requirements.md#header-parsing), [SAF-001](../reference/requirements.md#hostile-input) |
| FLD-081 | fuzz | The header section parser | it is fuzzed | it never panics, and the property of FLD-080 holds | [SAF-001](../reference/requirements.md#hostile-input), [SAF-003](../reference/requirements.md#hostile-input) |
