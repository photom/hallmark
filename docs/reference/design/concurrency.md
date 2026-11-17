# Concurrency

hallmark starts no threads. A check is a state machine that the caller drives one call at a time ([ADR 0017](../../adr/0017-the-core-does-no-io-and-the-caller-drives-it.md)), and the driver only polls futures ([ADR 0048](../../adr/0048-the-resolver-the-suffix-source-and-the-signer-return-futures-that-are-send.md)). This page covers what the library does and doesn't do with threads, which types move between threads, how a caller runs checks in parallel, and how cancellation and memory work. [Lifecycle](lifecycle.md) gives the states.

## Contents

- [What hallmark never does](#what-hallmark-never-does)
- [Send and Sync](#send-and-sync)
- [Checks in parallel](#checks-in-parallel)
- [Inside one check](#inside-one-check)
- [The driver](#the-driver)
- [WebAssembly and blocking callers](#webassembly-and-blocking-callers)
- [Cancellation](#cancellation)
- [Memory](#memory)

## What hallmark never does

| hallmark never | So | Requirements |
|---|---|---|
| Starts a thread or a task | The caller decides where each check runs | DNS-003 |
| Takes a lock, or uses atomics or interior mutability | A check has no shared state to protect | |
| Keeps mutable global state | Two checks never affect each other. The IDNA tables and the English texts are immutable statics | CHK-003 |
| Blocks, sleeps or reads a clock | A call returns as soon as its work is done. Time limits belong to the caller | IO-001, DNS-009 |
| Calls the caller back from inside a check | Only the driver calls the caller's traits, and only between calls to the check | ADR 0017 |
| Uses `unsafe` | The workspace forbids it | |

## Send and Sync

| Type | `Send` | `Sync` | Why |
|---|---|---|---|
| `Check` | Yes | Yes | It owns all its data. Methods that change it take it by unique reference |
| `Options`, `Limits`, `Envelope`, `VerificationTime` | Yes | Yes | Plain values |
| `Question`, `Answer`, `SuffixQuestion`, `SuffixAnswer`, `SigningRequest`, `Signature` | Yes | Yes | Plain values |
| `Results` and every result type | Yes | Yes | Plain values |
| `Event`, `Position` | Yes | Yes | Plain values |
| `Canonicalizer` | Yes | Yes | It owns its state |
| `Snapshot`, `SnapshotResolver` | Yes | Yes | Read only after creation |
| `MemorySigner` | Yes | Yes | Its keys are read only after creation and zeroed on drop (SEC-001) |
| The driver's future | Yes | | The traits require `Send` futures and `Send + Sync` implementers (ADR 0048) |

A test asserts each row at compile time, so a change that breaks one fails the build.

## Checks in parallel

```mermaid
flowchart LR
    subgraph caller["Caller, such as hallmark-milter on Tokio"]
        t1["Task for message 1"]
        t2["Task for message 2"]
        t3["Task for message 3"]
    end
    t1 --> c1["Check 1"]
    t2 --> c2["Check 2"]
    t3 --> c3["Check 3"]
    shared["Shared by reference: resolver, suffix source, MemorySigner"]
    t1 -.-> shared
    t2 -.-> shared
    t3 -.-> shared
```

| Rule | Why |
|---|---|
| One check serves one message. The caller runs as many checks at once as it likes | Checks share nothing |
| The caller may share a resolver, a suffix source, a `SnapshotResolver` or a `MemorySigner` among checks | They are `Sync`, and checks only read them |
| `Options` and `Limits` are copied into each check | INV-015: options never change after creation |
| A long-running caller can change options between messages | Each check takes its own options, so the next check sees new ones, verification of signatures included (SGN-006) |

## Inside one check

A check takes one call at a time, because each input method takes it by unique reference. The caller can resolve many questions at once and hand back the answers one by one, in any order.

```mermaid
sequenceDiagram
    participant Task as Caller's task
    participant Check
    participant R1 as Resolution 1
    participant R2 as Resolution 2
    Task->>Check: end of header fields
    Check-->>Task: questions Q1 and Q2
    par
        Task->>R1: resolve Q1
    and
        Task->>R2: resolve Q2
    end
    R2-->>Task: answer to Q2
    Task->>Check: answer to Q2
    R1-->>Task: answer to Q1
    Task->>Check: answer to Q1
```

The order of the answers changes neither the result nor the sorted trace (CHK-003, INV-013).

## The driver

The driver polls several futures from one task and starts none of its own.

| Rule | Requirements |
|---|---|
| The driver keeps up to `driver_in_flight` futures pending at once, and starts the next when one completes | CHK-005, [Limits](limits.md) |
| It polls its futures in turn from its own `poll`, so it needs no executor beyond the caller's | DNS-003 |
| It hands each answer to the check as soon as its future completes, between polls | CHK-003 |
| It holds shared references to the resolver, the suffix source and the signer, and a unique reference to the check | ADR 0048 |

## WebAssembly and blocking callers

| Caller | How it drives a check | Requirements |
|---|---|---|
| `hallmark-wasm` | Its own loop on the browser's one thread: take the requests, resolve them through `fetch`, hand back the answers. The library's `MemorySigner` signs synchronously in the same loop | WSM-002, ADR 0048 |
| Blocking code | Its own loop: take a request, resolve it, hand back the answer. The adapters of STR-004 read the message from a standard reader and write the trace to a standard writer | STR-004 |
| Async code with `Send` futures | The driver, on any runtime | CHK-005 |

## Cancellation

A caller cancels by dropping the check, or by dropping the driver's future at any await.

| Dropped | What happens | What the caller handles |
|---|---|---|
| A check | Its requests, answers and unread trace bytes are freed. Nothing else holds them | Its own resolutions still running, which it can drop too |
| The driver's future | The resolver, suffix source and signer futures it held are dropped. The check stays as the last delivered answer left it, and the caller can drive it again or drop it | What a dropped resolver or signer future leaves behind, such as a socket |
| A `MemorySigner` | Its keys are zeroed | |

A check never waits, so it needs no timeout of its own. A caller that stops waiting for an answer reports a temporary failure for that question (CHK-002, DNS-009).

## Memory

| Rule | Requirements |
|---|---|
| A check copies no body chunk, and keeps nothing that points into one after the call returns | STR-007, INV-004 |
| The body pipeline keeps a fixed amount of state for each body hash, whatever the size of the body | STR-001, INV-005 |
| Everything else a check keeps is bounded by its limits: the header section, the questions and answers, the trace bytes and the signing data | SAF-004, [Limits](limits.md) |
| At the off level, the trace allocates nothing | TRC-004 |
