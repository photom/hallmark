# ADR 0010: A DKIM `l=` larger than the body gets no special rule

- Status: Accepted
- Date: 2026-11-13
- Affects: DKIM verification and the trace

## Context

A DKIM signature's `l=` tag gives the number of canonicalized body octets that the body hash covers. RFC 6376 §3.5 says the value "MUST NOT be larger than the actual number of octets in the canonicalized message body", which constrains signers. For verifiers, §3.7 hashes the canonicalized body "truncated to the length specified in the "l=" tag". When the body is shorter than `l=`, that truncation changes nothing.

`l=` is a contested tag. RFC 6376 §8.2 explains how it lets an intermediary append content to a signed message, says signers "should be extremely wary of using this tag", and says assessors might ignore signatures that use it. hallmark never sets it when signing (DKS-004).

A verifier that finds a body shorter than `l=` can't tell which of two things happened:
- The signer set `l=` larger than the body, and the body arrived whole.
- The body was cut short after signing.

## Options considered

- (a) Follow §3.7 as written: hash the whole shorter body and compare the hash with `bh=`.
- (b) Always report `fail`.
- (c) Always report `permerror`.
- (d) Always report `neutral`.

## Decision

Option (a).

- Verification hashes the canonicalized body truncated to `l=` octets, as §3.7 says. When the body is shorter, that is the whole body. The body hash comparison decides the result, as it does for every signature.
- The trace records, as a fact, that `l=` is larger than the canonicalized body and by how many octets, citing §3.5.
- The code has no other path for this case.

## Consequences

Positive:
- hallmark adds no rule of its own and no extra code path for a contested tag.
- A body cut short after signing fails, as it would without `l=`, because its hash differs from the one the signer computed.
- A signature passes only when the whole body it covers arrived intact, so no unsigned content is accepted.

Negative:
- A signature that breaks the MUST NOT in §3.5 can still pass. The trace says that it broke the rule.
- A verifier that rejects such signatures reports a different result for the same message.

## References

- [RFC 6376 §3.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.5), [§3.7](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.7) and [§8.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-8.2)
- [requirements.md: DKIM verification](../reference/requirements.md#dkim-verification)
- [ADR 0009](0009-requirements-and-tests-come-only-from-the-specifications.md), which asks for an ADR where a specification leaves room for more than one reading
