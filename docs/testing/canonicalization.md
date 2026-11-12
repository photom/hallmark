# Test scenarios: Canonicalization

Scenarios for body canonicalization: DKIM's `simple` and `relaxed` algorithms, DKIM2's body hash, line endings, chunk boundaries, and `l=`. Conventions are in [README.md](README.md).

Each scenario feeds the body to the canonicalizer and checks the canonical bytes, the body hash, or both. Unless a scenario says otherwise, the body arrives in one chunk with CRLF line endings.

## DKIM simple body

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| CAN-001 | unit | An empty body | the body is canonicalized with `simple` | the canonical body is `\r\n`, 2 octets, and its hash is `frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY=` | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.3) |
| CAN-002 | unit | The body `A\r\n\r\n\r\n` | the body is canonicalized with `simple` | the canonical body is `A\r\n`, and its hash is `Jv/ViGJTkGo2p+oPbiYFb8NkcmJstIlLyxAKNNxp0ds=` | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.3) |
| CAN-003 | unit | The body `A`, with no line ending | the body is canonicalized with `simple` | the canonical body is `A\r\n` | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.3) |
| CAN-004 | unit | The body of RFC 6376's canonicalization example: ` C \r\nD \t E\r\n\r\n\r\n` | the body is canonicalized with `simple` | the canonical body is ` C \r\nD \t E\r\n`, and its hash is `NOeivbQlDH9TmNKJUw7D53wZfsk8YMZ/hTuVVwTgi8s=` | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.5) |
| CAN-005 | unit | The body `\r\n\r\n`, only empty lines | the body is canonicalized with `simple` | the canonical body is `\r\n` | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.3) |

