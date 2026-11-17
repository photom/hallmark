# ADR 0040: DMARC supports RFC 7489 and DMARCbis, with DMARCbis the default

- Status: Accepted
- Date: 2026-11-16
- Supersedes: [ADR 0019](0019-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md)
- Affects: the DMARC code, its public options, and the trace

## Context

DMARCbis was published as RFC 9989 in May 2026, and it obsoletes RFC 7489. The two differ in ways that change results:

- RFC 7489 finds the organizational domain with a public suffix list, and names no list (§3.2, Appendix A.6.1). RFC 9989 walks the DNS tree instead.
- RFC 9989 adds the `np`, `psd`, and `t` tags, and drops `pct`, `rf`, and `ri`.
- RFC 7489's `pct` asks receivers to apply the policy to a percentage of messages. RFC 9989 Appendix A.6 reports that only `pct=0` and `pct=100` were applied accurately, and that its `t=y` and `t=n` are analogous to those two values.

Domains will keep publishing records written for RFC 7489 for years, and receivers will want to compare the two specifications while they move.

ADR 0019 made the same choice. It said that the organizational domain in RFC 7489 mode comes from the Public Suffix List, left open where the list comes from, and said that the record parser accepts the tags of both specifications. [ADR 0038](0038-in-rfc-7489-mode-the-caller-answers-public-suffix-questions.md) has since settled the list: the caller answers public suffix questions from the list of its choice. The requirements have the parser read only the tags of the mode in use (REC-031).

## Options considered

- (a) RFC 9989 only.
- (b) RFC 7489 only.
- (c) Both, chosen per call, with RFC 9989 as the default.

## Decision

Option (c).

- A caller picks RFC 9989 or RFC 7489 for each DMARC evaluation. RFC 9989 is the default.
- The record parser reads the tags of the mode in use and ignores unknown tags, so `pct`, `rf` and `ri` are unknown in RFC 9989 mode, and `np`, `psd` and `t` are unknown in RFC 7489 mode (REC-031).
- In RFC 7489 mode, the organizational domain comes from the public suffix that the caller gives for each domain, from the list of its choice ([ADR 0038](0038-in-rfc-7489-mode-the-caller-answers-public-suffix-questions.md)). In RFC 9989 mode, it comes from the DNS tree walk, with the query limit that RFC 9989 sets.
- Test mode comes from `t=y` in RFC 9989 mode and from `pct=0` in RFC 7489 mode, and the result reports it as [ADR 0020](0020-test-modes-are-reported-and-the-caller-decides-what-they-mean.md) says. hallmark takes no random value.
- The trace records which specification applied and each step of policy discovery.
- hallmark parses `rua` and `ruf` but sends no reports.

## Consequences

Positive:
- A receiver can evaluate the same message under both specifications and see where they differ, in the demo or in its own tests.
- New users get the current standard by default.
- A DMARC evaluation in either mode is a pure function of the message, the DNS answers, the public suffix answers and the options, so it replays without a random value.

Negative:
- Two ways to find the organizational domain mean two code paths and two sets of scenarios.
- A caller that uses RFC 7489 mode has to keep a public suffix list and answer the questions (ADR 0038).

## References

- [RFC 9989](https://www.rfc-editor.org/info/rfc9989), DMARC, and its [Appendix A.6](https://www.rfc-editor.org/rfc/rfc9989.html#appendix-A.6)
- [RFC 7489](https://www.rfc-editor.org/info/rfc7489), the previous DMARC specification, and its [§3.2](https://www.rfc-editor.org/rfc/rfc7489.html#section-3.2)
- [ADR 0020](0020-test-modes-are-reported-and-the-caller-decides-what-they-mean.md), test modes
- [ADR 0038](0038-in-rfc-7489-mode-the-caller-answers-public-suffix-questions.md), public suffix questions
- [requirements.md: DMARC](../reference/requirements.md#dmarc)
