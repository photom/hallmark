# ADR 0020: DMARC supports RFC 7489 and DMARCbis, with DMARCbis the default

- Status: Accepted
- Date: 2026-11-14
- Supersedes: [ADR 0007](0007-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md)
- Affects: the DMARC code, its public options, and the trace

## Context

DMARCbis was published as RFC 9989 in May 2026, and it obsoletes RFC 7489. The two differ in ways that change results:
- RFC 7489 finds the organizational domain with the Public Suffix List. RFC 9989 walks the DNS tree instead.
- RFC 9989 adds the `np`, `psd`, and `t` tags, and drops `pct`, `rf`, and `ri`.
- RFC 7489's `pct` asks receivers to apply the policy to a percentage of messages. RFC 9989 Appendix A.6 reports that only `pct=0` and `pct=100` were applied accurately, and that its `t=y` and `t=n` are analogous to those two values.

Domains will keep publishing records written for RFC 7489 for years, and receivers will want to compare the two specifications while they move.

ADR 0007 had hallmark sample `pct` with a random value from the caller. hallmark now reads `pct=0` as test mode and ignores other values ([ADR 0021](0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md)), so the random value is gone.

## Options considered

- (a) RFC 9989 only.
- (b) RFC 7489 only.
- (c) Both, chosen per call, with RFC 9989 as the default.

## Decision

Option (c).

- A caller picks RFC 9989 or RFC 7489 for each DMARC evaluation. RFC 9989 is the default.
- The record parser accepts the tags of both specifications, and ignores unknown tags.
- In RFC 7489 mode, the organizational domain comes from the Public Suffix List. In RFC 9989 mode, it comes from the DNS tree walk, with the query limit RFC 9989 sets.
- Test mode comes from `t=y` in RFC 9989 mode and from `pct=0` in RFC 7489 mode, and the result reports it as ADR 0021 says. hallmark takes no random value.
- The trace records which specification applied and each step of policy discovery.
- hallmark parses `rua` and `ruf` but sends no reports.

## Consequences

Positive:
- A receiver can evaluate the same message under both specifications and see where they differ, in the demo or in its own tests.
- New users get the current standard by default.
- A DMARC evaluation in either mode is a pure function of the message, the DNS answers and the options, so it replays without a random value.

Negative:
- Two ways to find the organizational domain mean two code paths and two sets of scenarios.
- RFC 7489 mode needs a Public Suffix List. Whether the caller supplies it or the crate bundles a copy is still open (requirements.md, Open items).

## References

- [RFC 9989](https://www.rfc-editor.org/info/rfc9989), DMARC, and its [Appendix A.6](https://www.rfc-editor.org/rfc/rfc9989.html#appendix-A.6)
- [RFC 7489](https://www.rfc-editor.org/info/rfc7489), the previous DMARC specification
- [ADR 0021](0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md), test modes
- [requirements.md: DMARC](../reference/requirements.md#dmarc)
