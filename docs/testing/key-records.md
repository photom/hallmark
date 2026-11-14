# Test scenarios: Key records

Scenarios for DKIM key records, DKIM2 key records and DKIM2 Domain Policy Records: selection, tags, key data, several records at one name, and the outcome of each problem. Conventions are in [README.md](README.md).

Unless a row says otherwise, the signature has `d=void.invalid` and `s=sel`, the key record is the only TXT record at `sel._domainkey.void.invalid`, its key is generated while the test runs and signed the message, and DNS answers come from a snapshot. The snapshot answers every question the check asks, and a name that a row doesn't mention has no records of the type asked. `<key>` stands for that key's `p=` value.

## Contents

- [DKIM key records](#dkim-key-records)
- [RSA and Ed25519 key data](#rsa-and-ed25519-key-data)
- [Several records at a DKIM selector](#several-records-at-a-dkim-selector)
- [DKIM2 key records](#dkim2-key-records)
- [Domain Policy Records](#domain-policy-records)
- [Properties](#properties)

## DKIM key records

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| KEY-001 | unit | The key record `v=DKIM1; p=<key>` | the message is verified | `pass` | [REC-050](../reference/requirements.md#dns-records), [DKV-005](../reference/requirements.md#dkim-verification), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1) |
| KEY-002 | unit | The key record `v=DKIM2; p=<key>` | the message is verified | PERMFAIL (no key for signature). hallmark drops the record for its version, and the trace says so | [REC-050](../reference/requirements.md#dns-records), [REC-001](../reference/requirements.md#dns-records), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1) |
| KEY-003 | unit | The key record `k=rsa; v=DKIM1; p=<key>` | the message is verified | PERMFAIL (key syntax error): `v=` isn't the first tag | [REC-050](../reference/requirements.md#dns-records), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1), [RFC 6376 §6.1.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.2) |
| KEY-004 | unit | The key record `v=DKIM1; p=<key>; p=<key>` | the message is verified | PERMFAIL (key syntax error): a repeated tag makes the tag list invalid | [REC-050](../reference/requirements.md#dns-records), [HDR-006](../reference/requirements.md#header-parsing), [RFC 6376 §3.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.2) |
| KEY-005 | unit | A selector name that doesn't exist, then a selector name with no TXT records, then a temporary failure | the message is verified each time | PERMFAIL (no key for signature) twice, then TEMPFAIL (key unavailable) | [REC-050](../reference/requirements.md#dns-records), [RFC 6376 §6.1.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.2) |
| KEY-006 | unit | The key record `v=DKIM1; h=sha1; p=<key>`, and an `rsa-sha256` signature | the message is verified | PERMFAIL (inappropriate hash algorithm) | [REC-051](../reference/requirements.md#dns-records), [RFC 6376 §6.1.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.2) |
| KEY-007 | unit | The key record `v=DKIM1; h=sha256:future-hash; p=<key>` | the message is verified | `pass`. hallmark ignores `future-hash` | [REC-050](../reference/requirements.md#dns-records), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1) |
| KEY-008 | unit | The key record `v=DKIM1; s=other; p=<key>` | the message is verified | PERMFAIL (no key for signature): the record doesn't apply to email, and hallmark ignores it | [REC-050](../reference/requirements.md#dns-records), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1) |
| KEY-009 | unit | The key record `v=DKIM1; s=email:other; p=<key>` | the message is verified | `pass` | [REC-050](../reference/requirements.md#dns-records), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1) |
| KEY-010 | unit | The key record `v=DKIM1; t=y:future-flag; p=<key>` | the message is verified | `pass`, flagged as testing. hallmark ignores `future-flag` | [REC-050](../reference/requirements.md#dns-records), [DKV-009](../reference/requirements.md#dkim-verification), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1) |
| KEY-011 | unit | The key record `v=DKIM1; n=caf=C3=A9; p=<key>` | the message is verified | `pass`. hallmark doesn't interpret `n=` | [REC-050](../reference/requirements.md#dns-records), [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1) |

## RSA and Ed25519 key data

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| KEY-020 | unit | The key record of RFC 6376 Appendix C | the record is read | a 1024-bit RSA key, and the trace records that `p=` held a SubjectPublicKeyInfo | [REC-051](../reference/requirements.md#dns-records), [ADR 0027](../adr/0027-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md), [RFC 6376 Appendix C](https://www.rfc-editor.org/rfc/rfc6376.html#appendix-C) |
| KEY-021 | unit | A 2048-bit RSA key published in `p=` as an RSAPublicKey | the message is verified | `pass`, and the trace records the RSAPublicKey form | [REC-051](../reference/requirements.md#dns-records), [ADR 0027](../adr/0027-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md), [RFC 8017 Appendix A.1.1](https://www.rfc-editor.org/rfc/rfc8017.html#appendix-A.1.1) |
| KEY-022 | unit | The key of KEY-021 published as a SubjectPublicKeyInfo | the message is verified | `pass` | [REC-051](../reference/requirements.md#dns-records), [ADR 0027](../adr/0027-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md), [RFC 5280 §4.1](https://www.rfc-editor.org/rfc/rfc5280.html#section-4.1) |
| KEY-023 | unit | The key of KEY-021 in a SubjectPublicKeyInfo that names `id-RSASSA-PSS` | the message is verified | PERMFAIL (key syntax error) | [REC-051](../reference/requirements.md#dns-records), [ADR 0027](../adr/0027-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md), [RFC 4055 §1.2](https://www.rfc-editor.org/rfc/rfc4055.html#section-1.2) |
| KEY-024 | unit | The key of KEY-021 in a SubjectPublicKeyInfo with `rsaEncryption` and no parameters | the message is verified | PERMFAIL (key syntax error): the parameters must be NULL | [REC-051](../reference/requirements.md#dns-records), [ADR 0027](../adr/0027-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md), [RFC 4055 §1.2](https://www.rfc-editor.org/rfc/rfc4055.html#section-1.2) |
| KEY-025 | unit | The RSAPublicKey of KEY-021 followed by one more byte | the message is verified | PERMFAIL (key syntax error) | [REC-051](../reference/requirements.md#dns-records), [ADR 0027](../adr/0027-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md) |
| KEY-026 | unit | The Ed25519 public key of RFC 8463 Appendix A.2 in the key record `v=DKIM1; k=ed25519; p=11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=`, and a message the test signs with the secret key of Appendix A.1 | the message is verified | `pass` | [REC-051](../reference/requirements.md#dns-records), [RFC 8463 §4.2](https://www.rfc-editor.org/rfc/rfc8463.html#section-4.2), [RFC 8463 Appendix A](https://www.rfc-editor.org/rfc/rfc8463.html#appendix-A) |
| KEY-027 | unit | A `k=ed25519` record whose `p=` decodes to 31 octets | the message is verified | PERMFAIL (key syntax error) | [REC-051](../reference/requirements.md#dns-records), [dns-03 §4.3](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-4.3) |
| KEY-028 | unit | The key record `v=DKIM1; p=` | the message is verified | PERMFAIL (key revoked) | [REC-051](../reference/requirements.md#dns-records), [RFC 6376 §6.1.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.2) |
| KEY-029 | unit | The key record `v=DKIM1; k=ed25519; p=<Ed25519 key>`, and an `rsa-sha256` signature | the message is verified | PERMFAIL (inappropriate key algorithm) | [REC-051](../reference/requirements.md#dns-records), [RFC 6376 §6.1.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.2) |

## Several records at a DKIM selector

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| KEY-030 | unit | The records `v=DKIM1; p=<old key>` and `v=DKIM1; p=<key>`, and a message signed with `<key>` | the message is verified with the records in each order | `pass` both times. The trace holds both records and a caution that cites RFC 6376 §3.6.2.2 and dns-03 §3 | [REC-052](../reference/requirements.md#dns-records), [ADR 0026](../adr/0026-a-dkim-selector-with-several-key-records-passes-if-any-of-them-verifies-the-signature.md), [RFC 6376 §6.1.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.2) |
| KEY-031 | unit | The records `v=DKIM1; p=` and `v=DKIM1; p=<old key>`, and a message signed with `<key>` | the message is verified | the signature fails, and the result lists both reasons: key revoked, and a signature that didn't verify | [REC-052](../reference/requirements.md#dns-records), [ADR 0026](../adr/0026-a-dkim-selector-with-several-key-records-passes-if-any-of-them-verifies-the-signature.md) |
| KEY-032 | unit | The limit of key records per selector lowered to 2 for the test, and three records, the third with `<key>` | the message is verified | PERMFAIL before hallmark tries any record, and a trace step that names the limit | [REC-052](../reference/requirements.md#dns-records), [SAF-002](../reference/requirements.md#hostile-input) |
| KEY-033 | unit | The records `v=DKIM1; p=<key>` and `v=DKIM1; t=y; p=<key>` | the message is verified | `pass`, with both records listed and the signature flagged as testing | [REC-052](../reference/requirements.md#dns-records), [DKV-009](../reference/requirements.md#dkim-verification) |
| KEY-034 | unit | The records `v=DKIM2; p=<old key>` and `v=DKIM1; p=<key>` | the message is verified | `pass` with no caution: one record remains after hallmark drops the `v=DKIM2` record | [REC-050](../reference/requirements.md#dns-records), [REC-052](../reference/requirements.md#dns-records) |
| KEY-035 | property | Any set of up to 4 key records, each valid or not, and a signed message | the message is verified with the records in every order | every order gives the same result | [REC-052](../reference/requirements.md#dns-records), [ADR 0026](../adr/0026-a-dkim-selector-with-several-key-records-passes-if-any-of-them-verifies-the-signature.md) |

## DKIM2 key records

In these rows the message has one DKIM2-Signature, with `i=1` and the selector `sel`.

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| KEY-050 | unit | Two records with valid keys at `sel._domainkey.void.invalid` | the message is verified | PERMERROR `DKIM2-Signature i=1 public key sel has multiple records` | [REC-053](../reference/requirements.md#dns-records), [draft-09 §14.5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.5) |
| KEY-051 | unit | A temporary failure for the key question | the message is verified | TEMPERROR `DKIM2-Signature i=1 public key sel could not be fetched` | [REC-053](../reference/requirements.md#dns-records), [draft-09 §14.5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.5) |
| KEY-052 | unit | A key name that doesn't exist, then only the record `v=DKIM2; p=<key>` | the message is verified each time | PERMERROR `DKIM2-Signature i=1 public key sel does not exist` both times | [REC-053](../reference/requirements.md#dns-records), [dns-03 §3](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-3) |
| KEY-053 | unit | The record `k=ed25519; v=DKIM1; p=<key>` | the message is verified | PERMERROR `DKIM2-Signature i=1 public key sel has a syntax error` | [REC-053](../reference/requirements.md#dns-records), [dns-03 §4.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-4.1) |
| KEY-054 | unit | The record `v=DKIM1; k=ed25519; p=` | the message is verified | PERMERROR `DKIM2-Signature i=1 public key sel has been revoked` | [REC-053](../reference/requirements.md#dns-records), [dns-03 §4.3](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-4.3) |
| KEY-055 | unit | A `k=mldsa44` record whose `p=` decodes to 1,311 octets | the message is verified | PERMERROR `DKIM2-Signature i=1 public key sel is malformed` | [REC-053](../reference/requirements.md#dns-records), [DK2-011](../reference/requirements.md#dkim2), [dns-03 §4.3](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-4.3) |
| KEY-056 | unit | A `k=ed25519` record, and an `mldsa44` signature | the message is verified | PERMERROR `DKIM2-Signature i=1 public key sel algorithm mismatch` | [REC-053](../reference/requirements.md#dns-records), [draft-09 §14.5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.5) |
| KEY-057 | unit | A 2048-bit RSA key published as a SubjectPublicKeyInfo, and an `rsa-sha256` signature | the message is verified | PASS | [REC-053](../reference/requirements.md#dns-records), [DK2-011](../reference/requirements.md#dkim2), [ADR 0027](../adr/0027-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md), [dns-03 §4.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-4.2) |
| KEY-058 | unit | The record `v=DKIM1; k=ed25519; h=sha1; s=other; t=y; p=<key>` | the message is verified | PASS: DKIM2 ignores `h=`, `s=` and `t=` | [REC-053](../reference/requirements.md#dns-records), [DK2-011](../reference/requirements.md#dkim2), [dns-03 §4](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-4) |

## Domain Policy Records

In these rows the DKIM2-Signature has `i=1` and `d=mail.void.invalid`, the From field is `a@void.invalid`, and the signature is valid. The From domain matches only because the signing domain is its subdomain, so the check fetches the TXT records at `_domainkey.void.invalid`.

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| KEY-070 | unit | No TXT records at `_domainkey.void.invalid` | the From check runs | PASS: the From field is Authentic | [REC-054](../reference/requirements.md#dns-records), [dns-03 §5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-5) |
| KEY-071 | unit | The record `v=DKIM1; f=strict` | the From check runs | FAIL `DKIM2-Signature i=1 From void.invalid forbids subdomain signing` | [REC-054](../reference/requirements.md#dns-records), [DK2-012](../reference/requirements.md#dkim2), [draft-09 §14.8](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.8) |
| KEY-072 | unit | The record `v=DKIM1; f=` | the From check runs | PASS: an empty `f=` sets no flag | [REC-054](../reference/requirements.md#dns-records), [dns-03 §5.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-5.2) |
| KEY-073 | unit | The record `v=DKIM1; f=strict:future-flag` | the From check runs | the FAIL of KEY-071. hallmark ignores `future-flag` | [REC-054](../reference/requirements.md#dns-records), [dns-03 §5.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-5.2) |
| KEY-074 | unit | The record `v=DKIM1; f=strict; f=strict` | the From check runs | PERMERROR with no human-readable text, and a trace step that names the repeated tag and says that draft-09 §14.8 has no text for this case | [REC-054](../reference/requirements.md#dns-records), [DK2-013](../reference/requirements.md#dkim2), [ADR 0028](../adr/0028-a-malformed-dkim2-domain-policy-record-gives-permerror.md) |
| KEY-075 | unit | The record `v=DKIM1; f=1x` | the From check runs | PERMERROR with no human-readable text: a flag must start with a letter | [REC-054](../reference/requirements.md#dns-records), [ADR 0028](../adr/0028-a-malformed-dkim2-domain-policy-record-gives-permerror.md), [dns-03 §5.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-5.2) |
| KEY-076 | unit | The records `v=DKIM1` and `v=DKIM1; f=strict` | the From check runs | PERMERROR `DKIM2-Signature i=1 multiple records for void.invalid policy` | [REC-054](../reference/requirements.md#dns-records), [draft-09 §14.8](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.8) |
| KEY-077 | unit | A temporary failure for `_domainkey.void.invalid` | the From check runs | TEMPERROR `DKIM2-Signature i=1 policy for void.invalid unavailable` | [REC-054](../reference/requirements.md#dns-records), [draft-09 §14.8](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.8) |
| KEY-078 | unit | The record `f=strict`, with no `v=` tag | the From check runs | PASS: hallmark drops the record, so the domain has no policy | [REC-054](../reference/requirements.md#dns-records), [dns-03 §5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-5) |
| KEY-079 | unit | The From field `a@sub.void.invalid`, a signature with `d=mail.sub.void.invalid`, the record `v=DKIM1; f=strict` at `_domainkey.void.invalid`, and no TXT records at `_domainkey.sub.void.invalid` | the From check runs | PASS. The check fetches `_domainkey.sub.void.invalid` only: a policy record applies only to the domain that holds it | [REC-054](../reference/requirements.md#dns-records), [dns-03 §5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-5) |

## Properties

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| KEY-090 | property | Any byte string of up to 64 KiB, as a DKIM key record and as a DKIM2 key record | the record is read | the parser returns without panicking, with a key or with the outcome that REC-050 to REC-053 give | [REC-050](../reference/requirements.md#dns-records), [REC-053](../reference/requirements.md#dns-records), [SAF-001](../reference/requirements.md#hostile-input) |
| KEY-091 | fuzz | The reader of RSA key data | it is fuzzed | it never panics, and each key it accepts has a modulus and a public exponent | [SAF-003](../reference/requirements.md#hostile-input), [ADR 0027](../adr/0027-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md) |
