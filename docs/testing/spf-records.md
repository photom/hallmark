# Test scenarios: SPF records

Scenarios for the SPF record parser: record selection, terms, addresses and CIDR lengths, domain-specs and macros, target names after expansion, explanations, and the address answers that SPF reads. Conventions are in [README.md](README.md).

Unless a row says otherwise, the record is the only TXT record at `void.invalid`, MAIL FROM is `<user@void.invalid>`, the client is 192.0.2.1, and DNS answers come from a snapshot. The snapshot answers every question the check asks, and a name that a row doesn't mention has no records of the type asked.

## Contents

- [Record selection](#record-selection)
- [Terms](#terms)
- [Addresses and CIDR lengths](#addresses-and-cidr-lengths)
- [Domain-specs and macros](#domain-specs-and-macros)
- [Target names after expansion](#target-names-after-expansion)
- [Explanations](#explanations)
- [Address answers](#address-answers)
- [Properties](#properties)

## Record selection

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SPR-001 | unit | The TXT record `V=SPF1 -all` | SPF runs | `fail`. hallmark selects the record, and the trace records the version as published, `V=SPF1` | [SPF-007](../reference/requirements.md#spf-rfc-7208), [ADR 0022](../adr/0022-spf-reads-v-spf1-without-regard-to-case-and-allows-c-r-and-t-only-in-explanations.md), [RFC 7208 §4.5](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.5), [RFC 5234 §2.3](https://www.rfc-editor.org/rfc/rfc5234.html#section-2.3) |
| SPR-002 | unit | The TXT record `v=spf10 -all` | SPF runs | `none`, and the trace records that hallmark dropped the record because its version is `v=spf10` | [SPF-007](../reference/requirements.md#spf-rfc-7208), [REC-001](../reference/requirements.md#dns-records), [RFC 7208 §4.5](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.5) |
| SPR-003 | unit | The TXT record `v=spf1\t-all`, with a tab after the version | SPF runs | `none`: neither a space nor the end of the record follows the version, so hallmark drops the record | [SPF-007](../reference/requirements.md#spf-rfc-7208), [RFC 7208 §4.5](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.5) |
| SPR-004 | unit | The TXT records `hello` and `v=spf1 -all` | SPF runs | `fail`, and the trace records that hallmark dropped `hello` | [REC-001](../reference/requirements.md#dns-records), [RFC 7208 §4.5](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.5) |
| SPR-005 | unit | The TXT records `v=spf1 -all` and `v=spf1 +all` | SPF runs | `permerror`, and the trace holds both records | [SPF-007](../reference/requirements.md#spf-rfc-7208), [RFC 7208 §4.5](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.5) |
| SPR-006 | unit | The TXT record `v=spf1` | SPF runs | `neutral`, the default result of a record with no terms | [REC-010](../reference/requirements.md#dns-records), [RFC 7208 §4.7](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.7) |
| SPR-007 | unit | The TXT record `v=spf1 a:café.void.invalid -all`, with `é` in UTF-8 | SPF runs | `permerror`, and the error names the US-ASCII rule at the offset of `é`. hallmark selects the record before it parses it | [REC-001](../reference/requirements.md#dns-records), [REC-010](../reference/requirements.md#dns-records), [RFC 7208 §3.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-3.1), [RFC 7208 §4.6](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6) |

## Terms

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SPR-010 | unit | The record `v=spf1 -all foo` | SPF runs | `permerror`, and the error names the mechanism rule at the offset of `foo`, though `-all` comes first | [REC-010](../reference/requirements.md#dns-records), [RFC 7208 §4.6](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6), [RFC 7208 §5.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.1) |
| SPR-011 | unit | The record `v=spf1  ip4:192.0.2.1   -all  `, with extra spaces between the terms and at the end | SPF runs | `pass` | [REC-010](../reference/requirements.md#dns-records), [RFC 7208 §4.5](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.5), [RFC 7208 §4.6.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.1) |
| SPR-012 | unit | The record `v=spf1 ip4:192.0.2.1\t-all` | SPF runs | `permerror`, and the error names the term separator at the offset of the tab | [REC-010](../reference/requirements.md#dns-records), [RFC 7208 §4.6.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.1) |
| SPR-013 | unit | The record `v=spf1 IP4:192.0.2.1 -ALL` | SPF runs | `pass` | [REC-011](../reference/requirements.md#dns-records), [RFC 7208 §4.6.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.1) |
| SPR-014 | unit | The record `v=spf1 ip4=192.0.2.1 -all` | SPF runs | `fail`. `ip4=192.0.2.1` is an unknown modifier, which hallmark ignores, and the trace lists it | [REC-011](../reference/requirements.md#dns-records), [RFC 7208 §4.6.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.1), [RFC 7208 §6](https://www.rfc-editor.org/rfc/rfc7208.html#section-6) |
| SPR-015 | unit | The record `v=spf1 foo=bar foo=baz -all` | SPF runs | `fail`, with both copies of `foo` ignored and no error | [REC-011](../reference/requirements.md#dns-records), [RFC 7208 §6](https://www.rfc-editor.org/rfc/rfc7208.html#section-6) |
| SPR-016 | unit | The record `v=spf1 redirect=a.void.invalid redirect=b.void.invalid` | SPF runs | `permerror`, before any question for `a.void.invalid` or `b.void.invalid` | [REC-011](../reference/requirements.md#dns-records), [RFC 7208 §6](https://www.rfc-editor.org/rfc/rfc7208.html#section-6) |
| SPR-017 | unit | The record `v=spf1 -all exp=a.void.invalid exp=b.void.invalid` | SPF runs | `permerror` | [REC-011](../reference/requirements.md#dns-records), [RFC 7208 §6](https://www.rfc-editor.org/rfc/rfc7208.html#section-6) |
| SPR-018 | unit | The record `v=spf1 -redirect=a.void.invalid` | SPF runs | `permerror`: a qualifier can't come before a modifier | [REC-011](../reference/requirements.md#dns-records), [RFC 7208 §4.6.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.1) |
| SPR-019 | unit | The record `v=spf1 redirect=other.void.invalid -all` | SPF runs | `fail`, no question for `other.void.invalid`, and a trace step that says `redirect` had no effect because the record holds `all` | [REC-011](../reference/requirements.md#dns-records), [RFC 7208 §5.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.1), [RFC 7208 §6.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.1) |
| SPR-020 | unit | The record `v=spf1 ip5:192.0.2.1 -all` | SPF runs | `permerror`, and the error names the mechanism rule at the offset of `ip5` | [REC-011](../reference/requirements.md#dns-records), [RFC 7208 §4.6.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.1) |

## Addresses and CIDR lengths

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SPR-030 | unit | The record `v=spf1 ip4:192.0.2.01 -all` | SPF runs | `permerror`, and the error names the `qnum` rule at the offset of `01` | [REC-012](../reference/requirements.md#dns-records), [RFC 7208 §5.6](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.6) |
| SPR-031 | unit | The record `v=spf1 ip4:192.0.2 -all` | SPF runs | `permerror`: an address can't leave out parts | [REC-012](../reference/requirements.md#dns-records), [RFC 7208 §5.6](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.6) |
| SPR-032 | unit | The record `v=spf1 ip4:192.0.2.0/032 -all` | SPF runs | `permerror`: a CIDR length has no leading zeros | [REC-012](../reference/requirements.md#dns-records), [RFC 7208 §5.6](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.6) |
| SPR-033 | unit | The client 2001:db8::1 and the record `v=spf1 ip6:2001:db8::/32 -all` | SPF runs | `pass` | [REC-012](../reference/requirements.md#dns-records), [RFC 7208 §5.6](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.6), [RFC 4291 §2.2](https://www.rfc-editor.org/rfc/rfc4291.html#section-2.2) |
| SPR-034 | unit | The client 2001:db8::c000:201 and the record `v=spf1 ip6:2001:db8::192.0.2.1 -all` | SPF runs | `pass`: the dotted quad gives the last 32 bits of the address | [REC-012](../reference/requirements.md#dns-records), [RFC 4291 §2.2](https://www.rfc-editor.org/rfc/rfc4291.html#section-2.2) |
| SPR-035 | unit | The record `v=spf1 ip6:2001:db8::1::2 -all` | SPF runs | `permerror`: `::` appears twice | [REC-012](../reference/requirements.md#dns-records), [RFC 4291 §2.2](https://www.rfc-editor.org/rfc/rfc4291.html#section-2.2) |
| SPR-036 | unit | The record `v=spf1 ip6:fe80::1%eth0 -all`, with a zone after the address | SPF runs | `permerror` | [REC-012](../reference/requirements.md#dns-records), [RFC 4291 §2.2](https://www.rfc-editor.org/rfc/rfc4291.html#section-2.2) |
| SPR-037 | unit | The record `v=spf1 ip6:2001:db8::/129 -all` | SPF runs | `permerror`: the length is over 128 | [REC-012](../reference/requirements.md#dns-records), [PAR-003](../reference/requirements.md#parsers), [RFC 7208 §5.6](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.6) |
| SPR-038 | unit | The client 2001:db8::1, the record `v=spf1 a//64 -all`, and the AAAA record 2001:db8::ffff for `void.invalid` | SPF runs | `pass`: the IPv6 length 64 applies to an IPv6 client | [REC-012](../reference/requirements.md#dns-records), [REC-017](../reference/requirements.md#dns-records), [RFC 7208 §5.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.3), [RFC 7208 §5.6](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.6) |

## Domain-specs and macros

Rows SPR-055 to SPR-058 use the values of the examples in RFC 7208 §7.4, with `email.void.invalid` in place of `email.example.com`: the sender `strong-bad@email.void.invalid`, the IPv4 client 192.0.2.3 and the IPv6 client 2001:db8::cb01. `<domain>` is `email.void.invalid`.

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SPR-040 | unit | The record `v=spf1 a:localhost -all` | SPF runs | `permerror`, and the error names the `domain-end` rule | [REC-013](../reference/requirements.md#dns-records), [RFC 7208 §7.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.1) |
| SPR-041 | unit | The record `v=spf1 a:192.0.2.1 -all` | SPF runs | `permerror`: the top label `1` is all digits | [REC-013](../reference/requirements.md#dns-records), [RFC 7208 §7.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.1) |
| SPR-042 | unit | The record `v=spf1 a:mail.void.invalid. -all`, and the A record 192.0.2.1 for `mail.void.invalid` | SPF runs | `pass`: the dot at the end is accepted | [REC-013](../reference/requirements.md#dns-records), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3) |
| SPR-043 | unit | The record `v=spf1 -exists:%(ir).void.invalid -all` | SPF runs | `permerror`: a `%` is followed by `(` | [REC-013](../reference/requirements.md#dns-records), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3) |
| SPR-044 | unit | The record `v=spf1 exists:%{d0}.void.invalid -all` | SPF runs | `permerror`: a digit transformer can't be 0 | [REC-013](../reference/requirements.md#dns-records), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3) |
| SPR-045 | unit | The record `v=spf1 exists:%{i99999999999999999999r}.void.invalid -all` and the client 192.0.2.1 | the check builds the question | A `1.2.0.192.void.invalid`: a transformer too large to hold uses every part | [REC-013](../reference/requirements.md#dns-records), [PAR-003](../reference/requirements.md#parsers), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3) |
| SPR-046 | unit | The record `v=spf1 exists:%{t}.void.invalid -all` | SPF runs | `permerror`, before any DNS question: `t` is allowed only in explanation text | [REC-013](../reference/requirements.md#dns-records), [ADR 0022](../adr/0022-spf-reads-v-spf1-without-regard-to-case-and-allows-c-r-and-t-only-in-explanations.md), [RFC 7208 §7.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.2) |
| SPR-047 | unit | The record `v=spf1 -all exp=%{r}.void.invalid` | SPF runs | `permerror`: the `domain-spec` of `exp=` is a term argument, not explanation text | [REC-013](../reference/requirements.md#dns-records), [ADR 0022](../adr/0022-spf-reads-v-spf1-without-regard-to-case-and-allows-c-r-and-t-only-in-explanations.md), [RFC 7208 §7.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.2) |
| SPR-055 | unit | The IPv4 client and the macros `%{s}`, `%{o}`, `%{d}`, `%{d4}`, `%{d3}`, `%{d2}`, `%{d1}`, `%{dr}`, `%{d2r}`, `%{l}`, `%{l-}`, `%{lr}`, `%{lr-}` and `%{l1r-}` | each macro is expanded | in order: `strong-bad@email.void.invalid`, `email.void.invalid` four times, `void.invalid`, `invalid`, `invalid.void.email`, `void.email`, `strong-bad`, `strong.bad`, `strong-bad`, `bad.strong` and `strong` | [REC-014](../reference/requirements.md#dns-records), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3), [RFC 7208 §7.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.4) |
| SPR-056 | unit | The macro string `%{ir}.%{v}._spf.%{d2}`, with the IPv4 client and then with the IPv6 client | it is expanded | `3.2.0.192.in-addr._spf.void.invalid`, then `1.0.b.c.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.0.8.b.d.0.1.0.0.2.ip6._spf.void.invalid` | [REC-014](../reference/requirements.md#dns-records), [RFC 7208 §7.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.4) |
| SPR-057 | unit | The macro `%{S}` | it is expanded | `strong-bad%40email.void.invalid`: `@` is outside the unreserved set, and `-` and `.` are inside it | [REC-014](../reference/requirements.md#dns-records), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3), [RFC 3986 §2.3](https://www.rfc-editor.org/rfc/rfc3986.html#section-2.3) |
| SPR-058 | unit | The IPv4 client, the PTR answer `a.other.invalid` and `mx.email.void.invalid`, in that order, and the A record 192.0.2.3 for both names | `%{p}` is expanded | `mx.email.void.invalid`: a validated subdomain of `<domain>` comes before the first name of the answer | [REC-014](../reference/requirements.md#dns-records), [RFC 7208 §5.5](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.5), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3) |
| SPR-059 | unit | The client 192.0.2.3 and no PTR records for it | `%{p}` is expanded | `unknown` | [REC-014](../reference/requirements.md#dns-records), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3) |

## Target names after expansion

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SPR-070 | unit | MAIL FROM `<"a..b"@void.invalid>` and the record `v=spf1 exists:%{l}.void.invalid -all` | SPF runs | `fail`. `exists` matches nothing, no question goes out for it, and the trace records the name `"a..b".void.invalid` and its empty label | [REC-015](../reference/requirements.md#dns-records), [ADR 0023](../adr/0023-an-spf-target-name-that-macro-expansion-makes-invalid-matches-nothing.md), [RFC 7208 §4.8](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.8) |
| SPR-071 | unit | A local part of 64 `a` characters, and the record `v=spf1 exists:%{l}.a.void.invalid exists:%{l}.b.void.invalid exists:%{l}.c.void.invalid -all` | SPF runs | `fail`, not `permerror`: each name has a label over 63 octets and matches nothing. The trace counts three terms toward the lookup limit and no void lookups | [REC-015](../reference/requirements.md#dns-records), [ADR 0023](../adr/0023-an-spf-target-name-that-macro-expansion-makes-invalid-matches-nothing.md), [RFC 7208 §4.6.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.4) |
| SPR-072 | unit | The local part of SPR-071, and a record with ten terms `exists:%{l}.void.invalid` followed by `a -all` | SPF runs | `permerror` for the lookup limit at `a`, though no question went out for the first ten terms | [REC-015](../reference/requirements.md#dns-records), [SPF-006](../reference/requirements.md#spf-rfc-7208), [RFC 7208 §4.6.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.4) |
| SPR-073 | unit | The local part of SPR-071 and the record `v=spf1 include:%{l}._spf.void.invalid -all` | SPF runs | `permerror`: check_host() gives `none` for the malformed name, and `include` turns `none` into `permerror` | [REC-015](../reference/requirements.md#dns-records), [RFC 7208 §4.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.3), [RFC 7208 §5.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.2) |
| SPR-074 | unit | The local part of SPR-071 and the record `v=spf1 redirect=%{l}._spf.void.invalid` | SPF runs | `permerror` | [REC-015](../reference/requirements.md#dns-records), [RFC 7208 §6.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.1) |
| SPR-075 | unit | The local part of SPR-071 and the record `v=spf1 -all exp=%{l}._spf.void.invalid` | SPF runs | `fail` with no explanation, and no question for the explanation | [REC-015](../reference/requirements.md#dns-records), [REC-016](../reference/requirements.md#dns-records), [RFC 7208 §6.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.2) |

## Explanations

Unless a row says otherwise, the record is `v=spf1 -all exp=explain.void.invalid`.

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SPR-080 | unit | The client 2001:db8::cb01, the verification time 2026-11-14T00:00:00Z, no receiving host name, and the TXT record `%{c} at %{t} via %{r}` at `explain.void.invalid` | SPF runs | `fail` with the explanation `2001:db8::cb01 at 1794614400 via unknown`, marked as text from a third party | [REC-014](../reference/requirements.md#dns-records), [REC-016](../reference/requirements.md#dns-records), [TIM-001](../reference/requirements.md#verification-time), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3), [RFC 5952 §4](https://www.rfc-editor.org/rfc/rfc5952.html#section-4) |
| SPR-081 | unit | The case of SPR-080, with the receiving host name `mx.void.invalid` from the caller | SPF runs | the explanation `2001:db8::cb01 at 1794614400 via mx.void.invalid` | [GEN-002](../reference/requirements.md#general), [REC-014](../reference/requirements.md#dns-records), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3) |
| SPR-082 | unit | Two TXT records at `explain.void.invalid` | SPF runs | `fail` with no explanation | [REC-016](../reference/requirements.md#dns-records), [RFC 7208 §6.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.2) |
| SPR-083 | unit | The TXT record `café` at `explain.void.invalid`, with `é` in UTF-8 | SPF runs | `fail` with no explanation: an explanation is US-ASCII | [REC-016](../reference/requirements.md#dns-records), [RFC 7208 §6.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.2) |
| SPR-084 | unit | The TXT record `100% sure` at `explain.void.invalid` | SPF runs | `fail` with no explanation: `%` followed by a space is a syntax error | [REC-016](../reference/requirements.md#dns-records), [RFC 7208 §7.1](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.1), [RFC 7208 §7.3](https://www.rfc-editor.org/rfc/rfc7208.html#section-7.3) |
| SPR-085 | unit | A temporary failure for `explain.void.invalid` | SPF runs | `fail` with no explanation, not `temperror` | [REC-016](../reference/requirements.md#dns-records), [RFC 7208 §6.2](https://www.rfc-editor.org/rfc/rfc7208.html#section-6.2) |

## Address answers

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SPR-090 | unit | The record `v=spf1 mx -all` and an MX answer of 11 records | SPF runs | `permerror`, before any address question | [REC-017](../reference/requirements.md#dns-records), [SPF-006](../reference/requirements.md#spf-rfc-7208), [RFC 7208 §4.6.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.4) |
| SPR-091 | unit | The record `v=spf1 mx -all`, no MX records for `void.invalid`, and the A record 192.0.2.1 for it | SPF runs | `fail`, and no A question for `void.invalid`: SPF applies no implicit MX | [REC-017](../reference/requirements.md#dns-records), [RFC 7208 §5.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.4) |
| SPR-092 | unit | The client 2001:db8::1 and the record `v=spf1 exists:%{d} -all` | the check builds the question | A `void.invalid`, not AAAA | [REC-017](../reference/requirements.md#dns-records), [RFC 7208 §5.7](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.7) |
| SPR-093 | unit | The client 192.0.2.3, the record `v=spf1 ptr -all`, and a PTR answer of 12 names in which only the 11th has the A record 192.0.2.3 | SPF runs | `fail`. The check asks for the addresses of the first 10 names only | [REC-017](../reference/requirements.md#dns-records), [RFC 7208 §4.6.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-4.6.4), [RFC 7208 §5.5](https://www.rfc-editor.org/rfc/rfc7208.html#section-5.5) |

## Properties

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SPR-100 | property | Any byte string of up to 64 KiB after `v=spf1 ` | it is parsed as an SPF record | the parser returns without panicking, with a record or with an error whose offset lies inside the input | [REC-010](../reference/requirements.md#dns-records), [PAR-001](../reference/requirements.md#parsers), [SAF-001](../reference/requirements.md#hostile-input) |
| SPR-101 | fuzz | The SPF record parser | it is fuzzed | it never panics, and the property of SPR-100 holds | [SAF-003](../reference/requirements.md#hostile-input) |
| SPR-102 | fuzz | The IPv6 address parser | it is fuzzed | it never panics, and each address it accepts, written in the form of RFC 5952 §4, parses back to the same address | [SAF-003](../reference/requirements.md#hostile-input), [REC-012](../reference/requirements.md#dns-records), [RFC 5952 §4](https://www.rfc-editor.org/rfc/rfc5952.html#section-4) |
