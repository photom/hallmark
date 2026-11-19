# ADR 0055: The IDNA tables use Unicode 12.0.0

- Status: Accepted
- Date: 2026-11-19
- Affects: ADS-002, the `idna` module, `crates/hallmark-tables`, scenario DOM-014, and the license check of ADR 0045

## Context

[ADR 0012](0012-hallmark-converts-domain-names-to-a-labels-with-its-own-rfc-5891-code.md) computes the IDNA code point categories for one fixed Unicode version, by the rules of RFC 5892 §2 and §3 as RFC 8753 updates them, and asks the documents to name that version.

RFC 8753 has the IETF review each new Unicode version for IDNA before its derived property values are published. IANA keeps a table of those values for each version it has: the newest is 12.0.0, from RFC 9233, and the registry page was last updated on 2024-04-26. Scenario DOM-014 compares the generated tables with that registry.

RFC 5891 §5.4 has lookup reject a label that holds a code point unassigned in the Unicode version the application uses.

## Options considered

- (a) Unicode 12.0.0.
- (b) The newest Unicode version, with no IETF review and no IANA table.

## Decision

Option (a).

- The generator in `crates/hallmark-tables` reads the Unicode Character Database files of version 12.0.0.
- The generated tables are compared with the IANA table for 12.0.0 (DOM-014), which goes into `crates/hallmark-conformance` as ADR 0045 says.
- The trace of each conversion records the version, as ADR 0012 requires.
- Moving to another version needs a new ADR, after the IETF has reviewed that version.

Option (b) gives newer characters a lookup form, but leaves the tables without a published reference to test against.

## Consequences

Positive:
- The tables follow a version that the IETF reviewed, and DOM-014 tests them against published data.

Negative:
- A domain name with a character added after Unicode 12.0.0 gets no lookup form, as RFC 5891 §5.4 requires for unassigned code points.
- ADR 0045 names the license of the Unicode data as `Unicode-3.0`. The license that covers the 12.0.0 files has to be checked before they enter `crates/hallmark-tables`. If it isn't `Unicode-3.0`, a new ADR replaces ADR 0045.

## References

- [ADR 0012](0012-hallmark-converts-domain-names-to-a-labels-with-its-own-rfc-5891-code.md), hallmark's own RFC 5891 code
- [ADR 0045](0045-the-hallmark-package-is-licensed-mit-and-unicode-3-0-and-its-source-data-stays-outside-it.md), the package license and the place of the source data
- [requirements.md: Addresses and domain names](../reference/requirements.md#addresses-and-domain-names)
- [IANA: IDNA Rules and Derived Property Values](https://www.iana.org/assignments/idna-tables-12.0.0/idna-tables-12.0.0.xhtml)
