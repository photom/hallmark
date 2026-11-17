# ADR 0047: One check type runs any set of checks and shares their DNS answers

- Status: Accepted
- Date: 2026-11-17
- Affects: the public API of `crates/hallmark`, the driver, `crates/hallmark-wasm`, and every caller

## Context

A caller can run each check on its own, or all of them in one call that feeds the SPF and DKIM results into DMARC (GEN-001). Within one call, the core asks each DNS name and type once, and every step that needs the answer reads the same one (DNS-007). The trace gives each kind of check a fixed slot under its root, signing included, so a position names the same step in every call. A DMARC alignment step can then point at an SPF or DKIM result from another call (DMC-005, [Trace codes](../reference/trace-codes.md#slots-under-the-root)).

The API reference needs a shape before it can be written: the types that a caller creates, feeds and takes results from.

## Options considered

- (a) One `Check` type. Its options name the kinds of check it runs, any subset of the seven slots. All of them read the same input, share one table of DNS questions and write one trace. A DMARC check whose SPF or DKIM results come from another call takes those results as options.
- (b) One type per kind of check, plus a combined type that holds them and shares their questions.
- (c) One type per kind of check, and the caller combines them.

## Decision

Option (a).

- `Check` is the only type that a caller feeds. Its options name the kinds of check it runs: SPF for the HELO identity, SPF for the MAIL FROM identity, DKIM, DKIM2, DMARC, DKIM signing and DKIM2 signing.
- Every kind reads from the one input sequence of CHK-001 and skips the input it doesn't need.
- One question table holds each question type and name once (DNS-007). One trace holds the slots of the kinds that run.
- DMARC takes its authenticated identifiers from SPF and DKIM in the same check, or from results that the caller passes from earlier checks, each with its position in that check's trace (DMC-005). The options name the source of each, or say that there is none.
- Options that break a rule of the API give an error when the check is created, before any input.

Option (b) gives every type its own copy of the input methods and makes the combined type keep their questions in step. Option (c) leaves DNS-007 to the caller, and a caller that ran DKIM and DMARC separately would ask for the same names twice.

## Consequences

Positive:
- One set of input methods and one request loop serve every combination, in the driver and in `hallmark-wasm` alike.
- A question goes out once per call, whichever kinds need it.
- The trace of a call that runs several kinds is one document.

Negative:
- `Check` carries state for every kind, so the type grows with each kind added. A kind that doesn't run holds no state.
- A caller that wants the SPF result before the message arrives, such as a milter that acts at MAIL FROM, runs a separate SPF check and passes its result to the check that runs DMARC.

## References

- [requirements.md: General, Running a check, DNS](../reference/requirements.md#general)
- [Trace codes: Slots under the root](../reference/trace-codes.md#slots-under-the-root)
- [ADR 0017](0017-the-core-does-no-io-and-the-caller-drives-it.md), the core does no I/O
