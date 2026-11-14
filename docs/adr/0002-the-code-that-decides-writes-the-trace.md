# ADR 0002: The code that decides writes the trace

- Status: Accepted
- Date: 2026-11-12
- Affects: every check in `crates/hallmark`, the result types, the demo, and the Authentication-Results output

## Context

hallmark's main feature is a trace that explains each verification step by step: which SPF mechanism matched, which DNS answer ended a lookup, why a body hash differed, which domain aligned for DMARC. Operators today get one word, such as `fail`, and work out the reason by hand.

A trace that disagrees with the result is worse than no trace. If a trace grows with hostile input, it is a denial-of-service risk. Message content in a trace raises privacy questions for a milter that handles other people's mail. hallmark-milter also checks mail all day, so tracing must cost nothing when it is off.

## Options considered

- (a) Emit log events through the `tracing` crate and let the caller collect them.
- (b) Return a result and build an explanation from it afterwards.
- (c) Record structured trace steps in the code that makes each decision, into a recorder that the check carries, with a level chosen per call.

## Decision

Option (c).

- Each decision point records a step as it decides: what it checked, its inputs (each DNS question and answer included), its outcome, and the RFC or draft section that requires it. The library never rebuilds a trace from a result.
- Steps form a tree. An SPF `include` is a child of the term that caused it, and each DKIM signature has its own subtree.
- A step is either a fact, what the check found, or a hint, a labeled guess about the cause. Hints never change a result.
- The caller picks the level per call: off, summary, or full. At the off level the recorder allocates nothing, and every recording call checks the level before it builds anything.
- The trace has size limits. When a limit stops it, the trace records that it was cut short; the check itself goes on.
- The body enters the trace as hashes, byte counts and line counts. Body content goes in only when the caller asks, and then it streams out as the body passes, without being kept (TRC-010).
- Private key material never enters the trace. Key types offer no way to read their bytes back out.
- The trace serializes to a documented format with its own schema version. A later ADR chooses the format before the first release.

## Consequences

Positive:
- The trace and the result come from the same code path, so they can't disagree.
- Tests can assert on the path a check took, such as "permerror at the eleventh lookup, on this term", which catches more than a result-only test (TRC-009).
- The demo and hallmark-milter read one format.

Negative:
- Every decision point carries recording code, which makes the checks longer.
- The trace schema is a public interface with its own compatibility rules (STB-001).
- A full trace holds addresses and header content, and callers must decide where to keep it (TRC-006).

## References

- [requirements.md: Trace and reasoning](../reference/requirements.md#trace-and-reasoning)
- [ADR 0001](0001-the-core-does-no-io-and-the-caller-drives-it.md), which makes every check replayable
