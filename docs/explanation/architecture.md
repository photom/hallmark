# Architecture

hallmark checks whether a message comes from the domain it claims, signs messages with DKIM and DKIM2, and explains each step in a trace. Its architecture has two views, each in its own document. The [requirements](../reference/requirements.md) list what hallmark must do, the [detailed design](../reference/design/README.md) describes its modules, objects and limits, and the [ADRs](../adr/README.md) record each decision.

| View | Holds | Changes |
|---|---|---|
| [External architecture](external-architecture.md) | What callers and the systems around hallmark see and can rely on: the parts, how a caller drives a check or a signing, the rules for streaming, header fields, addresses, DNS records and DKIM2 chains, the trace, and where hallmark-milter fits | Within a major version, only by adding (STB-001). DKIM2 rules can change in any release while `unstable-dkim2` lasts (DK2-005) |
| [Internal architecture](internal-architecture.md) | How hallmark meets those rules: how the code is divided, what a check fixes, how the protocols meet, and how the core streams, parses, hashes and records | In any release, as long as the external rules hold |

A topic with both a rule and a way of meeting it, such as streaming, has a section in each view, and each section links to the other. The external section states only what a caller can test from outside.
