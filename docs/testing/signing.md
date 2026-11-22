# Test scenarios: Signing

Scenarios for signing through a signer and verifying what it returns ([ADR 0042](../adr/0042-signing-goes-through-a-signer-the-caller-supplies-and-the-caller-can-have-each-signature-verified.md)), the library's signer for keys in memory, and the changes a DKIM2 Reviser describes. Conventions are in [README.md](README.md).

Keys in these scenarios are generated while the test runs. A test signer records each request it gets and answers with a signature from such a key.

## Contents

- [Signers](#signers)
- [Header fields to sign](#header-fields-to-sign)
- [Verification and key material](#verification-and-key-material)
- [Changes that a Reviser describes](#changes-that-a-reviser-describes)

## Signers

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SIGN-001 | unit | DKIM signing with `rsa-sha256` through the test signer | the body ends | the signing hands out one request: the algorithm `rsa-sha256` and the 32-byte SHA-256 hash of the header hash input | [SGN-002](../reference/requirements.md#signers), [RFC 6376 §3.3.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.3.2) |
| SIGN-002 | unit | DKIM signing with `ed25519-sha256` through the test signer | the body ends | one request: the algorithm `ed25519-sha256` and the 32-byte SHA-256 hash, which the signer signs as its Ed25519 message | [SGN-002](../reference/requirements.md#signers), [RFC 8463 §3](https://www.rfc-editor.org/rfc/rfc8463.html#section-3) |
| SIGN-003 | unit | DKIM2 signing with `mldsa44` through the test signer | the body ends | one request: the algorithm `mldsa44`, the whole signing input of draft-09 §12.8, unhashed, and the context string `DKIM2` | [SGN-002](../reference/requirements.md#signers), [draft-09 §3.4](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-3.4), [draft-09 §12.8](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-12.8) |
| SIGN-004 | unit | A signer that states a 512-bit RSA key | DKIM signing starts | an error that names the key size, before the signing hands out any request | [SGN-003](../reference/requirements.md#signers), [DKS-001](../reference/requirements.md#dkim-signing), [RFC 8301 §3.2](https://www.rfc-editor.org/rfc/rfc8301.html#section-3.2) |
| SIGN-005 | unit | A signer that states a 2048-bit RSA key with the public exponent 3 | DKIM2 signing starts | an error that names the exponent: DKIM2 requires 65537 | [SGN-003](../reference/requirements.md#signers), [draft-09 §3.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-3.2) |
| SIGN-006 | unit | The signer of SIGN-005 | DKIM signing starts | signing goes ahead: RFC 6376 §3.3.2 says only that the exponent SHOULD be 65537 | [SGN-003](../reference/requirements.md#signers), [RFC 6376 §3.3.2](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.3.2) |
| SIGN-007 | unit | A signer that answers an `ed25519-sha256` request with 63 bytes | the signature comes back | an error that names the signer and the size, and no header field to add | [SGN-004](../reference/requirements.md#signers), [RFC 8032 §5.1.6](https://www.rfc-editor.org/rfc/rfc8032.html#section-5.1.6) |
| SIGN-008 | unit | A signer that answers with an error | the error comes back | signing stops with an error that names the signer. No DKIM-Signature field comes out | [SGN-004](../reference/requirements.md#signers) |
| SIGN-009 | unit | A signer that answers only after the signing caller has made other calls to the signing | the signature comes back | the same header fields as with a signer that answers at once | [SGN-001](../reference/requirements.md#signers), [CHK-003](../reference/requirements.md#running-a-check) |
| SIGN-010 | unit | The library's signer with an ML-DSA-44 key, and the same input signed twice with two different random values from the caller | each signature is made | two different signatures, and hallmark verifies both | [SGN-005](../reference/requirements.md#signers), [draft-09 §3.4](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-3.4) |
| SIGN-011 | unit | The library's signer with an Ed25519 key | a message is signed and then verified by hallmark | `pass` | [SGN-005](../reference/requirements.md#signers), [DKS-001](../reference/requirements.md#dkim-signing) |
| SIGN-012 | unit | A key held by the library's signer | the key is formatted with `Debug` and `Display` | neither output holds any byte of the key | [SGN-005](../reference/requirements.md#signers), [SEC-001](../reference/requirements.md#private-keys) |
| SIGN-026 | unit | With `unstable-dkim2`, DKIM2 signing with the `exploded` flag, and a message with no DKIM2-Signature field | the header section ends | signing stops with `signature.exploded_first_signature`, no DKIM2 field comes out, and DKIM signing in the same signing still finishes | [SGN-004](../reference/requirements.md#signers), [DK2-003](../reference/requirements.md#dkim2), [draft-09 §12.5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-12.5) |

## Header fields to sign

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SIGN-013 | unit | A message with one From field and one DKIM-Signature field, and a signing caller that asks to sign From and DKIM-Signature fields with oversigning on | DKIM signing builds `h=` | `h=` names `from` twice and `dkim-signature` once: oversigning adds no extra `dkim-signature`, so no name can reach the new signature's own field | [DKS-006](../reference/requirements.md#dkim-signing), [DKS-003](../reference/requirements.md#dkim-signing), [ADR 0028](../adr/0028-dkim-h-selection-counts-the-signatures-own-dkim-signature-field.md), [RFC 6376 §5.4](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.4) |
| SIGN-014 | unit | The signing caller of SIGN-013 and a message with no DKIM-Signature field | DKIM signing builds `h=` | `h=` doesn't name `dkim-signature` | [DKS-006](../reference/requirements.md#dkim-signing), [RFC 6376 §3.7](https://www.rfc-editor.org/rfc/rfc6376.html#section-3.7) |
| SIGN-015 | unit | A message that hallmark signed as in SIGN-013 | hallmark verifies it | `pass`, and the trace holds no fact or caution about the signature's own field | [DKS-006](../reference/requirements.md#dkim-signing), [HCN-004](../reference/requirements.md#header-canonicalization) |
| SIGN-027 | unit | DKIM signing with `header_fields` set to 2, and a message with three header fields | the header section ends | signing stops with `signature.header_limit`, which names the limit, and no DKIM-Signature field comes out | [SAF-002](../reference/requirements.md#hostile-input), [SGN-004](../reference/requirements.md#signers) |
| SIGN-028 | unit | DKIM signing with `h=` naming Subject, and a message whose Subject value holds a bare CR | the message is signed | the signature covers the bare CR as an octet, and the trace adds a caution that cites RFC 6376 §5.3, RFC 5322 §2.2 and RFC 5321 §2.3.8 | [HDR-012](../reference/requirements.md#header-parsing), [ADR 0051](../adr/0051-a-bare-cr-or-lf-in-the-header-section-is-an-octet-of-the-line-that-holds-it.md), [RFC 6376 §5.3](https://www.rfc-editor.org/rfc/rfc6376.html#section-5.3) |

## Verification and key material

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SIGN-016 | unit | `verify` turned on in the signing options, and a test signer that states one Ed25519 public key and signs with another key | the signature comes back | an error that names the signer and says that the signature doesn't verify with the stated key, and no header field to add | [SGN-006](../reference/requirements.md#signers), [ADR 0042](../adr/0042-signing-goes-through-a-signer-the-caller-supplies-and-the-caller-can-have-each-signature-verified.md) |
| SIGN-017 | unit | The signer of SIGN-016, with `verify` left at its default | the signature comes back | a header field that holds the signature: by default the core checks only its size | [SGN-006](../reference/requirements.md#signers), [SGN-004](../reference/requirements.md#signers) |
| SIGN-018 | unit | One signing caller that signs two messages through the signer of SIGN-016, with `verify` off in the options of the first signing and on in those of the second, and nothing else changed between the calls | each message is signed | the first call returns a header field, and the second returns the error of SIGN-016 | [SGN-006](../reference/requirements.md#signers), [ADR 0042](../adr/0042-signing-goes-through-a-signer-the-caller-supplies-and-the-caller-can-have-each-signature-verified.md) |
| SIGN-019 | unit | The library's signer with a 2048-bit RSA key, signing at the full trace level, and the same signer with a 512-bit key, which the core refuses | each run ends | no trace event and no error message holds any byte of either private key, raw, in hex or in base64 | [SEC-002](../reference/requirements.md#private-keys), [SGN-003](../reference/requirements.md#signers), [TRC-006](../reference/requirements.md#trace-and-reasoning) |

## Changes that a Reviser describes

| ID | Level | Given | When | Then | Spec |
|----|-------|-------|------|------|------|
| SIGN-020 | unit | A Reviser that added a 2-line footer to a 10-line body, and describes lines 1 to 10 as unchanged | it signs | a Message-Instance whose body recipe copies lines 1 to 10. The version rebuilt from it has the body hash of the highest earlier Message-Instance | [DK2-014](../reference/requirements.md#dkim2), [draft-09 §5.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-5.2), [draft-09 §12.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-12.1) |
| SIGN-021 | unit | A Reviser that removed line 5, `Old text`, from a 10-line body, and describes that removal with its content | it signs | a body recipe that copies lines 1 to 4, gives `Old text` as a line, then copies lines 5 to 9 of the new body | [DK2-014](../reference/requirements.md#dkim2), [draft-09 §5.2](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-5.2) |
| SIGN-022 | unit | A Reviser that describes no body change, while the body hash differs from the one in the highest Message-Instance | it signs | an error that says the description misses a body change, and no signature | [DK2-014](../reference/requirements.md#dkim2), [draft-09 §12.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-12.1) |
| SIGN-023 | unit | A Reviser that added a 3-line footer to a 10-line body, and describes lines 1 to 11 as unchanged | it signs | an error: the rebuilt version's body hash differs from the one in the highest Message-Instance. No signature | [DK2-014](../reference/requirements.md#dkim2), [DK2-006](../reference/requirements.md#dkim2) |
| SIGN-024 | unit | A Reviser that asks for a null body recipe | it signs | a Message-Instance whose recipes hold `"b": null` | [DK2-014](../reference/requirements.md#dkim2), [draft-09 §5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-5), [draft-09 §12.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-12.1) |
| SIGN-025 | unit | A Reviser that changed the Subject field and gives its earlier value | it signs | a header recipe for Subject that rebuilds the earlier value, and a rebuilt header hash that matches the highest earlier Message-Instance | [DK2-014](../reference/requirements.md#dkim2), [draft-09 §5.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-5.1) |
