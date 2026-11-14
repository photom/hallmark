# Handle DMARC test modes

Use this guide when your program acts on hallmark's DMARC result, for example a milter that decides whether to reject or quarantine a message. hallmark reports what the domain published and whether it is testing. It computes no policy of its own, so these steps are yours to take ([ADR 0021](../adr/0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md)).

## Contents

- [What you need from the result](#what-you-need-from-the-result)
- [Find the policy the domain owner asks for](#find-the-policy-the-domain-owner-asks-for)
- [Records with other `pct` values](#records-with-other-pct-values)
- [A pass that rests only on testing-mode DKIM keys](#a-pass-that-rests-only-on-testing-mode-dkim-keys)
- [DKIM results outside DMARC](#dkim-results-outside-dmarc)
- [Related](#related)

## What you need from the result

The DMARC result (DMC-006 to DMC-008) gives you:

- the result: `pass`, `fail`, `none`, `temperror` or `permerror`
- the policy tag that applied, `p`, `sp` or `np`, and its published value: `none`, `quarantine` or `reject`
- whether the record asks for test mode, and which tag set it: `t=y` in RFC 9989 mode, or `pct=0` in RFC 7489 mode
- whether a pass rests only on DKIM signatures from testing-mode keys

## Find the policy the domain owner asks for

Take these steps when the result is `fail`. A `pass` needs no policy.

1. Read the published value of the policy tag that applied.
2. If the record doesn't ask for test mode, the domain owner asks for that value.
3. If it does, look up the published value in this table:

   | Published value | RFC 9989 mode, `t=y` (§4.7) | RFC 7489 mode, `pct=0` (§6.6.4) |
   |---|---|---|
   | `reject` | `quarantine` | treat the message as if the policy were `quarantine` |
   | `quarantine` | `none` | apply your local message classification as normal |
   | `none` | `none` | `none` |

4. Decide what to do with the message. RFC 9989 §5.4 leaves the final handling to you, says that you SHOULD NOT reject a message only because the policy is `reject`, and says that `p=none` MUST NOT change your existing mail handling.

RFC 9989 §4.7 also asks a receiver to apply "any special handling rules it might have in place" when `t=y` is set, such as rewriting the From header field. Appendix A.6 explains that mailing lists and other intermediaries used `pct=0` that way. If your program resends mail, check whether your rewriting rules should also run on a test-mode `fail`.

## Records with other `pct` values

In RFC 7489 mode, hallmark ignores a `pct` value other than 0 and reports the record as if `pct` were absent, which means 100. RFC 9989 Appendix A.6 reports that other values were seldom applied accurately. The trace records the value hallmark ignored, so you can find such records in the trace.

## A pass that rests only on testing-mode DKIM keys

RFC 6376 §3.6.1 says that verifiers "MUST NOT treat messages from Signers in testing mode differently from unsigned email". RFC 9989 §4.4.1 still counts such a signature for DMARC, and hallmark does too. If the result says that the pass rests only on signatures from testing-mode keys, choose one of these:

- Accept the pass as RFC 9989 §4.4.1 gives it.
- Treat those signatures as absent. SPF gave no aligned pass, so the message then fails DMARC, and you find the policy as in the steps above.

The two RFCs don't say which is right, and the question stays open in the [requirements](../reference/requirements.md#open-items).

## DKIM results outside DMARC

If you score DKIM results on their own, read the testing flag in each DKIM result (DKV-009). RFC 6376 §3.6.1 asks you to treat a message signed with a testing-mode key like unsigned mail, so a failed signature from such a key shouldn't count against the message. The same section allows you to keep testing-mode results to help the signer.

## Related

- [Requirements: DMARC](../reference/requirements.md#dmarc)
- [ADR 0020](../adr/0020-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md), the two DMARC modes
- [RFC 9989 §4.7](https://www.rfc-editor.org/rfc/rfc9989.html#section-4.7), [§5.4](https://www.rfc-editor.org/rfc/rfc9989.html#section-5.4) and [Appendix A.6](https://www.rfc-editor.org/rfc/rfc9989.html#appendix-A.6)
- [RFC 7489 §6.6.4](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.6.4)
- [RFC 6376 §3.6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.6.1)
