# Results

The values that `Check::take_results` gives: one result for each kind of check, each with a typed reason (GEN-004) and its position in the trace (DMC-005). [Domain model](../design/domain-model.md#results) draws the types.

## Contents

- [Signatures](#signatures)
- [Parameters](#parameters)
- [Results](#results-1)
- [Domain](#domain)
- [SPF](#spf)
- [DKIM](#dkim)
- [DKIM2](#dkim2)
- [DMARC](#dmarc)
- [Signing](#signing)
- [Validation](#validation)
  - [Result values](#result-values)
  - [`HeaderFieldOut::read_value`](#headerfieldoutread_value)

## Signatures

Every type on this page is built by hallmark. Structs are `#[non_exhaustive]` with public fields, apart from `HeaderFieldOut`, whose fields are private.

| API | Signature | Returns | Requirements |
|---|---|---|---|
| `HeaderFieldOut::name` | `fn name(&self) -> &str` | The field name, such as `DKIM-Signature` | STR-009 |
| `HeaderFieldOut::value` | `fn value(&self) -> &[u8]` | The whole value in the form `field_form` states ([FieldForm](options.md#fieldform)), without the line ending that ends the field | STR-009 |
| `HeaderFieldOut::read_value` | `fn read_value(&self, offset: usize, buffer: &mut [u8]) -> Result<usize, ResultsError>` | The bytes of the value from `offset` written into `buffer`, as many as fit. 0 when `offset` is the length of the value | STR-002 |

## Parameters

| API | Parameter | Meaning and form |
|---|---|---|
| `HeaderFieldOut::read_value` | `offset` | The octet of the value to start from, counted from 0 |
| `HeaderFieldOut::read_value` | `buffer` | Where the bytes go, from the start of the buffer |

## Results

| Field | Type | Present when | Requirements |
|---|---|---|---|
| `spf_helo` | `Option<SpfResult>` | The kinds hold `SpfHelo` | SPF-002 |
| `spf_mail_from` | `Option<SpfResult>` | The kinds hold `SpfMailFrom` | SPF-001 |
| `dkim` | `Option<DkimResults>` | The kinds hold `Dkim` | DKV-001 |
| `dkim2` | `Option<Dkim2Result>`, with `unstable-dkim2` | The kinds hold `Dkim2` | DK2-013 |
| `dmarc` | `Vec<DmarcResult>` | The kinds hold `Dmarc`: one result, or one for each author domain when `several_author_domains` is on. Empty otherwise | DMC-002 |
| `dkim_signing` | `Option<Signing>` | The kinds hold `DkimSign` | DKS-005 |
| `dkim2_signing` | `Option<Signing>`, with `unstable-dkim2` | The kinds hold `Dkim2Sign` | DK2-014 |
| `time_checks` | `bool` | Always: whether time checks were on | TIM-005 |

## Domain

`Domain` is a domain name as a result names it.

| Field | Type | Meaning | Requirements |
|---|---|---|---|
| `written` | `Data` ([Trace](trace.md#data)) | The name as the input wrote it, with the parties that control it | ADS-002, TRC-013 |
| `lookup` | `Option<String>` | The lookup form, in ASCII with A-labels, or `None` when the conversion failed or the labels don't fit | ADS-002 to ADS-004 |

## SPF

| Field | Type | Meaning | Requirements |
|---|---|---|---|
| `identity` | `SpfIdentity`: `Helo` or `MailFrom` | The identity checked | SPF-001, SPF-002 |
| `domain` | `Domain` | The `<domain>` of `check_host()` | ADS-023 |
| `outcome` | `SpfOutcome` | The result of RFC 7208 §2.6 | SPF-003 |
| `reason` | `SpfReason` | Why the evaluation gave that outcome | GEN-004 |
| `explanation` | `Option<Data>` | The expanded `exp=` text for `fail`, marked as from a third party | REC-016, TRC-013 |
| `position` | `Position` ([Trace](trace.md#position)) | The result's place in the trace | DMC-005 |

`SpfOutcome` is an exhaustive enum, `Copy`: `None`, `Neutral`, `Pass`, `Fail`, `Softfail`, `Temperror`, `Permerror` (RFC 7208 §2.6).

`SpfReason` is a `#[non_exhaustive]` enum. A `Position` in it points at the trace step that the reason rests on.

| Variant | Carries | Outcomes | Meaning | Requirements |
|---|---|---|---|---|
| `Matched` | `term: Position` | `Pass`, `Fail`, `Softfail`, `Neutral` | The mechanism at `term` matched | SPF-003 |
| `Default` | Nothing | `Neutral` | No mechanism matched, and no `redirect` applied (RFC 7208 §4.7) | SPF-003 |
| `NoValidDomain` | Nothing | `None` | The domain isn't a valid multi-label domain name | RFC 7208 §4.3 |
| `NoRecord` | Nothing | `None` | The domain publishes no SPF record | RFC 7208 §4.5 |
| `SeveralRecords` | Nothing | `Permerror` | The domain publishes more than one SPF record | SPF-007 |
| `TemporaryFailure` | `record_type: QuestionType`, `name: Vec<u8>`, `description: Option<Vec<u8>>` | `Temperror` | The question that failed, and the resolver's description | GEN-005, DNS-009 |
| `Syntax` | `ParseError` ([Errors](errors.md#parse-errors)) | `Permerror` | The record doesn't parse | REC-010 |
| `LookupLimit` | `term: Position` | `Permerror` | The term at `term` was the eleventh that causes a DNS lookup | SPF-006 |
| `VoidLookupLimit` | `term: Position` | `Permerror` | The term at `term` went over `spf_void_lookups` | SPF-006 |
| `MxLimit` | `term: Position` | `Permerror` | The `mx` at `term` got more than 10 MX records | SPF-006 |
| `CheckLimit` | `limit: Limit`, `term: Position` | `Permerror` | The term at `term` reached a limit of the check, such as `dns_questions` | SAF-002 |
| `Nested` | `term: Position` | `Temperror`, `Permerror` | The `include` or `redirect` at `term` gave this outcome | RFC 7208 §5.2, §6.1 |

## DKIM

`DkimResults` holds the results of DKIM verification.

| Field | Type | Meaning | Requirements |
|---|---|---|---|
| `signatures` | `Vec<DkimResult>` | One result for each DKIM-Signature field, in header order | DKV-004 |
| `stopped` | `Option<Limit>` | The header limit that stopped DKIM before any signature, which gives `permerror` for the whole check | SAF-002 |

| `DkimResult` field | Type | Meaning | Requirements |
|---|---|---|---|
| `domain` | `Option<Domain>` | `d=`, or `None` when the field gives no valid `d=` | DKV-001 |
| `identity` | `Option<Data>` | `i=` | RFC 6376 §3.5 |
| `selector` | `Option<Data>` | `s=`, or `None` when the field gives none | DKV-001 |
| `algorithm` | `Option<Data>` | `a=` as written, or `None` when the field gives none | DKV-002 |
| `outcome` | `DkimOutcome` | The result of RFC 6376 §3.9 | DKV-004 |
| `reason` | `DkimReason` | Why the signature got that outcome | GEN-004 |
| `testing` | `bool` | The key record has `t=y` | DKV-009 |
| `unsigned_body` | `Option<u64>` | Body octets after the `l=` cut | DKV-006 |
| `position` | `Position` | The result's place in the trace | DMC-005 |

`DkimOutcome` is an exhaustive enum, `Copy`: `Success`, `Permfail`, `Tempfail` (RFC 6376 §3.9).

`DkimReason` is a `#[non_exhaustive]` enum.

| Variant | Carries | Outcome | Meaning | Requirements |
|---|---|---|---|---|
| `Verified` | Nothing | `Success` | The signature verified | DKV-001 |
| `Field` | `step: Position` | `Permfail` | A check of the DKIM-Signature field failed at `step`, such as a missing tag or an expired `x=` | RFC 6376 §6.1.1 |
| `Key` | `step: Position` | `Permfail` | A check of the key record failed at `step`, such as a revoked key or a key too short | RFC 6376 §6.1.2, DKV-003 |
| `BodyHash` | `step: Position` | `Permfail` | The body hash didn't match `bh=` | RFC 6376 §6.1.3 |
| `Signature` | `step: Position` | `Permfail` | The signature didn't verify over the header hash | RFC 6376 §6.1.3 |
| `Limit` | `Limit` | `Permfail` | The signature reached a limit, such as `dkim_signatures` or `key_records` | SAF-002 |
| `KeyTemporaryFailure` | `description: Option<Vec<u8>>` | `Tempfail` | The key question failed, with the resolver's description | GEN-005, DNS-009 |

## DKIM2

`Dkim2Result` and the types in this section exist only with `unstable-dkim2`.

| Field | Type | Meaning | Requirements |
|---|---|---|---|
| `state` | `Dkim2State` | The output state of draft-09 §14.1 | DK2-013 |
| `reason` | `Dkim2Reason` | The step that decided the state | GEN-004 |
| `draft_text` | `Option<String>` | The human-readable string of draft-09 §14 with its placeholders filled in, for a state other than PASS. `None` where the draft has no string | DK2-013 |
| `header_d` | `Option<Data>` | The `header.d` property of authres-02 §3.2 | AR-004 |
| `header_i` | `Option<Data>` | The `header.i` property of authres-02 §3.2 | AR-004 |
| `position` | `Position` | The result's place in the trace | DMC-005 |

`Dkim2State` is an exhaustive enum, `Copy`: `None`, `Pass`, `Fail`, `Permerror`, `Temperror` (draft-09 §14.1).

`Dkim2Reason` is a `#[non_exhaustive]` enum. Each step variant names one step of [DKIM2](../design/dkim2.md#steps) and carries `step: Position`, the trace step that decided the state.

| Variant | Carries | States | Requirements |
|---|---|---|---|
| `NoFields` | Nothing | `None` | DK2-013, AR-004 |
| `Verified` | Nothing | `Pass` | DK2-013 |
| `Fields` | `step: Position` | `Permerror` | draft-09 §14.2 |
| `Timestamps` | `step: Position` | `Fail` | draft-09 §14.3 |
| `ChainOfCustody` | `step: Position` | `Fail` | draft-09 §14.4 |
| `Keys` | `step: Position` | `Temperror`, `Permerror` | REC-053 |
| `Signatures` | `step: Position` | `Fail`, `Permerror` | draft-09 §14.6 |
| `Hashes` | `step: Position` | `Fail` | draft-09 §14.7 |
| `AuthenticFrom` | `step: Position` | `Fail`, `Permerror` | draft-09 §14.8 |
| `Limit` | `Limit` | `Permerror` | SAF-002 |

## DMARC

| Field | Type | Meaning | Requirements |
|---|---|---|---|
| `author_domain` | `Option<Domain>` | The author domain, absent when DMARC wasn't applied for lack of one | DMC-002 |
| `outcome` | `DmarcOutcome` | The DMARC result | DMC-002, DMC-006 |
| `policy_tag` | `PolicyTag` | The policy tag that applied | DMC-006, REC-032 |
| `published` | `PublishedPolicy` | Every policy tag of the record, as published | REC-032 |
| `test_mode` | `Option<TestModeTag>`: `T` or `Pct` | The tag that set test mode: `t=y`, or `pct=0` in RFC 7489 mode | DMC-007 |
| `rests_on_testing_keys` | `bool` | A pass rests only on DKIM keys with `t=y` | DMC-008 |
| `alignments` | `Vec<Alignment>` | Each identifier checked, in the order checked | DMC-005 |
| `position` | `Position` | The result's place in the trace | DMC-005 |

| Type | Field or variant | Type or carries | Meaning | Requirements |
|---|---|---|---|---|
| `DmarcOutcome` | `Pass`, `Fail`, `None`, `Temperror`, `Permerror` | Nothing | The result of RFC 9989 or RFC 7489 | DMC-006 |
| `DmarcOutcome` | `NotApplied` | `NotApplied` | DMARC wasn't applied, and why | DMC-002 |
| `NotApplied` | `NoFrom` | Nothing | The message has no From field | DMC-002 |
| `NotApplied` | `FromUnparsable` | Nothing | A From field doesn't parse | DMC-002, FLD-055 |
| `NotApplied` | `NoMailbox` | Nothing | The From field holds a group with no mailbox | DMC-002, RFC 6854 |
| `NotApplied` | `DomainLiteral` | Nothing | The From address has a domain literal | ADS-005, [ADR 0013](../../adr/0013-a-from-domain-literal-gives-dmarc-no-author-domain.md) |
| `NotApplied` | `NoLookupForm` | Nothing | The author domain has no lookup form | ADS-002 |
| `NotApplied` | `SeveralAuthorDomains` | Nothing | More than one domain, and `several_author_domains` is off | DMC-002 |
| `NotApplied` | `AuthorDomainLimit` | Nothing | More author domains than `author_domains` | DMC-002, SAF-002 |
| `NotApplied` | `NoValidPolicy` | Nothing | The policy tag is missing or invalid, and `rua` holds no valid URI | REC-032, [ADR 0023](../../adr/0023-dmarc-follows-rfc-9989-section-4-10-1-for-invalid-policy-tags-and-section-4-7-for-nonexistent-subdomains.md) |
| `PolicyTag` | `Applied` | `tag: PolicyTagName`, `value: Data` | The tag that applied, `P`, `Sp` or `Np`, with its value as published | DMC-006 |
| `PolicyTag` | `Undetermined` | Nothing | The question about the author domain failed, so the tag couldn't be determined | REC-032 |
| `PolicyTag` | `NoPolicy` | Nothing | No DMARC Policy Record applied, as for `None` and `NotApplied` | DMC-006 |
| `PublishedPolicy` | `p`, `sp`, `np` | `Option<Data>` each | Each tag's value as published, or `None` when absent | REC-032 |
| `Alignment` | `identifier` | `Position` | The SPF or DKIM result the identifier came from | DMC-005 |
| `Alignment` | `domain` | `Domain` | The identifier's domain | DMC-005 |
| `Alignment` | `mode` | `AlignmentMode`: `Relaxed` or `Strict` | The mode from `aspf` or `adkim` | DMC-005 |
| `Alignment` | `aligned` | `bool` | Whether the domain aligned with the author domain | DMC-005 |

`DmarcOutcome`, `PolicyTag`, `PolicyTagName`, `TestModeTag` and `AlignmentMode` are exhaustive. `NotApplied` is `#[non_exhaustive]`.

## Signing

`Signing` is an exhaustive enum.

| Variant | Carries | Requirements |
|---|---|---|
| `Written` | `Vec<HeaderFieldOut>`: the header fields to add, in the order to add them | DKS-005, STR-009 |
| `Stopped` | `SigningError`: the error that stopped it ([Errors](errors.md#signing-errors-in-the-results)) | SGN-004 |

`HeaderFieldOut` holds a field name and a whole value. `value` gives the value at once, and `read_value` gives it in chunks of the caller's size (STR-002).

## Validation

### Result values

| ID | Input | Rule | Error code | Requirement | Scenario |
|---|---|---|---|---|---|
| | None | No rule: results are output, and reading a field takes no input | | GEN-004 | ARH-008 |

### `HeaderFieldOut::read_value`

| ID | Input | Rule | Error code | Requirement | Scenario |
|---|---|---|---|---|---|
| VAL-301 | `buffer` | Holds at least 1 byte | `results.buffer_empty` | STR-002 | API-301 |
| VAL-302 | `offset` | At most the length of the value | `results.offset_out_of_range` | STR-002 | API-302 |
