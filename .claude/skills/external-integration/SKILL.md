---
name: external-integration
description: Rules for code that talks to DNS for hallmark, such as the DNS over HTTPS resolver in `hallmark-wasm`, the snapshot resolver, and resolvers that callers write. Use when writing or reviewing a resolver, its retries and caching, or how it checks and reports answers.
---

# External system integration

The hallmark core reaches nothing outside itself. It hands DNS questions to a resolver that the caller supplies and takes the answers back (DNS-001, ADR 0001). The outside system is DNS, reached through a resolver: the native one that hallmark-milter supplies, the DNS over HTTPS resolver in `hallmark-wasm`, or the snapshot resolver that tests and the demo use (DNS-004). Apply these rules when you write or review a resolver.

## 1. The resolver boundary

- Keep DNS behind the resolver interface. The core sees questions and answers, and no socket, HTTP status or server format reaches it.
- Map every answer to one of the four outcomes that DNS-002 names: records found, name does not exist, no records of that type, and temporary failure. SPF void lookups and temporary errors depend on the difference.
- hallmark keeps no store of its own. A resolver that caches answers drops each record when its TTL runs out (RFC 1035 §3.2.1), and drops a negative answer when the TTL taken from the SOA record runs out (RFC 2308 §5).

## 2. Trust and validation

- Never trust an answer. A DNS server, a DoH server, or a record pasted into the demo can return malformed or hostile data at any time.
- Check that the question section of a response matches the question you asked before you hand the answer to the core (RFC 1035 §7.3).
- The trace records every DNS question and answer (TRC-008). A resolver that also logs them treats the names as message data, because they come from addresses in the message (TRC-006). Keep message content and private keys out of every log.

## 3. Retries and time limits

- Retry on connection errors, timeouts, and HTTP `5xx` status codes.
- Use exponential backoff with jitter ("full jitter", for example), starting from a short interval. Jitter keeps many clients from retrying in lockstep.
- Treat `429 Too Many Requests` (RFC 6585 §4) and `503` as retryable. If the response carries a `Retry-After` header (RFC 9110 §10.2.3), wait that long in place of the computed backoff.
- Stop retrying before the time limit the caller sets, and report a temporary failure. RFC 7208 §4.6.4 asks for a limit on the time an SPF check takes, of at least 20 seconds, with `temperror` when the check runs past it.

## 4. DNS over HTTPS

- A DoH server sends every valid DNS response with a `2xx` status, SERVFAIL and NXDOMAIN included, so read the outcome from the DNS response code (RFC 8484 §4.2.1).
- A response with any other status carries no DNS reply (RFC 8484 §4.2.1). Retry it as section 3 says, or report a temporary failure.

## 5. Failure tests

Write unit tests for the unlikely cases: malformed DNS messages, empty answers, an answer to a different question, SERVFAIL and NXDOMAIN inside a `200` response, `403`, and `429` both with and without `Retry-After`. Tests make no network calls and use recorded responses.

## Checklist

- [ ] Map every answer to one of the four outcomes of DNS-002.
- [ ] Keep cached answers no longer than their TTL.
- [ ] Retry 5xx, 429, and timeout errors with exponential backoff and jitter, honor `Retry-After`, and stop within the caller's time limit.
- [ ] Check that each response answers the question you asked.
- [ ] Keep message content and private keys out of logs.
- [ ] Unit-test malformed and unexpected responses.

## Sources

- RFC 1035 §3.2.1 and §7.3, RFC 2308 §5: caching answers and matching them to questions.
- RFC 7208 §4.6.4: the time limit on an SPF check.
- RFC 8484 §4.2.1: DNS and HTTP errors in DNS over HTTPS.
- RFC 9110 §10.2.3 and RFC 6585 §4: `Retry-After` and `429`.
- AWS Architecture Blog, "Exponential Backoff and Jitter": retry strategy.
- Microsoft Azure design patterns, Anti-Corruption Layer: keeping an outside format out of the internal domain.

## Related skills

- design-master: architectural boundaries such as Anti-Corruption Layers.
- canon-tdd: testing resolvers.
- rust-master: async rules for the driver and `hallmark-wasm`.
- docs-first: diagram the external connections before you build them.
