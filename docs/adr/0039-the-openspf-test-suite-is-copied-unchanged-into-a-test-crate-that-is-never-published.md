# ADR 0039: The openspf test suite is copied unchanged into a test crate that is never published

- Status: Accepted
- Date: 2026-11-16
- Affects: SPF tests, the repository layout, the license of the hallmark package, and the domain names in tests

## Context

hallmark may use published conformance test data after a license check (TST-002). For SPF, the openspf test suite holds 203 cases for RFC 7208 in one YAML file, `rfc7208-tests.yml`. Each case gives a client address, a HELO name, a MAIL FROM, the result it expects and the section of RFC 7208 it checks. Six cases accept either of two results. The file also holds the DNS records that the cases read.

The suite's main author, Stuart D Gathman, keeps it in the pyspf repository. Its header calls it "the openspf.org test suite (release 2014.04)", and its change log lists the cases added in 2019 under a release 2019.08 marked unreleased. The file last changed in commit `4bf96ea` on 2019-08-31.

The suite has a license file of its own, `rfc7208-tests.LICENSE`, apart from the default license of the openspf.org website. It is the 3-clause BSD license, with the copyright held by Stuart D Gathman, Julian Mehnle and Scott Kitterman. It lets anyone copy and change the suite on three conditions. Copies of the source keep the copyright notice, the conditions and the disclaimer. Binary copies reproduce them in their documentation. Nobody uses the authors' names to endorse a product made from the suite without their written permission. hallmark is MIT-licensed (STB-002). `cargo package` puts the files under a crate's directory into its package, apart from those that the manifest or `.gitignore` leaves out.

The suite names domains outside `.invalid`: `example.com`, `example.net` and `example.org`, and registrable names such as `example1.com` and `pair.com`. It also uses addresses outside the documentation ranges, such as 1.2.3.4. Its expected results depend on those names, because macros expand them into query names and explanation strings. TST-004 puts the domain names of tests under `.invalid`.

The file uses YAML anchors, folded strings and inline maps, so a test needs a full YAML parser to read it.

## Options considered

- (a) Leave the suite out, and write every SPF case from RFC 7208.
- (b) Copy the suite into the tests of the `hallmark` crate.
- (c) Copy the suite into a crate that is never published, and rename its domains under `.invalid`.
- (d) Copy the suite unchanged into a crate that is never published.

## Decision

Option (d).

- hallmark copies two files from the pyspf repository at commit `4bf96ea`, byte for byte: `test/rfc7208-tests.yml` and `test/rfc7208-tests.LICENSE`. It takes nothing else from that repository.
- The files live in `crates/hallmark-conformance`, a workspace crate with `publish = false`, next to the tests that run them. None of the suite's files enter the `hallmark` package, so its license doesn't change.
- The suite keeps its domain names and addresses. Its tests answer DNS from the suite's own records and make no network calls. The `.invalid` rule of TST-004 covers hallmark's own tests.
- [docs/testing/README.md](../testing/README.md#third-party-test-data) records the source, the commit, the files and the license. A newer copy comes in only through a change that updates that record.
- RFC 7208 and its errata stay the source of each expected result (TST-001). A case passes when hallmark gives the expected result, or either result where the case accepts two. If RFC 7208 or its errata contradict a case's expected result, the test skips that case by name, and a scenario records the reason and the section.
- The files come into the repository with the first test of the SPF check. That change also decides how the test reads YAML: through a YAML parser as a dev-dependency of `hallmark-conformance`, or through a conversion done once.

Option (a) gives up 203 cases that people outside the project wrote against RFC 7208, each tied to a section. Option (b) puts BSD-licensed files into a package published as MIT, and leaving them out of the package breaks the tests for anyone who runs them from it. Option (c) changes the query names and explanation strings that the cases expect, risks mistakes in 79 KB of data, and makes each upstream change harder to compare.

## Consequences

Positive:
- The SPF check gets 203 cases written outside the project, each tied to a section of RFC 7208.
- The `hallmark` package holds none of the BSD-licensed files.
- A later upstream version compares byte for byte with the copy.

Negative:
- The workspace gains a crate that holds only tests and their data.
- The repository holds BSD-licensed data. Every copy of it keeps the notice, and hallmark's documents never use the authors' names to promote hallmark.
- The repository holds test domains outside `.invalid`, some of them registrable.
- Each case that contradicts RFC 7208 needs a recorded reason.

## References

- [RFC 7208](https://www.rfc-editor.org/rfc/rfc7208.html)
- [requirements.md: Quality and process](../reference/requirements.md#quality-and-process)
- [docs/testing/README.md: Third-party test data](../testing/README.md#third-party-test-data)
- [The suite's license file at commit 4bf96ea](https://github.com/sdgathman/pyspf/blob/4bf96ea63af4999663809bae9b5530bce25e6f16/test/rfc7208-tests.LICENSE)
