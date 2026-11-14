# Test scenarios: DMARC records

Scenarios for the DMARC record parser: record selection, tag syntax, invalid values and the rule of RFC 9989 §4.10.1, reporting URIs, and RFC 7489 mode. Conventions are in [README.md](README.md).

Unless a row says otherwise, DMARC runs in RFC 9989 mode, the author domain is `void.invalid`, the record is the only TXT record at `_dmarc.void.invalid`, the message has no aligned pass, and DNS answers come from a snapshot. The snapshot answers every question the check asks, and a name that a row doesn't mention, such as `_dmarc.invalid` in the tree walk, has no TXT records.

## Contents

- [Record selection](#record-selection)
- [Tag syntax](#tag-syntax)
- [Invalid values and the policy](#invalid-values-and-the-policy)
- [Reporting URIs](#reporting-uris)
- [RFC 7489 mode](#rfc-7489-mode)
- [Properties](#properties)

## Record selection

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| DMR-001 | unit | The record `v=DMARC1; p=reject` | DMARC runs | `fail`, with the policy tag `p` and its published value `reject` | [REC-030](../reference/requirements.md#dns-records), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7) |
| DMR-002 | unit | The record `v=dmarc1; p=reject`, and no record at `_dmarc.invalid` | DMARC runs | DMARC isn't applied. hallmark drops the record because its version doesn't match exactly, the trace records why, and the tree walk asks for `_dmarc.invalid` | [REC-030](../reference/requirements.md#dns-records), [REC-001](../reference/requirements.md#dns-records), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7), [RFC 9989 §4.10](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10) |
| DMR-003 | unit | The record `p=reject; v=DMARC1` | DMARC runs | hallmark drops the record: `v` isn't the first tag | [REC-030](../reference/requirements.md#dns-records), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7) |
| DMR-004 | unit | The author domain `mail.void.invalid`, the records `v=DMARC1; p=reject` and `v=DMARC1; p=none` at `_dmarc.mail.void.invalid`, and the record `v=DMARC1; p=quarantine` at `_dmarc.void.invalid` | DMARC runs | hallmark discards both records at `_dmarc.mail.void.invalid`, the tree walk finds `_dmarc.void.invalid`, and the policy is `quarantine` from its `p` | [REC-030](../reference/requirements.md#dns-records), [RFC 9989 §4.10](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10), [RFC 9989 §4.10.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10.1) |
| DMR-005 | unit | The records of DMR-004 | DMARC runs in RFC 7489 mode | DMARC isn't applied, because two records remain at the first name | [REC-030](../reference/requirements.md#dns-records), [RFC 7489 §6.6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.6.3) |

## Tag syntax

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| DMR-010 | unit | The records `v=DMARC1;p=reject` and `v = DMARC1 ; p = reject` in two snapshots | DMARC runs on each | the policy `reject` both times | [REC-031](../reference/requirements.md#dns-records), [RFC 9989 §4.8](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.8) |
| DMR-011 | unit | The record `v=DMARC1; P=reject; rua=mailto:dmarc@void.invalid` | DMARC runs | the policy `none`, set by the rule of RFC 9989 §4.10.1: `P` is an unknown tag, which the trace lists, so the record has no `p` | [REC-031](../reference/requirements.md#dns-records), [REC-032](../reference/requirements.md#dns-records), [ADR 0025](../adr/0025-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md), [RFC 6376 §3.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.2) |
| DMR-012 | unit | The record `v=DMARC1; p=REJECT` | DMARC runs | the policy `reject`, with the published value `REJECT` | [REC-031](../reference/requirements.md#dns-records), [ADR 0025](../adr/0025-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md), [RFC 5234 §2.3](https://www.rfc-editor.org/rfc/rfc5234.html#section-2.3) |
| DMR-013 | unit | The records `v=DMARC1; p=reject ` and `v=DMARC1; p=reject\t`, which end with a space and a tab, in two snapshots | DMARC runs on each | the policy `reject` both times | [REC-031](../reference/requirements.md#dns-records), [ADR 0025](../adr/0025-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md) |
| DMR-014 | unit | The record `v=DMARC1; p=` | DMARC runs | DMARC isn't applied: the empty `p` is invalid, and the record has no `rua` | [REC-031](../reference/requirements.md#dns-records), [REC-032](../reference/requirements.md#dns-records), [RFC 9989 §4.8](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.8) |
| DMR-015 | unit | The record `v=DMARC1; p=reject; p=none; rua=mailto:dmarc@void.invalid` | DMARC runs | the policy `none`, set by the rule of RFC 9989 §4.10.1. The trace marks both copies of `p` as invalid, and the result lists both as published | [REC-031](../reference/requirements.md#dns-records), [REC-032](../reference/requirements.md#dns-records), [ADR 0025](../adr/0025-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md) |
| DMR-016 | unit | The record `v=DMARC1; p=reject; adkim=s; adkim=r` | DMARC runs | the policy `reject` and relaxed DKIM alignment: the repeated `adkim` takes its default | [REC-031](../reference/requirements.md#dns-records), [REC-032](../reference/requirements.md#dns-records), [ADR 0025](../adr/0025-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md) |
| DMR-017 | unit | The record `v=DMARC1; p=reject; v=DMARC2` | DMARC runs | the policy `reject`. hallmark ignores the second `v` | [REC-031](../reference/requirements.md#dns-records), [ADR 0025](../adr/0025-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md) |
| DMR-018 | unit | The record `v=DMARC1; p=reject; foo=bar` | DMARC runs | the policy `reject`, and the trace lists `foo` as an unknown tag | [REC-031](../reference/requirements.md#dns-records), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7) |
| DMR-019 | unit | The record `v=DMARC1; p=reject; pct=50; rf=afrf; ri=3600` | DMARC runs | the policy `reject` and no test mode. `pct`, `rf` and `ri` are unknown tags in RFC 9989 mode | [REC-031](../reference/requirements.md#dns-records), [DMC-004](../reference/requirements.md#dmarc), [RFC 9989 Appendix C.5.2](https://www.rfc-editor.org/rfc/rfc9989.html#appendix-C.5.2) |

## Invalid values and the policy

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| DMR-030 | unit | The record `v=DMARC1` | DMARC runs | DMARC isn't applied: the record has no `p` and no `rua`. The result names the rule of RFC 9989 §4.10.1 | [REC-032](../reference/requirements.md#dns-records), [ADR 0024](../adr/0024-dmarc-follows-rfc-9989-section-4-10-1-for-invalid-policy-tags-and-section-4-7-for-nonexistent-subdomains.md), [RFC 9989 §4.10.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10.1) |
| DMR-031 | unit | The record `v=DMARC1; rua=mailto:dmarc@void.invalid` | DMARC runs | the policy `none`, set by the rule of RFC 9989 §4.10.1 | [REC-032](../reference/requirements.md#dns-records), [RFC 9989 §4.10.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10.1) |
| DMR-032 | unit | The record `v=DMARC1; p=reject; sp=bogus` | DMARC runs | DMARC isn't applied. The result lists `p=reject` and `sp=bogus` as published, and names the rule of RFC 9989 §4.10.1 | [REC-032](../reference/requirements.md#dns-records), [ADR 0024](../adr/0024-dmarc-follows-rfc-9989-section-4-10-1-for-invalid-policy-tags-and-section-4-7-for-nonexistent-subdomains.md), [RFC 9989 §4.10.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10.1) |
| DMR-033 | unit | The record `v=DMARC1; p=reject; sp=bogus; rua=mailto:dmarc@void.invalid` | DMARC runs | the policy `none`, though the result lists `p=reject` as published | [REC-032](../reference/requirements.md#dns-records), [ADR 0024](../adr/0024-dmarc-follows-rfc-9989-section-4-10-1-for-invalid-policy-tags-and-section-4-7-for-nonexistent-subdomains.md) |
| DMR-034 | unit | The record `v=DMARC1; p=reject; np=bogus; rua=mailto:dmarc@void.invalid` | DMARC runs | the policy `none` | [REC-032](../reference/requirements.md#dns-records), [RFC 9989 §4.10.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10.1) |
| DMR-035 | unit | The author domain `gone.void.invalid`, which doesn't exist, and the record `v=DMARC1; p=reject; sp=none` at `_dmarc.void.invalid` | DMARC runs | the policy tag `sp` with the value `none`: with no `np`, a nonexistent subdomain takes `sp` | [REC-032](../reference/requirements.md#dns-records), [ADR 0024](../adr/0024-dmarc-follows-rfc-9989-section-4-10-1-for-invalid-policy-tags-and-section-4-7-for-nonexistent-subdomains.md), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7), [RFC 9989 §3.2.13](https://www.rfc-editor.org/rfc/rfc9989.html#section-3.2.13) |
| DMR-036 | unit | The author domain of DMR-035 and the record `v=DMARC1; p=reject; sp=quarantine; np=none` | DMARC runs | the policy tag `np` with the value `none` | [REC-032](../reference/requirements.md#dns-records), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7) |
| DMR-037 | unit | The record `v=DMARC1; p=reject; adkim=x; aspf=s` | DMARC runs | the policy `reject`, relaxed DKIM alignment from the default, and strict SPF alignment | [REC-032](../reference/requirements.md#dns-records), [RFC 9989 §4.8](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.8) |
| DMR-038 | unit | The record `v=DMARC1; p=quarantine; t=maybe` | DMARC runs | the policy `quarantine` and no test mode: the invalid `t` takes its default, `n` | [REC-032](../reference/requirements.md#dns-records), [RFC 9989 §4.8](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.8) |
| DMR-039 | unit | The record `v=DMARC1; p=reject; psd=x` | DMARC runs | the policy `reject`, and the record counts as `psd=u` in the tree walk | [REC-032](../reference/requirements.md#dns-records), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7) |
| DMR-040 | unit | The record `v=DMARC1; p=bogus; adkim=s; t=y; rua=mailto:dmarc@void.invalid` | DMARC runs | the policy `none`, strict DKIM alignment, and test mode: the record's other valid tags keep their values | [REC-032](../reference/requirements.md#dns-records), [ADR 0024](../adr/0024-dmarc-follows-rfc-9989-section-4-10-1-for-invalid-policy-tags-and-section-4-7-for-nonexistent-subdomains.md) |

## Reporting URIs

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| DMR-050 | unit | The record `v=DMARC1; p=reject; rua=mailto:a@void.invalid , mailto:b@void.invalid!10m` | DMARC runs | the result lists both URIs as published, both valid, and hallmark ignores the size `!10m` | [REC-033](../reference/requirements.md#dns-records), [RFC 9989 §4.8](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.8) |
| DMR-051 | unit | The record `v=DMARC1; p=bogus; rua=not a uri` | DMARC runs | DMARC isn't applied: the only `rua` entry has spaces, which RFC 3986 doesn't allow | [REC-033](../reference/requirements.md#dns-records), [REC-032](../reference/requirements.md#dns-records), [RFC 3986 §3](https://www.rfc-editor.org/rfc/rfc3986.html#section-3) |
| DMR-052 | unit | The record `v=DMARC1; p=bogus; rua=not a uri, mailto:dmarc@void.invalid` | DMARC runs | the policy `none`. The result marks the first entry as invalid and the second as valid | [REC-033](../reference/requirements.md#dns-records), [REC-032](../reference/requirements.md#dns-records) |

## RFC 7489 mode

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| DMR-060 | unit | The record `v=DMARC1; rua=mailto:dmarc@void.invalid; p=reject` | DMARC runs in RFC 7489 mode | the policy `reject`: `p` may come anywhere after `v` | [REC-031](../reference/requirements.md#dns-records), [ADR 0025](../adr/0025-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md), [RFC 7489 §6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.3) |
| DMR-061 | unit | The record `v=DMARC1; p=reject; sp=bogus` | DMARC runs in RFC 7489 mode | DMARC isn't applied | [REC-032](../reference/requirements.md#dns-records), [RFC 7489 §6.6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.6.3) |
| DMR-062 | unit | The record `v=DMARC1; p=reject; rf=afrf; ri=3600` | DMARC runs in RFC 7489 mode | the policy `reject`, and the result lists `rf` and `ri` as published | [REC-031](../reference/requirements.md#dns-records), [RFC 7489 §6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.3) |
| DMR-063 | unit | The record `v=DMARC1; p=reject; t=y` | DMARC runs in RFC 7489 mode | the policy `reject` and no test mode: `t` is an unknown tag in RFC 7489 mode | [REC-031](../reference/requirements.md#dns-records), [DMC-004](../reference/requirements.md#dmarc), [RFC 7489 §6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.3) |

## Properties

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| DMR-080 | property | Any byte string of up to 64 KiB after `v=DMARC1;` | it is parsed as a DMARC record | the parser returns without panicking, and each tag it returns has a byte range inside the input | [REC-031](../reference/requirements.md#dns-records), [PAR-002](../reference/requirements.md#parsers), [SAF-001](../reference/requirements.md#hostile-input) |
| DMR-081 | fuzz | The DMARC record parser | it is fuzzed | it never panics, and the property of DMR-080 holds | [SAF-003](../reference/requirements.md#hostile-input) |
| DMR-082 | fuzz | The parser of DMARC reporting URIs | it is fuzzed | it never panics | [SAF-003](../reference/requirements.md#hostile-input), [REC-033](../reference/requirements.md#dns-records) |
