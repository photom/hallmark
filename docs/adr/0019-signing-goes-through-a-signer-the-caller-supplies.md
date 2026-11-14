# ADR 0019: Signing goes through a signer the caller supplies

- Status: Accepted
- Date: 2026-11-14
- Affects: DKIM and DKIM2 signing in `crates/hallmark`, the private key types, hallmark-milter, and the demo

## Context

hallmark signs with DKIM (RFC 6376, RFC 8463) and DKIM2 (`draft-ietf-dkim-dkim2-spec-09`). Each signature needs a private key. Operators often keep signing keys in a hardware security module or a remote key service, where the key never leaves the device, and those services answer asynchronously. hallmark is a library and does no I/O ([ADR 0018](0018-the-core-does-no-io-and-the-caller-drives-it.md)), so it can't call such a service itself.

The algorithms sign different data:
- `rsa-sha256` signs a SHA-256 hash with RSASSA-PKCS1-v1_5 (RFC 6376 §3.3.2, draft-09 §3.2, RFC 8017 §8.2).
- `ed25519-sha256` signs a SHA-256 hash, which Ed25519 takes as its message (RFC 8463 §3, draft-09 §3.3).
- `mldsa44` is pure ML-DSA: the field values go into the signature algorithm without being hashed first, with the context string `DKIM2` (draft-09 §3.4). A signer for ML-DSA-44 therefore needs the whole signing input, not a hash. Draft-09 §3.4 also recommends the hedged variant, which needs a random value for each signature.

Tests and the demo still need to sign with a key held in memory (DEM-006).

## Options considered

- (a) The caller passes a private key, and the core signs with it.
- (b) The core hands the data to sign to a signer that the caller supplies and takes the signature back, as it hands DNS questions to the resolver. The library ships a signer for keys held in memory.
- (c) Both: a key-in-memory path inside the core, and a separate path for outside signers.

## Decision

Option (b).

- Once a signature's input is complete, the check hands a signing request to the caller: the algorithm, and the data that algorithm signs. For `rsa-sha256` and `ed25519-sha256` that data is the SHA-256 hash. For `mldsa44` it is the whole signing input with the context string `DKIM2`. The caller passes the request to its signer and gives the signature back to the check.
- Before signing, a signer states its algorithm and public key. The core refuses an RSA key under 1024 bits (RFC 8301 §3.2), and for DKIM2 an RSA key whose public exponent isn't 65537 (draft-09 §3.2).
- The core checks that each signature it gets back has the size that its algorithm and key give. A signer error, or a signature of the wrong size, stops signing with an error that names the signer. It never turns into a protocol result.
- The library ships a signer for keys held in memory. Its key types follow SEC-001 to SEC-003. It signs ML-DSA-44 with the hedged variant and takes a random value from the caller for each signature, so it draws no random numbers itself.
- The driver of ADR 0018 accepts a signer, as it accepts a resolver.

## Consequences

Positive:
- A key in a hardware security module or a key service works without any change to the core, and the key never enters hallmark's memory.
- One signing path serves every kind of key, so tests of the built-in signer exercise the same core code that an outside signer uses.
- The core holds no private key, so the trace and errors have no key to leak (SEC-002).

Negative:
- An ML-DSA-44 signer receives the whole signing input. The header field limits (SAF-002) bound its size, but it can be several kilobytes.
- Signing has one more round trip, from the core to the signer and back.
- An outside signer's quality is outside hallmark's control. hallmark can check a signature's size but not, without verifying it, that the right key made it.

## References

- [requirements.md: Signers, Private keys](../reference/requirements.md#signers)
- [ADR 0018](0018-the-core-does-no-io-and-the-caller-drives-it.md), the core does no I/O
- [RFC 6376 §3.3.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.3.2), [RFC 8463 §3](https://www.rfc-editor.org/rfc/rfc8463.html#section-3), [RFC 8017 §8.2](https://www.rfc-editor.org/rfc/rfc8017.html#section-8.2)
- [draft-ietf-dkim-dkim2-spec-09 §3](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-3)