## DKIM relaxed body

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| CAN-020 | unit | An empty body | the body is canonicalized with `relaxed` | the canonical body is empty, and its hash is `47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=` | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.4) |
| CAN-021 | unit | The body of RFC 6376's canonicalization example: ` C \r\nD \t E\r\n\r\n\r\n` | the body is canonicalized with `relaxed` | the canonical body is ` C\r\nD E\r\n`, and its hash is `unak6JHq0wL+Q1HP7dW1tjBx9FLA6DffoZ0qrLwbbpo=` | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.5) |
| CAN-022 | unit | The body `A \t \r\n` | the body is canonicalized with `relaxed` | the canonical body is `A\r\n`: whitespace at the end of the line is gone | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.4) |
| CAN-023 | unit | The body `A\t\t B\r\n` | the body is canonicalized with `relaxed` | the canonical body is `A B\r\n`: the run of whitespace is one space | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.4) |
| CAN-024 | unit | The body `A  `, with trailing spaces and no line ending | the body is canonicalized with `relaxed` | the canonical body is `A\r\n` | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.4) |
| CAN-025 | unit | The body `  \r\n\t\r\n`, lines that hold only whitespace | the body is canonicalized with `relaxed` | the canonical body is empty: the lines become empty, and empty lines at the end are ignored | [DKV-001](../reference/requirements.md#dkim-verification), [RFC 6376 §3.4.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.4) |

## DKIM2 body

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| CAN-040 | unit | The body of RFC 6376's canonicalization example: ` C \r\nD \t E\r\n\r\n\r\n` | the DKIM2 body hash is computed | the hash is `NOeivbQlDH9TmNKJUw7D53wZfsk8YMZ/hTuVVwTgi8s=`, the same as DKIM's `simple` | [DK2-003](../reference/requirements.md#dkim2), [draft-09 §6.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-6.1) |
| CAN-041 | unit | An empty body | the DKIM2 body hash is computed | the hash is `frcCV1k9oG9oKj3dpUqdJg1PxRT2RSN/XKdLCPjaYaY=`, the hash of `\r\n` | [DK2-003](../reference/requirements.md#dkim2), [draft-09 §6.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-6.1) |

## Line endings

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| CAN-060 | unit | The body `A\nB\n`, declared as LF line endings | the body is canonicalized with `simple` | the canonical body is `A\r\nB\r\n` | [STR-011](../reference/requirements.md#streaming), [RFC 6376 §5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.3) |
| CAN-061 | unit | The body `A\r\nB\n`, declared as LF line endings | the body is canonicalized with `simple` | the canonical body is `A\r\nB\r\n`: the existing CRLF stays one line ending, and the bare LF becomes CRLF | [STR-011](../reference/requirements.md#streaming), [RFC 6376 §5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.3) |
| CAN-062 | unit | The body `A\r\nB\n`, declared as LF line endings, split into the chunks `A\r` and `\nB\n` | the chunks are canonicalized with `simple` | the canonical body is `A\r\nB\r\n`, the same as CAN-061 | [STR-011](../reference/requirements.md#streaming), [STR-005](../reference/requirements.md#streaming) |
| CAN-063 | unit | The body `A\nB\r\n`, declared as CRLF line endings, with a bare LF | the body is canonicalized with `simple` | open: whether a verifier in CRLF mode leaves a bare LF as it is or converts it ([open item](../reference/requirements.md#open-items)) | [STR-011](../reference/requirements.md#streaming), [RFC 6376 §5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.3) |

## Chunk boundaries

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| CAN-080 | unit | The body `Hello   \r\nWorld\r\n`, split into `Hello   ` and `\r\nWorld\r\n` | the chunks are canonicalized with `relaxed` | the canonical body is `Hello\r\nWorld\r\n`: the spaces held back from the first chunk are dropped | [STR-005](../reference/requirements.md#streaming), [RFC 6376 §3.4.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.4) |
| CAN-081 | unit | The body `Hello   World\r\n`, split into `Hello   ` and `World\r\n` | the chunks are canonicalized with `relaxed` | the canonical body is `Hello World\r\n`: the spaces held back become one space | [STR-005](../reference/requirements.md#streaming), [RFC 6376 §3.4.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.4) |
| CAN-082 | unit | The body `A\r\nB\r\n`, split into `A\r` and `\nB\r\n` | the chunks are canonicalized with `simple` | the canonical body is `A\r\nB\r\n` | [STR-005](../reference/requirements.md#streaming), [RFC 6376 §3.4.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.3) |
| CAN-083 | unit | The body `A\r\n\r\n\r\nB\r\n`, split into `A\r\n\r\n` and `\r\nB\r\n` | the chunks are canonicalized with `simple` | the canonical body is `A\r\n\r\n\r\nB\r\n`: empty lines held back are hashed once text follows | [STR-005](../reference/requirements.md#streaming), [RFC 6376 §3.4.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.3) |
| CAN-084 | unit | The body `A\r\n\r\n\r\n`, split into `A\r\n\r\n` and `\r\n` | the chunks are canonicalized with `simple` and the body ends | the canonical body is `A\r\n`: empty lines held back are dropped at the end | [STR-005](../reference/requirements.md#streaming), [RFC 6376 §3.4.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.3) |
| CAN-085 | unit | The body of RFC 6376's canonicalization example, delivered one byte per chunk | the chunks are canonicalized with `simple` and with `relaxed` | the hashes are those of CAN-004 and CAN-021 | [STR-005](../reference/requirements.md#streaming), [RFC 6376 §3.4.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4.5) |
| CAN-086 | property | Any body of up to 4,096 bytes, mixing text, spaces, tabs, CR, and LF, split at any set of points | the chunks are canonicalized with `simple`, `relaxed`, and the DKIM2 body hash, in CRLF and LF modes | each result equals the result of canonicalizing the body in one chunk | [STR-005](../reference/requirements.md#streaming) |

## Memory

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| CAN-100 | unit | A body of `A\r\n` followed by 1,000,000 empty lines, in chunks of 65,535 bytes | the chunks are canonicalized with `simple` and the body ends | the canonical body is `A\r\n`, and between calls the canonicalizer holds a count of pending line endings and no input bytes | [STR-001](../reference/requirements.md#streaming), [SAF-004](../reference/requirements.md#hostile-input) |
| CAN-101 | unit | A body of `A`, 1,000,000 spaces, and `B\r\n`, in chunks of 65,535 bytes | the chunks are canonicalized with `relaxed` | the canonical body is `A B\r\n`, and between calls the canonicalizer holds a flag for pending whitespace and no input bytes | [STR-001](../reference/requirements.md#streaming), [SAF-004](../reference/requirements.md#hostile-input) |

## Body length (`l=`)

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| CAN-120 | unit | The body of RFC 6376's canonicalization example and `l=5` | the body is canonicalized with `relaxed` | the hash covers the first 5 canonical octets, ` C\r\nD`, and is `lXbMuizzz61KKfSrf4wavTUrzVz/yI6aAWKyhRpIQiM=`, and the result reports 4 canonical octets left unsigned | [DKV-006](../reference/requirements.md#dkim-verification), [RFC 6376 §3.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.5) |
| CAN-121 | unit | The body `A\r\n` and `l=10`, larger than its 3 canonical octets | the body is canonicalized with `simple` | the signature fails verification, and the trace names `l=` with both lengths | [DKV-006](../reference/requirements.md#dkim-verification), [RFC 6376 §3.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.5) |
