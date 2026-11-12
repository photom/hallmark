# ADR 0009: Requirements and tests come only from the specifications

- Status: Accepted
- Date: 2026-11-12
- Affects: the requirements, the design, every test, the documentation, and outside contributions

## Context

SPF, DKIM, and DMARC have many implementations, and some of them depart from the RFCs in places. A new implementation could study them, copy their behavior, or test against them. Doing so would carry their departures into hallmark, and it would make hallmark's concepts and design look borrowed. hallmark's trace cites the rule behind each step, so the rules need a single source.

## Options considered

- (a) Study other implementations and test hallmark against them for compatibility.
- (b) Build from the specifications only, and use published conformance test data after a license check.

## Decision

Option (b).

- Requirements, design, and tests come from the RFCs and IETF drafts that the requirements name. Each test cites the section it checks, and each trace step cites the section behind it.
- Contributors do not read, copy, or compare behavior against other sender-authentication implementations, and documents, tests, and commit messages do not cite them.
- Published conformance test data, such as the openspf SPF test suite, may be used after a license check. The documentation records each source and its license.
- Where a specification leaves a question open, an ADR records hallmark's reading, as ADR 0008 does, and the trace records the assumption when it applies.

## Consequences

Positive:
- Every behavior traces back to a section of a specification, which the trace can show to its reader.
- hallmark's design is its own.

Negative:
- hallmark has no differential testing against other implementations, so a difference in behavior surfaces only through users and interoperability reports.
- Where implementations agree on behavior that the specifications leave open, hallmark may choose differently. Each such choice needs an ADR.

## References

- [requirements.md: Quality and process](../reference/requirements.md#quality-and-process)
- [ADR 0008](0008-dkim2-recipe-line-numbers-count-the-raw-body.md), an example of a recorded reading
