# ADR 0062: Verification and signing have separate entry types

- Status: Accepted
- Date: 2026-11-22
- Supersedes: ADR 0047
- Affects: the public API of `crates/hallmark`, the drivers, the `io` adapters, `crates/hallmark-wasm`, and every caller

## Context

A verifying caller runs SPF, DKIM, DKIM2 and DMARC, each on its own or several in one call that feeds the SPF and DKIM results into DMARC (GEN-001). Within one call, the core asks each DNS name and type once, and every step that needs the answer reads the same one (DNS-007). The trace gives each kind of check a fixed slot under its root, so a position names the same step in every call, and a DMARC alignment step can point at an SPF or DKIM result from another call (DMC-005, [Trace codes](../reference/trace-codes.md#slots-under-the-root)).

A signing caller adds DKIM and DKIM2 signatures. Signing takes the header fields and the body but no envelope: DKIM2 takes `mf=` and `rt=` from its own options (draft-09 §10.5, §10.6). It asks no DNS question. It hands signing requests to a signer that the signing caller supplies (SGN-001), and it has settings that verification never reads: the signatures to add, their keys and header lists, and whether hallmark checks the signer's output (SGN-006). Verification has settings that signing never reads, such as time checks and the DMARC sources. The two sides share only the time, the trace level, the message ID, the limits and the form of header field values.

A DKIM2 forwarder verifies the message it received and signs the message it sends, so even a caller that does both works on two different bodies.

## Options considered

- (a) Two entry types: `Check` runs any set of the five checks and shares their DNS answers, and `Signing` runs DKIM signing, DKIM2 signing or both. Each type has options of its own, and each options type holds its own copy of the settings that both sides use.
- (b) One type that runs any set of checks and signings, with one set of options.
- (c) One type for each kind of check and each kind of signing, combined by the caller.

## Decision

Option (a).

| | `Check` | `Signing` |
|---|---|---|
| Runs | SPF for HELO, SPF for MAIL FROM, DKIM, DKIM2, DMARC: the kinds that `CheckOptions` lists | DKIM signing when `SigningOptions::dkim` holds an entry, DKIM2 signing when `SigningOptions::dkim2` is `Some`. At least one of them |
| Options | `CheckOptions`: the time, the trace level, the message ID, the limits, the field form, and the settings of verification | `SigningOptions`: its own time, trace level, message ID, limits and field form, and the signatures to add |
| Input | The envelope, the header fields, the body and the end of the message | The header fields, the body and the end of the message |
| Requests | DNS questions and public suffix questions | Signing requests |
| Results | `CheckResults` | `SigningResults`, with one `SigningOutcome` for DKIM and one for DKIM2 |
| Errors | `CheckError` for its calls, `CheckOptionsError` for its options | `SigningError` for its calls, `SigningOptionsError` for its options, and `SignatureError` in a stopped `SigningOutcome` |
| Driver | `drive`, with a resolver and, in RFC 7489 mode, a source of public suffixes | `drive_signing`, with a signer |
| Blocking adapters | `io::read_message`, `io::write_trace` | `io::read_message_to_sign`, `io::write_signing_trace` |

- Within a check, one question table holds each question type and name once (DNS-007), and one trace holds the slots of the kinds that run. DMARC takes its authenticated identifiers from SPF and DKIM in the same check, or from results that the verifying caller passes from earlier checks, each with its position in that check's trace (DMC-005).
- A signing's trace uses the slots `[6]` for DKIM signing, `[7]` for DKIM2 signing and `[8]` for the header section, the same positions as a check's trace would give them, so a position keeps its meaning across both kinds of trace.
- Each options type holds its own copy of the shared settings, and no rule compares the two copies, because no call takes both.
- Inside the crate, both types use the same header parser, body pipeline, trace recorder and limits.
- Options that break a rule of the API give an error when the check or the signing is created, before any input.

Option (b) shows a signing caller every verification setting, a list of kinds that has to agree with the signatures it names, and a resolver parameter it never uses. A shared setting such as the time would have one value for both sides, though only one side reads each of the others, and signing shares no DNS answer with verification. Option (c) leaves DNS-007 to the verifying caller, which would ask for the same names twice when it runs DKIM and DMARC apart.

## Consequences

Positive:
- Each caller sees only its own settings, input, requests, results and errors.
- No rule has to check that a list of kinds agrees with the signatures to add, so that error can't happen.
- Each driver takes only the trait that its side needs: a verifying caller never names a signer, and a signing caller never names a resolver.
- One set of input methods and one request loop serve every combination of checks, and a question goes out once per check.

Negative:
- The input methods exist twice, once on each type.
- A caller that verifies and signs passes each body chunk to both values.
- The shared settings appear in both options types, so a change to one of them changes both.
- `Check` carries state for every kind of check, so the type grows with each kind added. A kind that doesn't run holds no state.
- A verifying caller that wants the SPF result before the message arrives, such as a milter that acts at MAIL FROM, runs a separate SPF check and passes its result to the check that runs DMARC.

## References

- [requirements.md: General, Running a check, Signers](../reference/requirements.md#running-a-check)
- [Trace codes: Slots under the root](../reference/trace-codes.md#slots-under-the-root)
- [ADR 0017](0017-the-core-does-no-io-and-the-caller-drives-it.md), the core does no I/O
- [ADR 0042](0042-signing-goes-through-a-signer-the-caller-supplies-and-the-caller-can-have-each-signature-verified.md), signing through a signer
- [ADR 0063](0063-checks-and-signings-fix-their-evaluations-when-created-and-each-input-once-it-arrives.md), what a check and a signing hold
- [draft-ietf-dkim-dkim2-spec-09 §10.5 and §10.6](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-10.5)
