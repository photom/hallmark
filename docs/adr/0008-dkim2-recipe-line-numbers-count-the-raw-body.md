# ADR 0008: DKIM2 recipe line numbers count the raw body

- Status: Accepted
- Date: 2026-11-12
- Affects: DKIM2 verification and recipe writing, and the trace

## Context

`draft-ietf-dkim-dkim2-spec-09` §5.2 numbers body lines "top down", starting at 1, for the `c` (copy) steps of a body recipe. §6.1 hashes the body in the manner of DKIM1's simple scheme: it ignores all empty lines at the end of the body, and adds a CRLF when the body has none at the end. The draft doesn't say whether recipe line numbers count the body as transmitted or the body after the §6.1 step.

Both readings number lines the same way from line 1. They differ in one thing only: whether the empty lines at the end of the current body exist as lines that a `c` step can copy. Take a current body of `A` followed by one empty line. Under the raw reading it has two lines, and `{"c":[1,2]}` is valid. Under the §6.1 reading it has one line, and the same step is invalid.

## Options considered

- (a) Count the raw body: the body as transmitted, with CRLF line endings, after transfer encoding and before dot-stuffing (§17), and before §6.1 drops the empty lines at the end.
- (b) Count the body after §6.1.
- (c) Wait for the working group to settle the question.

## Decision

Option (a), until a draft revision settles the question.

- When verifying, recipe line numbers count the raw body.
- When writing a recipe for a change the caller describes, hallmark never copies empty lines at the end of the body with `c` steps. It writes them as `{"d":""}` steps, which both readings accept.
- When a recipe copies a line that §6.1 would drop, the trace records the assumption: "recipe line numbers count the raw body, so line N exists".

## Consequences

Positive:
- The raw reading accepts every recipe that a signer using the §6.1 reading writes, plus recipes that copy trailing empty lines, so hallmark rejects fewer valid recipes.
- Recipes that hallmark writes verify under either reading.

Negative:
- A recipe that copies trailing empty lines verifies in hallmark and may fail in a verifier that uses the §6.1 reading. The trace makes the case visible.
- A related question stays open: under the raw reading, a last body line with no CRLF counts as a line, and whether copying it adds a CRLF needs its own decision ([hallmark#1](https://github.com/photom/hallmark/issues/1)).

## References

- [draft-ietf-dkim-dkim2-spec-09, §5.2 and §6.1](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#name-body-recipes)
- [hallmark#1](https://github.com/photom/hallmark/issues/1)
- [requirements.md: DKIM2](../reference/requirements.md#dkim2)
