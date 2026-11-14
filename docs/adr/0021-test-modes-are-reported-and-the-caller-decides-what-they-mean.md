# ADR 0021: Test modes are reported, and the caller decides what they mean

- Status: Accepted
- Date: 2026-11-14
- Affects: the DKIM and DMARC result types, the trace, the how-to guides, and every caller that acts on a DMARC result

## Context

Three records can say that their owner is testing:

- A DKIM key record with `t=y`. RFC 6376 §3.6.1: "Verifiers MUST NOT treat messages from Signers in testing mode differently from unsigned email, even should the signature fail to verify. Verifiers MAY wish to track testing mode results to assist the Signer." The verification steps of RFC 6376 §6 never read the flag, so the signature verifies as any other does.
- A DMARC record with `t=y` under RFC 9989. §4.7 calls it "a request" not to apply the policy, with "an expectation that the policy applied to any failing messages will be one level below the specified policy": `quarantine` becomes `none`, and `reject` becomes `quarantine`. It has no effect on `p=none` or on reports.
- A DMARC record with `pct` under RFC 7489. §6.6.4 says receivers "MUST NOT enact the requested policy" on more than the stated percent of messages, and gives the treatment for a message that falls outside it. RFC 9989 Appendix A.6 reports that `pct` "was usually not accurately applied, unless the value specified was either 0 or 100", and calls `t=y` and `t=n` analogous to `pct=0` and `pct=100`.

RFC 9989 §5.4 leaves the final handling of every message to the receiver's local policy. hallmark is a library, so its callers are those receivers.

The DKIM flag also meets DMARC. RFC 9989 §4.4.1 makes the signing domain of any signature that validates a DKIM-Authenticated Identifier, and says nothing about the key's flags. Applied as written, RFC 9989 lets a signature from a testing-mode key give DMARC a pass, which RFC 6376 §3.6.1 seems to forbid. Neither document says how the two combine. Only a pass is affected: DMARC uses only identifiers that passed, so the "even should the signature fail" half changes nothing in DMARC.

## Options considered

- (a) hallmark applies the test modes: DMARC leaves out signatures from testing-mode keys, the result carries the policy one level down, and RFC 7489 mode samples `pct` with a random value from the caller.
- (b) hallmark reports the test modes and computes no policy from them. The results carry each flag and the published policy, and a how-to guide shows callers the steps that the RFCs describe.

## Decision

Option (b).

- The DKIM result carries the key's `t=y` flag. Verification runs as for any other key.
- DMARC counts a signature from a testing-mode key as RFC 9989 §4.4.1 says. If a DMARC pass rests only on such signatures, that is, SPF gives no aligned pass and every aligned DKIM pass comes from a key with `t=y`, the DMARC result says so.
- The DMARC result carries the policy tag that applied (`p`, `sp` or `np`) with its published value, and whether the record asks for test mode. hallmark computes no policy one level down.
- Test mode comes from `t=y` in RFC 9989 mode and from `pct=0` in RFC 7489 mode, as RFC 9989 Appendix A.6 pairs them. In RFC 7489 mode, hallmark ignores any other `pct` value and reads the record as if `pct` were absent, which means 100. It samples nothing and takes no random value. The trace records an ignored `pct` value.
- The trace records each test mode where hallmark found it, with the tag that set it.
- The how-to guide [Handle DMARC test modes](../how-to/handle-dmarc-test-modes.md) shows callers how RFC 9989 §4.7, RFC 7489 §6.6.4 and RFC 6376 §3.6.1 apply.
- The gap between RFC 6376 §3.6.1 and RFC 9989 §4.4.1 stays an open item until a specification closes it.

## Consequences

Positive:
- Callers keep the decisions that RFC 9989 §5.4 leaves to the receiver, and see exactly what each domain published.
- Apart from the `pct` rule, the results hold only what each domain published, so a caller that disagrees with the how-to can apply its own policy.
- DMARC needs no random value, so every evaluation replays exactly.

Negative:
- Each caller has to handle test modes itself, or follow the how-to. A caller that ignores the flags applies published policies at full strength.
- A record written for RFC 7489 with a `pct` from 1 to 99 is reported as if it asked for 100%. A caller that follows the result then applies the policy to more messages than RFC 7489 §6.6.4 allows. RFC 9989 obsoletes RFC 7489 and drops `pct` for the reason Appendix A.6 gives, and the trace records the ignored value so an operator can see it.

## References

- [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1), the DKIM key flags
- [RFC 9989 §4.4.1](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.4.1), [§4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7), [§5.4](https://www.rfc-editor.org/rfc/rfc9989.html#section-5.4) and [Appendix A.6](https://www.rfc-editor.org/rfc/rfc9989.html#appendix-A.6)
- [RFC 7489 §6.6.4](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.6.4)
- [ADR 0020](0020-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md), the two DMARC modes
- [requirements.md: DKIM verification, DMARC](../reference/requirements.md#dmarc)
