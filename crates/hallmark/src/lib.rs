//! Email sender authentication: SPF, DKIM, DKIM2 and DMARC.
//!
//! hallmark verifies SPF, DKIM, DKIM2 and DMARC, signs messages with DKIM and
//! DKIM2, and records a trace that explains every step of every check. The
//! core does no I/O of its own: the caller supplies DNS answers, the time, and
//! the message as a stream. The same code runs natively and in WebAssembly.
//!
//! No item is implemented yet. Each item on these pages is a declaration that
//! only rustdoc compiles
//! ([ADR 0052](https://github.com/photom/hallmark/blob/main/docs/adr/0052-the-api-reference-is-rustdoc-published-on-github-pages.md)),
//! and the names are a first draft until the first release (STB-001).
//!
//! # Overview
//!
//! A caller builds [`Options`], creates a [`Check`], and passes it an
//! [`Envelope`] and the message. The check hands out [`Request`]s: DNS
//! questions, public suffix questions and signing requests. The caller
//! answers each one, and when the check finishes, takes the [`Results`] and
//! reads the trace.
//! [Lifecycle](https://github.com/photom/hallmark/blob/main/docs/reference/design/lifecycle.md#a-check-from-start-to-end)
//! draws the calls in order, and the
//! [domain model](https://github.com/photom/hallmark/blob/main/docs/reference/design/domain-model.md#what-the-core-and-the-caller-exchange)
//! draws the values that pass between the core and the caller.
//!
//! | Tool | Works on |
//! |---|---|
//! | [`drive`], with a [`Resolver`], a [`SuffixSource`] and a [`Signer`] | A check, with the caller's resolver, suffix source and signer |
//! | [`io::read_message`], [`io::write_trace`] | A check, with a standard blocking reader and writer |
//! | [`AuthenticationResults`] | Authentication-Results header fields, written from results or read from upstream |
//! | [`Canonicalizer`] | A body, outside any check |
//! | [`sort_trace`], [`Event`] | Trace bytes |
//! | [`Snapshot`], [`SnapshotResolver`], [`PastedRecords`] | DNS answers kept for replay |
//! | [`MemorySigner`] | Keys held in memory |
//!
//! # How to read an entry
//!
//! The doc comment of a function or method says what it does, which states
//! accept it when its type has states, and the meaning and form of each
//! parameter. A table then gives its validation rules, with these columns:
//!
//! | Column | Content |
//! |---|---|
//! | ID | `VAL-` and three digits, unique across the crate |
//! | Input | The parameter or the state that the rule reads |
//! | Rule | The condition that must hold |
//! | Error code | The code that a broken rule gives, linked to its variant, or "Not an error" and what happens instead |
//! | Requirement | The requirement or the section of a specification behind the rule |
//! | Scenario | The test: an `API-` scenario with the rule's number, or a scenario in another file |
//!
//! A function or method with no rule to check says so. IDs such as CHK-004
//! name the [requirements](https://github.com/photom/hallmark/blob/main/docs/reference/requirements.md),
//! and scenario IDs such as API-101 name the
//! [test scenarios](https://github.com/photom/hallmark/blob/main/docs/testing/README.md).
//! The `API-` scenarios are in
//! [api.md](https://github.com/photom/hallmark/blob/main/docs/testing/api.md).
//! Each field and variant names the requirements behind it in parentheses.
//!
//! # Conventions
//!
//! | Item | Convention |
//! |---|---|
//! | Rule order | A method checks its rules in the order its table lists them and gives the code of the first one broken |
//! | Strong types | A rule that a type already enforces, such as a 32-octet key or a `Kind` that exists only with `unstable-dkim2`, gets no row |
//! | Errors | A function that can fail returns `Result`, and its error type is an enum with one variant for each code |
//! | Panics | No function or method panics on any input (SAF-001, [ADR 0004](https://github.com/photom/hallmark/blob/main/docs/adr/0004-no-input-makes-hallmark-panic.md)), so no entry has a Panics section |
//! | Closures | No API takes a closure, a function pointer or a type bound on `Fn` ([ADR 0003](https://github.com/photom/hallmark/blob/main/docs/adr/0003-rust-code-is-written-in-a-procedural-style.md)). A caller supplies behavior by implementing a trait |
//! | `unstable-dkim2` | APIs, fields and variants that say they are available with the feature exist only when it is on, and can change in any release (DK2-005) |
//!
//! # Types
//!
//! | Written in a signature | Meaning |
//! |---|---|
//! | `&[u8]` | Bytes the call borrows and doesn't keep past its return (STR-007). Any octet is allowed unless the parameter says otherwise |
//! | `Vec<u8>` | Bytes a value owns |
//! | `&str`, `String` | UTF-8 text, borrowed or owned |
//! | `&mut [u8]` | A buffer the call writes into, from its start |
//! | `u64`, `usize` | `usize` counts bytes in memory, and `u64` counts octets and positions in input |
//!
//! | Rule | Why |
//! |---|---|
//! | Plain data types have public fields | The caller reads each field by name |
//! | Types that hallmark builds, such as results and requests, are `#[non_exhaustive]`, so the caller reads their fields and can't build them | A minor version can add a field (STB-001) |
//! | Types that the caller builds, such as [`Envelope`], are `#[non_exhaustive]` and have a `new` function or implement `Default`. The caller starts from that value and sets the public fields | A minor version can add a field with a default (STB-001) |
//! | Enums whose values a specification fixes, such as [`SpfOutcome`], are exhaustive. Reason enums and error enums are `#[non_exhaustive]` | STB-001 |
//! | An enum with a variant that exists only with `unstable-dkim2`, such as [`Kind`], is `#[non_exhaustive]` | Another crate in the build can turn the feature on, and a `match` without a wildcard arm would then stop compiling |
//! | Every public type implements `Debug`. Plain data types also implement `Clone`, `PartialEq` and `Eq` | Tests compare requests and results |
//! | Types that hold a private key implement neither `Debug` nor `Clone` | SEC-001, SEC-002 |
//!
//! [Concurrency](https://github.com/photom/hallmark/blob/main/docs/reference/design/concurrency.md#send-and-sync)
//! gives which types are `Send` and `Sync`.
//!
//! # Errors
//!
//! | Where a failure comes from | What happens |
//! |---|---|
//! | The caller breaks a rule of an API | The call returns an error, and the value called stays as it was |
//! | The signer fails, or its signature is wrong | That signing stops, with a [`SigningError`] in the results |
//! | The message, the envelope or a DNS record | The protocol's result, and a trace step |
//! | A limit on a check's work | The protocol's error result, and a trace step that names the limit |
//! | The trace limit | The trace is cut short, and the results stay the same |
//! | The caller's reader or writer | An [`io::StreamError`] that names the stream |
//!
//! | Rule | Requirements |
//! |---|---|
//! | An error returned from a call never becomes a protocol result, and a protocol result never comes back as an error | CHK-004, INV-010 |
//! | A call that returns an error changes nothing in the value called | CHK-004, INV-003 |
//! | Input from a sender or a DNS publisher never gives a returned error. It gives a protocol result | GEN-005, SAF-001 |
//! | No error, in its text, its data or its `Debug` output, holds a byte of a private key | SEC-002, THR-012 |
//! | An error names input by its role and offset, such as the header field and the byte, and doesn't copy body content | PAR-001, TRC-010 |
//!
//! Each error type is a `#[non_exhaustive]` enum with one variant for each of
//! its codes, and its doc comment lists the codes with the APIs and rules
//! that give them. A variant's fields hold the data its code needs: the
//! limit, the offset, the line, the member, or the error underneath. Each
//! error type has a `code` method, which gives the code as text, such as
//! `check.out_of_order`. It implements `Display`, which writes English text
//! from the code and the data, and `std::error::Error`, whose `source` gives
//! the error that a variant carries.
//!
//! | Type | Returned by | Code prefix |
//! |---|---|---|
//! | [`CheckError`] | Every method of [`Check`] except [`Check::new`] | `check.` |
//! | [`OptionsError`] | [`Check::new`], [`VerificationTime::from_unix_seconds`] | `options.` |
//! | [`LimitsError`] | The setters of [`Limits`] and [`DocumentLimits`] | `limits.` |
//! | [`ResultsError`] | [`HeaderFieldOut::read_value`] | `results.` |
//! | [`SigningError`] | Held in [`Signing::Stopped`] in the results | `signing.` |
//! | [`KeyError`] | The methods of [`MemorySigner`] that add a key | `key.` |
//! | [`TraceError`] | [`sort_trace`], [`Event::parse`] | `trace.` |
//! | [`CanonicalizerError`] | Every method of [`Canonicalizer`] | `canonicalizer.` |
//! | [`AuthResError`] | [`AuthservId::new`], [`AuthenticationResults::parse`], [`AuthenticationResults::carries_authserv_id`] | `authres.` |
//! | [`DriveError`] | [`drive`] | `driver.` |
//! | [`io::StreamError`] | [`io::read_message`], [`io::write_trace`] | `stream.` |
//! | [`DocumentError`] | [`Snapshot::parse`], [`Snapshot::from_trace`], [`PastedRecords::parse`] | `document.` |
//!
//! [`SignerFailure`] is a struct. [`MemorySigner::sign`] puts the code of a
//! broken rule, a `signer.` code, in its description.
//!
//! A code keeps its meaning within a major version of the crate, and a minor
//! version can add codes and variants (STB-001). The codes of DKIM2 features
//! can change in any release while `unstable-dkim2` lasts (DK2-005). Error
//! codes are separate from trace codes, and the trace format's version
//! doesn't cover them.
//!
//! # Features
//!
//! | Feature | Default | Turns on |
//! |---|---|---|
//! | `unstable-dkim2` | Off | DKIM2 verification and signing, ML-DSA-44, the DKIM2 mode of the canonicalizer, and the `dkim2` method of Authentication-Results (DK2-001, DK2-005) |

