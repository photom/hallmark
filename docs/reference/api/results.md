# Results

The values that `Check::take_results` gives: one result for each kind of check, each with a typed reason (GEN-004) and its position in the trace (DMC-005). [Domain model](../design/domain-model.md#results) draws the types.

## Contents

- [Members](#members)
- [SPF](#spf)
- [DKIM](#dkim)
- [DKIM2](#dkim2)
- [DMARC](#dmarc)
- [Signing](#signing)
- [Validation](#validation)
  - [Result values](#result-values)
  - [`HeaderFieldOut::read_value`](#headerfieldoutread_value)

## Members

| Member | Type | Present when | Requirements |
|---|---|---|---|
| `spf_helo` | `SpfResult` | The kinds hold SPF HELO | SPF-002 |
| `spf_mail_from` | `SpfResult` | The kinds hold SPF MAIL FROM | SPF-001 |
| `dkim` | `DkimResults` | The kinds hold DKIM | DKV-001 |
| `dkim2` | `Dkim2Result` | The kinds hold DKIM2 | DK2-013 |
| `dmarc` | List of `DmarcResult` | The kinds hold DMARC: one result, or one for each author domain when `several_author_domains` is on | DMC-002 |
| `dkim_signing` | `Signing` | The kinds hold DKIM signing | DKS-005 |
| `dkim2_signing` | `Signing` | The kinds hold DKIM2 signing | DK2-014 |
| `time_checks` | Boolean | Always: whether time checks were on | TIM-005 |

## SPF

| Member | Type | Meaning | Requirements |
|---|---|---|---|
| `identity` | HELO or MAIL FROM | The identity checked | SPF-001 |
| `domain` | `Domain` | The `<domain>` of `check_host()` | ADS-023 |
| `outcome` | `none`, `neutral`, `pass`, `fail`, `softfail`, `temperror`, `permerror` | The result of RFC 7208 §2.6 | SPF-003 |
| `reason` | `SpfReason` | The mechanism that matched and its position, no match, no record, the limit or the syntax error that stopped the evaluation, or the question that failed | GEN-004 |
| `explanation` | Data, optional | The expanded `exp=` text for `fail`, marked as from a third party | REC-016, TRC-013 |
| `position` | `Position` | The result's place in the trace | DMC-005 |

## DKIM

`DkimResults` holds a `DkimResult` for each DKIM-Signature field, in header order, and a check error when a header limit stopped DKIM before any signature (`permerror`, SAF-002).

| Member | Type | Meaning | Requirements |
|---|---|---|---|
| `domain` | `Domain` | `d=` | DKV-001 |
| `identity` | Data, optional | `i=` | RFC 6376 §3.5 |
| `selector` | Data | `s=` | DKV-001 |
| `algorithm` | Text | `a=` as written | DKV-002 |
| `outcome` | `success`, `permfail`, `tempfail` | The result of RFC 6376 §3.9 | DKV-004 |
| `reason` | `DkimReason` | The cause of a failure, or the limit the signature reached | GEN-004 |
| `testing` | Boolean | The key record has `t=y` | DKV-009 |
| `unsigned_body` | Count, optional | Body octets after the `l=` cut | DKV-006 |
| `position` | `Position` | The result's place in the trace | DMC-005 |

## DKIM2

| Member | Type | Meaning | Requirements |
|---|---|---|---|
| `state` | `NONE`, `PASS`, `FAIL`, `PERMERROR`, `TEMPERROR` | The output state of draft-09 §14.1 | DK2-013 |
| `reason` | `Dkim2Reason` | The case that decided the state | GEN-004 |
| `draft_text` | Text, optional | The human-readable string of draft-09 §14 with its placeholders filled in, for a state other than PASS | DK2-013 |
| `header_d`, `header_i` | Data, optional | The `header.d` and `header.i` properties of authres-02 §3.2 | AR-004 |
| `position` | `Position` | The result's place in the trace | DMC-005 |

## DMARC

| Member | Type | Meaning | Requirements |
|---|---|---|---|
| `author_domain` | `Domain`, optional | The author domain, absent when DMARC wasn't applied for lack of one | DMC-002 |
| `outcome` | `pass`, `fail`, `none`, `temperror`, `permerror`, or not applied with the reason | The DMARC result | DMC-002, DMC-006 |
| `policy_tag` | `p`, `sp` or `np` with its published value, or undetermined | The policy tag that applied | DMC-006, REC-032 |
| `published` | The `p`, `sp` and `np` values as published | Every policy tag of the record | REC-032 |
| `test_mode` | The tag that set it, or none | `t=y`, or `pct=0` in RFC 7489 mode | DMC-007 |
| `rests_on_testing_keys` | Boolean | A pass rests only on DKIM keys with `t=y` | DMC-008 |
| `alignments` | List of `Alignment`: the identifier's position, its domain, `relaxed` or `strict`, and whether it aligned | Each identifier checked | DMC-005 |
| `position` | `Position` | The result's place in the trace | DMC-005 |

## Signing

| `Signing` | Carries | Requirements |
|---|---|---|
| `Written` | The header fields to add, as `HeaderFieldOut` values in the order to add them | DKS-005, STR-009 |
| `Stopped` | The `SigningError` that stopped it ([Errors](errors.md#signing-errors-in-the-results)) | SGN-004 |

`HeaderFieldOut` holds a field name and a whole value in the form `field_form` states (STR-009). `value` gives the value at once, and `read_value` gives it in chunks of the caller's size (STR-002).

## Validation

### Result values

| ID | Input | Rule | Error code | Requirement | Scenario |
|---|---|---|---|---|---|
| | None | No rule: results are output, and reading a member takes no input | | GEN-004 | ARH-008 |

### `HeaderFieldOut::read_value`

| ID | Input | Rule | Error code | Requirement | Scenario |
|---|---|---|---|---|---|
| VAL-301 | `buffer` | Holds at least 1 byte | `results.buffer_empty` | STR-002 | API-301 |
| VAL-302 | `offset` | At most the length of the value | `results.offset_out_of_range` | STR-002 | API-302 |
