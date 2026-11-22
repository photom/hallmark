# ADR 0059: The modules of the hallmark crate are bounded contexts in five layers

- Status: Accepted
- Date: 2026-11-22
- Affects: the module layout of `crates/hallmark`, [Modules](../reference/design/modules.md), the context map of the [domain model](../reference/design/domain-model.md), and the module that holds `Canonicalizer`

## Context

The domain model names hallmark's bounded contexts: Check, Message, Addresses and domain names, Exchange, SPF, DKIM, DKIM2, DMARC, Authentication-Results and Trace. Each context owns its words, such as "selector" in DKIM and "alignment" in DMARC.

DKIM and DKIM2 build on body canonicalization, cryptography and tag lists, which are not protocols themselves. The public `Canonicalizer` streams the canonical body that a check hashes, and to stream an earlier DKIM2 version it reads the recipes of Message-Instance fields (STR-012, [ADR 0041](0041-the-trace-records-places-in-the-body-and-a-separate-api-streams-the-canonical-body.md)), which DKIM2 defines (draft-09 §5). Code that DKIM2 builds on can't hold the `Canonicalizer` without depending on DKIM2.

Each specification changes on its own schedule. DKIM2 follows drafts that can move to a new revision (DK2-002), DMARC has two RFCs (DMC-001), and SPF and DKIM have errata. hallmark-milter depends on the one published `hallmark` crate (REP-001), which follows semantic versioning (STB-001).

## Options considered

- (a) One crate whose modules are grouped by bounded context into five layers, with the dependency rule checked outside the compiler.
- (b) The layers grouped by the kind of code: kernel, exchange values, syntax, protocols, application and adapters.
- (c) One crate per bounded context in the workspace, so the compiler enforces each boundary.

## Decision

Option (a). From the innermost layer out:

| Layer | Modules, by context | A module may use |
|---|---|---|
| Kernel | `parse`, `base64`, `json`, `limits`, `time`, `trace` | The kernel |
| Supporting contexts | Message: `header`, `canonical`. Tag lists: `tags`. Addresses and domain names: `address`, `idna`, `ip`. Exchange: `dns`, `suffix`, `signing`. Cryptography: `crypto` | The kernel, and the modules of its own context |
| Protocol contexts | `spf`, `dkim`, `dkim2`, `dmarc` | The kernel and the supporting contexts. Never another protocol context |
| Application | `check`, `authres`, `canonicalizer`, `render` | Every layer below |
| Adapters | `driver`, `io`, `snapshot`, `keys` | Every layer below |

- `canonical` keeps the body state machine, the `l=` cut, the places in the body, the `simple` and `relaxed` forms of a header field, and a rebuild that applies copy and insert steps to the body as it streams. `dkim2` turns the recipes of Message-Instance fields into those steps.
- The public `Canonicalizer` sits in the application module `canonicalizer`, which joins `canonical` with the recipe reading of `dkim2`.
- A hash input that selects header fields belongs to the protocol whose specification defines it. HCN-003 and HCN-004 go in `dkim`, and HCN-005 to HCN-007 in `dkim2`. Both take the field forms from `canonical`.
- `uri`, which only DMARC uses (REC-033), is a submodule of `dmarc`. `ip` belongs to Addresses and domain names, because the envelope parser reads IPv4 and IPv6 address literals (RFC 5321 §4.1.3), and SPF (REC-012) and pasted records (DNS-010) read the same text forms.
- `crypto` wraps the cryptography crates of [ADR 0049](0049-hallmark-depends-on-cryptography-crates-zeroize-and-thiserror-and-writes-everything-else-itself.md), and no trait stands between it and the protocols. Hashing and signature verification do no I/O and give the same output for the same input, and hallmark has no second implementation to swap in.
- Each public type sits in the module of its context, and `lib.rs` re-exports it at the crate root, so the module never shows in a public path. Every other item is `pub(crate)` at most, and the inner modules of a context are private to its module tree.
- Review checks each `use crate::` path against the table. Once the modules hold code, a CI job checks them too.

Option (b) puts parsers in one layer and protocols and canonicalization in another. That splits the Message context, with `header` in one layer and `canonical` in the other, and needs an exception so that `dkim` and `dkim2` can use `canonical` and `crypto`, which sit in the protocol layer without being protocols.

Option (c) lets the compiler enforce each boundary. But hallmark-milter depends on the published crate, so each context crate would have to be published too, with its own versions under STB-001, and the `unstable-dkim2` feature would have to reach every crate that DKIM2 touches.

## Consequences

Positive:
- Each context's words stay inside its module tree, and a change to one specification, such as a new DKIM2 draft, touches one protocol context.
- The dependency graph has no cycle and no exception.
- A caller sees every public type at the crate root, whatever module holds it.

Negative:
- The compiler doesn't check the layers. A `use` that breaks the table compiles until review or the CI job catches it.
- A supporting context can't use another one, so a value that two of them need has to move to the kernel.

## References

- [ADR 0003](0003-rust-code-is-written-in-a-procedural-style.md), the procedural style
- [ADR 0041](0041-the-trace-records-places-in-the-body-and-a-separate-api-streams-the-canonical-body.md), the canonicalizer
- [ADR 0049](0049-hallmark-depends-on-cryptography-crates-zeroize-and-thiserror-and-writes-everything-else-itself.md), the outside crates
- [ADR 0054](0054-the-public-api-follows-one-set-of-conventions.md), the public API conventions
- [draft-ietf-dkim-dkim2-spec-09 §5](https://datatracker.ietf.org/doc/html/draft-ietf-dkim-dkim2-spec-09#section-5), recipes
- [RFC 5321 §4.1.3](https://www.rfc-editor.org/rfc/rfc5321.html#section-4.1.3), address literals
- [Modules](../reference/design/modules.md)
