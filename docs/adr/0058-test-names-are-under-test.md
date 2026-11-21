# ADR 0058: Test names are under `.test`

- Status: Accepted
- Date: 2026-11-21
- Affects: TST-004, the test scenarios, `SECURITY.md`, and the issue and pull request templates

## Context

hallmark's tests make no network calls and answer DNS from snapshots (TST-004). They need names that no one can register and that can't reach a real domain, and any reserved name meets that.

RFC 2606 §2 reserves four top-level domains. It recommends `.test` "for use in testing of current or new DNS related code", and `.invalid` "for use in online construction of domain names that are sure to be invalid and which it is obvious at a glance are invalid". hallmark is DNS-related code, and its tests don't build names that are meant to be invalid.

rust-milter's interoperability stack answers DNS for its test names from a local Unbound, under `void.test` ([rust-milter testing ADR 0007](https://github.com/photom/rust-milter/blob/main/docs/testing/adr/0007-test-names-are-in-void-test-and-resolve-with-a-local-unbound.md)). RFC 6761 §6.2 items 5 and 6 let an administrator configure an authoritative server to give positive answers for `.test` names, and RFC 6761 §6.4 item 6 says that the data for `.invalid` names "cannot be modified by local configuration". With the same names in hallmark, a message, its records and its snapshot can move between hallmark's scenarios and a stack that resolves them.

## Options considered

- (a) Names under `.test`, with `void.test` as the usual domain.
- (b) Names under `.invalid`.
- (c) The example domains, such as `example.com`. RFC 6761 §6.5 reserves them for documentation, and asks caching DNS servers to resolve them normally.

## Decision

Option (a).

- hallmark's own tests, scenarios and examples use names under `.test`, such as `void.test`. The tests make no network calls and answer DNS from snapshots (TST-004).
- Published conformance data keeps its own names ([ADR 0039](0039-the-openspf-test-suite-is-copied-unchanged-into-a-test-crate-that-is-never-published.md)). The rule of TST-004 that ADR 0039 refers to is the rule of this record.

## Consequences

Positive:
- The names follow the use that RFC 2606 §2 recommends for testing DNS-related code.
- A stack that answers DNS from local data can resolve the names of a scenario, which RFC 6761 §6.4 rules out for `.invalid`.
- hallmark and rust-milter share one test domain.

Negative:
- A `.test` name isn't invalid at a glance. A network whose servers answer for `.test` can resolve it (RFC 6761 §6.2 item 1), so a caller that copies a name from a scenario into a live setup may get an answer. hallmark's own tests make no network calls.
- A scenario whose expected value depends on the bytes of a name, such as a hash, a length or a base64 value, has to be worked out for the `.test` name.

## References

- [RFC 2606 §2](https://www.rfc-editor.org/rfc/rfc2606.html#section-2)
- [RFC 6761 §6.2, §6.4 and §6.5](https://www.rfc-editor.org/rfc/rfc6761.html#section-6.2)
- [ADR 0039](0039-the-openspf-test-suite-is-copied-unchanged-into-a-test-crate-that-is-never-published.md), the conformance data and its names
- [rust-milter testing ADR 0007](https://github.com/photom/rust-milter/blob/main/docs/testing/adr/0007-test-names-are-in-void-test-and-resolve-with-a-local-unbound.md)
- [requirements.md: Quality and process](../reference/requirements.md#quality-and-process)
