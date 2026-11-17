# ADR 0028: DKIM `h=` selection counts the signature's own DKIM-Signature field

- Status: Accepted
- Date: 2026-11-15
- Affects: DKIM verification, DKIM signing, the DKIM header hash input, and the trace

## Context

A DKIM signature lists in `h=` the names of the header fields it signs. For a name that the message holds more than once, the signer signs the instances "in order from the bottom of the header field block to the top", and it "MAY include more instances of a header field name in "h=" than there are actual corresponding header fields so that the signature will not verify if additional header fields of that name are added" (RFC 6376 §5.4.2). An informative note in §5.4 adds that listing a name once more than the message holds is enough to block any later additions. The verifier picks instances the same way, from the bottom up (DKV-011), and §6.1.3 says only that names match without regard to case.

Three passages restrict the name `dkim-signature`:

- §3.5, on `h=`: "The field MUST NOT include the DKIM-Signature header field that is being created or verified but may include others."
- §3.7: "The DKIM-Signature header field MUST NOT be included in its own "h=" tag, although other DKIM-Signature header fields MAY be signed (see Section 4)."
- §5.4: "The DKIM-Signature header field is always implicitly signed and MUST NOT be included in the "h=" tag except to indicate that other preexisting signatures are also signed."

The field being signed or verified enters the hash last, after the fields of `h=`, with its `b=` value emptied (§3.5, §3.7). At signing time that field isn't in the message yet. At verification it is, and §3.5 says it SHOULD be prepended, so it sits above the signatures that came before it. Take a message with one older DKIM-Signature field, S1, and a signer that creates S2 with `h=from:dkim-signature:dkim-signature`. The signer hashes S1 for the first `dkim-signature` and nothing for the second. At verification, the bottom-up selection finds S1 for the first name and S2 itself for the second, so the header hash differs from the signer's.

A correct `h=` can select the field being verified too. If a DKIM-Signature field below it is removed after signing, or moved above it, the names that pointed at the older fields reach the field being verified. §5.4 warns that "DKIM-Signature header fields may be reordered by some intermediate MTAs".

§6.1.1 lists the checks that make a DKIM-Signature field invalid, and none of them counts `dkim-signature` names. It also lets a verifier return PERMFAIL "for any other reason".

## Options considered

- (a) Skip the field being verified when picking instances for `dkim-signature`.
- (b) Pick instances as DKV-011 does for every name, the field being verified included.
- (c) Return PERMFAIL for a signature whose `h=` names `dkim-signature` more times than there are DKIM-Signature fields below it.

## Decision

Option (b).

- The bottom-up selection of DKV-011 counts the DKIM-Signature field being verified as one instance of `dkim-signature`. If a name selects it, hallmark hashes it in the canonical form that `c=` gives, like any other selected field.
- If a name selects the field being verified, the trace records that fact. It adds a caution (TRC-003) that cites RFC 6376 §3.5, §3.7 and §5.4: the signer broke the MUST NOT of those sections, unless a DKIM-Signature field below this one was removed after signing or moved, as §5.4 warns some MTAs do.
- The header hash then differs from the signer's, and the signature fails as any signature does whose header hash doesn't verify (§6.1.3). The caution doesn't change the result.
- In signatures that hallmark makes, `h=` never includes the new signature's own field: it names `dkim-signature` at most once for each DKIM-Signature field in the message, oversigning included (DKS-006).

§5.4.2 and §6.1.3 describe one selection for every name, and no section gives the field being verified an exception. Option (a) would add one and pass a signature whose `h=` breaks §3.5, §3.7 and §5.4. Option (c) is the PERMFAIL that §6.1.1 lets a verifier return for reasons of its own. Choices that the specification leaves to the receiver stay with the caller (GEN-006).

## Consequences

Positive:
- hallmark selects and hashes header fields as §5.4.2 and §6.1.3 describe, with no exception.
- The trace says why the header hash differs and which rule the `h=` breaks.
- Signatures that hallmark makes never select their own field.

Negative:
- A signature whose signer listed `dkim-signature` once more than the older signatures, to block later ones, fails even if nothing changed after signing.
- A DKIM-Signature field removed or moved after signing gives the same fact, so the caution names that cause as well as the signer.

## References

- [RFC 6376 §3.5](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.5), [§3.7](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.7), [§5.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.4), [§5.4.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.4.2), [§6.1.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.1) and [§6.1.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-6.1.3)
- [requirements.md: DKIM verification, DKIM signing, Header canonicalization](../reference/requirements.md#header-canonicalization)
