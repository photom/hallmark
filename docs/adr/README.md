# Architecture decision records

Decisions that shape hallmark's design and specification, with their context and consequences. These records sit outside the Diátaxis structure of the rest of `docs/`: each one is a dated record of a decision, kept as it was made.

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-the-core-does-no-io-and-the-caller-drives-it.md) | The core does no I/O, and the caller drives it | Superseded by ADR 0017 |
| [0002](0002-the-code-that-decides-writes-the-trace.md) | The code that decides writes the trace | Superseded by ADR 0031 |
| [0003](0003-rust-code-is-written-in-a-procedural-style.md) | Rust code is written in a procedural style | Accepted |
| [0004](0004-no-input-makes-hallmark-panic.md) | No input makes hallmark panic | Accepted |
| [0005](0005-the-demo-lives-in-the-library-workspace.md) | The demo lives in the library's workspace | Accepted |
| [0006](0006-dkim2-targets-draft-09-behind-an-unstable-feature.md) | DKIM2 targets draft-09 behind an unstable feature | Accepted |
| [0007](0007-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md) | DMARC supports RFC 7489 and DMARCbis, with DMARCbis the default | Superseded by ADR 0019 |
| [0008](0008-dkim2-recipe-line-numbers-count-the-raw-body.md) | DKIM2 recipe line numbers count the raw body | Accepted |
| [0009](0009-a-dkim-l-tag-larger-than-the-body-gets-no-special-rule.md) | A DKIM `l=` larger than the body gets no special rule | Accepted |
| [0010](0010-header-parsers-follow-the-abnf-and-never-repair-input.md) | Header parsers follow the ABNF and never repair input | Accepted |
| [0011](0011-hallmark-reads-no-mime-structure.md) | hallmark reads no MIME structure | Accepted |
| [0012](0012-hallmark-converts-domain-names-to-a-labels-with-its-own-rfc-5891-code.md) | hallmark converts domain names to A-labels with its own RFC 5891 code | Accepted |
| [0013](0013-a-from-domain-literal-gives-dmarc-no-author-domain.md) | A From domain literal gives DMARC no author domain | Accepted |
| [0014](0014-hallmark-looks-up-header-domains-as-written-whatever-ascii-they-hold.md) | hallmark looks up header domains as written, whatever ASCII they hold | Accepted |
| [0015](0015-spf-macros-expand-the-local-part-as-written.md) | SPF macros expand the local part as written | Accepted |
| [0016](0016-dkim2-compares-local-parts-as-written-and-ignores-source-routes.md) | DKIM2 compares local parts as written and ignores source routes | Accepted |
| [0017](0017-the-core-does-no-io-and-the-caller-drives-it.md) | The core does no I/O, and the caller drives it | Accepted |
| [0018](0018-signing-goes-through-a-signer-the-caller-supplies.md) | Signing goes through a signer the caller supplies | Superseded by ADR 0042 |
| [0019](0019-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md) | DMARC supports RFC 7489 and DMARCbis, with DMARCbis the default | Superseded by ADR 0040 |
| [0020](0020-test-modes-are-reported-and-the-caller-decides-what-they-mean.md) | Test modes are reported, and the caller decides what they mean | Accepted |
| [0021](0021-spf-reads-v-spf1-without-regard-to-case-and-allows-c-r-and-t-only-in-explanations.md) | SPF reads `v=spf1` without regard to case, and allows `c`, `r` and `t` only in explanations | Accepted |
| [0022](0022-an-spf-target-name-that-macro-expansion-makes-invalid-matches-nothing.md) | An SPF target name that macro expansion makes invalid matches nothing | Accepted |
| [0023](0023-dmarc-follows-rfc-9989-section-4-10-1-for-invalid-policy-tags-and-section-4-7-for-nonexistent-subdomains.md) | DMARC follows RFC 9989 §4.10.1 for invalid policy tags and §4.7 for nonexistent subdomains | Accepted |
| [0024](0024-dmarc-tag-names-keep-their-case-white-space-around-a-value-is-not-part-of-it-and-a-repeated-tag-is-invalid.md) | DMARC tag names keep their case, white space around a value is not part of it, and a repeated tag is invalid | Accepted |
| [0025](0025-a-dkim-selector-with-several-key-records-passes-if-any-of-them-verifies-the-signature.md) | A DKIM selector with several key records passes if any of them verifies the signature | Accepted |
| [0026](0026-rsa-keys-in-key-records-may-be-rsapublickey-or-subjectpublickeyinfo.md) | RSA keys in key records may be RSAPublicKey or SubjectPublicKeyInfo | Accepted |
| [0027](0027-a-malformed-dkim2-domain-policy-record-gives-permerror.md) | A malformed DKIM2 Domain Policy Record gives PERMERROR | Accepted |
| [0028](0028-dkim-h-selection-counts-the-signatures-own-dkim-signature-field.md) | DKIM `h=` selection counts the signature's own DKIM-Signature field | Accepted |
| [0029](0029-the-dkim2-header-hash-leaves-out-lines-that-are-not-header-fields.md) | The DKIM2 header hash leaves out lines that are not header fields | Accepted |
| [0030](0030-a-last-header-line-without-a-line-ending-is-a-header-field-that-canonicalization-ends-with-crlf.md) | A last header line without a line ending is a header field that canonicalization ends with CRLF | Accepted |
| [0031](0031-the-code-that-decides-writes-the-trace-as-events-with-tree-positions.md) | The code that decides writes the trace as events with tree positions | Accepted |
| [0032](0032-the-trace-records-places-in-the-body-and-a-separate-api-streams-the-canonical-body.md) | The trace records places in the body, and a separate API streams the canonical body | Superseded by ADR 0041 |
| [0033](0033-trace-events-are-i-json-texts-in-a-json-text-sequence.md) | Trace events are I-JSON texts in a JSON text sequence | Accepted |
| [0034](0034-trace-step-codes-are-strings-listed-in-a-reference-page.md) | Trace step codes are strings listed in a reference page | Accepted |
| [0035](0035-dns-snapshots-and-demo-cases-are-documents-in-the-trace-formats-conventions.md) | DNS snapshots and demo cases are documents in the trace format's conventions | Accepted |
| [0036](0036-the-minimum-supported-rust-version-starts-at-1-88-and-rises-when-a-dependency-needs-it.md) | The minimum supported Rust version starts at 1.88 and rises when a dependency needs it | Accepted |
| [0037](0037-the-core-takes-crlf-line-endings-only-and-a-bare-cr-or-lf-in-the-body-is-an-octet.md) | The core takes CRLF line endings only, and a bare CR or LF in the body is an octet | Accepted |
| [0038](0038-in-rfc-7489-mode-the-caller-answers-public-suffix-questions.md) | In RFC 7489 mode, the caller answers public suffix questions | Accepted |
| [0039](0039-the-openspf-test-suite-is-copied-unchanged-into-a-test-crate-that-is-never-published.md) | The openspf test suite is copied unchanged into a test crate that is never published | Accepted |
| [0040](0040-dmarc-supports-rfc-7489-and-dmarcbis-with-dmarcbis-the-default.md) | DMARC supports RFC 7489 and DMARCbis, with DMARCbis the default | Accepted |
| [0041](0041-the-trace-records-places-in-the-body-and-a-separate-api-streams-the-canonical-body.md) | The trace records places in the body, and a separate API streams the canonical body | Accepted |
| [0042](0042-signing-goes-through-a-signer-the-caller-supplies-and-the-caller-can-have-each-signature-verified.md) | Signing goes through a signer the caller supplies, and the caller can have each signature verified | Accepted |
| [0043](0043-rsa-public-exponents-have-a-size-limit-in-dkim-and-must-be-65537-in-dkim2.md) | RSA public exponents have a size limit in DKIM and must be 65537 in DKIM2 | Accepted |
| [0044](0044-a-dkim-signature-that-breaks-the-s-flag-of-its-key-gets-permfail-domain-mismatch.md) | A DKIM signature that breaks the `s` flag of its key gets PERMFAIL (domain mismatch) | Accepted |
| [0045](0045-the-hallmark-package-is-licensed-mit-and-unicode-3-0-and-its-source-data-stays-outside-it.md) | The hallmark package is licensed MIT AND Unicode-3.0, and its source data stays outside it | Accepted |
| [0046](0046-dmarc-asks-whether-the-author-domain-exists-only-when-np-can-change-the-policy.md) | DMARC asks whether the author domain exists only when `np` can change the policy | Superseded by ADR 0050 |
| [0047](0047-one-check-type-runs-any-set-of-checks-and-shares-their-dns-answers.md) | One check type runs any set of checks and shares their DNS answers | Accepted |
| [0048](0048-the-resolver-the-suffix-source-and-the-signer-return-futures-that-are-send.md) | The resolver, the suffix source and the signer return futures that are `Send` | Accepted |
| [0049](0049-hallmark-depends-on-cryptography-crates-zeroize-and-thiserror-and-writes-everything-else-itself.md) | hallmark depends on cryptography crates, `zeroize` and `thiserror`, and writes everything else itself | Accepted |
| [0050](0050-dmarc-asks-whether-the-author-domain-exists-when-np-can-change-the-policy-and-needs-the-answer-only-for-a-message-that-fails.md) | DMARC asks whether the author domain exists when `np` can change the policy, and needs the answer only for a message that fails | Accepted |
| [0051](0051-a-bare-cr-or-lf-in-the-header-section-is-an-octet-of-the-line-that-holds-it.md) | A bare CR or LF in the header section is an octet of the line that holds it | Accepted |
| [0052](0052-the-api-reference-is-rustdoc-published-on-github-pages.md) | The API reference is rustdoc, published on GitHub Pages | Accepted |
| [0053](0053-the-tests-run-on-rocky-linux-9-and-debian-13.md) | The tests run on Rocky Linux 9 and Debian 13 | Accepted |
| [0054](0054-the-public-api-follows-one-set-of-conventions.md) | The public API follows one set of conventions | Accepted |
| [0055](0055-the-idna-tables-use-unicode-12-0-0.md) | The IDNA tables use Unicode 12.0.0 | Accepted |
| [0056](0056-authentication-results-map-each-typed-reason-to-one-result-value.md) | Authentication-Results map each typed reason to one result value | Accepted |
| [0057](0057-the-conversion-to-a-labels-runs-the-contexto-rules.md) | The conversion to A-labels runs the CONTEXTO rules | Accepted |

Conventions:
- One file per decision, named `NNNN-short-title.md`, numbered in order.
- Sections: status and date, context, options considered, decision, consequences, references.
- Status is `Proposed`, `Accepted`, `Superseded by ADR NNNN`, or `Deprecated`.
- An accepted record is not rewritten; only its status line changes.
- When a decision turns out wrong or changes, a new ADR restates the whole decision, so that it can be read alone. The old record's status becomes `Superseded by ADR NNNN`, and its body stays as it was.
- Specifications and other documents link only to records in force, and describe current behavior without its history.
