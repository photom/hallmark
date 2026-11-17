# ADR 0045: The hallmark package is licensed MIT AND Unicode-3.0, and its source data stays outside it

- Status: Accepted
- Date: 2026-11-16
- Affects: the license of the `hallmark` package, the repository layout, STB-002, and the IDNA tables of ADR 0012

## Context

hallmark converts domain names to A-labels with its own code (ADR 0012). The checks of RFC 5891 §5.4 need properties of each code point: whether RFC 5892 allows it, its normalization data for the NFC check, its bidirectional class for RFC 5893, and its joining type and script for the contextual rules. ADR 0012 has a generator read the Unicode Character Database (UCD) files of one Unicode version and write tables, which the `hallmark` crate holds and compiles in. A test compares the tables with the IANA registry of IDNA derived property values for that version (DOM-014).

The Unicode Consortium publishes the UCD under the Unicode License v3, whose SPDX identifier is `Unicode-3.0`. It lets anyone copy, change and distribute the data files, as long as its copyright and permission notice goes with every copy or with the documentation. Tables computed from the files are changed copies of them. `deny.toml` already allows `Unicode-3.0` among the licenses compatible with MIT.

hallmark is MIT-licensed (STB-002), and `cargo package` puts the files under a crate's directory into its package, apart from those that the manifest or `.gitignore` leaves out. [ADR 0039](0039-the-openspf-test-suite-is-copied-unchanged-into-a-test-crate-that-is-never-published.md) keeps the BSD-licensed openspf suite out of the package by putting it in `crates/hallmark-conformance`, a crate that is never published.

The tables are the only Unicode-derived files that the library needs at compile time. The UCD files and the generator serve only to make the tables, and the IANA registry serves only the test.

## Options considered

- (a) The package declares `MIT AND Unicode-3.0` and carries the Unicode notice. The UCD files and the generator live in a crate that is never published, and the IANA registry goes into `crates/hallmark-conformance`.
- (b) The package keeps `MIT` and only adds the Unicode notice as a file.
- (c) The repository holds no tables, and a build script generates them from the UCD files at build time.

## Decision

Option (a).

- The `license` of `crates/hallmark` becomes `MIT AND Unicode-3.0` in the change that brings the generated tables, and the package then carries the Unicode License v3 with its copyright notice next to the tables. Until then, the license stays `MIT`.
- The generator and the UCD files it reads live in `crates/hallmark-tables`, a workspace crate with `publish = false`. The crate records the Unicode version and where the files came from, and the files keep the Unicode notice.
- The IANA registry that DOM-014 compares against is published data. It goes into `crates/hallmark-conformance` after a license check, unchanged, and `docs/testing/README.md` records its source, version and terms (TST-002).
- STB-002 states the package license: MIT, with the generated Unicode tables under Unicode-3.0.

Option (b) makes the package metadata say less than the package holds, so a caller's license check, cargo-deny included, would miss the Unicode data. Option (c) still puts the UCD files into the package, because the build script needs them, and adds a generation step to every build.

## Consequences

Positive:
- The package metadata names every license in the package, and callers' license checks see it.
- The package holds the tables and no source data, so it stays small.
- The test against the IANA registry runs in the repository, and the registry never enters a published package.

Negative:
- Callers who accept only MIT have to accept `Unicode-3.0` too.
- The workspace gains another crate that is never published.

## References

- [ADR 0012](0012-hallmark-converts-domain-names-to-a-labels-with-its-own-rfc-5891-code.md), the IDNA tables
- [ADR 0039](0039-the-openspf-test-suite-is-copied-unchanged-into-a-test-crate-that-is-never-published.md), data outside the package
- [RFC 5891 §5.4](https://www.rfc-editor.org/rfc/rfc5891.html#section-5.4), [RFC 5892](https://www.rfc-editor.org/rfc/rfc5892.html) and [RFC 5893](https://www.rfc-editor.org/rfc/rfc5893.html)
- [Unicode License v3](https://www.unicode.org/license.txt)
- [requirements.md: Stability and licensing](../reference/requirements.md#stability-and-licensing)
