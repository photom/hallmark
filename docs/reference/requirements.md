# Requirements

The requirements for hallmark, with an ID for each one. Test scenarios in [docs/testing](../testing/README.md) link to these IDs, and the [architecture decision records](../adr/README.md) record the choices behind them. [Architecture](../explanation/architecture.md) explains how the parts fit together.

hallmark is a Rust library for email sender authentication. It verifies SPF, DKIM, DKIM2 and DMARC, signs messages with DKIM and DKIM2, and runs both natively and in WebAssembly. Its main feature is a detailed trace that explains every step of every check, so an operator can see why a message passed or failed instead of getting a one-word result.

hallmark is a library. It reports results and leaves every action to its caller: whether to reject or quarantine a message, which header fields to delete, and how to treat a domain that says it is testing (GEN-006).

Two programs use the library:

- hallmark-milter, a separate repository, signs and verifies mail inside a Sendmail or Postfix MTA.
- The browser demo, in this repository, shows what hallmark does and doubles as a test bench.

## Contents

- [Context](#context)
- [Scope](#scope)
- [Functional requirements](#functional-requirements)
  - [General](#general)
  - [Running a check](#running-a-check)
  - [SPF (RFC 7208)](#spf-rfc-7208)
  - [DKIM verification](#dkim-verification)
  - [DKIM signing](#dkim-signing)
  - [Signers](#signers)
  - [DKIM2](#dkim2)
  - [DMARC](#dmarc)
  - [Authentication-Results (RFC 8601)](#authentication-results-rfc-8601)
  - [Trace and reasoning](#trace-and-reasoning)
  - [Verification time](#verification-time)
  - [DNS](#dns)
  - [Parsers](#parsers)
  - [Header parsing](#header-parsing)
  - [Addresses and domain names](#addresses-and-domain-names)
- [Non-functional requirements](#non-functional-requirements)
  - [I/O only through the caller](#io-only-through-the-caller)
  - [Streaming](#streaming)
  - [WebAssembly](#webassembly)
  - [Hostile input](#hostile-input)
  - [Private keys](#private-keys)
  - [Stability and licensing](#stability-and-licensing)
- [Repository and deliverables](#repository-and-deliverables)
- [Browser demo](#browser-demo)
- [Quality and process](#quality-and-process)
- [Relationship to hallmark-milter](#relationship-to-hallmark-milter)
- [Open items](#open-items)

## Context

```mermaid
flowchart LR
    msg["Message, envelope and options"] --> core["hallmark core"]
    time["Verification time"] --> core
    core -->|"DNS questions"| resolver["Resolver supplied by the caller"]
    resolver -->|"DNS answers"| core
    core -->|"Data to sign"| signer["Signer supplied by the caller"]
    signer -->|"Signatures"| core
    core --> out["Results, trace and Authentication-Results"]
    milter["hallmark-milter"] -->|"uses the published crate"| core
    wasm["hallmark-wasm and the browser demo"] -->|"uses the workspace crate"| core
```

The caller supplies everything that comes from outside: the message, the DNS answers, the time, and the signatures, which a signer of its choosing computes (SGN-001). The core takes no random values. With the same inputs, the core always produces the same results and the same trace.

```mermaid
flowchart LR
    spf["SPF result for MAIL FROM"] --> align["Identifier alignment"]
    dkim["DKIM results, one per signature"] --> align
    from["Author domain from the From header"] --> align
    policy["DMARC policy discovery"] --> eval["DMARC evaluation"]
    align --> eval
    eval --> result["DMARC result and the published policy"]
```

## Scope

In scope:

- SPF verification, RFC 7208
- DKIM verification and signing, RFC 6376 with RFC 8301, RFC 8463 and RFC 8616
- DKIM2 verification and signing, from `draft-ietf-dkim-dkim2-spec-09` and its key record draft `draft-ietf-dkim-dkim2-dns-03`, as an experimental feature
- DMARC evaluation under RFC 9989 (DMARCbis, May 2026) and under RFC 7489, which RFC 9989 obsoletes
- Generating and parsing Authentication-Results header fields, RFC 8601, with the `dkim2` method of `draft-gondwana-dkim2-authres-02`
- The trace and reasoning for every check
- WebAssembly bindings and a browser demo published on GitHub Pages

Out of scope:

- ARC, which DKIM2 aims to replace
- Sender ID, ADSP and BIMI
- Sending DMARC aggregate or failure reports. The library parses `rua` and `ruf` but sends nothing.
- Network code in the core: no DNS resolver implementation and no SMTP handling
- MIME parsing. For now, this leaves out checking DKIM2 delivery status notifications (draft-09 §16.4) ([ADR 0012](../adr/0012-hallmark-reads-no-mime-structure.md)).
- Received-SPF header fields (RFC 7208 §9.1), for now. SPF results go in Authentication-Results (RFC 7208 §9.2, AR-001).

## Functional requirements

### General

| ID | Requirement |
|---|---|
| GEN-001 | A caller can run each check (SPF, DKIM, DKIM2, DMARC) on its own, or run all of them in one call that feeds the SPF and DKIM results into DMARC. |
| GEN-002 | Checks take the message as bytes in RFC 5322 form, plus envelope data: the client IP address, the HELO name, the MAIL FROM address and the RCPT TO addresses. DKIM2's chain-of-custody check uses MAIL FROM and RCPT TO (draft-09 §14.4). |
| GEN-003 | DKIM and DKIM2 accept a message as a stream (STR-001). A caller with the whole message can pass it in one piece. |
| GEN-004 | Every result is a typed value that carries a machine-readable reason: which mechanism matched, why a signature failed, which domain aligned. |
| GEN-005 | A temporary DNS failure produces the protocol's temporary-error result (`temperror`). It never turns into a pass or a fail. |
| GEN-006 | hallmark returns results, a trace and header fields to add, and acts on nothing. Rejecting or quarantining a message (RFC 9989 §5.4) and deleting header fields (AR-005) stay with the caller, as does every choice that the specifications leave to the receiver, such as how to treat test modes (DKV-009, DMC-007, [ADR 0021](../adr/0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md)). |

### Running a check

A check is a value the caller drives, so the same core runs under any async runtime, in blocking code and in the browser ([ADR 0018](../adr/0018-the-core-does-no-io-and-the-caller-drives-it.md)).

| ID | Requirement |
|---|---|
| CHK-001 | The caller passes a check the envelope, then the header fields, then the body in chunks, then the end of the message. After each input, the check lists the DNS questions and signing requests (SGN-001) it now has, and the caller answers them whenever it has the answers. SPF needs only the envelope (DNS-005). |
| CHK-002 | A check finishes when its input is complete and every answer it needs has arrived, and it then returns its result and its trace. A caller that stops waiting for an answer reports a temporary failure for that question (DNS-009). |
| CHK-003 | A check's result and trace depend only on its inputs: not on when answers arrive, how the caller groups them, or where body chunks are cut (TRC-012, STR-005). |
| CHK-004 | Input that the check can't accept, such as a body chunk before the header fields end, an answer to a question it never asked, or a second answer to the same question, gives an error to the caller. It never turns into a protocol result. |
| CHK-005 | The driver of DNS-003 runs a check to the end with a resolver and, for signing, a signer that the caller implements. It may resolve several questions at once, and the result and trace stay the same (CHK-003). |

### SPF (RFC 7208)

| ID | Requirement |
|---|---|
| SPF-001 | The library implements `check_host()` for the MAIL FROM identity. When MAIL FROM is empty, it checks `postmaster@` the HELO name, as RFC 7208 §2.4 says. |
| SPF-002 | A caller can also check the HELO identity on its own (RFC 7208 §2.3). |
| SPF-003 | Results use all seven values: `none`, `neutral`, `pass`, `fail`, `softfail`, `temperror`, `permerror`. |
| SPF-004 | Every mechanism and modifier works, including `ptr`, which RFC 7208 §5.5 tells publishers not to use, plus `exp=` and `redirect=`. |
| SPF-005 | Macro expansion follows RFC 7208 §7. |
| SPF-006 | The library enforces the processing limits of RFC 7208 §4.6.4: at most 10 terms that cause DNS lookups, the void-lookup limit (default 2), and the MX and PTR address-lookup limits, each with the result the RFC gives for it. |
| SPF-007 | Record selection follows RFC 7208 §4.5. A TXT record is an SPF record only if it begins with exactly `v=spf1` followed by a space or the end of the record, so a record that begins `v=spf10` is discarded. No SPF record gives `none`, and more than one gives `permerror`. |

### DKIM verification

| ID | Requirement |
|---|---|
| DKV-001 | The library verifies DKIM-Signature header fields under RFC 6376, with the `simple` and `relaxed` canonicalizations for headers and body. |
| DKV-002 | It supports `rsa-sha256` and `ed25519-sha256` (RFC 8463). An `rsa-sha1` signature is never valid (RFC 8301). |
| DKV-003 | It verifies RSA keys from 1024 to 4096 bits. A key under 1024 bits makes the signature invalid (RFC 8301). The caller can choose whether to accept larger keys. |
| DKV-004 | It verifies each signature on a message and reports a result for each one. The number of signatures it checks has a limit with a default (RFC 6376 §6.1). |
| DKV-005 | It parses key records under RFC 6376 §3.6.1, including an empty `p=` (revoked key), `h=`, `s=`, and the `t=y` and `t=s` flags. |
| DKV-006 | A signature with `l=` can verify, and both the result and the trace report how much of the body was left unsigned (RFC 6376 §8.2). |
| DKV-007 | Internationalized addresses and domains follow RFC 8616. |
| DKV-008 | An `l=` larger than the canonicalized body gets no special rule ([ADR 0010](../adr/0010-a-dkim-l-tag-larger-than-the-body-gets-no-special-rule.md)). Verification follows RFC 6376 §3.7 as written: truncating the body to `l=` octets leaves it whole, and the body hash comparison decides the result. The trace records, as a fact, that `l=` exceeds the body, which RFC 6376 §3.5 forbids signers to do. |
| DKV-009 | A key record with `t=y` marks the signature as testing. Verification runs as for any other key, because the steps of RFC 6376 §6 never read the flag, and the result carries the flag. RFC 6376 §3.6.1 tells verifiers to treat mail from a testing-mode key like unsigned mail, and the caller applies that ([ADR 0021](../adr/0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md)). |
| DKV-010 | The trace records a caution (TRC-003) on a signature that passes with an RSA key under 2048 bits (RFC 8301 §3.2), with an `l=` tag (RFC 6376 §8.2), or with an `h=` that names From only as many times as the message has From fields, so that a From field added later leaves the signature valid (RFC 6376 §5.4 and §8.15). A caution never changes the result. |
| DKV-011 | For each name in `h=`, the trace records the header field instance it selected, from the bottom up, or that no instance was left for it. A name with no instance contributes nothing to the hash (RFC 6376 §3.5 and §5.4). |
| DKV-012 | If the body hash matches `bh=` but the signature doesn't verify, the trace records as a fact that the body was unchanged and the signature over the header hash failed: a signed header field, the DKIM-Signature field or the key changed after signing. It also records that DKIM keeps nothing that tells which header field changed. A hint may name the header fields that mailing lists commonly change (RFC 6377 §3.3, TRC-003). |

### DKIM signing

| ID | Requirement |
|---|---|
| DKS-001 | The library signs with `rsa-sha256` and `ed25519-sha256`, through a signer (SGN-001). It refuses RSA keys under 1024 bits. |
| DKS-002 | A caller can add more than one signature to a message, for example RSA and Ed25519 together, as RFC 8463 suggests. |
| DKS-003 | The caller chooses the canonicalizations and the header fields to sign. The library has a default header list, always signs From (RFC 6376 §5.4), and can oversign: sign one more instance of a field than the message has, so that a later added instance breaks the signature. |
| DKS-004 | The library sets `t=` from the verification time parameter. `x=` is optional. It never sets `l=`. |
| DKS-005 | Signing returns only the header fields to add, and the caller puts them in the message. The library never returns the whole signed message, because it can compute a signature only after the body has streamed past (STR-001). |

### Signers

Signing hands the data to sign to a signer and takes the signature back, as DNS questions go to the resolver ([ADR 0019](../adr/0019-signing-goes-through-a-signer-the-caller-supplies.md)).

| ID | Requirement |
|---|---|
| SGN-001 | The caller supplies the signer. A signer can keep its key in memory, in a hardware security module or in a remote key service, and it can answer at once or later. The core never holds a key that an outside signer keeps. |
| SGN-002 | The signer gets the data its algorithm signs: the SHA-256 hash for `rsa-sha256`, signed with RSASSA-PKCS1-v1_5 (RFC 6376 §3.3.2, draft-09 §3.2, RFC 8017 §8.2); the SHA-256 hash for `ed25519-sha256`, which Ed25519 takes as its message (RFC 8463 §3, draft-09 §3.3); and the whole signing input, unhashed, with the context string `DKIM2`, for `mldsa44` (draft-09 §3.4). The header field limits of SAF-002 bound the size of that input. |
| SGN-003 | Before signing, a signer states its algorithm and public key. The core refuses an RSA key under 1024 bits (RFC 8301 §3.2, draft-09 §3.2), and for DKIM2 an RSA key whose public exponent isn't 65537 (draft-09 §3.2). |
| SGN-004 | The core checks that each signature it gets back has the size that its algorithm and key give. A signer error or a signature of the wrong size stops signing with an error that names the signer, never a protocol result. |
| SGN-005 | The library ships a signer for keys held in memory, which tests and the demo use (DEM-006). Its key types follow SEC-001 to SEC-003. It signs ML-DSA-44 with the hedged variant that draft-09 §3.4 recommends, taking a random value from the caller for each signature. |

### DKIM2

| ID | Requirement |
|---|---|
| DK2-001 | DKIM2 support sits behind the Cargo feature `unstable-dkim2` until the specification becomes an RFC ([ADR 0006](../adr/0006-dkim2-targets-draft-09-behind-an-unstable-feature.md)). |
| DK2-002 | The library targets three drafts together, the latest revisions on 13 November 2026 ([ADR 0006](../adr/0006-dkim2-targets-draft-09-behind-an-unstable-feature.md)): `draft-ietf-dkim-dkim2-spec-09` (1 November 2026) for DKIM2 itself, `draft-ietf-dkim-dkim2-dns-03` (2 November 2026) for key records and the Domain Policy Record, and `draft-gondwana-dkim2-authres-02` (2 November 2026) for Authentication-Results. Moving any of them to a new revision is a deliberate change with its own tests. |
| DK2-003 | The library verifies and signs DKIM2. |
| DK2-004 | The trace covers every DKIM2 signature on the message and every change to the message that the draft records. |
| DK2-005 | The DKIM2 public API may change between minor versions while the feature is experimental. |
| DK2-006 | Verification rebuilds every earlier version of the body that the recipes describe, for all signatures in the chain, in the same single pass over the body. Draft-09 §5.1 and §5.2 require the `c` (copy) steps of a recipe to be in ascending order and not to overlap, so each earlier version can take its lines as they stream past, and the core keeps only a position in each recipe. It needs no storage and no random access. |
| DK2-007 | A recipe that matches the JSON schema but breaks the ordering rules (draft-09 §5.1 and §5.2 warn this can happen) is hostile input. Verification of the signatures that depend on it fails with a reason, and the trace names the step that broke the rule. A null body recipe, which declares that the earlier body can't be rebuilt, also appears in the result and the trace. |
| DK2-008 | Body recipe line numbers count the raw body, before §6.1 drops empty lines at the end. Recipes that hallmark writes never copy those empty lines with `c` steps, and the trace records the assumption whenever a recipe copies one ([ADR 0008](../adr/0008-dkim2-recipe-line-numbers-count-the-raw-body.md)). |
| DK2-009 | Verification implements the five algorithms that draft-09 §3 requires of every verifier: the `sha256` and `sha512` hashes, and the `rsa-sha256`, `ed25519-sha256` and `mldsa44` signatures. ML-DSA-44 is the pure variant, with the context string `DKIM2` (§3.4). Signing offers the same algorithms through a signer (SGN-001), and a caller can sign with several at once. |
| DK2-010 | Several signatures in one `s=` tag combine by the `sig-all` and `sig-any` rules of draft-09 §14.6, and several hash sets in one `h=` tag combine by the `hash-all` and `hash-any` rules of §14.7. An algorithm that hallmark doesn't implement or treats as insecure, and a key it treats as too short, give a PERMERROR intermediate result. |
| DK2-011 | DKIM2 key records follow dns-03 §4: the `rsa`, `ed25519` and `mldsa44` key types, an empty `p=` for a revoked key, and exact sizes of 32 octets for an Ed25519 key and 1,312 octets for an ML-DSA-44 key. DKIM2 ignores the deprecated `h=`, `n=`, `s=` and `t=` tags (draft-09 §14.5). DKIM keys keep following RFC 6376 (DKV-005). |
| DK2-012 | Verification checks that each From header field present since the first signature is Authentic (draft-09 §11 and §14.8). It uses the relaxed domain match of §7 in both directions. When the match succeeds only because the signing domain is a subdomain of the From domain, it fetches the From domain's Domain Policy Record (dns-03 §5), and a `strict` flag there cancels the match. A From header field with more than one address, or with a domain literal, is never Authentic. |
| DK2-013 | Results use the five output states of draft-09 §14.1: NONE, PASS, FAIL, PERMERROR and TEMPERROR. A result other than PASS carries the human-readable string that draft-09 §14 gives for its case, with the placeholders filled in, and the trace records the same string. |
| DK2-014 | A Reviser (draft-09 §2.3) describes each change it made: for the body, which lines of the new body came unchanged from the earlier version, and which earlier lines it removed or replaced, with their content; for header fields, the earlier value of each field it changed or removed. hallmark writes the Message-Instance and its recipes from that description (§5, §12.1), or a null body recipe when the caller asks for one. It never compares two versions of the body, which would need both in memory (STR-001). In the same pass, it rebuilds the earlier version from the recipes it wrote, as verification does (DK2-006), and signing stops with an error if that version's hashes differ from the ones in the highest Message-Instance. |

### DMARC

| ID | Requirement |
|---|---|
| DMC-001 | The library evaluates DMARC under RFC 9989 (DMARCbis) and under RFC 7489. The caller picks one per call, and RFC 9989 is the default ([ADR 0020](../adr/0020-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md)). |
| DMC-002 | The author domain is the domain in the RFC5322.From header field (ADS-012 covers a message with several), converted from a U-label to an A-label (RFC 5890 §2.3), as RFC 9989 §5.3.1 and RFC 7489 §6.6.1 require. DMARC applies only when exactly one domain is found. When none is found, as with a missing From field, a group with no addresses (RFC 6854) or an address with a domain literal (ADS-005), or when more than one is found, the result says that DMARC was not applied and why. A caller can choose to evaluate a message with several domains anyway, taking each one as an author domain up to a limit (SAF-002), and then the most strict policy among the checks that fail applies (RFC 9989 §11.5, RFC 7489 §6.6.1). The trace records which case applied. |
| DMC-003 | In RFC 7489 mode, the library finds the organizational domain with the Public Suffix List. In DMARCbis mode, it uses the DNS tree walk and the query limit DMARCbis sets. |
| DMC-004 | The record parser accepts the tags of both specifications: DMARCbis adds `np`, `psd` and `t` and drops `pct`, `rf` and `ri`. Unknown tags are ignored. |
| DMC-005 | Identifier alignment checks the SPF-authenticated MAIL FROM domain and each DKIM `d=` domain against the author domain, in relaxed or strict mode from `aspf` and `adkim`. Each alignment step names the SPF or DKIM result it used, so a caller that ran the checks one at a time (GEN-001) can match them. |
| DMC-006 | The result is `pass`, `fail`, `none`, `temperror` or `permerror`, together with the alignment details, the policy tag that applied (`p`, `sp`, or in RFC 9989 mode `np`) with its published value, and whether the record asks for test mode (DMC-007). hallmark computes no other policy from them. The caller decides what to do with the message ([ADR 0021](../adr/0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md)), and the how-to guide [Handle DMARC test modes](../how-to/handle-dmarc-test-modes.md) shows the steps that RFC 9989 §4.7 and RFC 7489 §6.6.4 describe. |
| DMC-007 | Test mode comes from `t=y` in RFC 9989 mode (§4.7) and from `pct=0` in RFC 7489 mode, as RFC 9989 Appendix A.6 pairs them. In RFC 7489 mode, hallmark ignores any other `pct` value and reads the record as if `pct` were absent, which means 100, because Appendix A.6 reports that other values were seldom applied accurately. The trace records an ignored `pct` value. hallmark samples nothing and takes no random value ([ADR 0021](../adr/0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md)). |
| DMC-008 | If a DMARC pass rests only on signatures from keys with `t=y` (DKV-009), that is, SPF gives no aligned pass and every aligned DKIM pass comes from such a key, the result says so. hallmark still counts those signatures, as RFC 9989 §4.4.1 counts any signature that validates, and leaves to the caller how RFC 6376 §3.6.1 applies ([ADR 0021](../adr/0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md)). |

### Authentication-Results (RFC 8601)

| ID | Requirement |
|---|---|
| AR-001 | The library generates an Authentication-Results header field from its results, with the authserv-id supplied by the caller. |
| AR-002 | Each method result carries a short explanation in its `reason` (the reasonspec of RFC 8601 §2.2) and in comments. hallmark builds it from the result's typed reason (GEN-004), not from the trace, so it is there at every trace level, off included (TRC-004). |
| AR-003 | The library parses Authentication-Results header fields, so that a caller can read results from an upstream host it trusts. |
| AR-004 | DKIM2 results use the `dkim2` method of `draft-gondwana-dkim2-authres-02` §3: one result for the whole message, the result values of §3.1, and the `header.d` and `header.i` properties of §3.2, each left out in the cases §3.2 lists. A message with no DKIM2 header fields is reported as `dkim2=none`, not left out (§3.1). |
| AR-005 | The library reports which Authentication-Results fields carry an authserv-id the caller names, so that an MTA can delete the ones that claim to come from inside its trust boundary without coming from a trusted MTA (RFC 8601 §5). In internationalized messages, it compares authserv-ids after converting A-labels to U-labels, as §5 says. hallmark deletes nothing itself. |

### Trace and reasoning

The trace is the main feature. It records every step a check takes, with the rule that required the step ([ADR 0002](../adr/0002-the-code-that-decides-writes-the-trace.md)).

| ID | Requirement |
|---|---|
| TRC-001 | The code that makes a decision also records it in the trace, while the check runs. The library never rebuilds a trace after the fact, so the trace can't disagree with the result. |
| TRC-002 | The trace is structured data: a tree of steps. Each step records what it checked, its inputs (including each DNS question and answer), its outcome, and the RFC or draft section that requires it, as codes and parameters (TRC-011). |
| TRC-003 | Facts and hints stay apart. A fact states what the check found ("the body hash didn't match `bh=`"). A fact that a specification warns about carries a caution, which cites the section with the warning ("the signature passed with a 1024-bit RSA key", RFC 8301 §3.2). A hint is a labeled guess about the cause ("a mailing list footer may have changed the body"). Cautions and hints never change a result. Facts, cautions and results cite the specifications they apply, and hints may also cite RFC 6377 and RFC 7960, which describe how mailing lists and other mediators change messages. The first release has a small set of hints, and new hints arrive with test cases that justify them. |
| TRC-004 | The caller picks a trace level per call: off, summary or full. The summary holds, for each check, the steps on the path from its top step down to the steps that decided its result, and every caution. The full trace holds every step. At the off level, the library allocates nothing for the trace. |
| TRC-005 | The trace has size limits. Hostile DNS answers or messages can't make it grow without bound, and when it reaches a limit, the trace records that it was cut short. A trace limit never changes a result: the check goes on and returns what it would have returned with no trace. |
| TRC-006 | The trace never contains private key material. It can contain addresses and header contents, and the reference docs say so, so that callers can decide where to store it. |
| TRC-007 | The trace serializes to a documented format with its own schema version. The demo, hallmark-milter and outside tools all read that format. |
| TRC-008 | The trace records every DNS question and answer as the resolver gave it, each TXT record as its separate strings (DNS-006), so a caller can save them as a snapshot and replay the check offline (DNS-004). |
| TRC-009 | Tests can assert on the path a check took through the trace as well as on its result. They match step codes (TRC-011), so a change of wording breaks no test. |
| TRC-010 | By default, the trace records the body as hashes, byte counts and line counts, plus the point where `l=` cut it. Body content goes into the trace only when the caller asks for it, and then it streams out through STR-002 as the body passes, without being kept, so trace memory stays within the limits of STR-001 and TRC-005. |
| TRC-011 | Each step carries a kind code and an outcome code from a documented list, and its parameters, such as a domain name, a tag value or a count. The text a person reads is rendered from the codes and parameters. The library renders English, and a caller can render another language, as the demo may. The human-readable strings of draft-09 §14 (DK2-013) keep a field of their own, in the words the draft gives them. |
| TRC-012 | Each step keeps its logical place in the tree, whatever order the DNS answers arrive in and wherever the body chunks are cut. An answer sits under the step whose question it answers, SPF terms follow their record, DKIM signatures follow the header section, and a step that finishes when the body ends, such as the assumption of DK2-008, stays under the step it belongs to. The same inputs give the same trace, and the same bytes once serialized. |
| TRC-013 | The trace marks each value that a sender or a DNS publisher controls: header field values, addresses, DNS record contents, the SPF explanation string (RFC 7208 §6.2), macro expansions and recipe content. The library escapes nothing, because the right escaping depends on where a value is shown. The trace format documentation tells readers, the demo included, to escape marked values for their output, and to show an SPF explanation as coming from a third party, as RFC 7208 §6.2 asks. |

What the trace shows for each protocol:

| Protocol | Steps the trace records |
|---|---|
| SPF | The evaluation tree through `include` and `redirect`, each DNS question and answer, each macro expansion and its result, the running lookup and void-lookup counts, the mechanism that matched, or the limit that was hit and the term that hit it, and for a `fail` the `exp=` lookup and the expanded explanation |
| DKIM | Tag parsing, the key lookup and key record with its flags, the header field instance chosen for each name in `h=` or the lack of one (DKV-011), the canonicalized header, the body as described in TRC-010, the computed and stated body hashes, the signature check, what a body hash match with a failed signature shows (DKV-012), and the cautions of DKV-010 |
| DKIM2 | The same steps as DKIM for each signature, plus the chain of custody, the timestamp checks, each recipe step, the relaxed domain matches and Domain Policy Record lookups of the From check, how signatures and hash sets combined, and the human-readable string that draft-09 §14 gives for the result |
| DMARC | The author domain, each step of policy discovery (Public Suffix List lookup or tree walk), alignment for each identifier with its mode and the SPF or DKIM result it came from, which of `p`, `sp` or `np` applied, test mode and the tag that set it, an ignored `pct` value, whether a pass rests only on testing-mode keys, and the final result |

### Verification time

| ID | Requirement |
|---|---|
| TIM-001 | The verification time is a parameter, and the core never reads a clock. It is the time the message was received. RFC 6376 §3.5 asks for the time the message was first received at the verifier's administrative domain, when that is reliably known, and draft-09 §14.3 measures DKIM2 timestamps against the time the message was accepted over SMTP. A caller that checks a stored message later passes the time it was received, not the current time. |
| TIM-002 | The caller can turn off checks that compare a signature time with the verification time: an expired DKIM `x=`, a DKIM `t=` in the future, and the DKIM2 checks of draft-09 §14.3, which fail a signature created more than 14 days before the message was accepted or more than one day after it. The checks are on by default. RFC 6376 §3.5 makes both DKIM checks optional for verifiers. |
| TIM-003 | Checks that don't use the verification time still run when time checks are off. For example, `x=` must still be later than `t=`. |
| TIM-004 | When a time check is off, the library still works out what it would have said and records that in the trace as a skipped step ("this signature expired on 2024-03-02; the expiration check was off"). |
| TIM-005 | An Authentication-Results header field generated with time checks off carries a comment that says so. |
| TIM-006 | The option exists on every target, native and WebAssembly, so native tests cover it. |

### DNS

| ID | Requirement |
|---|---|
| DNS-001 | The core asks for DNS records by handing questions to the caller and taking the answers back ([ADR 0018](../adr/0018-the-core-does-no-io-and-the-caller-drives-it.md)). It needs TXT, A, AAAA, MX and PTR records. |
| DNS-002 | The resolver's answer tells apart: records found, name does not exist, no records of that type, and temporary failure. SPF void lookups and temporary errors depend on that difference. |
| DNS-003 | The core never waits for an answer, so a caller can resolve with any async runtime, with blocking code, or in the browser. The library also offers a driver that runs a check to the end with a resolver the caller implements, and the driver depends on no async runtime. |
| DNS-004 | The library ships a resolver that answers from a snapshot: the questions of a check, each with the answer its resolver gave (TRC-008). Tests, bug reports and the demo's replay of a shared case (DEM-004) use it. It matches a question by its type and its name, without regard to ASCII case (RFC 4343 §2). A question the snapshot doesn't hold gets a temporary failure that says so (DNS-009). |
| DNS-005 | The DNS questions for DKIM and DKIM2 keys, for DKIM2 Domain Policy Records and for DMARC policy are available once the header fields are complete, so the caller can resolve them while the body streams. SPF needs only the envelope, so its questions are available before the message. |
| DNS-006 | Answers come in fixed shapes: each TXT record as its list of character-strings, in order and not joined (RFC 1035 §3.3 and §3.3.14), A and AAAA records as addresses, MX records as a preference and an exchange name, and PTR records as a name. The core joins TXT strings with nothing between them, as RFC 7208 §3.3, RFC 6376 §3.6.2.2, RFC 9989 §4.5 and dns-03 §3 require, so a resolver needs to know none of those rules. |
| DNS-007 | Each question has an identifier, unique within the check, and each answer names the question it answers. Within one check, and within one call that runs several checks (GEN-001), the core asks each name and type once, and every step that needs the answer reads the same one. Reusing an answer changes none of the counts that RFC 7208 §4.6.4 keeps. |
| DNS-008 | The core builds every question name itself: `<selector>._domainkey.<domain>` for DKIM and DKIM2 keys (RFC 6376 §3.6.2.1, dns-03 §3), `_domainkey.<domain>` for a DKIM2 Domain Policy Record (dns-03 §5), `_dmarc.<domain>` for a DMARC policy record (RFC 9989 §4.5) and the names of the DNS tree walk (§4.10), reverse names under `in-addr.arpa` and `ip6.arpa` for SPF `ptr` and `%{p}` (RFC 1035 §3.5, RFC 3596 §2.5), and SPF target names after macro expansion, cut from the left to 253 characters (RFC 7208 §7.3). Every name goes out in lookup form (ADS-002) and within the limits of ADS-004. |
| DNS-009 | A temporary failure can carry the resolver's own description, such as a timeout or SERVFAIL, and the trace records it as the resolver gave it. The core reads no clock, so the caller's resolver applies time limits, such as the one RFC 7208 §4.6.4 asks for, and reports a timeout as a temporary failure. |
| DNS-010 | The snapshot resolver also answers from records that a user pastes into the demo (DEM-002), one per line in the presentation format of RFC 1035 §5.1: an absolute owner name, an optional TTL and class, the type, and the data. Pasted records act as the complete zone data. A name with no pasted records and none below it doesn't exist (RFC 8020 §2, RFC 9989 §3.2.13), and a name with records of other types, or with records below it, has no records of the asked type. A line that doesn't match the format gives an error with its line and offset (PAR-001), never a repaired record. |

### Parsers

Every parser in hallmark, for header fields, addresses, tag lists, DNS records, recipe JSON and pasted records, shares the rules below ([ADR 0011](../adr/0011-header-parsers-follow-the-abnf-and-never-repair-input.md)).

| ID | Requirement |
|---|---|
| PAR-001 | Input that doesn't match its grammar gives an error, never a repaired value. The error names the rule that failed, the byte offset, and the input that the parser read, such as a header field or a DNS record. HDR-010 applies this rule to header fields. |
| PAR-002 | Each parsed value keeps the byte range it came from, so the trace can point at the text in a header field or a record that a step read. |
| PAR-003 | Parsers read numbers with overflow checks, as HDR-013 describes for tag values. A number outside the range that its grammar or its protocol allows gets that protocol's rule, such as `permerror` for an SPF CIDR length over 32 (RFC 7208 §5.6, §4.6). |
| PAR-004 | The limits of SAF-002 form one set with documented defaults, and a caller can change them per call. Each parser checks its limits as it reads. |
| PAR-005 | Parsers read input left to right without recursion, as HDR-011 says for header fields. Nesting, such as JSON depth in a recipe, counts against a depth limit (SAF-002). |

### Header parsing

hallmark parses every header field it uses with its own parsers, written from the ABNF ([ADR 0011](../adr/0011-header-parsers-follow-the-abnf-and-never-repair-input.md)), and reads no MIME structure ([ADR 0012](../adr/0012-hallmark-reads-no-mime-structure.md)). Addresses, in header fields and in the envelope, have a design of their own, which [Addresses and domain names](#addresses-and-domain-names) describes.

| ID | Requirement |
|---|---|
| HDR-001 | The header section parser splits a message's header section into header fields, each with its name, its unfolded value and its exact raw bytes. The header section ends at the first empty line, or at the end of the input when the message has no body (RFC 5322 §2.1 and §3.5). The parser never changes the raw bytes, because `simple` header canonicalization hashes them as they arrived (RFC 6376 §3.4.1). |
| HDR-002 | Field names match without regard to case, as ABNF strings do (RFC 5234 §2.3) and as RFC 6376 §6.1.3 requires when it matches `h=` against the message. |
| HDR-003 | Unfolding removes each CRLF that is followed by WSP (RFC 5322 §2.2.3). The parser also accepts the obsolete forms: folded lines made only of white space (§4.2), and white space between a field name and its colon (§4.5). |
| HDR-004 | Field values may contain UTF-8 (RFC 6532 §3.2). Field names stay printable US-ASCII without the colon (RFC 5322 §2.2). A line that isn't a header field stays a separate malformed line with its raw bytes ([ADR 0011](../adr/0011-header-parsers-follow-the-abnf-and-never-repair-input.md)). |
| HDR-005 | The same parsers serve both input forms of STR-008: a raw header section, and fields that arrive one at a time from a milter. |
| HDR-006 | One parser reads the tag=value lists of DKIM-Signature, Message-Instance and DKIM2-Signature header fields, and of DKIM keys, DKIM2 keys and Domain Policy Records, because RFC 6376 §3.2 and draft-09 §8 define the same grammar. Tag names are case-sensitive, a duplicate tag makes the whole list invalid, checks ignore an unrecognized tag, and an empty value differs from an omitted tag. In a DKIM-Signature field of a message with internationalized header fields, tag values may also hold well-formed UTF-8 (RFC 3629 §4): RFC 8616 §5 says `d=`, `i=` and `s=` should carry U-labels there, and RFC 6376 §3.2 advises against ruling out UTF-8 in tag values. DKIM2 tag values stay ASCII, as draft-09 §8 defines them. |
| HDR-007 | The decoders follow the rules of each tag. The base64 decoder ignores folding white space (RFC 6376 §2.10 and §3.5, draft-09 §2.14). The dkim-quoted-printable decoder accepts uppercase hex digits only (RFC 6376 §2.11). In internationalized messages, it also accepts well-formed UTF-8 left unencoded, as RFC 8616 §5 allows (RFC 3629 §4). DKIM `h=` holds header field names separated by colons (§3.5). In DKIM `c=`, a single name sets the header algorithm and leaves the body `simple`, and a missing `c=` means `simple/simple` (§3.5). In the DKIM2 `s=` and `h=` groups, "," binds more tightly than "\|" (draft-09 §9.3 and §10.9). |
| HDR-008 | DKIM2 `mf=` and `rt=` values decode from base64 to the RFC 5321 paths they record, angle brackets included (draft-09 §10.5 and §10.6). An `r=` value decodes from base64 to a JSON object (draft-09 §9.2, RFC 8259), parsed within the limits of SAF-002. |
| HDR-009 | The Authentication-Results parser follows RFC 8601 §2.2: the authserv-id, the optional version, `none`, and each result with its method, method version, reason and properties, with comments wherever the grammar allows CFWS. Values follow the token and quoted-string rules of RFC 2045 §5.1, with the UTF-8 that RFC 6532 adds to quoted strings, as RFC 8601 §2.2 says. AR-003 uses this parser. |
| HDR-010 | Input that doesn't match its grammar yields an error, never a repaired value. The error names the rule that failed and the byte offset, and the trace records it ([ADR 0011](../adr/0011-header-parsers-follow-the-abnf-and-never-repair-input.md)). |
| HDR-011 | Parsers read input left to right without recursion. Nested comments (RFC 5322 §3.2.2) count against a depth limit (SAF-002). |
| HDR-012 | A raw header section uses the line endings that the caller states for the message (STR-011): CRLF, or LF, as in many `.eml` files and in a browser text area. In LF mode, the parser reads each LF as a line ending, and canonicalization gets each line ending as CRLF, the form RFC 6376 §5.3 and draft-09 §17 require for signing. A CRLF already in the input stays one line ending. |
| HDR-013 | Parsers read numeric tag values with overflow checks, so a long value can't make them panic or wrap around (SAF-001). A DKIM `t=` or `x=` longer than 12 digits counts as infinite, as RFC 6376 §3.5 allows. Any other value too large to hold counts as larger than whatever the check compares it with: a DKIM `l=` of up to 76 digits (RFC 6376 §3.5) is then larger than the body ([ADR 0010](../adr/0010-a-dkim-l-tag-larger-than-the-body-gets-no-special-rule.md)), a DKIM2 `t=` (draft-09 §10.4) lies more than a day in the future (§14.3), and a DKIM2 `m=` or `i=` exceeds the limits of SAF-002. |

### Addresses and domain names

Checks read addresses from two places with different grammars: header fields, under RFC 5322, and the SMTP envelope, under RFC 5321. They compare what they find across both, so the two parsers return the same parts. The parsers follow [ADR 0011](../adr/0011-header-parsers-follow-the-abnf-and-never-repair-input.md): input that matches no grammar gives an error that names the rule and the byte offset (HDR-010).

| ID | Requirement |
|---|---|
| ADS-001 | hallmark has two address parsers, one for each grammar. The header address parser reads RFC 5322 addresses (§3.4), and the envelope address parser reads RFC 5321 paths and HELO names (§4.1.1.1, §4.1.2). Both return the same parts: a local part, and either a domain name or an address literal. The envelope address parser also reads DKIM `i=` values, whose local part follows RFC 5321, and the header address parser reads the addresses in Authentication-Results property values, whose local part follows RFC 5322 (ADS-013). Checks can then compare an envelope domain with a header domain, as DMARC alignment does (RFC 9989 §4.4), and as the DKIM2 relaxed domain match does (draft-09 §7). |
| ADS-002 | A domain name keeps the form it had in the input and also gets a lookup form, in which each U-label becomes an A-label through the lookup protocol of RFC 5891 §5: the checks of §5.4, with the code points of RFC 5892 and RFC 8753 and the bidirectional rule of RFC 5893, then Punycode (RFC 3492). Every DNS question and every comparison uses the lookup form, as RFC 8616 §3, RFC 7208 §4.3, RFC 9989 §5.3.1 and draft-09 §18 require. A domain name that fails the conversion has no lookup form, and the trace names the rule it broke. hallmark carries out the conversion itself, with tables computed for one fixed Unicode version ([ADR 0013](../adr/0013-hallmark-converts-domain-names-to-a-labels-with-its-own-rfc-5891-code.md)). |
| ADS-003 | Domain names compare in their lookup form without regard to ASCII case (RFC 5321 §2.4, RFC 4343), as RFC 9989 §4.4 also says for alignment. |
| ADS-004 | A domain name used in a DNS question has labels of 1 to 63 octets and at most 255 octets in all, in its lookup form (RFC 1035 §2.3.4). A check that meets a domain name outside these limits applies its own rule, such as `none` for SPF (RFC 7208 §4.3). A header domain may hold ASCII characters other than letters, digits and hyphens, as RFC 5322 §3.4.1 allows, and checks look it up as written, because DNS labels may hold any octets (RFC 2181 §11, [ADR 0015](../adr/0015-hallmark-looks-up-header-domains-as-written-whatever-ascii-they-hold.md)). Envelope domains keep the letters, digits and hyphens of RFC 5321 §4.1.2. |
| ADS-005 | Address literals stay apart from domain names, and no check looks them up in the DNS. The header parser returns RFC 5322 domain literals (§3.4.1), and the envelope parser returns RFC 5321 address literals: IPv4, IPv6 and general literals (§4.1.3). DKIM2 never matches a literal (draft-09 §7.2), and a From address with a domain literal gives DMARC no author domain ([ADR 0014](../adr/0014-a-from-domain-literal-gives-dmarc-no-author-domain.md)). |
| ADS-006 | A local part keeps the form it had in the input and also gets a value without its quoting: the quote characters and the backslash of each quoted pair (RFC 5322 §3.2.4, RFC 5321 §4.1.2). The value also leaves out the comments and folding white space that the obsolete local parts of RFC 5322 §4.4 allow between words. Local parts compare with their case kept (RFC 5321 §2.4). hallmark never converts a UTF-8 local part (RFC 6532 §3.2, RFC 6531 §3.3, draft-09 §18). |
| ADS-007 | The parsers accept addresses longer than the minimum sizes of RFC 5321 §4.5.3.1 (local parts of 64 octets, domains of 255 octets, paths of 256 octets), up to the limits of SAF-002, because that section asks implementations to avoid limits on these lengths where they can. |
| ADS-008 | Domain names outside addresses follow ADS-002 to ADS-004 too: DKIM `d=` and `s=`, the domain of DKIM `i=`, DKIM2 `d=` and `nd=`, the domains that SPF reaches through its mechanisms, modifiers and macros, and the DMARC policy domains. DKIM `d=` and `s=` use the labels of RFC 5321 §4.1.2 (RFC 6376 §3.5), and DKIM2 `d=` and `nd=` use its Domain rule (draft-09 §2.13), so these labels hold only letters, digits and hyphens. In a DKIM-Signature of an internationalized message, `d=`, `i=` and `s=` may hold U-labels (RFC 8616 §5, HDR-006). DNS questions use the lookup form, and the DKIM hash uses the domain as it appears in the header field (RFC 8616 §5). |
| ADS-010 | The header address parser reads each From header field as an address list (RFC 5322 §3.4 and §3.6.2), groups included (RFC 6854), with comments and folding white space (§3.2.2), quoted strings (§3.2.4), domain literals (§3.4.1), the obsolete forms of §4.4, and UTF-8 (RFC 6532 §3.2). It returns each mailbox in order, with the group it belongs to, and ignores an obsolete route, as §4.4 says. |
| ADS-011 | Display names and comments keep their raw form, encoded words included. An encoded word can't appear in an addr-spec (RFC 2047 §5), so no check needs one decoded ([ADR 0012](../adr/0012-hallmark-reads-no-mime-structure.md)). |
| ADS-012 | The From field is the only header field whose addresses a check reads. DMARC takes the author domain from it (RFC 9989 §5.3.1, DMC-002), and DKIM2 checks that it is Authentic (draft-09 §11, DK2-012). A message can carry more than one From field, although RFC 5322 §3.6 allows exactly one. The parser then reads all of them, the domains of all of them count toward DMC-002, and the trace records that the message had several From fields (RFC 9989 §11.5, RFC 7489 §6.6.1). |
| ADS-013 | Two tag values hold addresses. A DKIM `i=` value, after dkim-quoted-printable decoding, is an address whose local part may be empty and follows the RFC 5321 grammar (RFC 6376 §2.9 and §3.5), so the envelope address parser reads it. Its domain must equal the `d=` domain or be a subdomain of it (RFC 6376 §6.1.1). An Authentication-Results property value can take the form `local-part@domain`, with the local part of RFC 5322 §3.4.1 as RFC 6531 extends it (RFC 8601 §2.2), so the header address parser reads it. |
| ADS-020 | The envelope address parser reads MAIL FROM as a Reverse-path: the null path `<>`, or a Path (RFC 5321 §4.1.1.2 and §4.1.2). The caller passes the path without its Mail-parameters, which DKIM2 `mf=` also leaves out (draft-09 §10.5). The parser accepts and ignores a source route, as §4.1.2 says. |
| ADS-021 | It reads each RCPT TO value as a Forward-path, and also accepts `<Postmaster>` without a domain, matching "Postmaster" without regard to case (RFC 5321 §4.1.1.3). |
| ADS-022 | It reads the HELO or EHLO name as a domain name or an address literal (RFC 5321 §4.1.1.1). An internationalized HELO name must already be in A-labels (RFC 8616 §4), so a HELO name with U-labels is malformed. |
| ADS-023 | SPF takes its `<sender>` and `<domain>` (RFC 7208 §4.1) from these parts: the MAIL FROM mailbox, or `postmaster@` the HELO name when MAIL FROM is null (§2.4), with "postmaster" as the local part of a sender that has none (§4.3). SPF checks the HELO identity only when it is a valid multi-label domain name (§2.3). The macros `%{s}`, `%{l}` and `%{o}` expand from the same parts (§7.2 and §7.3), and a term in which `%{s}` or `%{l}` expands a non-ASCII local part matches nothing (RFC 8616 §4). `%{l}` expands the local part as written in MAIL FROM, quoting included, and `%{s}` joins that local part, `@` and the domain in its lookup form ([ADR 0016](../adr/0016-spf-macros-expand-the-local-part-as-written.md)). |
| ADS-024 | The envelope address parser reads the decoded DKIM2 `mf=` and `rt=` values (HDR-008) as a Reverse-path and Forward-paths. The chain-of-custody check compares them with MAIL FROM and RCPT TO, angle brackets included: domains in lookup form without regard to case, and local parts as written, quoting included (draft-09 §12.2, §14.4 and §18). It ignores a source route on either side (RFC 5321 §4.1.2). If two local parts differ only in quoting, the trace adds a hint that RFC 5321 §4.1.2 treats them as one mailbox ([ADR 0017](../adr/0017-dkim2-compares-local-parts-as-written-and-ignores-source-routes.md)). The relaxed domain match uses only the domain part (§7.2). |
| ADS-030 | The trace records each address a check reads: where it came from, its local part, its domain name in both forms or its literal, and the result of each conversion to an A-label. |

## Non-functional requirements

### I/O only through the caller

The core does no I/O of its own. Where data is too large to hold in memory, it streams input and output through interfaces the caller supplies ([ADR 0018](../adr/0018-the-core-does-no-io-and-the-caller-drives-it.md)).

| ID | Requirement |
|---|---|
| IO-001 | The core opens no sockets and files, reads no clock or environment variables, and draws no random numbers. DNS answers, the time and signatures come from the caller, and the core takes no random values. |
| IO-002 | Large input and output pass through streaming interfaces that the caller supplies as parameters (STR-001 to STR-011). The DNS resolver (DNS-001) and the signer (SGN-001) are the other caller-supplied interfaces. |

### Streaming

```mermaid
flowchart LR
    src["Caller's source: MTA, file or browser"] -->|"body chunks"| core["hallmark core: hash state and a bounded buffer"]
    core -->|"output chunks"| sink["Caller's sink: MTA, file or browser"]
```

| ID | Requirement |
|---|---|
| STR-001 | Signing and verification accept the message body as a stream of chunks of any size. Memory use doesn't grow with the body: the core keeps hash state and a bounded buffer, not the body. |
| STR-002 | Output that can grow large streams to the caller in chunks of bounded size. This covers the serialized trace (TRC-007) and two kinds of DKIM2 header field: Message-Instance fields, which record changes to the message, and DKIM2-Signature fields, where one ML-DSA-44 signature takes 3,228 base64 characters (draft-09 §3.4). |
| STR-003 | The caller drives the stream: it pushes input chunks and pulls output chunks. The same interface works with any async runtime, with blocking code, and in the browser, and the library never waits on I/O itself. |
| STR-004 | The library provides adapters that take a standard blocking reader or writer as a parameter, for callers that don't need async. |
| STR-005 | Streamed output is byte-for-byte the same as buffered output, whatever the chunk sizes, and tests check this with chunks from 1 byte up to the whole message. |
| STR-006 | An error from the caller's source or sink stops the operation with an error that names the stream. The library never treats a stream error as a protocol result such as `fail`. |
| STR-007 | The core keeps no reference to an input chunk after the call that received it returns. A caller that accounts for memory per command, such as a milter engine, can then release a chunk's memory as soon as the call ends. |
| STR-008 | Header fields can arrive one at a time as a name and a value, in the order of the message, as a milter receives them. The caller states the form of the value: LF or CRLF between folded lines, and whether the receiving side removed one leading space. The core rebuilds the bytes that canonicalization needs, and the trace records any assumption it made, for example "one leading space was assumed after the colon". |
| STR-009 | A generated header field comes out as a whole value in the form the caller asks for: LF or CRLF between folded lines, with or without a leading space. |
| STR-010 | The single pass covers DKIM2 chains too (DK2-006). No check needs to read the body twice. |
| STR-011 | The caller states the line endings of the message: CRLF, as a milter receives it, or LF, as in many `.eml` files and in a browser text area. The core converts bare LF to CRLF as the body streams, including a line ending split across two chunks (draft-09 §17, RFC 6376 §5.3). HDR-012 applies the same rule to the header section. |

### WebAssembly

| ID | Requirement |
|---|---|
| WSM-001 | The core builds for `wasm32-unknown-unknown`, and CI checks the build on every change. |
| WSM-002 | The core depends on no browser crates such as `wasm-bindgen`. Browser code lives in `hallmark-wasm`. |
| WSM-003 | Cryptography uses pure Rust crates that build for WebAssembly, ML-DSA-44 included (DK2-009). |

### Hostile input

| ID | Requirement |
|---|---|
| SAF-001 | No input makes the library panic: not a message, a DNS answer, or a record ([ADR 0004](../adr/0004-no-input-makes-hallmark-panic.md)). |
| SAF-002 | Limits with defaults cover the header field count and size, the nesting depth of comments, the mailboxes in one address list, the length of an address, the entries in one `rt=` tag, the signatures checked per message, the author domains evaluated per message (DMC-002), the DNS questions per check, the key size, recursion depth, and for DKIM2 the number of Message-Instances, the recipe steps, and the size and nesting depth of recipe JSON. Draft-09 §20 sets no limits on recipes, and §14.2 lets a verifier refuse recipes that are too complex. Hitting a limit gives the protocol's error result and a trace step. |
| SAF-003 | Every parser has a fuzz target: the header section, the tag=value list and its value decoders for DKIM and DKIM2 fields and key records, header addresses, envelope paths and HELO names, conversion to A-labels, DKIM2 recipe JSON, the SPF record, the DMARC record, Authentication-Results, and canonicalization. |
| SAF-004 | Memory use stays bounded by the input size and the configured limits. |

### Private keys

| ID | Requirement |
|---|---|
| SEC-001 | Private key types neither print nor format their contents, and memory holding a key is zeroed when the key is dropped. |
| SEC-002 | Private keys never appear in traces, errors or logs. |
| SEC-003 | Signing must not leak the private key through timing. If a chosen crate has a known timing advisory, an ADR records the decision and its limits. |

### Stability and licensing

| ID | Requirement |
|---|---|
| STB-001 | The crate follows semantic versioning. The trace schema version (TRC-007) changes separately from the crate version. |
| STB-002 | hallmark is MIT-licensed. Dependencies must have compatible licenses, and CI checks them. |
| STB-003 | The core has a stated minimum supported Rust version, and CI tests it. |

## Repository and deliverables

The hallmark repository is a Cargo workspace ([ADR 0005](../adr/0005-the-demo-lives-in-the-library-workspace.md)):

| Path | Contents | Published |
|---|---|---|
| `crates/hallmark` | The core library | crates.io |
| `crates/hallmark-wasm` | `wasm-bindgen` bindings and a DNS-over-HTTPS resolver for the browser | not at first |
| `demo/` | The static demo site | GitHub Pages |

| ID | Requirement |
|---|---|
| REP-001 | The core crate's package leaves out `demo/`. |
| REP-002 | A GitHub Actions workflow builds the demo and publishes it to GitHub Pages, without a `gh-pages` branch. |
| REP-003 | The demo stays small: plain TypeScript or JavaScript with few npm dependencies. If it grows into a large front-end app with its own contributors, it moves to its own repository. |

## Browser demo

| ID | Requirement |
|---|---|
| DEM-001 | A user pastes a message, enters the client IP address, HELO name, MAIL FROM and RCPT TO, and gets the SPF, DKIM, DKIM2 and DMARC results with the trace drawn as a tree. |
| DEM-002 | The user picks the DNS source: DNS over HTTPS, or records pasted into the page (DNS-010). |
| DEM-003 | The page names the DNS-over-HTTPS provider before any query goes out. Only those queries leave the browser, and the message itself never does. |
| DEM-004 | A user can save a case (message, DNS snapshot, time and options) and share it. Anyone can replay a shared case offline. |
| DEM-005 | The user can turn off time checks (TIM-002), so that old messages can still be explained. |
| DEM-006 | The user can sign a message with a key generated in the page or pasted into it, through the library's signer for keys in memory (SGN-005). The key never leaves the browser, and the page tells users not to paste production keys. |
| DEM-007 | CI runs browser tests against the demo on every change, so a library change that breaks the demo fails the same pull request. |

## Quality and process

| ID | Requirement |
|---|---|
| TST-001 | Every requirement and every test comes from the RFC or draft text and cites the section it checks. |
| TST-002 | The project builds only from the specifications. It does not read, copy or compare its behavior against other sender-authentication implementations, and its documents do not cite them ([ADR 0009](../adr/0009-requirements-and-tests-come-only-from-the-specifications.md)). |
| TST-003 | Published conformance test data, such as the openspf SPF test suite, may be used after a license check. The docs record each source and its license. |
| TST-004 | Test vectors from the RFCs themselves (RFC 6376, RFC 8463) are part of the test suite. |
| TST-005 | Tests use domain names under `.invalid` and answer DNS from snapshots. Tests make no network calls. |
| TST-006 | Fuzz targets (SAF-003) run in CI for a bounded time on every change, and for longer on a schedule. |
| TST-007 | Code in this repository uses only short, well-established regular expressions, such as one that matches a run of digits, because people find long or complex ones hard to read. hallmark parses structured input, such as header fields, addresses, tag lists and DNS records, by hand from the ABNF ([ADR 0011](../adr/0011-header-parsers-follow-the-abnf-and-never-repair-input.md)). |

Process:

- Documentation follows Diátaxis, with one type per document.
- Each document with two or more sections opens with a contents list. ADRs keep the form they were accepted in.
- Architecture decisions are ADRs in [docs/adr](../adr/README.md).
- Test scenarios are Given/When/Then tables in [docs/testing](../testing/README.md), written before their tests, with IDs that link to the requirement IDs here.
- Code follows Canon TDD, the procedural Rust style ([ADR 0003](../adr/0003-rust-code-is-written-in-a-procedural-style.md)), and the panic rule ([ADR 0004](../adr/0004-no-input-makes-hallmark-panic.md)).
- Diagrams are Mermaid.

## Relationship to hallmark-milter

hallmark-milter depends on the published hallmark crate, never on its internals. It supplies a native resolver, the time each message was received (TIM-001) and a signer for its keys (SGN-001). The milter signs with DKIM and DKIM2 and verifies SPF, DKIM, DKIM2 and DMARC. It deletes the Authentication-Results fields that carry its own authserv-id but didn't come from a trusted MTA, as RFC 8601 §5 requires (AR-005).

hallmark-milter does not offer the switch that turns off time checks (TIM-002), because skipping expiration on live mail weakens protection against replayed signatures. Full traces are off by default because they hold message data (TRC-006), and an operator can turn them on. The `reason` of each Authentication-Results result comes from the typed result (AR-002), so it is there with traces off.

The milter feeds hallmark from rust-milter's callbacks, which match STR-001, STR-003 and STR-007: one body chunk of at most 65,535 bytes per `SMFIC_BODY` call, with CRLF line endings, and memory counted until each command ends. Two limits on the milter side shape its design:

- Header fields arrive with LF between folded lines and with one leading space removed, unless the milter negotiates `SMFIP_HDR_LEADSPC`. Without that flag, the milter can't tell `From: x` from `From:x`, so a `simple` header canonicalization can't always be rebuilt (STR-008). rust-milter doesn't let a handler request that flag yet ([rust-milter#61](https://github.com/photom/rust-milter/issues/61)).
- An added header field goes out whole in one reply packet. Neither Sendmail nor Postfix limits the size of a milter's reply, and Postfix stores an added header field without cutting it. The limits apply where the mail is received: a Postfix with milters cuts each incoming header field to 60,000 bytes, and Sendmail limits all incoming header fields of a message together to `MaxHeadersLength`, 32,768 bytes by default. So a very large DKIM2 header field leaves this host whole, and the next hop cuts it (Postfix) or rejects the message (Sendmail). A cut field fails verification. Those limits belong to the receiving MTA's configuration. hallmark-milter can only check the length of each field before it adds it and decide what to do with one the next hop is likely to cut or reject.

## Open items

| Item | To decide |
|---|---|
| Public Suffix List | Whether the caller supplies the list in RFC 7489 mode or the crate bundles a copy |
| openspf suite | Its license (TST-003) |
| MSRV | Which Rust version (STB-003) |
| Trace format | The serialization format and its first schema (TRC-007). Snapshots (DNS-004) use the same format |
| Last recipe line without CRLF | Under DK2-008, a last body line with no CRLF counts as a line. Whether a `c` step that copies it adds a CRLF ([hallmark#1](https://github.com/photom/hallmark/issues/1)) |
| Bare LF in CRLF mode | Whether a verifier whose caller declared CRLF line endings leaves a bare LF in the body as it is or converts it (STR-011, scenario CAN-063) |
| First hints | The starting set of hints (TRC-003) |
| Testing-mode DKIM keys in DMARC | RFC 6376 §3.6.1 tells verifiers to treat mail from a testing-mode key like unsigned mail, and RFC 9989 §4.4.1 counts any signature that validates. Neither says how the two combine. [ADR 0021](../adr/0021-test-modes-are-reported-and-the-caller-decides-what-they-mean.md) counts the signature and reports it (DMC-008) until a specification says otherwise |
| DK2-008 and the working group | Whether to ask the ietf-dkim working group about the reading of [ADR 0008](../adr/0008-dkim2-recipe-line-numbers-count-the-raw-body.md), as [hallmark#1](https://github.com/photom/hallmark/issues/1) asks about the last line |
| DKIM2 local-part comparison | Draft-09 §14.4 says local parts must match exactly, and RFC 5321 §4.1.2 says all quoted forms of a local part are equivalent when compared. [ADR 0017](../adr/0017-dkim2-compares-local-parts-as-written-and-ignores-source-routes.md) compares them as written until a revision of the draft says whether `<"a"@void.invalid>` matches `<a@void.invalid>` (ADS-024, [hallmark#4](https://github.com/photom/hallmark/issues/4)) |
| Demo "as of signing time" | Whether the demo should offer to verify as of the signature's `t=`. The library needs no change for it, because the time is already a parameter. |
