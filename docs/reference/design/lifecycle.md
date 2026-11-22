# Lifecycle

The states of each object that lives longer than one call: a check, a signing, their requests, each signature, the canonicalizer, the trace output and the driver, and how long each object stays in memory. [Concurrency](concurrency.md) covers threads and cancellation.

## Contents

- [A check from start to end](#a-check-from-start-to-end)
- [Check](#check)
- [Requests](#requests)
- [Signing](#signing)
- [Trace output](#trace-output)
- [Canonicalizer](#canonicalizer)
- [Driver](#driver)
- [How long objects live](#how-long-objects-live)

## A check from start to end

```mermaid
sequenceDiagram
    participant Caller
    participant Check
    participant Resolver as Caller's resolver
    Caller->>Check: create with options
    Caller->>Check: envelope
    Check-->>Caller: SPF questions
    Caller->>Resolver: resolve
    Caller->>Check: header fields, then their end
    Check-->>Caller: DKIM, DKIM2 and DMARC questions
    Caller->>Resolver: resolve while the body arrives
    loop each body chunk
        Caller->>Check: body chunk
    end
    Resolver-->>Caller: answers
    Caller->>Check: answers, in any order
    Check-->>Caller: follow-up questions, such as an SPF include
    Caller->>Check: end of message
    Caller->>Check: read trace bytes, at any time
    Caller->>Check: take the results once finished
```

After each call, the caller takes the requests the check now has. It can answer them at once, later, or together (CHK-001).

## Check

```mermaid
stateDiagram-v2
    [*] --> Created : create with valid options
    Created --> Envelope : envelope
    Envelope --> Headers : header field
    Headers --> Headers : header field
    Envelope --> Body : header section, or end of header fields
    Headers --> Body : end of header fields
    Envelope --> Ended : whole message
    Body --> Body : body chunk
    Body --> Ended : end of message
    Ended --> Finished : no request waiting or taken
    Finished --> [*] : verifying caller takes the results
```

A check whose kinds are all SPF needs only the envelope, so it moves from Created to Ended when the envelope arrives (CHK-001, DNS-005).

| State | Input it accepts | Next state | Requests that can appear |
|---|---|---|---|
| Created | Envelope | Envelope, or Ended for SPF alone | SPF questions. In RFC 7489 mode, the public suffix question for the MAIL FROM domain, or the HELO name when MAIL FROM is null (DMC-010) |
| Envelope | One header field (STR-008), the raw header section, the end of header fields, or the whole message | Headers, Body or Ended | As for Body, once the header section is complete |
| Headers | One header field, or the end of header fields | Headers or Body | None |
| Body | A body chunk, or the end of the message | Body or Ended | DKIM and DKIM2 key questions, DKIM2 Domain Policy Record questions, DMARC policy questions, and in RFC 7489 mode public suffix questions for the author domain and each DKIM `d=` domain (DNS-005) |
| Ended | None of the message input | Finished | Questions that follow from answers, such as SPF follow-ups, the DMARC tree walk and the author domain question of REC-032 |
| Finished | None | | None |

In every state from Envelope on, the check also accepts an answer to a question it handed out and a public suffix answer, and these can make new requests appear. Reading trace bytes is allowed in every state.

| Input that breaks the order | What happens | Requirement |
|---|---|---|
| Message input in the wrong state, such as a body chunk before the header section ends, or a second envelope | An error to the caller. The check stays in its state and loses nothing | CHK-004 |
| An answer to a question the check never handed out, or a second answer | An error to the caller. The check keeps the first answer | CHK-004 |
| Taking the results before Finished | The check names what it still waits for, and gives no result | CHK-002 |

How a check keeps its other parts, such as its inputs and which evaluations it runs, is internal, and [Modules](modules.md#inside-the-check) gives it.

## Requests

Questions, public suffix questions and signing requests share one lifecycle.

```mermaid
stateDiagram-v2
    [*] --> Waiting : a step needs it
    Waiting --> Taken : caller takes it
    Taken --> Answered : caller answers it
    Answered --> [*] : check or signing finishes or is dropped
    Waiting --> [*] : check or signing is dropped
    Taken --> [*] : check or signing is dropped
```

| Event | Rule | Requirement |
|---|---|---|
| A second step needs the same question type and name | It reads the existing question and its answer. No new request appears | DNS-007 |
| The caller answers a Waiting request | Error: the caller can know an ID only after taking the request | CHK-004 |
| The caller answers an Answered request | Error, and the first answer stays | CHK-004 |
| The caller gives up waiting | It answers with a temporary failure, which becomes the protocol's temporary error | CHK-002, DNS-009 |
| A step no longer needs an answer, such as the author domain question of a message with an aligned identifier | The request stays, and its answer is still accepted | REC-032 |

## Signing

A signing takes no envelope and asks no DNS question, so its states start at the header fields ([ADR 0062](../../adr/0062-verification-and-signing-have-separate-entry-types.md)).

```mermaid
sequenceDiagram
    participant Caller as Signing caller
    participant Signing
    participant Signer as Signing caller's signer
    Caller->>Signing: create with signing options
    Caller->>Signing: header fields, then their end
    loop each body chunk
        Caller->>Signing: body chunk
    end
    Caller->>Signing: end of message
    Signing-->>Caller: signing requests
    Caller->>Signer: sign
    Signer-->>Caller: signatures
    Caller->>Signing: signatures
    Caller->>Signing: take the results once finished
```

```mermaid
stateDiagram-v2
    [*] --> Created : create with valid signing options
    Created --> Headers : header field
    Headers --> Headers : header field
    Created --> Body : header section, or end of header fields
    Headers --> Body : end of header fields
    Created --> Ended : whole message
    Body --> Body : body chunk
    Body --> Ended : end of message
    Ended --> Finished : no signing request waiting or taken
    Finished --> [*] : signing caller takes the results
```

| State | Input it accepts | Next state | Requests that can appear |
|---|---|---|---|
| Created | One header field (STR-008), the raw header section, the end of header fields, or the whole message | Headers, Body or Ended | None |
| Headers | One header field, or the end of header fields | Headers or Body | None |
| Body | A body chunk, or the end of the message | Body or Ended | None |
| Ended | None of the message input | Finished | Signing requests |
| Finished | None | | None |

In every state, the signing also accepts a signature for a request it handed out, and reading trace bytes is allowed. Input that breaks the order gives an error, as it does for a check (CHK-004, CHK-006).

Each signature that the options ask for follows these states. DKIM2 signing with several algorithms has one signing request for each.

```mermaid
stateDiagram-v2
    [*] --> Ready : signer statement accepted at creation
    Ready --> Hashing : header section complete
    Hashing --> Requested : end of message, data to sign ready
    Requested --> Checking : signature handed back
    Requested --> Stopped : signer failure handed back
    Checking --> Written : right size, and verified if verify is on
    Checking --> Stopped : wrong size, or verification failed
    Written --> [*]
    Stopped --> [*]
```

| State | Meaning | Requirements |
|---|---|---|
| Ready | The options gave the signer statement, and the signing accepted its algorithm and key | SGN-003 |
| Hashing | The body streams through the body hashes the signature needs | STR-001 |
| Requested | The signing request is waiting or taken | SGN-002 |
| Checking | The size check, then verification if `verify` is on in the signing options | SGN-004, SGN-006 |
| Written | The header field is ready, and the results carry it | DKS-005, STR-009 |
| Stopped | That kind of signing stops with an error that names the signer. No header field of that kind comes out, and the other kind still finishes | SGN-004, SGN-006 |

## Trace output

```mermaid
flowchart LR
    step["Step finishes"] --> level{"Level"}
    level -->|"off"| nothing["Nothing is kept"]
    level -->|"full"| out["Event bytes go to the output at once"]
    level -->|"summary"| held["Event is held"]
    held --> fin["Check or signing finishes"]
    fin --> pick["The deciding paths and the cautions go to the output, in position order"]
    out --> read["Caller reads bytes in chunks"]
    pick --> read
```

| Rule | Requirements |
|---|---|
| At the full level, the root event goes out last, when the check or the signing finishes. At the summary level, it goes out first | [Trace format](../trace-format.md#levels-and-order) |
| Every byte the recorder writes counts toward `trace_size`, whether the caller has read it or not | TRC-005 |
| When the limit is reached, the recorder writes nothing more, and the root event says the trace was cut short. Results don't change | TRC-005, INV-009 |
| At the off level, the recorder allocates nothing | TRC-004 |

## Canonicalizer

The public canonicalizer of STR-012 is separate from any check.

```mermaid
stateDiagram-v2
    [*] --> Streaming : create with a canonicalization
    Streaming --> Streaming : body chunk in, canonical bytes out
    Streaming --> Ended : end of body
    Ended --> Drained : caller reads the last bytes
    Drained --> [*]
```

A body chunk after the end of the body gives an error. The output doesn't depend on where the chunks are cut (STR-005).

## Driver

`drive` runs a check, and `drive_signing` a signing, until no request is open (CHK-005).

```mermaid
stateDiagram-v2
    [*] --> Collecting : drive the check or the signing
    Collecting --> Awaiting : requests taken and futures started
    Awaiting --> Delivering : a future completes
    Delivering --> Collecting : answer handed over
    Collecting --> Done : no request open
    Delivering --> Failed : the check refuses an answer, which a signing never does
    Done --> [*]
    Failed --> [*]
    Awaiting --> [*] : caller drops the driver's future
```

| Rule | Requirements |
|---|---|
| A driver hands each answer over in the order its future completes. The result and the sorted trace stay the same | CHK-003, CHK-005 |
| A signer failure is an answer like any other: the signing stops that kind of signing, and `drive_signing` goes on | SGN-004 |
| The driver reads no clock. The caller's resolver applies its own time limits | DNS-009 |

## How long objects live

| Object | Created by | Owned by | Freed | Bound on its size |
|---|---|---|---|---|
| `Check`, `Signing` | The caller | The caller | When the caller drops it | [Limits](limits.md) |
| Body chunk | The caller | The caller | When the call that passed it returns | None. No chunk is copied (STR-007) |
| Header section | The check or the signing, from the input | The check or the signing | With it | `header_section_size` |
| Question and its answer | A step that needs it, and the caller's answer | The check. The caller gets a copy of the question | With the check | `dns_questions`, `answer_size` |
| Signing request data | The signing, at the end of the message | The signing. The signing caller gets a copy | With the signing | `header_section_size` (SGN-002) |
| Trace bytes not yet read | The recorder | The check or the signing | When the caller reads them | `trace_size` |
| `CheckResults`, `SigningResults` | The check or the signing, when it finishes | The caller | When the caller drops them | The limits of the value that made them |
| Private key | The caller, inside a `MemorySigner` or its own signer | The signer | When the signer is dropped, zeroed (SEC-001) | |
