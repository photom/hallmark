# ADR 0054: The public API follows one set of conventions

- Status: Accepted
- Date: 2026-11-19
- Affects: every public item of the `hallmark` crate, its doc comments, `docs/reference/design/modules.md` and `docs/testing/api.md`

## Context

ADR 0052 moved the API reference into the doc comments of the crate. The pull request that wrote those declarations made sixteen choices that no ADR covers: how a caller builds a value, how the traits take their requests, which enums are `#[non_exhaustive]`, what a doc comment holds, and some choices about single items. Each item implemented from now on has to follow the same rules, so they need one place.

Two questions came up after that pull request. `MemorySigner` had an inherent `sign` and the `Signer` trait's `sign`, with different parameters. And `modules.md` listed `LocalPart`, `Mailbox` and `Path` as public types of the `address` module, but no API takes or returns them.

## Options considered

- (a) Record the conventions and the single choices in one ADR.
- (b) Leave the rustdoc as the only record, with the reasons in the pull request.
- (c) Write one ADR for each choice.

## Decision

Option (a).

Building and reading values:

| Rule | Example |
|---|---|
| A type that the caller builds has public fields, is `#[non_exhaustive]`, and has a `new` function or implements `Default`. The caller starts from that value and sets the fields | `Options`, `Envelope`, `DkimSigning` |
| A type whose values have rules checks them when it is built or changed, and keeps its fields private | `Limits` and its setters, `VerificationTime`, `AuthservId` |
| A type that hallmark builds has public fields and is `#[non_exhaustive]`, so the caller reads it and can't build it | Results, requests, `Event` |
| An enum whose values a specification fixes is exhaustive. A reason enum or an error enum is `#[non_exhaustive]` | `SpfOutcome`; `SpfReason`, `CheckError` |
| An enum with a variant behind `unstable-dkim2` is `#[non_exhaustive]`, because another crate in the build can turn the feature on | `Kind`, `Algorithm` |
| A field avoids a Rust keyword by naming its role | `Question::record_type` |
| A trait method takes its request by value, so the future it returns owns what it needs ([ADR 0048](0048-the-resolver-the-suffix-source-and-the-signer-return-futures-that-are-send.md)) | `Resolver::resolve` |
| A caller that has no implementation of an optional trait names an empty enum in `None` | `NoSuffixSource`, `NoSigner` |
| An inherent method doesn't take the name of a trait method that the type implements with other parameters | `MemorySigner::sign_now` beside `Signer::sign` |
| A type that only the parsers use stays crate-private, and the public API reports its parts through public types | `LocalPart`, `Mailbox` and `Path` stay private, and results carry `Domain` and `Data` |

Errors:

| Rule |
|---|
| Each API area has one error enum, with one variant for each error code. A variant's fields hold the data its code needs, such as the limit, the offset, the line or the error underneath |
| Each error enum has `code`, implements `Display` from the code and the data, and implements `std::error::Error`, whose `source` gives the error that a variant carries |
| An error enum's doc comment holds a table of its codes, with the APIs and rules that give them |

Layout and doc comments:

| Rule |
|---|
| The items are re-exported at the crate root. The blocking adapters stay in the public module `io` |
| A function's doc comment says what it does, which states accept it when its type has states, and the meaning and form of each parameter under `# Parameters`. Its rules go in a table under `# Validation`, with the ID, the input, the rule, the error code linked to its variant or "Not an error", the requirement and the scenario. The heading isn't `# Errors`, because some rules aren't errors |
| Each field and variant names its requirements in parentheses |
| rustdoc doesn't draw Mermaid diagrams, so the diagrams stay in the design documents. A doc comment links to the design section that draws its flow, or gives a mapping as a table |
| A doc comment links to a document by its GitHub URL on `main`, because a rustdoc page can't reach the repository with a relative link. CI checks these links against the working tree |

Single choices that the declarations made:

| Item | Choice |
|---|---|
| `Check::answer_signing` | Takes `Result<Signature, SignerFailure>` |
| `FieldForm` | States what each variant means for a value taken and for a value generated |
| `SpfReason`, `DkimReason`, `Dkim2Reason`, `NotApplied`, `PolicyTag` | Take their variants from the result tables of the design documents |
| `DkimResult` | `domain`, `selector` and `algorithm` are optional, because a field that lacks them still gets PERMFAIL (RFC 6376 §6.1.1). `key_records` lists each key record with its own reason (REC-052) |
| `Value::Unknown` | Holds the JSON text of a parameter of a code that the library doesn't list |
| DKIM2's `Revision`, `BodyRevision`, `EarlierLines`, `FieldRevision` | Describe a Reviser's changes as DK2-014 and draft-09 §5 read |
| Error data | `TraceError` carries offsets, `DocumentError::TooLarge` the limit, `SigningError` the domain and selector, and `ParseError` the rule, the offset, the input and the limit |
| VAL-412 | Sits on the `Signer` implementation of `MemorySigner`, the only path it applies to |
| `EnvelopeField` | A public enum that `CheckError::EnvelopeIncomplete` carries |

Option (b) leaves the reasons in a closed pull request, where an implementer won't look. Option (c) spreads closely related rules over many records.

## Consequences

Positive:
- An item added later follows rules that one record states.
- The rustdoc and the design documents agree on which types are public.

Negative:
- A change to one convention needs a new ADR that restates the whole set.
- A developer reads the private address types in the source or with `cargo doc --document-private-items`, because the published reference leaves them out.

## References

- [ADR 0003](0003-rust-code-is-written-in-a-procedural-style.md), the procedural style, which rules out closures in the API
- [ADR 0048](0048-the-resolver-the-suffix-source-and-the-signer-return-futures-that-are-send.md), the futures of the traits
- [ADR 0052](0052-the-api-reference-is-rustdoc-published-on-github-pages.md), the API reference as rustdoc
- [modules.md](../reference/design/modules.md), the public types of each module