#[cfg(doc)]
mod address;
#[cfg(doc)]
mod authres;
#[cfg(doc)]
mod canonical;
#[cfg(doc)]
mod check;
#[cfg(doc)]
mod dkim;
#[cfg(all(doc, feature = "unstable-dkim2"))]
mod dkim2;
#[cfg(doc)]
mod dmarc;
#[cfg(doc)]
mod dns;
#[cfg(doc)]
mod driver;
#[cfg(doc)]
mod header;
#[cfg(doc)]
pub mod io;
#[cfg(doc)]
mod keys;
#[cfg(doc)]
mod limits;
#[cfg(doc)]
mod parse;
#[cfg(doc)]
mod signing;
#[cfg(doc)]
mod snapshot;
#[cfg(doc)]
mod spf;
#[cfg(doc)]
mod suffix;
#[cfg(doc)]
mod time;
#[cfg(doc)]
mod trace;

#[cfg(doc)]
pub use address::Domain;
#[cfg(doc)]
pub use authres::{AuthResError, AuthenticationResults, AuthservId, MethodResult, Property};
#[cfg(doc)]
pub use canonical::{
    Canonicalizer, CanonicalizerError, CanonicalizerMode, Finish, Progress, Rebuild,
};
#[cfg(doc)]
pub use check::{
    CanonAlgorithm, Check, CheckError, DkimCanonicalization, DkimSigning, DkimSource, DmarcMode,
    Envelope, EnvelopeField, ExpectedInput, HeaderFieldOut, Kind, Options, OptionsError, Request,
    Results, ResultsError, Signing, SigningError, SpfSource, TraceRead, Waiting,
};
#[cfg(doc)]
pub use dkim::{DkimOutcome, DkimReason, DkimResult, DkimResults};
#[cfg(doc)]
pub use dmarc::{
    Alignment, AlignmentMode, DmarcOutcome, DmarcResult, NotApplied, PolicyTag, PolicyTagName,
    PublishedPolicy, TestModeTag,
};
#[cfg(doc)]
pub use dns::{Answer, Question, QuestionId, QuestionType, Record};
#[cfg(doc)]
pub use driver::{DriveError, NoSigner, NoSuffixSource, Resolver, Signer, SuffixSource, drive};
#[cfg(doc)]
pub use header::{FieldForm, LeadingSpace, LineBreak};
#[cfg(doc)]
pub use keys::{KeyError, MemorySigner, RandomFailure, RandomSource};
#[cfg(doc)]
pub use limits::{DocumentLimits, Limit, Limits, LimitsError};
#[cfg(doc)]
pub use parse::ParseError;
#[cfg(doc)]
pub use signing::{
    Algorithm, RequestId, Signature, SignerFailure, SignerStatement, SigningKind, SigningRequest,
};
#[cfg(doc)]
pub use snapshot::{DocumentError, PastedRecords, Snapshot, SnapshotResolver};
#[cfg(doc)]
pub use spf::{SpfIdentity, SpfOutcome, SpfReason, SpfResult};
#[cfg(doc)]
pub use suffix::{SuffixAnswer, SuffixQuestion};
#[cfg(doc)]
pub use time::VerificationTime;
#[cfg(doc)]
pub use trace::{
    Data, Event, EventKind, Level, Param, Parties, Place, Position, TraceError, Value, Version,
    sort_trace,
};

#[cfg(all(doc, feature = "unstable-dkim2"))]
pub use check::{
    BodyRevision, Dkim2Flags, Dkim2Signer, Dkim2Signing, EarlierLines, FieldRevision, Revision,
};

#[cfg(all(doc, feature = "unstable-dkim2"))]
pub use dkim2::{Dkim2Reason, Dkim2Result, Dkim2State};
