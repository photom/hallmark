# ADR 0049: hallmark depends on cryptography crates, `zeroize` and `thiserror`, and writes everything else itself

- Status: Accepted
- Date: 2026-11-17
- Affects: `crates/hallmark/Cargo.toml`, `deny.toml`, the minimum supported Rust version, and every parser in `crates/hallmark`

## Context

hallmark reads messages and DNS answers from strangers and holds private keys for tests and the demo, so every dependency adds code that reads hostile input or touches keys, and releases to follow.

The requirements shape the choice:
- Cryptography uses pure Rust crates that build for WebAssembly, ML-DSA-44 included (WSM-003).
- Private key memory is zeroed when the key is dropped (SEC-001). If a chosen crate has a known timing advisory, an ADR records the decision and its limits (SEC-003). RUSTSEC-2023-0071, the Marvin attack, affects every release of the `rsa` crate before 0.10.0. Version 0.10.0, released on 2026-11-08, is the first that the advisory lists as patched.
- Parsers follow the ABNF and give errors that name the rule and the byte offset ([ADR 0010](0010-header-parsers-follow-the-abnf-and-never-repair-input.md), PAR-001). hallmark converts domain names to A-labels with its own code and maps nothing ([ADR 0012](0012-hallmark-converts-domain-names-to-a-labels-with-its-own-rfc-5891-code.md)). Trace events, snapshots and cases are written in one fixed form of bytes, with members in a set order ([ADR 0033](0033-trace-events-are-i-json-texts-in-a-json-text-sequence.md)).
- The core draws no random numbers (IO-001).
- CI checks the licenses of dependencies (STB-002), and the minimum supported Rust version rises when a dependency needs it ([ADR 0036](0036-the-minimum-supported-rust-version-starts-at-1-88-and-rises-when-a-dependency-needs-it.md)).

## Options considered

- (a) Cryptography crates, `zeroize` and `thiserror`. hallmark writes its own parsers and encoders.
- (b) Option (a), plus crates for base64, JSON, URIs and IDNA.
- (c) A cryptography library that wraps C code.

## Decision

Option (a). The `hallmark` crate depends on these crates and no others:

| Crate | Used for | Requirements |
|---|---|---|
| `sha2` | SHA-256 and SHA-512 | DKV-001, DK2-009 |
| `rsa`, 0.10.0 or later | RSASSA-PKCS1-v1_5 verification, signing with keys in memory, and decoding RSAPublicKey and SubjectPublicKeyInfo | DKV-002, DKV-003, REC-051, SGN-005, [ADR 0026](0026-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md) |
| `ed25519-dalek` | Ed25519 verification and signing with keys in memory | DKV-002, DK2-009, SGN-005 |
| `ml-dsa` | ML-DSA-44 verification and signing with keys in memory, only with the `unstable-dkim2` feature | DK2-009, SGN-005 |
| `zeroize` | Zeroing private key memory on drop | SEC-001 |
| `thiserror` | The error types of the public API | the rust-master skill |

hallmark writes these itself:

| Part | Rules it follows |
|---|---|
| base64 decoder and encoder | RFC 4648, with the folding white space of RFC 6376 §2.10 and draft-09 §2.14 (HDR-007) |
| dkim-quoted-printable decoder | RFC 6376 §2.11 and RFC 8616 §5 (HDR-007) |
| JSON reader | Recipe JSON (HDR-008), snapshots and demo cases (ADR 0035), and trace events for sorting and for extracting snapshots (TRC-008, TRC-012) |
| JSON writer | The fixed form of [Trace format: Bytes](../reference/trace-format.md#bytes) |
| URI checker | RFC 3986 §3 for `rua` and `ruf` (REC-033) |
| IDNA and Punycode | RFC 5891 §5 and RFC 3492 (ADS-002, ADR 0012) |
| IP address text forms | RFC 7208 §5.6 and RFC 4291 §2.2 for SPF (REC-012) |

Rules for these dependencies:
- Each crate comes in with the first code that needs it.
- No dependency draws random numbers for the core (IO-001), so features that bring in `getrandom` stay off. The WebAssembly build in CI fails if `getrandom` comes in, because it has no default source of random numbers on `wasm32-unknown-unknown`.
- `rsa` stays at 0.10.0 or later. cargo-deny fails CI on an earlier release, because RUSTSEC-2023-0071 affects it. While no other timing advisory affects a chosen crate, SEC-003 needs no further ADR. A new advisory gets one.
- A new dependency of the `hallmark` crate needs an ADR that supersedes this one. Dependencies of tests, fuzz targets and `hallmark-wasm` don't, and cargo-deny checks them like the others.

Option (b) brings parsers whose errors don't name the ABNF rule and the offset, an IDNA crate that maps input under UTS #46 where ADR 0012 maps nothing, and a JSON writer that doesn't keep the member order and escaping of ADR 0033. Option (c) breaks WSM-003.

## Consequences

Positive:
- Less outside code reads hostile input, and every parser reports errors the same way.
- Fewer releases to follow, and a smaller list for cargo-deny.
- SEC-003 holds for RSA signing with keys in memory without a recorded exception.

Negative:
- hallmark maintains its own base64, JSON, URI, IDNA and IP address code, each with a fuzz target (SAF-003).
- A release of a cryptography crate that needs a newer Rust raises the minimum supported Rust version (ADR 0036).

## References

- [requirements.md: WebAssembly, Private keys, Stability and licensing](../reference/requirements.md#webassembly)
- [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html)
- [ADR 0010](0010-header-parsers-follow-the-abnf-and-never-repair-input.md), [ADR 0012](0012-hallmark-converts-domain-names-to-a-labels-with-its-own-rfc-5891-code.md), [ADR 0033](0033-trace-events-are-i-json-texts-in-a-json-text-sequence.md), [ADR 0036](0036-the-minimum-supported-rust-version-starts-at-1-88-and-rises-when-a-dependency-needs-it.md)
