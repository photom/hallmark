# Test scenarios: Test modes and published policy

Scenarios for how DKIM and DMARC results report test modes and the published policy, which the caller then applies ([ADR 0021](../adr/0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md)). Conventions are in [README.md](README.md).

Unless a row says otherwise, the message fails DMARC, the author domain is `void.invalid`, and the record is published at `_dmarc.void.invalid`.

## DMARC policy and test mode

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| POL-001 | unit | RFC 9989 mode, and the record `v=DMARC1; p=reject; t=y` | DMARC runs | `fail`, the tag `p` with the published value `reject`, and test mode on, set by `t`. The result holds no other policy | [DMC-006](../reference/requirements.md#dmarc), [DMC-007](../reference/requirements.md#dmarc), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7) |
| POL-002 | unit | RFC 9989 mode, and the record `v=DMARC1; p=reject` | DMARC runs | `fail`, `p` with `reject`, and test mode off | [DMC-006](../reference/requirements.md#dmarc), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7) |
| POL-003 | unit | RFC 9989 mode, and the record `v=DMARC1; p=none; t=y` | DMARC runs | `fail`, `p` with `none`, and test mode on. The result reports `t=y` as published, though it has no effect on `none` | [DMC-006](../reference/requirements.md#dmarc), [DMC-007](../reference/requirements.md#dmarc), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7) |
| POL-004 | unit | RFC 9989 mode, the author domain `sub.void.invalid` with no record of its own, and the record `v=DMARC1; p=reject; sp=quarantine; t=y` at `_dmarc.void.invalid` | DMARC runs | `fail`, the tag `sp` with `quarantine`, and test mode on | [DMC-006](../reference/requirements.md#dmarc), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7), [RFC 9989 §4.10.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10.1) |
| POL-005 | unit | RFC 7489 mode, and the record `v=DMARC1; p=reject; pct=0` | DMARC runs | `fail`, `p` with `reject`, and test mode on, set by `pct` | [DMC-007](../reference/requirements.md#dmarc), [RFC 9989 Appendix A.6](https://www.rfc-editor.org/rfc/rfc9989.html#appendix-A.6) |
| POL-006 | unit | RFC 7489 mode, and the record `v=DMARC1; p=reject; pct=50` | DMARC runs | `fail`, `p` with `reject`, and test mode off. The trace records that hallmark ignored `pct=50` and read the record as if `pct` were absent | [DMC-007](../reference/requirements.md#dmarc), [ADR 0021](../adr/0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md), [RFC 9989 Appendix A.6](https://www.rfc-editor.org/rfc/rfc9989.html#appendix-A.6) |
| POL-007 | unit | RFC 7489 mode, and the record `v=DMARC1; p=reject; pct=100` | DMARC runs | `fail`, `p` with `reject`, test mode off, and no ignored value in the trace | [DMC-007](../reference/requirements.md#dmarc), [RFC 7489 §6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.3) |
| POL-008 | unit | RFC 9989 mode, and the record `v=DMARC1; p=reject; pct=0` | DMARC runs | `fail`, `p` with `reject`, and test mode off: RFC 9989 has no `pct` tag | [DMC-004](../reference/requirements.md#dmarc), [DMC-007](../reference/requirements.md#dmarc), [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7) |
| POL-009 | unit | RFC 7489 mode, and the record `v=DMARC1; p=reject; t=y` | DMARC runs | `fail`, `p` with `reject`, and test mode off: RFC 7489 has no `t` tag | [DMC-004](../reference/requirements.md#dmarc), [DMC-007](../reference/requirements.md#dmarc), [RFC 7489 §6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.3) |
| POL-010 | unit | DMARC run with SPF and DKIM results that the caller got from separate calls | DMARC runs | each alignment step names the SPF result or the DKIM signature, by its `d=` and `s=` and its place in the header section, that it used | [DMC-005](../reference/requirements.md#dmarc), [GEN-001](../reference/requirements.md#general) |
| POL-011 | unit | RFC 9989 mode, and the record `v=DMARC1; p=reject` | DMARC runs and the caller reads the result | the result holds the published value and no verdict: no field tells the caller to reject or quarantine, and hallmark changes nothing in the message | [GEN-006](../reference/requirements.md#general), [DMC-006](../reference/requirements.md#dmarc), [RFC 9989 §5.4](https://www.rfc-editor.org/rfc/rfc9989.html#section-5.4) |

## Testing-mode DKIM keys

In these rows the message passes DMARC unless a row says otherwise.

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| POL-020 | unit | A valid signature whose key record holds `t=y` | the signature is verified | `pass`, with the testing flag set | [DKV-009](../reference/requirements.md#dkim-verification), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1) |
| POL-021 | unit | A signature that doesn't verify, whose key record holds `t=y` | the signature is verified | `fail`, with the testing flag set | [DKV-009](../reference/requirements.md#dkim-verification), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1) |
| POL-022 | unit | A valid signature whose key record holds `t=y:s` | the signature is verified | `pass`, with the testing flag set, and the `s` flag applied as RFC 6376 §3.6.1 says | [DKV-005](../reference/requirements.md#dkim-verification), [DKV-009](../reference/requirements.md#dkim-verification), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1) |
| POL-023 | unit | An SPF result that doesn't align, and one aligned DKIM pass from a key with `t=y` | DMARC runs | `pass`, and the result says that the pass rests only on testing-mode keys | [DMC-008](../reference/requirements.md#dmarc), [RFC 9989 §4.4.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.4.1) |
| POL-024 | unit | An SPF result that doesn't align, an aligned DKIM pass from a key with `t=y`, and another from a key without it | DMARC runs | `pass`, and the result doesn't say that the pass rests only on testing-mode keys | [DMC-008](../reference/requirements.md#dmarc) |
| POL-025 | unit | An aligned SPF pass, and an aligned DKIM pass from a key with `t=y` | DMARC runs | `pass`, and the result doesn't say that the pass rests only on testing-mode keys | [DMC-008](../reference/requirements.md#dmarc) |
| POL-026 | unit | An SPF result that doesn't align, an aligned DKIM pass from a key with `t=y`, and a DKIM pass with `d=other.invalid`, which doesn't align, from a key without it | DMARC runs | `pass`, and the result says that the pass rests only on testing-mode keys: only aligned passes count | [DMC-008](../reference/requirements.md#dmarc), [RFC 9989 §4.4.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.4.1) |
