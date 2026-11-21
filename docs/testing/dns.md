# Test scenarios: DNS questions and answers

Scenarios for the resolver boundary: the shapes of answers, the names hallmark asks for, temporary failures, snapshots and demo cases, records pasted into the demo, and public suffix answers in snapshots. The parsers of SPF, DMARC and key records get files of their own. Conventions are in [README.md](README.md).

## Contents

- [Answers](#answers)
- [Question names](#question-names)
- [Temporary failures and snapshots](#temporary-failures-and-snapshots)
- [Pasted records](#pasted-records)
- [Public suffix answers](#public-suffix-answers)

## Answers

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| RES-001 | unit | The four strings of the RSA key record in RFC 8463 Appendix A.2, answered as one TXT record for `test._domainkey.void.test` | the key record is read | one record whose `p=` holds the four strings joined with nothing between them | [DNS-006](../reference/requirements.md#dns), [RFC 6376 §3.6.2.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.2.2), [RFC 8463 Appendix A.2](https://www.rfc-editor.org/rfc/rfc8463.html#appendix-A.2) |
| RES-002 | unit | A TXT record with the strings `v=spf1 ip4:192.0.2.0/2` and `4 -all` | the SPF record is read | `v=spf1 ip4:192.0.2.0/24 -all` | [DNS-006](../reference/requirements.md#dns), [RFC 7208 §3.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-3.3) |
| RES-003 | unit | A TXT record at `_dmarc.void.test` with the strings `v=DMARC1; p=rej` and `ect` | the DMARC record is read | the policy `reject` | [DNS-006](../reference/requirements.md#dns), [RFC 9989 §4.5](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.5) |
| RES-004 | unit | The answer of RES-001 | the trace is read | the answer step holds the four strings as the resolver gave them, not joined, and the key record step holds the joined value | [DNS-006](../reference/requirements.md#dns), [TRC-008](../reference/requirements.md#trace-and-reasoning) |
| RES-005 | unit | The MX answer with preference 10 and exchange `mx.void.test` for `void.test` | SPF evaluates `mx` | the check asks for the addresses of `mx.void.test` | [DNS-006](../reference/requirements.md#dns), [RFC 7208 §5.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.4) |

## Question names

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| RES-010 | unit | A DKIM-Signature with `d=void.test` and `s=foo.bar` | the key question is built | TXT `foo.bar._domainkey.void.test` | [DNS-008](../reference/requirements.md#dns), [RFC 6376 §3.6.2.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.2.1) |
| RES-011 | unit | A DKIM2 From check that needs the Domain Policy Record of `void.test` | the question is built | TXT `_domainkey.void.test` | [DNS-008](../reference/requirements.md#dns), [dns-03 §5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-5) |
| RES-012 | unit | DMARC in RFC 9989 mode for the author domain `mail.void.test` | policy discovery starts | the first question is TXT `_dmarc.mail.void.test` | [DNS-008](../reference/requirements.md#dns), [RFC 9989 §4.5](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.5), [RFC 9989 §4.10](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.10) |
| RES-013 | unit | An SPF `ptr` term and the client 192.0.2.3 | the check evaluates `ptr` | PTR `3.2.0.192.in-addr.arpa` | [DNS-008](../reference/requirements.md#dns), [RFC 1035 §3.5](https://www.rfc-editor.org/rfc/rfc1035.html#section-3.5), [RFC 7208 §5.5](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.5) |
| RES-014 | unit | An SPF `ptr` term and the client 2001:db8::1 | the check evaluates `ptr` | PTR `1.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.8.b.d.0.1.0.0.2.ip6.arpa` | [DNS-008](../reference/requirements.md#dns), [RFC 3596 §2.5](https://www.rfc-editor.org/rfc/rfc3596.html#section-2.5) |
| RES-015 | unit | MAIL FROM with a local part of 61 `a` characters, and the term `exists:%{l}.%{l}.%{l}.%{l}.%{l}.void.test`, which expands to 319 characters | the check builds the question | the name keeps its last three `a` labels and `void.test`, 195 characters: whole labels go from the left until the name fits in 253 | [DNS-008](../reference/requirements.md#dns), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3) |

## Temporary failures and snapshots

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| RES-020 | unit | A key question that gets a temporary failure with the description `SERVFAIL from 192.0.2.53` | the signature is verified | TEMPFAIL (key unavailable), and the trace holds the description as the resolver gave it | [DNS-009](../reference/requirements.md#dns), [RFC 6376 §6.1.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.2) |
| RES-021 | unit | A snapshot extracted from the full-level trace of a DMARC check with SPF and DKIM | the check is replayed with the snapshot resolver | the same results and the same sorted trace | [DNS-004](../reference/requirements.md#dns), [TRC-008](../reference/requirements.md#trace-and-reasoning) |
| RES-022 | unit | A snapshot that lacks one question the DKIM check asks | the check is replayed | TEMPFAIL for that signature, and the trace says that the snapshot held no answer for the question | [DNS-004](../reference/requirements.md#dns), [DNS-009](../reference/requirements.md#dns) |
| RES-023 | unit | A snapshot that holds `SEL._DomainKey.VOID.test`, and a check that asks for `sel._domainkey.void.test` | the check is replayed | the snapshot answers: names match without regard to ASCII case | [DNS-004](../reference/requirements.md#dns), [RFC 4343 §2](https://www.rfc-editor.org/rfc/rfc4343.html#section-2) |
| RES-024 | unit | The full-level trace of a DMARC check with SPF and DKIM, in which two steps read the same answer | a snapshot is extracted from its events in the order they went out, and again from the sorted events | one answer for each question type and name, in ascending order of the name in ASCII lowercase, then of the type. Both extractions are the same bytes | [DNS-004](../reference/requirements.md#dns), [TRC-008](../reference/requirements.md#trace-and-reasoning), [ADR 0035](../adr/0035-dns-snapshots-and-demo-cases-are-documents-in-the-trace-formats-conventions.md) |
| RES-025 | unit | A TXT answer with one record of two strings: `v=DKIM1; k=ed25519;` and the bytes FB FF | the answer goes into a snapshot | the record is the list `[{"text":"v=DKIM1; k=ed25519;","from":["dns"]},{"base64url":"-_8","from":["dns"]}]`: two strings, not joined | [DNS-006](../reference/requirements.md#dns), [Trace format](../reference/trace-format.md#snapshots) |
| RES-026 | unit | A temporary failure with the description `timeout`, and a name that doesn't exist | each answer goes into a snapshot, and the check is replayed from it | the snapshot holds `tempfail` with the description, and `nxdomain`. The replay gets the temporary failure with `timeout` and the name that doesn't exist | [DNS-002](../reference/requirements.md#dns), [DNS-009](../reference/requirements.md#dns), [Trace format](../reference/trace-format.md#snapshots) |
| RES-027 | unit | A snapshot whose `v` is `[2,0]` | the snapshot resolver reads it | an error that names the version, and no answer | [DNS-004](../reference/requirements.md#dns), [TRC-007](../reference/requirements.md#trace-and-reasoning) |
| RES-028 | unit | A demo case saved from a DMARC check: the message, the envelope, the time, the options of the call and a snapshot | the case is read back and replayed | the same results and the same sorted trace as the run that saved it | [DEM-004](../reference/requirements.md#browser-demo), [ADR 0035](../adr/0035-dns-snapshots-and-demo-cases-are-documents-in-the-trace-formats-conventions.md) |

## Pasted records

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| RES-030 | unit | The pasted line `sel._domainkey.void.test. 300 IN TXT "v=DKIM1; k=ed25519; " "p=11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo="` | the key question is answered | records found: one TXT record with two strings | [DNS-010](../reference/requirements.md#dns), [DNS-006](../reference/requirements.md#dns), [RFC 1035 §5.1](https://www.rfc-editor.org/rfc/rfc1035.html#section-5.1) |
| RES-031 | unit | Pasted records only for `void.test.`, all of type TXT | an A question for `void.test` | no records of that type | [DNS-010](../reference/requirements.md#dns) |
| RES-032 | unit | Pasted records only for `void.test.` | a TXT question for `other.void.test` | the name does not exist | [DNS-010](../reference/requirements.md#dns), [RFC 8020 §2](https://www.rfc-editor.org/rfc/rfc8020.html#section-2) |
| RES-033 | unit | A pasted record only for `sel._domainkey.void.test.` | a TXT question for `_domainkey.void.test` | no records of that type, because a name below it exists | [DNS-010](../reference/requirements.md#dns), [RFC 8020 §2](https://www.rfc-editor.org/rfc/rfc8020.html#section-2), [RFC 9989 §3.2.13](https://www.rfc-editor.org/rfc/rfc9989.html#section-3.2.13) |
| RES-034 | unit | The pasted line `sel._domainkey.void.test IN TXT "x"`, whose owner name has no final dot | the records are read | an error with line 1 and offset 0: the owner name must be absolute | [DNS-010](../reference/requirements.md#dns), [PAR-001](../reference/requirements.md#parsers), [RFC 1035 §5.1](https://www.rfc-editor.org/rfc/rfc1035.html#section-5.1) |
| RES-035 | unit | The pasted line `void.test. IN TXT "a\034b"` | the TXT question is answered | one string, `a"b`: `\034` is the decimal escape of a quote | [DNS-010](../reference/requirements.md#dns), [RFC 1035 §5.1](https://www.rfc-editor.org/rfc/rfc1035.html#section-5.1) |
| RES-036 | unit | The pasted line `void.test. IN MX 10 mx.void.test.` | the MX question is answered | one record with preference 10 and exchange `mx.void.test` | [DNS-010](../reference/requirements.md#dns), [DNS-006](../reference/requirements.md#dns) |
| RES-037 | fuzz | The parser of pasted records | it is fuzzed | it never panics, and each error it gives names a line and an offset inside the input | [SAF-003](../reference/requirements.md#hostile-input), [DNS-010](../reference/requirements.md#dns), [PAR-001](../reference/requirements.md#parsers) |

## Public suffix answers

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| RES-040 | unit | The full-level trace of the DMARC check of DMR-064 | a snapshot is extracted, and the check is replayed from it | the snapshot holds a suffix answer for `a.b.void.test` with `labels` 1 and `list` `test-1`, and the replay gives the same result and the same sorted trace with no list | [DNS-004](../reference/requirements.md#dns), [TRC-008](../reference/requirements.md#trace-and-reasoning), [DMC-011](../reference/requirements.md#dmarc) |
| RES-041 | unit | A snapshot that lacks the public suffix answer for an author domain with no record of its own | the DMARC check is replayed in RFC 7489 mode | `temperror`, and the `dmarc.suffix` step has the outcome `unavailable` and a description, marked as from the caller, that says the snapshot held no suffix answer for the domain | [DNS-004](../reference/requirements.md#dns), [DMC-010](../reference/requirements.md#dmarc), [DMC-011](../reference/requirements.md#dmarc) |
