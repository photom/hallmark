# ADR 0043: RSA public exponents have a size limit in DKIM and must be 65537 in DKIM2

- Status: Accepted
- Date: 2026-11-16
- Affects: the DKIM and DKIM2 key record parsers, the limits of SAF-002, verification, and the trace

## Context

An RSA public key is a modulus and a public exponent (RFC 8017 §3.1). The cost of verifying a signature grows with the length of the exponent.

For DKIM, RFC 6376 §3.3.1 and §3.3.2 say that the signing algorithm SHOULD use a public exponent of 65537, and say nothing about the exponents a verifier accepts. §8.13 describes an attack: "An attacker could create a large RSA signing key with a small exponent, thus requiring that the verification key have a large exponent. This will force Verifiers to use considerable computing resources to verify the signature. Verifiers might avoid this attack by refusing to verify signatures that reference selectors with public keys having unreasonable exponents." SAF-002 limits the size of a key's modulus, and the number of signatures and key records that hallmark tries, but not the exponent. An exponent as long as the modulus makes one verification cost about as much as one signing.

For DKIM2, draft-09 §3.2 says: "The signing algorithm MUST use a public exponent of 65537." So no valid DKIM2 `rsa-sha256` signature comes from a key with another exponent. Draft-09 §14.5 lists the outcomes of a key problem. It has `PERMERROR: DKIM2-Signature i=<x> public key <s1> algorithm mismatch` and `... is malformed`, and nothing for the exponent. dns-03 §4.2 puts no rule on the exponent.

The signing side already follows these sections: the core refuses a DKIM2 signer whose RSA key has an exponent other than 65537, and accepts other exponents for DKIM (SGN-003).

## Options considered

For DKIM:
- (a) A limit on the number of bits in the exponent, part of the limits of SAF-002, with a default of 32 bits.
- (b) Accept only 65537.
- (c) No limit of its own, and rely on the limits on signatures and key records.

For DKIM2:
- (d) PERMERROR "algorithm mismatch".
- (e) PERMERROR "is malformed".
- (f) Verify as usual, and add a caution.

## Decision

Options (a) and (d).

- SAF-002 limits the number of bits in the public exponent of an RSA key that DKIM verification uses. The default is 32 bits, which holds the exponents in common use, such as 3, 17 and 65537. A caller can change it for each call (PAR-004).
- A DKIM key whose exponent has more bits than the limit can't verify a signature. With one key record at the selector, the signature gets PERMFAIL. With several records, ADR 0025 lists the record with this reason, and another record can still verify the signature. The trace names the limit and cites RFC 6376 §8.13.
- A DKIM2 `rsa` key whose exponent isn't 65537 gets PERMERROR `DKIM2-Signature i=<x> public key <s1> algorithm mismatch`. The trace records the exponent and cites draft-09 §3.2.
- The rules apply to both forms of RSA key data that ADR 0026 accepts.

Option (b) would fail DKIM signatures that RFC 6376 allows, since its rule for the exponent is a SHOULD for signers. Option (c) lets each key record cost as much as a signing operation, times the number of signatures and records that the other limits allow. For DKIM2, "algorithm mismatch" is the draft text closest to a key that the algorithm, as §3.2 defines it, can't use. Option (e) would mix the case up with key data that doesn't decode, and option (f) would pass a signature that §3.2 rules out.

## Consequences

Positive:
- A hostile key record can't make one DKIM verification cost much more than a verification with 65537.
- DKIM2 verification follows §3.2 for verifiers as the core already does for signers.

Negative:
- A DKIM key with an exponent over 32 bits fails at the default limit, although RFC 6376 doesn't forbid it. The caller can raise the limit.
- A verifier that accepts any exponent gives a different result for such a key.
- The DKIM2 text "algorithm mismatch" also covers a `k=` that doesn't fit the signature's algorithm, so the trace has to say which case applied.

## References

- [RFC 6376 §3.3.1](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.3.1), [§3.3.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.3.2) and [§8.13](https://www.rfc-editor.org/rfc/rfc6376.html#section-8.13)
- [RFC 8017 §3.1](https://www.rfc-editor.org/rfc/rfc8017.html#section-3.1)
- [draft-ietf-dkim-dkim2-spec-09 §3.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-3.2) and [§14.5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-14.5)
- [draft-ietf-dkim-dkim2-dns-03 §4.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03#section-4.2)
- [ADR 0025](0025-a-dkim-selector-with-several-key-records-passes-if-any-of-them-verifies-the-signature.md) and [ADR 0026](0026-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md)
- [requirements.md: Hostile input, DNS records](../reference/requirements.md#dns-records)
