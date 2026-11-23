# ADR 0065: The public API follows one set of conventions, and each API checks the limits it takes

- Status: Accepted
- Date: 2026-11-23
- Affects: every public item of the `hallmark` crate, its doc comments, `docs/reference/design/modules.md`, `docs/reference/design/domain-model.md`, `docs/reference/design/limits.md` and `docs/testing/api.md`

## Context

The API reference lives in the doc comments of the crate (ADR 0052). Every item implemented from now on has to follow the same rules for how a caller builds a value, how the traits take their requests, which enums are `#[non_exhaustive]`, what a doc comment holds, and how errors look. Those rules need one place.

Some values a caller fills in have rules that tie two fields together. In `Limits`, `header_field_size` must stay at or below `header_section_size`. A type that checks each value as the caller sets it makes the result depend on the order of the assignments: raising both limits fails when the field size comes first and works when the section size comes first. `CheckOptions` and `SigningOptions` avoid that, because `Check::new` and `Signing::new` check all their rules at once.

## Options considered

For the record of the conventions:

- (a) Record the conventions and the single choices in one ADR.
- (b) Leave the rustdoc as the only record, with the reasons in the pull request.
- (c) Write one ADR for each choice.

For the limits:

- (d) `Limits` and `DocumentLimits` have public fields and check nothing while the caller sets them. Each API that takes them checks every rule at once, and a `validate` method runs the same checks for a caller that wants them earlier.
- (e) A builder collects every value, and its `build` checks them together and gives a `Limits` that is always valid.
- (f) Private fields with one setter for each limit, which checks the new value against the others.

## Decision

Options (a) and (d).

Building and reading values:

| Rule | Example |
|---|---|
| A type that the caller builds has public fields, is `#[non_exhaustive]`, and has a `new` function or implements `Default`. The caller starts from that value and sets the fields | `CheckOptions`, `SigningOptions`, `Envelope`, `DkimSigning`, `Limits`, `DocumentLimits` |
| Such a type checks nothing while the caller sets its fields. The API that takes it checks every rule at once, in the order of its validation table, before it reads any input. The order in which the caller set the fields never changes the result | `Check::new`, `Signing::new`, every API that takes `Limits` or `DocumentLimits` |
| A limits type has a `validate` method that runs the checks of the APIs that take it, for a caller that wants them before it calls one | `Limits::validate`, `DocumentLimits::validate` |
| A type whose value has rules of its own, with no field that constrains another, checks them when it is built and keeps its fields private | `VerificationTime`, `AuthservId` |
| A type that hallmark builds has public fields and is `#[non_exhaustive]`, so the caller reads it and can't build it | Results, requests, `Event` |
| An enum whose values a specification fixes is exhaustive. A reason enum or an error enum is `#[non_exhaustive]` | `SpfOutcome`; `SpfReason`, `CheckError` |
| An enum with a variant behind `unstable-dkim2` is `#[non_exhaustive]`, because another crate in the build can turn the feature on | `Kind`, `Algorithm` |
| A field avoids a Rust keyword by naming its role | `Question::record_type` |
| A trait method takes its request by value, so the future it returns owns what it needs ([ADR 0048](0048-the-resolver-the-suffix-source-and-the-signer-return-futures-that-are-send.md)) | `Resolver::resolve` |
| A caller that has no implementation of an optional trait names an empty enum in `None` | `NoSuffixSource` |
| An inherent method doesn't take the name of a trait method that the type implements with other parameters | `MemorySigner::sign_now` beside `Signer::sign` |
| A type that only the parsers use stays crate-private, and the public API reports its parts through public types | `LocalPart`, `Mailbox` and `Path` stay private, and results carry `Domain` and `Data` |

Errors:

| Rule |
|---|
| Each API area has one error enum, with one variant for each error code. A variant's fields hold the data its code needs, such as the limit, the offset, the line or the error underneath |
| Each error enum has `code`, implements `Display` from the code and the data, and implements `std::error::Error`, whose `source` gives the error that a variant carries |
| An error enum's doc comment holds a table of its codes, with the APIs and rules that give them |
| An API that takes limits reports limits that break their rules through one variant of its own error enum, `LimitsInvalid`, which carries the `LimitsError`. The rules and their codes stay in `LimitsError` |

Layout and doc comments:

| Rule |
|---|
| The items are re-exported at the crate root. The blocking adapters stay in the public module `io` |
| A function's doc comment says what it does, which states accept it when its type has states, and the meaning and form of each parameter under `# Parameters`. Its rules go in a table under `# Validation`, with the ID, the input, the rule, the error code linked to its variant or "Not an error", the requirement and the scenario. The heading isn't `# Errors`, because some rules aren't errors |
| Each field and variant names its requirements in parentheses |
| rustdoc doesn't draw Mermaid diagrams, so the diagrams stay in the design documents. A doc comment links to the design section that draws its flow, or gives a mapping as a table |
| A doc comment links to a document by its GitHub URL on `main`, because a rustdoc page can't reach the repository with a relative link. CI checks these links against the working tree |

Single choices:

| Item | Choice |
|---|---|
| `Signing::answer` | Takes `Result<Signature, SignerFailure>` |
| `FieldForm` | States what each setting means for a value taken and for a value generated |
| `SpfReason`, `DkimReason`, `Dkim2Reason`, `NotApplied`, `PolicyTag` | Take their variants from the result tables of the design documents |
| `DkimResult` | `domain`, `selector` and `algorithm` are optional, because a field that lacks them still gets PERMFAIL (RFC 6376 §6.1.1). `key_records` lists each key record with its own reason, in a `DkimKeyRecord` (REC-052) |
| `Value::Unknown` | Holds the JSON text of a parameter of a code that the library doesn't list |
| DKIM2's `Revision`, `BodyRevision`, `EarlierLines`, `FieldRevision` | Describe a Reviser's changes as DK2-014 and draft-09 §5 read |
| Error data | `TraceError` carries offsets, `DocumentError::TooLarge` the limit, `SigningError` the domain and selector, and `ParseError` the rule, the offset, the input and the limit |
| VAL-412 | Sits on the `Signer` implementation of `MemorySigner`, the only path it applies to |
| `EnvelopeField` | A public enum that `CheckError::EnvelopeIncomplete` carries |

Option (b) leaves the reasons in a closed pull request, where an implementer won't look. Option (c) spreads closely related rules over many records.

Option (d) gives the limits the same pattern as the other options a caller fills in. Option (e) also makes the order irrelevant, but adds a builder type beside every limits type, and a second way to build options. Option (f) makes the result depend on the order of the assignments.

## Consequences

Positive:
- An item added later follows rules that one record states.
- The rustdoc and the design documents agree on which types are public.
- A caller sets the options and the limits in any order, and gets every error from the API that takes them, or earlier from `validate`.

Negative:
- A change to one convention needs a new ADR that restates the whole set.
- A developer reads the private address types in the source or with `cargo doc --document-private-items`, because the published reference leaves them out.
- A `Limits` or `DocumentLimits` value can hold values that break its rules until an API takes it. Each API that takes limits has one more rule and one more error variant.

## References

- [ADR 0048](0048-the-resolver-the-suffix-source-and-the-signer-return-futures-that-are-send.md), the futures of the traits
- [ADR 0052](0052-the-api-reference-is-rustdoc-published-on-github-pages.md), the API reference as rustdoc
- [ADR 0064](0064-rust-code-is-procedural-and-lints-types-and-size-limits-keep-it-efficient-and-reviewable.md), the procedural style, which rules out closures in the API
- [modules.md](../reference/design/modules.md), the public types of each module
- [limits.md](../reference/design/limits.md), each limit's default and range, and the rules between limits
