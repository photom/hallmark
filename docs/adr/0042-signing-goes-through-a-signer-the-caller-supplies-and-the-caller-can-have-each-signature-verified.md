# ADR 0042: Signing goes through a signer the caller supplies, and the caller can have each signature verified

- Status: Accepted
- Date: 2026-11-16
- Supersedes: [ADR 0018](0018-signing-goes-through-a-signer-the-caller-supplies.md)
- Affects: DKIM and DKIM2 signing in `crates/hallmark`, the private key types, hallmark-milter, and the demo

## Context

hallmark signs with DKIM (RFC 6376, RFC 8463) and DKIM2 (`draft-ietf-dkim-dkim2-spec-09`). Each signature needs a private key. Operators often keep signing keys in a hardware security module or a remote key service, where the key never leaves the device, and those services answer asynchronously. hallmark is a library and does no I/O ([ADR 0017](0017-the-core-does-no-io-and-the-caller-drives-it.md)), so it can't call such a service itself.

The algorithms sign different data:
- `rsa-sha256` signs a SHA-256 hash with RSASSA-PKCS1-v1_5 (RFC 6376 §3.3.2, draft-09 §3.2, RFC 8017 §8.2).
- `ed25519-sha256` signs a SHA-256 hash, which Ed25519 takes as its message (RFC 8463 §3, draft-09 §3.3).
- `mldsa44` is pure ML-DSA: the field values go into the signature algorithm without being hashed first, with the context string `DKIM2` (draft-09 §3.4). A signer for ML-DSA-44 therefore needs the whole signing input, not a hash. Draft-09 §3.4 also recommends the hedged variant, which needs a random value for each signature.

Tests and the demo still need to sign with a key held in memory (DEM-006).

ADR 0018 settled all of the above. It had the core check only the size of each signature that comes back. A signer states its public key before signing, but the core can't tell whether the private key it then uses matches that public key. A key service set up with the wrong key ID, or a key changed in the signer but not in DNS, gives signatures that every receiver fails. A faulty signing device can also return a wrong signature. Some callers want hallmark to catch such signatures before they leave, at the cost of one verification per signature. A milter runs for a long time and shouldn't need a restart to start or stop checking.

## Options considered

For the signer:
- (a) The caller passes a private key, and the core signs with it.
- (b) The core hands the data to sign to a signer that the caller supplies and takes the signature back, as it hands DNS questions to the resolver. The library ships a signer for keys held in memory.
- (c) Both: a key-in-memory path inside the core, and a separate path for outside signers.

For checking the signatures that come back:
- (d) The core verifies every signature.
- (e) The caller turns verification on or off for each call, and it is off by default.
- (f) The library offers a check that signs a fixed input once when the caller sets up a signer, and verifies the result.
- (g) The core checks only the size.

## Decision

Options (b) and (e).

- Once a signature's input is complete, the check hands a signing request to the caller: the algorithm, and the data that algorithm signs. For `rsa-sha256` and `ed25519-sha256` that data is the SHA-256 hash. For `mldsa44` it is the whole signing input with the context string `DKIM2`. The caller passes the request to its signer and gives the signature back to the check.
- Before signing, a signer states its algorithm and public key. The core refuses an RSA key under 1024 bits (RFC 8301 §3.2), and for DKIM2 an RSA key whose public exponent isn't 65537 (draft-09 §3.2).
- The core checks that each signature it gets back has the size that its algorithm and key give. A signer error, or a signature of the wrong size, stops signing with an error that names the signer. It never turns into a protocol result.
- The caller can turn on verification for a call. With it on, the core verifies each signature it gets back with the public key that the signer stated, before it writes the header field. A signature that doesn't verify stops signing with an error that names the signer, as a signature of the wrong size does, and no header field comes out. Verification is off by default.
- The option belongs to each call, and hallmark keeps no state from one call to the next. A caller that runs for a long time, such as a milter, can turn verification on or off for the next message without a restart.
- The library ships a signer for keys held in memory. Its key types follow SEC-001 to SEC-003. It signs ML-DSA-44 with the hedged variant and takes a random value from the caller for each signature, so it draws no random numbers itself. Tests check its signatures, and it doesn't verify them at run time.
- The driver of ADR 0017 accepts a signer, as it accepts a resolver.

Option (a) puts every key into the core's memory and leaves keys in a hardware security module out. Option (c) gives the core two signing paths to keep in step. Option (d) costs a verification on every signature for callers that trust their signer. Option (f) needs the library or the caller to remember which signers passed the check, and it doesn't catch a fault that comes after the check.

## Consequences

Positive:
- A key in a hardware security module or a key service works without any change to the core, and the key never enters hallmark's memory.
- One signing path serves every kind of key, so tests of the built-in signer exercise the same core code that an outside signer uses.
- The core holds no private key, so the trace and errors have no key to leak (SEC-002).
- A caller that wants to know its signatures are good turns verification on, and turns it off again, without stopping.

Negative:
- An ML-DSA-44 signer receives the whole signing input. The header field limits (SAF-002) bound its size, but it can be several kilobytes.
- Signing has one more round trip, from the core to the signer and back.
- With verification off, a signature made with the wrong key leaves hallmark, and receivers fail it.
- With verification on, each signature costs one verification more.

## References

- [requirements.md: Signers, Private keys](../reference/requirements.md#signers)
- [ADR 0017](0017-the-core-does-no-io-and-the-caller-drives-it.md), the core does no I/O
- [RFC 6376 §3.3.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.3.2), [RFC 8463 §3](https://www.rfc-editor.org/rfc/rfc8463.html#section-3), [RFC 8017 §8.2](https://www.rfc-editor.org/rfc/rfc8017.html#section-8.2)
- [draft-ietf-dkim-dkim2-spec-09 §3](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-3)
