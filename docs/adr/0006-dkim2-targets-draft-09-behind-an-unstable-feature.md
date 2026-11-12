# ADR 0006: DKIM2 targets draft-09 behind an unstable feature

- Status: Accepted
- Date: 2026-11-12
- Affects: the DKIM2 code, the Cargo features of `crates/hallmark`, and the public API's semver promise

## Context

DKIM2 is an IETF DKIM working group draft, not an RFC. The latest revision is `draft-ietf-dkim-dkim2-spec-09`, dated 1 November 2026. Revisions change the header fields, the recipes, and the verification steps, so code written for one revision can fail against another. hallmark's users still want to try DKIM2 now, and the demo is a good place to show it.

## Options considered

- (a) Wait for the RFC before writing any DKIM2 code.
- (b) Follow each new draft revision as it appears.
- (c) Pin one revision, put DKIM2 behind a Cargo feature that is exempt from semver, and move to a new revision as a deliberate change.

## Decision

Option (c).

- hallmark implements `draft-ietf-dkim-dkim2-spec-09`.
- DKIM2 code compiles only with the Cargo feature `unstable-dkim2`. The `unstable-` prefix tells users the feature is outside the semver promise: its API and behavior can change in any release.
- Moving to a newer revision is its own change, with updated scenarios and tests, and a new ADR that supersedes this one and names the new revision.
- When DKIM2 becomes an RFC, a new ADR moves it out of the unstable feature.

## Consequences

Positive:
- Users and the demo can try DKIM2 today.
- The rest of the crate keeps its semver promise while the draft changes.

Negative:
- Mail signed by a different draft revision may fail to verify, and the trace has to say which revision hallmark implements.
- Each revision change costs a review of the draft against the code and the scenarios.

## References

- [draft-ietf-dkim-dkim2-spec-09](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09)
- [requirements.md: DKIM2](../reference/requirements.md#dkim2)
