# ADR 0001: The core does no I/O, and the caller drives it

- Status: Accepted
- Date: 2026-11-13
- Affects: the public API of `crates/hallmark`, `crates/hallmark-wasm`, and every caller, including hallmark-milter

## Context

hallmark runs in two different hosts. hallmark-milter runs on Tokio inside a mail server. The browser demo runs in WebAssembly, where DNS over HTTPS goes through `fetch` and its promises. The two hosts share no async runtime, and the browser has no sockets.

Every check needs data from outside: DNS records, the time the message was received, and the message itself. RFC 7489's `pct` sampling and DKIM2's ML-DSA-44 signing also need random numbers. Messages can be large, and a milter receives the body in chunks of up to 65,535 bytes, cut at arbitrary points.

The trace, hallmark's main feature, is only worth something if a check can be replayed: the same message, DNS answers and time must give the same result and the same trace, in a test, in the demo, and in a bug report.

A milter also wants to start DNS lookups early. It knows the DKIM selectors and the DMARC domain once the header fields are complete, while the body is still arriving, and SPF needs only the envelope.

## Options considered

- (a) The core awaits an async resolver trait that the caller implements, and reads the body from an async reader.
- (b) The core takes a blocking resolver and a blocking reader.
- (c) The core depends on Tokio and does its own DNS.
- (d) The core does no I/O. It is a state machine: the caller pushes input (envelope, header fields, body chunks), the core hands out the DNS questions it needs, and the caller pushes the answers back whenever it has them. Time and random values are parameters. A separate driver runs a check to the end with a resolver the caller implements.

## Decision

Option (d).

- The core opens no sockets and files, reads no clock or environment variables, and draws no random numbers.
- A check is a value the caller drives. It accepts the envelope, then header fields, then body chunks, and finally the end of the message. After each input it reports the DNS questions it now needs. The caller answers them in any order and at any time before the check needs them.
- The caller supplies the verification time, the random value for `pct` in RFC 7489 mode, and the random value for each hedged ML-DSA-44 signature (`draft-ietf-dkim-dkim2-spec-09` §3.4).
- The core keeps no reference to an input chunk after the call that received it returns.
- Large output, such as a serialized trace or a long DKIM2 header field, comes out in bounded chunks that the caller pulls.
- The library offers a driver that runs a check to the end with a `Resolver` the caller implements. The driver awaits the resolver's futures, which work on any executor, and depends on no async runtime. Adapters accept a standard blocking reader or writer for callers that don't need async.

## Consequences

Positive:
- The same core runs under Tokio, in the browser, and in blocking code.
- A check is a pure function of its inputs, so every case can be saved as a snapshot and replayed offline (TRC-008, DNS-004).
- A milter can resolve DKIM and DMARC questions while the body streams (DNS-005), and run SPF at MAIL FROM time.
- Memory stays bounded by the configured limits, not by the message size, because the core never holds the body.

Negative:
- The core's API is a state machine, which takes more code to call than one `async fn verify(message)`. The driver covers the common case.
- SPF needs answers one round after another (`include` reveals the next name to look up), so an SPF check passes through several question-and-answer rounds.
- Every check must handle answers that arrive late or never; a missing answer becomes the protocol's temporary error.

## References

- [requirements.md: I/O only through the caller, Streaming, DNS](../reference/requirements.md#io-only-through-the-caller)
- [Architecture](../explanation/architecture.md)
