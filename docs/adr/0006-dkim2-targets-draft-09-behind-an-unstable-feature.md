# ADR 0006: DKIM2 targets draft-09 behind an unstable feature

- Status: Accepted
- Date: 2026-11-13
- Affects: the DKIM2 code, the Cargo features of `crates/hallmark`, and the public API's semver promise

## Context

DKIM2 is IETF work in progress, not an RFC. Three drafts define it:
- `draft-ietf-dkim-dkim2-spec-09`, dated 1 November 2026: the protocol itself.
- `draft-ietf-dkim-dkim2-dns`: key records and the Domain Policy Record. Draft-09 cites it as [DKIMKEYS].
- `draft-gondwana-dkim2-authres`: the `dkim2` method for Authentication-Results. Draft-09 cites it as [AUTHRESULTS]. It is an individual draft, not yet a working group document.

Draft-09's reference list names `draft-ietf-dkim-dkim2-dns-02` and `draft-gondwana-dkim2-authres-01`, but its text matches the revisions published a day later, on 2 November 2026:
- Draft-09 §14.5 says [DKIMKEYS] deprecates the `h=`, `n=`, `s=` and `t=` key tags, and §3.8 sends readers to [DKIMKEYS] for the Domain Policy Record. Only dns-03 has either.
- Draft-09 §14.1 defines five output states, including NONE for a message with no DKIM2 header fields. Authres-02 maps those five states one to one. Authres-01 assumes four states and says DKIM2 reports that message as a PERMERROR.

Revisions change header fields, recipes, and verification steps, so code written for one revision can fail against another. hallmark's users still want to try DKIM2 now, and the demo is a good place to show it.

## Options considered

- (a) Wait for the RFC before writing any DKIM2 code.
- (b) Follow each new draft revision as it appears.
- (c) Pin one revision of each draft, put DKIM2 behind a Cargo feature that is exempt from semver, and move to new revisions as deliberate changes.

For the pin in (c):
- (c1) The revisions that draft-09's reference list names: dns-02 and authres-01.
- (c2) The revisions that draft-09's text matches: dns-03 and authres-02.

## Decision

Option (c), with (c2).

- hallmark implements `draft-ietf-dkim-dkim2-spec-09`, with `draft-ietf-dkim-dkim2-dns-03` for key records and the Domain Policy Record, and `draft-gondwana-dkim2-authres-02` for Authentication-Results.
- DKIM2 code compiles only with the Cargo feature `unstable-dkim2`. The `unstable-` prefix tells users the feature is outside the semver promise: its API and behavior can change in any release.
- Moving any of the three drafts to a newer revision is its own change, with updated scenarios and tests, and a new ADR that supersedes this one and names the new set.
- When DKIM2 becomes an RFC, a new ADR moves it out of the unstable feature.

## Consequences

Positive:
- Users and the demo can try DKIM2 today.
- The rest of the crate keeps its semver promise while the drafts change.
- Every rule that draft-09 sends readers to, including the key-tag deprecation and the Domain Policy Record, is in one of the pinned texts.

Negative:
- Mail signed under other draft revisions may fail to verify, and the trace has to say which revisions hallmark implements.
- Each revision change costs a review of the drafts against the code and the scenarios.
- The pinned set differs from draft-09's reference list, so a reader of draft-09 alone may expect dns-02 and authres-01.

## References

- [draft-ietf-dkim-dkim2-spec-09](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09)
- [draft-ietf-dkim-dkim2-dns-03](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-dns-03)
- [draft-gondwana-dkim2-authres-02](https://datatracker.ietf.org/doc/html/draft-gondwana-dkim2-authres-02)
- [requirements.md: DKIM2](../reference/requirements.md#dkim2)
