# Detailed design

The detailed design of hallmark: its modules, its domain model, the lifecycle of each object, its concurrency model, its limits, its threat model, the flows that parse and canonicalize a message, and the flows of SPF, DKIM, DKIM2 and DMARC. [Requirements](../requirements.md) say what hallmark must do, [Architecture](../../explanation/architecture.md) explains why the parts fit together the way they do, the [API reference](https://photom.github.io/hallmark/api/hallmark/) gives each method with its validation rules and error codes, and the [ADRs](../../adr/README.md) record each decision.

## Contents

- [Documents](#documents)
- [Conventions](#conventions)

## Documents

| Document | Content | Diagrams |
|---|---|---|
| [modules.md](modules.md) | Crates, modules, layers, dependency direction, features and outside crates | Flowcharts |
| [domain-model.md](domain-model.md) | Bounded contexts, entities, value objects and invariants | Context map, class diagrams |
| [lifecycle.md](lifecycle.md) | States and transitions of a check, its requests, its signing, the canonicalizer, the trace output and the driver, and how long each object lives | State diagrams |
| [concurrency.md](concurrency.md) | Threads, `Send` and `Sync`, parallel checks, the driver's concurrency, cancellation and memory | Flowchart, sequence diagram |
| [limits.md](limits.md) | Every limit: its default, its range, where hallmark checks it and what happens when input reaches it | Flowchart |
| [threat-model.md](threat-model.md) | Assets, actors, trust boundaries, threats and their mitigations | Data flow diagram |
| [header-section.md](header-section.md) | The steps from the header input to header fields and malformed lines, fields from a milter, bare CR and LF, and the header limits | Flowcharts, state diagram |
| [tag-lists.md](tag-lists.md) | The parse of a tag=value list, its value decoders, and the outcome of each error | Flowchart |
| [canonicalization.md](canonicalization.md) | The body pipeline and its state machine, the `l=` cut, the places in the body, and the header forms of DKIM and DKIM2 | Flowcharts, state diagram |
| [spf.md](spf.md) | `check_host()`, the terms, evaluation without recursion, the counters and limits, macros and results | Flowchart, state diagram |
| [dkim.md](dkim.md) | One signature from its field to its result, the field and key checks, time checks, cautions and hints, and signing | Flowcharts, sequence diagram |
| [dkim2.md](dkim2.md) | The verifier actions of draft-09 §14, recipe rebuilds, output states and signing | Flowchart, state diagram, sequence diagram |
| [dmarc.md](dmarc.md) | The author domain, policy discovery in both modes, identifiers, alignment, the policy tag, test mode and results | Flowcharts, sequence diagram |

## Conventions

| Item | Convention |
|---|---|
| Names in `code` | The names the API uses, in the Ubiquitous Language of [domain-model.md](domain-model.md) |
| IDs such as STR-001 | Requirements in [requirements.md](../requirements.md) |
| IDs such as INV-001 or THR-001 | Invariants and threats defined in these documents, unique within their prefix |
| Diagrams | Mermaid, as the [requirements](../requirements.md#quality-and-process) ask |
| Code | None: these are reference documents. Examples belong in tutorials and how-to guides |
