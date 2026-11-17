# ADR 0048: The resolver, the suffix source and the signer return futures that are `Send`

- Status: Accepted
- Date: 2026-11-17
- Affects: the driver in `crates/hallmark`, `crates/hallmark-wasm`, hallmark-milter, and every caller that implements a resolver, a suffix source or a signer

## Context

The driver runs a check to the end with a resolver, in RFC 7489 mode a source of public suffixes, and for signing a signer, each of which the caller implements (CHK-005). It awaits their futures and depends on no async runtime (DNS-003, [ADR 0017](0017-the-core-does-no-io-and-the-caller-drives-it.md)).

hallmark-milter runs on Tokio's multi-threaded runtime, which can move a spawned task between threads, so the task's future must be `Send`. The driver's future is `Send` only if the futures it awaits are. An async method in a trait states whether its future is `Send`, and a caller can't add the bound afterwards.

The browser demo resolves through `fetch`, and the futures that `wasm-bindgen` makes from its promises aren't `Send`. The browser runs the demo's WebAssembly on one thread.

The core's own methods are synchronous. A caller can drive a check without the driver, taking each request and handing back its answer (ADR 0017).

## Options considered

- (a) The traits require `Send` futures. `hallmark-wasm`, and any other caller whose futures aren't `Send`, drives the check without the driver.
- (b) Two sets of traits, one with `Send` futures and one without, and a driver for each.
- (c) No `Send` bound. A caller on a multi-threaded runtime runs the driver on a local task set.

## Decision

Option (a).

- The methods of the resolver, the suffix source and the signer return futures that are `Send`. The traits require `Send` and `Sync`, so the driver can hold a shared reference to each across an await.
- The driver's future is then `Send`, and a milter can spawn it on any Tokio runtime.
- `hallmark-wasm` drives its checks without the driver: it takes the requests, resolves them through `fetch`, and hands the answers back. The library's signer for keys in memory also signs synchronously, for loops like this one.
- A blocking caller drives a check the same way. The adapters of STR-004 read the message from a standard reader and write the trace to a standard writer.

Option (b) doubles the traits, the driver and their tests. Option (c) makes a caller on a multi-threaded runtime run each driver on a local task set, which ties every check to one thread.

## Consequences

Positive:
- One set of traits and one driver.
- hallmark-milter can spawn one task per message on its runtime.

Negative:
- The browser can't use the driver, so `hallmark-wasm` keeps its own loop over requests.
- A resolver or signer whose futures aren't `Send`, such as one built on a client library with handles bound to a thread, needs a wrapper that is `Send`, or its caller drives the check without the driver.

## References

- [requirements.md: Running a check, DNS, Streaming](../reference/requirements.md#running-a-check)
- [ADR 0017](0017-the-core-does-no-io-and-the-caller-drives-it.md), the core does no I/O
- [ADR 0042](0042-signing-goes-through-a-signer-the-caller-supplies-and-the-caller-can-have-each-signature-verified.md), signing through a signer
- [ADR 0047](0047-one-check-type-runs-any-set-of-checks-and-shares-their-dns-answers.md), one check type
