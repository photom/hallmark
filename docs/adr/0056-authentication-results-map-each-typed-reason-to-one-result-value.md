# ADR 0056: Authentication-Results map each typed reason to one result value

- Status: Accepted
- Date: 2026-11-20
- Affects: AR-001, AR-002, AR-006, `AuthenticationResults::generate`, `DkimReason`, `DmarcResult`, `docs/reference/design/authentication-results.md` and scenarios ARH-014 to ARH-030

## Context

AR-001 has hallmark write an Authentication-Results field from its results, and AR-002 has each result carry a `reason` built from its typed reason, at every trace level. The registries leave several choices open:

- RFC 6376 §3.9 has three DKIM outcomes, and RFC 8601 §2.7.1 has seven DKIM results. A PERMFAIL can be `fail`, `neutral`, `policy` or `permerror`.
- RFC 9989 §9.2 and RFC 7489 §11.2 register no result for a message that DMARC can't evaluate, such as one with no author domain (DMC-002).
- RFC 9989 §9.1 registers `policy.dmarc`, the policy applied after policy options. hallmark applies no policy (GEN-006).
- RFC 8601 §2.7.2 says the SPF `smtp.mailfrom` value SHOULD NOT hold the local part unless the record provides for authenticating it. Reported erratum 6027 reads the property as a mailbox.
- With a null MAIL FROM, SPF checks `postmaster@` the HELO name as the MAIL FROM identity (RFC 7208 §2.4), and RFC 8601 §4 asks for only the parameter that gave the result.
- `DkimReason::Field` and `DkimReason::Key` held only a trace position, and `DmarcResult` had no reason. With the trace off, the generator couldn't tell which check failed.

## Options considered

For a DKIM PERMFAIL:
- (a) A result for each reason: `fail` when the body hash or the signature didn't verify or the timestamps are out of range, `neutral` when the DKIM-Signature field can't be processed, and `permerror` for a key problem or a limit.
- (b) `fail` for every PERMFAIL.
- (c) `permerror` for every PERMFAIL.

For a message that DMARC can't evaluate:
- (d) `permerror` with a reason.
- (e) No `dmarc` result.

For the `reason` text:
- (f) A fixed phrase for each typed reason.
- (g) The phrase with the values the reason holds, such as a DNS name.

## Decision

Options (a), (d) and (f).

| Choice | Decision |
|---|---|
| DKIM `fail` | `BodyHash`, `Signature`, and the field checks `Expired` and `FutureTimestamp` |
| DKIM `neutral` | Every other field check: syntax, a missing tag, the version, the algorithm, `i=` outside `d=`, From not signed, an `x=` not later than `t=`, and the name rules |
| DKIM `permerror` | Every key check, no key record, a limit, and a header limit that stops DKIM before any signature |
| DKIM with several key records | When none verifies, the result follows the reason of the first record tried that passed its key checks, or of the first record if none did |
| DMARC not applied | `dmarc=permerror` with a reason that names the case |
| `policy` | hallmark never writes the `policy` result, the `policy` ptype or `policy.dmarc` |
| `smtp.mailfrom` | The domain only, never the local part. With a null MAIL FROM, the HELO name |
| `reason` | A fixed English phrase for each typed reason, the text of RFC 6376 §6.1 where it names the failure, and no value from the message, the envelope or DNS |
| API | `DkimReason::Field` and `DkimReason::Key` carry a `DkimFieldCheck` and a `DkimKeyCheck`. `DmarcResult` gains `reason: DmarcReason`, and `DmarcOutcome::NotApplied` moves its case into `DmarcReason::NotApplied` |

[authentication-results.md](../reference/design/authentication-results.md) gives the phrase for each reason, the properties, the comments, the quoting and the folding.

Option (b) gives a DKIM-Signature field that doesn't parse the same result as a signature that failed verification, although RFC 8601 §2.7.1 tells the two apart. Option (c) hides the difference between a signature that failed and a key that can't be used. Option (e) writes the same field for a message that DMARC couldn't evaluate as for a check that never ran DMARC. Option (g) puts DNS names and resolver text, which others control, into a field that MUAs may show. The trace keeps those details.

## Consequences

Positive:
- The field is the same at every trace level, and each result says why in words a person reads.
- No value that a sender or a DNS publisher controls enters a `reason`.

Negative:
- A reader that wants the failing DNS name or the limit reads the trace or the typed result.
- If erratum 6027 is verified, a new ADR replaces this one for `smtp.mailfrom`.

## References

- [RFC 8601 §2.2, §2.4, §2.7.1, §2.7.2, §4](https://www.rfc-editor.org/rfc/rfc8601.html)
- [RFC 6376 §3.9, §6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1)
- [RFC 9989 §9.1, §9.2](https://www.rfc-editor.org/rfc/rfc9989.html#section-9.1)
- [RFC 7489 §11.1, §11.2](https://www.rfc-editor.org/rfc/rfc7489.html#section-11.1)
- [RFC 7208 §2.4](https://www.rfc-editor.org/rfc/rfc7208.html#section-2.4)
- [ADR 0020](0020-test-modes-are-reported-and-the-caller-decides-what-they-mean.md), the caller decides what results mean
- [ADR 0025](0025-a-dkim-selector-with-several-key-records-passes-if-any-of-them-verifies-the-signature.md), several key records at a selector
