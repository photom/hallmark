# DNS questions and answers

The values a check hands the caller when it needs DNS records or a public suffix, and the values the caller hands back ([ADR 0017](../../adr/0017-the-core-does-no-io-and-the-caller-drives-it.md), [ADR 0038](../../adr/0038-in-rfc-7489-mode-the-caller-answers-public-suffix-questions.md)). The rules for an answer sit with the methods that take it, in [Check](check.md#checkanswer).

## Contents

- [Requests](#requests)
- [Questions](#questions)
- [Answers](#answers)
- [Suffix questions and answers](#suffix-questions-and-answers)
- [Validation](#validation)

## Requests

`Check::take_requests` gives a `Vec<Request>` ([Check](check.md#request)). Each request is one of three:

```mermaid
classDiagram
    class Request {
        <<enumeration>>
        Question
        Suffix
        Signing
    }
    class Question {
        +QuestionId id
        +QuestionType record_type
        +Vec~u8~ name
    }
    class SuffixQuestion {
        +QuestionId id
        +Vec~u8~ domain
    }
    class SigningRequest {
        +RequestId id
    }
    Request ..> Question
    Request ..> SuffixQuestion
    Request ..> SigningRequest
```

[Signing](signing.md#types) gives `SigningRequest`.

## Questions

`Question` is `#[non_exhaustive]` with public fields. The check builds it, and the caller reads it.

| Field | Type | Meaning | Requirements |
|---|---|---|---|
| `id` | `QuestionId` | Unique within the check. It depends on when the question was asked, so the trace never shows it | DNS-007, TRC-008 |
| `record_type` | `QuestionType` | The type of record asked for | DNS-001 |
| `name` | `Vec<u8>` | The name in lookup form, built by the check, as ASCII without a final dot | DNS-008, ADS-002 |

`QuestionType` is an exhaustive enum, `Copy`: `Txt`, `A`, `Aaaa`, `Mx` or `Ptr` (DNS-001).

A check holds each type and name once (INV-001), so the caller never gets the same question twice.

## Answers

`Answer` and `Record` are exhaustive enums. The caller builds them.

| `Answer` variant | Carries | Meaning | Requirements |
|---|---|---|---|
| `Records` | `Vec<Record>`, in the order the resolver gave them | Records found | DNS-002, DNS-006 |
| `NameNotFound` | Nothing | The name doesn't exist (NXDOMAIN) | DNS-002 |
| `NoRecords` | Nothing | The name exists with no records of this type (NODATA) | DNS-002 |
| `TemporaryFailure` | `Option<Vec<u8>>`: the resolver's description | The resolver couldn't get an answer, such as on a timeout or SERVFAIL | DNS-009 |

| `Record` variant | Carries | Requirements |
|---|---|---|
| `Txt` | `Vec<Vec<u8>>`: its character-strings, in order and not joined | DNS-006, RFC 1035 §3.3.14 |
| `A` | `Ipv4Addr` | DNS-006 |
| `Aaaa` | `Ipv6Addr` | DNS-006, RFC 3596 §2.2 |
| `Mx` | `preference: u16`, `exchange: Vec<u8>` | DNS-006, RFC 1035 §3.3.9 |
| `Ptr` | `Vec<u8>`: a name | DNS-006, RFC 1035 §3.3.12 |

A name in a record is the bytes of its labels joined by dots, without a final dot. The resolver needs to know no protocol rule. The check joins TXT strings itself (DNS-006), and a name in a record that doesn't parse gets the rule of the protocol that read it.

## Suffix questions and answers

In RFC 7489 mode, the check asks for the public suffix of each domain whose organizational domain it may need (DMC-010). RFC 9989 mode asks none.

`SuffixQuestion` is `#[non_exhaustive]` with public fields. `SuffixAnswer` is an exhaustive enum that the caller builds.

| Type | Field or variant | Type or carries | Meaning | Requirements |
|---|---|---|---|---|
| `SuffixQuestion` | `id` | `QuestionId` | Shares the ID space of DNS questions | DMC-010 |
| `SuffixQuestion` | `domain` | `Vec<u8>` | The domain in lookup form, as ASCII without a final dot | DMC-010 |
| `SuffixAnswer` | `Suffix` | `labels: u8`, `list: Option<String>` | The public suffix has `labels` labels, the x of RFC 7489 §3.2 step 3. `list` names the list, such as its date or a hash | DMC-010, DMC-011 |
| `SuffixAnswer` | `Unavailable` | Nothing | The caller has no suffix for the domain. It gives `temperror` if the result depends on that organizational domain | DMC-011 |

## Validation

These values are plain data, and building one checks nothing. A check checks each answer when the caller hands it back.

| Value | Checked by | Rules |
|---|---|---|
| `Answer`, `Record` | `Check::answer` | [VAL-120 to VAL-125](check.md#checkanswer) |
| `SuffixAnswer` | `Check::answer_suffix` | [VAL-130 to VAL-133](check.md#checkanswer_suffix) |
| `Answer` from a resolver | The driver, through `Check::answer` | [VAL-804](driver.md#drive) |
| `Answer` in a snapshot | `Snapshot::parse` | [VAL-901 to VAL-908](snapshot.md#snapshotparse) |
