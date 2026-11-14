# Test scenarios: Trace

Scenarios for the trace: its levels and limits, step codes, the order of steps, marked values, cautions and hints, and the trace items of DKIM and SPF. Conventions are in [README.md](README.md).

Unless a row says otherwise, the check runs at the full level, and DNS answers come from a snapshot.

## Contents

- [Levels and limits](#levels-and-limits)
- [Codes and order](#codes-and-order)
- [Marked values](#marked-values)
- [DKIM trace items](#dkim-trace-items)

## Levels and limits

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| STEP-001 | unit | A message whose only DKIM signature fails because the body hash doesn't match `bh=` | it is verified at the summary level | the trace holds the DKIM check's top step, the signature step and the body hash step, with their facts. It leaves out the key lookup and the canonicalized header, which didn't decide the result | [TRC-004](../reference/requirements.md#trace-and-reasoning), [RFC 6376 §6.1.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.3) |
| STEP-002 | unit | The message of STEP-001 | it is verified at the full level | the trace holds every step, the key lookup with its DNS question and answer and the canonicalized header included | [TRC-004](../reference/requirements.md#trace-and-reasoning) |
| STEP-003 | unit | The message of STEP-001 and an allocator that counts allocations | it is verified at the off level | the same result, no trace, and no allocation made for the trace | [TRC-004](../reference/requirements.md#trace-and-reasoning) |
| STEP-004 | unit | A message that passes DMARC through SPF alignment, and whose DKIM signature passes with an `l=` tag | DMARC runs at the summary level | the summary holds the path through SPF alignment, and also the `l=` caution of the DKIM signature, which lies off that path | [TRC-004](../reference/requirements.md#trace-and-reasoning), [DKV-010](../reference/requirements.md#dkim-verification), [RFC 6376 §8.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-8.2) |
| STEP-005 | unit | An SPF record chain that makes more steps than a trace size limit set low for the test | the check runs | the trace ends with a step that says it was cut short, and the SPF result equals the result at the off level | [TRC-005](../reference/requirements.md#trace-and-reasoning) |
| STEP-006 | property | Any message and snapshot | the checks run at the off, summary and full levels, and at the full level with a tiny trace limit | every run gives the same results | [TRC-004](../reference/requirements.md#trace-and-reasoning), [TRC-005](../reference/requirements.md#trace-and-reasoning) |

## Codes and order

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| STEP-010 | unit | An SPF check that reaches an eleventh term causing a DNS lookup | the check runs | `permerror`. The last step has the kind code of the lookup limit and the outcome code of `permerror`, with the term and the count 11 as parameters, and the test matches those codes, never the English text | [TRC-009](../reference/requirements.md#trace-and-reasoning), [TRC-011](../reference/requirements.md#trace-and-reasoning), [RFC 7208 §4.6.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.4) |
| STEP-011 | unit | The trace of STEP-010 | it is rendered by the library's English renderer, and by a test renderer that maps each code to other words | both texts come from the same codes and parameters, and rendering changes neither | [TRC-011](../reference/requirements.md#trace-and-reasoning) |
| STEP-012 | unit | A DKIM2-Signature with `i=1` whose tag list has a syntax error | the message is verified | the step's outcome code is PERMERROR, and its draft string field holds `PERMERROR: DKIM2-Signature i=1 syntax error` as draft-09 §14.2 words it | [TRC-011](../reference/requirements.md#trace-and-reasoning), [DK2-013](../reference/requirements.md#dkim2), [draft-09 §14.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.2) |
| STEP-013 | property | The SPF record `v=spf1 include:a.void.invalid include:b.void.invalid -all`, and a snapshot that answers every question | the answers are given in every order, one at a time and all together | every run gives the same trace: each answer sits under the `include` step that asked for it, and the two `include` steps keep their order in the record | [TRC-012](../reference/requirements.md#trace-and-reasoning), [CHK-003](../reference/requirements.md#running-a-check), [RFC 7208 §5.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.2) |
| STEP-014 | unit | A DKIM2 body recipe whose copy step takes an empty line at the end of the raw body | the body ends and the trace records the assumption of DK2-008 | the assumption step sits under the recipe step that copied the line, though hallmark recorded it only when the body ended | [TRC-012](../reference/requirements.md#trace-and-reasoning), [DK2-008](../reference/requirements.md#dkim2), [ADR 0008](../adr/0008-dkim2-recipe-line-numbers-count-the-raw-body.md) |
| STEP-015 | property | Any message and snapshot | the message is checked with body chunks of every size from 1 byte to the whole body, and with the answers in a random order, and each trace is serialized | the serialized traces are the same bytes | [TRC-012](../reference/requirements.md#trace-and-reasoning), [STR-005](../reference/requirements.md#streaming) |

## Marked values

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| STEP-020 | unit | `From: "<b>x</b>" <a@void.invalid>` | DMARC runs | the trace holds the display name `"<b>x</b>"` unchanged, with no escaping, and marks it as a value the sender controls | [TRC-013](../reference/requirements.md#trace-and-reasoning), [ADS-011](../reference/requirements.md#addresses-and-domain-names) |
| STEP-021 | unit | The SPF record `v=spf1 -all exp=explain.void.invalid`, and the TXT record `%{i} is not allowed <a href=x>` at `explain.void.invalid`, for the client 192.0.2.1 | the check runs | `fail`. The trace holds the `exp=` lookup and the explanation `192.0.2.1 is not allowed <a href=x>`, unescaped, marked as controlled by a DNS publisher and as a third-party explanation | [TRC-013](../reference/requirements.md#trace-and-reasoning), [RFC 7208 §6.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.2) |
| STEP-022 | unit | The SPF record `v=spf1 a:%{l}.void.invalid -all`, and MAIL FROM `<user@void.invalid>` | the check runs | the trace records the expansion of `%{l}.void.invalid` to `user.void.invalid`, marked as a value the sender controls | [TRC-013](../reference/requirements.md#trace-and-reasoning), [RFC 7208 §7.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.2) |

## DKIM trace items

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| STEP-030 | unit | A message with one From field, signed with `h=from:to:subject` | it is verified | `pass`, with a caution that a From field added later would leave the signature valid, citing RFC 6376 §8.15. The result is the same as without the caution | [DKV-010](../reference/requirements.md#dkim-verification), [TRC-003](../reference/requirements.md#trace-and-reasoning), [RFC 6376 §8.15](https://www.rfc-editor.org/rfc/rfc6376.html#section-8.15) |
| STEP-031 | unit | The same message, signed with `h=from:from:to:subject` | it is verified | `pass` with no From caution. The trace records that the second `from` had no instance left and contributed nothing | [DKV-010](../reference/requirements.md#dkim-verification), [DKV-011](../reference/requirements.md#dkim-verification), [RFC 6376 §5.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.4) |
| STEP-032 | unit | A message signed with a 1024-bit RSA key, and the same message signed with a 2048-bit RSA key, both generated by the test | each is verified | both `pass`. Only the 1024-bit signature has a caution, which cites RFC 8301 §3.2 | [DKV-010](../reference/requirements.md#dkim-verification), [RFC 8301 §3.2](https://www.rfc-editor.org/rfc/rfc8301.html#section-3.2) |
| STEP-033 | unit | A signature whose `l=` covers the whole body | it is verified | `pass`, 0 unsigned bytes (DKV-006), and a caution that cites RFC 6376 §8.2 | [DKV-010](../reference/requirements.md#dkim-verification), [DKV-006](../reference/requirements.md#dkim-verification), [RFC 6376 §8.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-8.2) |
| STEP-034 | unit | A signature whose `h=` names `reply-to`, and a message with no Reply-To field | it is verified | `pass`. The trace records that `reply-to` had no instance, and the signature verifies with nothing hashed for it | [DKV-011](../reference/requirements.md#dkim-verification), [RFC 6376 §3.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.5), [RFC 6376 §5.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.4) |
| STEP-035 | unit | A signed message whose Subject gained the prefix `[list] ` after signing, with `subject` in `h=` | it is verified | `fail`. The body hash matches, and the trace records as facts that the signature over the header hash failed and that DKIM keeps nothing that tells which field changed. A hint names Subject among the fields that mailing lists commonly change, citing RFC 6377 §3.3 | [DKV-012](../reference/requirements.md#dkim-verification), [TRC-003](../reference/requirements.md#trace-and-reasoning), [RFC 6377 §3.3](https://www.rfc-editor.org/rfc/rfc6377.html#section-3.3) |
