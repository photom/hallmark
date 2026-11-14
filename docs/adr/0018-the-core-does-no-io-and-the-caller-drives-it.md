# ADR 0018: The core does no I/O, and the caller drives it

- Status: Accepted
- Date: 2026-11-14
- Supersedes: [ADR 0001](0001-the-core-does-no-io-and-the-caller-drives-it.md)
- Affects: the public API of `crates/hallmark`, `crates/hallmark-wasm`, and every caller, including hallmark-milter

## Context

hallmark is a library. It runs in two different hosts. hallmark-milter runs on Tokio inside a mail server. The browser demo runs in WebAssembly, where DNS over HTTPS goes through `fetch` and its promises. The two hosts share no async runtime, and the browser has no sockets.

Every check needs data from outside: DNS records, the time the message was received, and the message itself. Signing needs a private key, and many operators keep keys in a hardware security module or a remote key service that the library can't reach. DKIM2's ML-DSA-44 signing also needs a random value for the hedged variant that `draft-ietf-dkim-dkim2-spec-09` §3.4 recommends. Messages can be large, and a milter receives the body in chunks of up to 65,535 bytes, cut at arbitrary points.

ADR 0001 also took a random value from the caller for RFC 7489 `pct` sampling. hallmark now samples nothing: `pct=0` sets test mode, and the caller decides what test mode means ([ADR 0021](0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md)).

The trace, hallmark's main feature, is only worth something if a check can be replayed: the same message, DNS answers and time must give the same result and the same trace, in a test, in the demo, and in a bug report.

A milter also wants to start DNS lookups early. It knows the DKIM selectors and the DMARC domain once the header fields are complete, while the body is still arriving, and SPF needs only the envelope.

## Options considered

- (a) The core awaits an async resolver trait that the caller implements, and reads the body from an async reader.
- (b) The core takes a blocking resolver and a blocking reader.
- (c) The core depends on Tokio and does its own DNS.
- (d) The core does no I/O. It is a state machine: the caller pushes input (envelope, header fields, body chunks), the core hands out the DNS questions and signing requests it has, and the caller pushes the answers and signatures back whenever it has them. Time is a parameter. A separate driver runs a check to the end with a resolver and a signer that the caller implements.

## Decision

Option (d).

- The core opens no sockets and files, reads no clock or environment variables, and draws no random numbers. It takes no random values either.
- A check is a value the caller drives. It accepts the envelope, then header fields, then body chunks, and finally the end of the message. After each input it reports the DNS questions it now needs. The caller answers them in any order, alone or together, and at any time before the check needs them.
- Each question has an identifier, and each answer names the question it answers. The trace places each step by what it belongs to, never by when its answer arrived, so the order of answers changes neither the result nor the trace.
- The caller supplies the verification time.
- Signing hands the data to sign to a signer the caller supplies and takes the signature back, the way DNS questions go to the resolver ([ADR 0019](0019-signing-goes-through-a-signer-the-caller-supplies.md)). The library's own signer for keys in memory takes the ML-DSA-44 random value from the caller.
- The core keeps no reference to an input chunk after the call that received it returns.
- Large output, such as a serialized trace or a long DKIM2 header field, comes out in bounded chunks that the caller pulls.
- The library offers a driver that runs a check to the end with a `Resolver` and, for signing, a signer that the caller implements. The driver awaits their futures, which work on any executor, and depends on no async runtime. Adapters accept a standard blocking reader or writer for callers that don't need async.

## Consequences

Positive:
- The same core runs under Tokio, in the browser, and in blocking code.
- A check is a pure function of its inputs, so every case can be saved as a snapshot and replayed offline (TRC-008, DNS-004). A replay needs no random value.
- A milter can resolve DKIM and DMARC questions while the body streams (DNS-005), and run SPF at MAIL FROM time.
- Keys can stay in a hardware security module or a key service, and the core never sees them.
- Memory stays bounded by the configured limits, not by the message size, because the core never holds the body.

Negative:
- The core's API is a state machine, which takes more code to call than one `async fn verify(message)`. The driver covers the common case.
- SPF needs answers one round after another (`include` reveals the next name to look up), so an SPF check passes through several question-and-answer rounds.
- Every check must handle answers that arrive late or never. The caller decides how long to wait and reports a question it gave up on as a temporary failure, which becomes the protocol's temporary error.
- Signing has one more round trip, from the core to the signer and back.

## References

- [requirements.md: I/O only through the caller, Running a check, DNS](../reference/requirements.md#io-only-through-the-caller)
- [ADR 0019](0019-signing-goes-through-a-signer-the-caller-supplies.md), signing through a signer
- [ADR 0021](0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md), test modes and `pct`
- [Architecture](../explanation/architecture.md)
