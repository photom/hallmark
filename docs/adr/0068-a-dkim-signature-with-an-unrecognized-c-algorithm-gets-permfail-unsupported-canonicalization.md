# ADR 0068: A DKIM signature with an unrecognized `c=` algorithm gets PERMFAIL (unsupported canonicalization)

- Status: Accepted
- Date: 2026-11-26
- Affects: the field checks of DKIM verification, `DkimFieldCheck`, the `dkim.fields` trace step, and the DKIM results of Authentication-Results

## Context

Under [ADR 0067](0067-dkim-value-keywords-match-only-as-written.md), a `c=` name other than `simple` and `relaxed`, such as `Relaxed` or `x-new`, is an unrecognized algorithm. Its syntax is valid: it matches `x-sig-c-tag-alg`. RFC 6376 says two things about such a signature.

- §3.4: "Verifiers MUST ignore any signatures that use unrecognized canonicalization algorithms."
- §6.1.1: "any inconsistency or unexpected values MUST cause the header field to be completely ignored and the Verifier to return PERMFAIL (signature syntax error)."

Neither settles the outcome alone. §6.1 says that returning PERMFAIL means the verifier stops processing the signature and ignores it completely, and most rules of §6.1.1 pair "ignore" with a PERMFAIL. §3.9 lets the output include "PERMFAILed or otherwise ignored signatures", so an ignored signature need not be a PERMFAIL. hallmark reports a result for each signature (DKV-004), and the status that RFC 6376 gives a signature it can't use is PERMFAIL, which §6.1.1 also gives for unexpected values. RFC 6376 names no reason for an unrecognized algorithm, so the question is which reason the result reports.

§3.3.4 uses the same wording for signing algorithms: "Verifiers MUST ignore any signatures using algorithms that they do not implement." hallmark gives an unknown `a=` the field check `Algorithm`, which Authentication-Results reports as `neutral` with the reason `unsupported algorithm` ([ADR 0056](0056-authentication-results-map-each-typed-reason-to-one-result-value.md)).

## Options considered

- (a) Reuse `DkimFieldCheck::Syntax`: PERMFAIL (signature syntax error), the reason §6.1.1 gives for unexpected values.
- (b) Reuse `DkimFieldCheck::Algorithm`: PERMFAIL (unsupported algorithm).
- (c) Add `DkimFieldCheck::Canonicalization`: PERMFAIL (unsupported canonicalization).

## Decision

Option (c).

- A signature whose `c=` names an unrecognized algorithm for the header or the body fails the field check `Canonicalization`, and its result is PERMFAIL, so it still appears in the results as DKV-004 asks. Verification stops there and asks no key question, as for the other field checks.
- Authentication-Results reports it as `dkim=neutral` with the reason `unsupported canonicalization`, the result that RFC 8601 §2.7.1 gives a signature that couldn't be processed.
- The `dkim.fields` step records the check as `canonicalization`, with the name and its range in the field, and cites RFC 6376 §3.4.
- A `c=` value that breaks the ABNF still fails the field check `Syntax`, as any value that doesn't decode does.

Option (a) has the RFC's own words, but it reports a syntax error for a value whose syntax is valid, and the result can't tell an unknown `c=` from a broken tag list. Reusing `Algorithm` would name the wrong tag, because it is the `a=` check. A new check treats the two "MUST ignore" rules of §3.3.4 and §3.4 the same way: each gets a check of its own.

## Consequences

Positive:
- The result names what failed, and the trace points at the name in `c=`.
- An unknown signing algorithm and an unknown canonicalization get parallel checks and the same Authentication-Results result.

Negative:
- The reason `unsupported canonicalization` isn't RFC 6376 text, as `unsupported algorithm` isn't either.
- `DkimFieldCheck` gains a variant. The enum is `#[non_exhaustive]`, so callers already handle variants they don't know.

## References

- [RFC 6376 §3.3.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.3.4)
- [RFC 6376 §3.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.4)
- [RFC 6376 §3.9](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.9)
- [RFC 6376 §6.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1)
- [RFC 6376 §6.1.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.1)
- [RFC 8601 §2.7.1](https://www.rfc-editor.org/rfc/rfc8601.html#section-2.7.1)
- [ADR 0056](0056-authentication-results-map-each-typed-reason-to-one-result-value.md)
- [ADR 0067](0067-dkim-value-keywords-match-only-as-written.md)
