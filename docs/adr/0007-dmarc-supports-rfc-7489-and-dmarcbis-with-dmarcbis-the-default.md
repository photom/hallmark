# ADR 0007: DMARC supports RFC 7489 and DMARCbis, with DMARCbis the default

- Status: Accepted
- Date: 2026-11-12
- Affects: the DMARC code, its public options, and the trace

## Context

DMARCbis was published as RFC 9989 in May 2026, and it obsoletes RFC 7489. The two differ in ways that change results:
- RFC 7489 finds the organizational domain with the Public Suffix List. RFC 9989 walks the DNS tree instead.
- RFC 9989 adds the `np`, `psd`, and `t` tags, and drops `pct`, `rf`, and `ri`.
- RFC 7489's `pct` sampling needs a random number.

Domains will keep publishing records written for RFC 7489 for years, and receivers will want to compare the two specifications while they move.

## Options considered

- (a) RFC 9989 only.
- (b) RFC 7489 only.
- (c) Both, chosen per call, with RFC 9989 as the default.

## Decision

Option (c).

- A caller picks RFC 9989 or RFC 7489 for each DMARC evaluation. RFC 9989 is the default.
- The record parser accepts the tags of both specifications, and ignores unknown tags.
- In RFC 7489 mode, the organizational domain comes from the Public Suffix List, and the caller supplies the random value for `pct` (ADR 0001). In RFC 9989 mode, it comes from the DNS tree walk, with the query limit RFC 9989 sets.
- The trace records which specification applied and each step of policy discovery.
- hallmark parses `rua` and `ruf` but sends no reports.

## Consequences

Positive:
- A receiver can evaluate the same message under both specifications and see where they differ, in the demo or in its own tests.
- New users get the current standard by default.

Negative:
- Two ways to find the organizational domain mean two code paths and two sets of scenarios.
- RFC 7489 mode needs a Public Suffix List. Whether the caller supplies it or the crate bundles a copy is still open (requirements.md, Open items).

## References

- [RFC 9989](https://www.rfc-editor.org/info/rfc9989), DMARC
- [RFC 7489](https://www.rfc-editor.org/info/rfc7489), the previous DMARC specification
- [requirements.md: DMARC](../reference/requirements.md#dmarc)
