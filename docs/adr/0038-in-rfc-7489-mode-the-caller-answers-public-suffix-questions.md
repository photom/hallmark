# ADR 0038: In RFC 7489 mode, the caller answers public suffix questions

- Status: Accepted
- Date: 2026-11-15
- Affects: DMARC in RFC 7489 mode, the check's questions and answers, the trace, snapshots, the driver, the demo and hallmark-milter

## Context

In RFC 7489 mode, DMARC finds the organizational domain of a domain with a public suffix list (RFC 7489 §3.2). Step 1 says to "Acquire a 'public suffix' list" and names none. Step 3 finds the list entry that matches the most labels of the domain and calls that number x, and step 4 puts the next label in front of the entry. §3.2 adds that "No list is guaranteed to be accurate or current". Appendix A.6.1 says that a list "can be obtained from various sources", that the Mozilla list is the most common, and that receivers who use different lists may not interoperate. Policy discovery queries the organizational domain when the author domain has no record (§6.6.3), and relaxed alignment compares organizational domains (§3.1.1 and §3.1.2). ADR 0019 left open where the list comes from.

The Mozilla list is published under the Mozilla Public License 2.0. publicsuffix.org updates its copy daily, says the list usually changes a few times a week, and asks apps that download it to do so no more than once a day. The list format, with its wildcards, exceptions and sections, comes from publicsuffix.org, not from an RFC.

Two Rust crates serve the list. `psl` builds the list into 2.6 MB of generated code, and its metadata says MIT or Apache-2.0 although it embeds the MPL-2.0 list. `publicsuffix` parses a list that its caller supplies. hallmark-milter runs for years, so it needs to update the list while it runs. Every crate it depends on adds to its supply chain.

## Options considered

- (a) The crate bundles a list.
- (b) The caller passes the list text, and hallmark parses it.
- (c) The check asks the caller for the public suffix of each domain, as it asks DNS questions ([ADR 0017](0017-the-core-does-no-io-and-the-caller-drives-it.md)).

## Decision

Option (c).

- In RFC 7489 mode, the check asks a public suffix question for each distinct domain whose organizational domain it may need: the author domain, the MAIL FROM domain and each DKIM `d=` domain. A question is available as soon as its domain is known, as DNS questions are. RFC 9989 mode asks none.
- The caller answers from the list of its choice. The answer gives the public suffix by its number of labels, the x of §3.2 step 3, from 1 to the number of labels in the domain. It can carry a name for the list, such as its date or a hash. The caller can instead answer that no suffix is available.
- hallmark applies step 4: the organizational domain is the domain's last x + 1 labels. If x equals the number of labels, the domain is its own organizational domain, and policy discovery makes no second query, since §6.6.3 step 3 queries the organizational domain only "if different".
- An answer outside that range gives an error to the caller, never a protocol result. An answer that no suffix is available gives `temperror` if the result depends on that organizational domain, as a temporary DNS failure does.
- The trace records each question, its answer, the list's name and the organizational domain. A snapshot keeps the answers next to the DNS answers, so a check replays without the list.
- hallmark ships no list, parses no list format, and depends on no list crate.

Option (a) has hallmark choose the list, which RFC 7489 leaves to the receiver. A list built into a release goes stale, and a milter could update it only with a new hallmark release and a rebuild. It also adds the list's size to the core and to the WebAssembly build, and brings MPL-2.0 data into an MIT crate. Option (b) has hallmark implement a list format that no RFC defines, against TST-001, and a replay would need the whole list.

## Consequences

Positive:
- The caller chooses the list and updates it when it likes, a running milter included.
- The core holds no list and no third-party list code, which keeps its size and its supply chain small.
- The trace shows which list answered each question, and a snapshot replays the check without the list.

Negative:
- A caller that uses RFC 7489 mode has to keep a list and answer the questions. hallmark-milter parses the list with its own code, and the demo uses a copy that its build bundles.
- Callers with different lists can get different results, as RFC 7489 Appendix A.6.1 warns. The list's name in the trace shows which list each used.
- Checks, traces and snapshots carry one more kind of question and answer.

## References

- [RFC 7489 §3.1.1](https://www.rfc-editor.org/rfc/rfc7489.html#section-3.1.1), [§3.1.2](https://www.rfc-editor.org/rfc/rfc7489.html#section-3.1.2), [§3.2](https://www.rfc-editor.org/rfc/rfc7489.html#section-3.2), [§6.6.3](https://www.rfc-editor.org/rfc/rfc7489.html#section-6.6.3) and [Appendix A.6.1](https://www.rfc-editor.org/rfc/rfc7489.html#appendix-A.6.1)
- [requirements.md: DMARC](../reference/requirements.md#dmarc)
- [Public Suffix List: use of the list](https://publicsuffix.org/list/)
- [ADR 0017](0017-the-core-does-no-io-and-the-caller-drives-it.md), [ADR 0019](0019-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md), [ADR 0035](0035-dns-snapshots-and-demo-cases-are-documents-in-the-trace-formats-conventions.md)
