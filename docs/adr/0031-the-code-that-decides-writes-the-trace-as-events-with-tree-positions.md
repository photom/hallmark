# ADR 0031: The code that decides writes the trace as events with tree positions

- Status: Accepted
- Date: 2026-11-15
- Supersedes: [ADR 0002](0002-the-code-that-decides-writes-the-trace.md)
- Affects: every check in `crates/hallmark`, the result types, the demo, hallmark-milter, and the Authentication-Results output

## Context

hallmark's main feature is a trace that explains each verification step by step: which SPF mechanism matched, which DNS answer ended a lookup, why a body hash differed, which domain aligned for DMARC. Operators today get one word, such as `fail`, and work out the reason by hand.

A trace that disagrees with the result is worse than no trace. If a trace grows with hostile input, it is a denial-of-service risk. Message content in a trace raises privacy questions for a milter that handles other people's mail. hallmark-milter also checks mail all day, so tracing must cost nothing when it is off.

ADR 0002 settled that the code that makes each decision records it, as structured steps in a tree. Three points it left open now matter:

- DNS answers arrive in any order, and the same inputs must give the same trace (TRC-012). A trace written step by step as the check runs comes out in a different order from run to run.
- A check can stall while it waits for an answer, and a process can die part-way. A trace that only goes out when the check finishes leaves nothing to look at in either case.
- An operator follows one message through the MTA's log, the milter's log and the trace, so all three need an ID in common.

## Options considered

- (a) Emit log events through the `tracing` crate and let the caller collect them.
- (b) Return a result and build an explanation from it afterwards.
- (c) Record a tree of steps, and write it out when the check finishes.
- (d) Record each step as an event that carries its position in the tree, send each event when its step finishes, and let readers sort the events by position to get the tree.

## Decision

Option (d).

- Each decision point records a step as it decides: what it checked, its inputs (each DNS question and answer included), its outcome, and the RFC or draft section that requires it. The library never rebuilds a trace from a result.
- Steps form a tree. An SPF `include` is a child of the term that caused it, and each DKIM signature has its own subtree. Each step goes out as one event, which carries the step's position: the path of child numbers from the root of the tree. The path also names the step's parent, so an event needs no other link.
- A step's position depends only on the inputs: the order that the specifications fix, and the order of items in the input, such as header fields, terms in a record and records in an answer. It never depends on when an answer arrived, where the body chunks were cut, or how many events went out before it.
- Readers combine the events by sorting them by position, which puts each parent before its children. The sorted events are the trace: the same inputs give the same events and, once sorted, the same bytes. The library offers the sort.
- Each event carries the ID that the caller gave the check for the message, such as the MTA's queue ID, so an operator can join hallmark's events with the MTA's and the milter's logs. hallmark never makes up an ID, because the core draws no random values.
- A step is a fact, what the check found, a caution, a fact that a specification warns about, or a hint, a labeled guess about the cause. Cautions and hints never change a result.
- The caller picks the level per call: off, summary, or full. At the off level the recorder allocates nothing, and every recording call checks the level before it builds anything. At the full level, each event goes out when its step finishes. At the summary level, hallmark holds the events back until the check finishes, then sends the events on the path from each check's top step to the steps that decided its result, and every caution, in position order.
- The trace has size limits. If a limit stops it, the trace records that it was cut short, and the check itself goes on.
- The body enters the trace only as hashes, counts and places, never as content ([ADR 0032](0032-the-trace-records-places-in-the-body-and-a-separate-api-streams-the-canonical-body.md)).
- Private key material never enters the trace. Key types offer no way to read their bytes back out.
- [ADR 0033](0033-trace-events-are-i-json-texts-in-a-json-text-sequence.md) chooses the format of the events, and [ADR 0034](0034-trace-step-codes-are-strings-listed-in-a-reference-page.md) the step codes.

Option (a) gives the caller log lines with no fixed form, so tests can't assert on the path a check took, and the demo can't read them. Option (b) lets the trace and the result drift apart. Option (c) leaves nothing behind when a check stalls or a process dies, and holds every step until the end.

## Consequences

Positive:
- The trace and the result come from the same code path, so they can't disagree.
- Tests can assert on the path a check took, such as "permerror at the eleventh lookup, on this term", which catches more than a result-only test (TRC-009).
- A stalled check or a dead process still leaves every event sent before it stopped.
- A milter can write events to its log as they arrive, under the MTA's queue ID.
- The demo and hallmark-milter read one format.

Negative:
- Every decision point carries recording code, which makes the checks longer.
- At the full level, the raw order of events differs between runs, so readers that need the tree, and tests, sort first.
- Positions are part of the public format: moving a step to another place in the tree is a breaking change (STB-001).
- A full trace holds addresses and header content, and callers must decide where to keep it (TRC-006).

## References

- [requirements.md: Trace and reasoning](../reference/requirements.md#trace-and-reasoning)
- [ADR 0017](0017-the-core-does-no-io-and-the-caller-drives-it.md), which makes every check replayable
