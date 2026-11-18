//! The public canonicalizer. Body and header canonicalization inside a check
//! stay crate-private.

use crate::*;

/// Streams out the canonical body that a check hashes, for callers that want
/// to see it, such as the demo (STR-012,
/// [ADR 0041](https://github.com/photom/hallmark/blob/main/docs/adr/0041-the-trace-records-places-in-the-body-and-a-separate-api-streams-the-canonical-body.md)).
///
/// It runs the same code as the checks and is separate from any check. The
/// caller passes each body chunk to [`canonicalize`](Self::canonicalize),
/// calling again with the rest of the chunk until all of it is read, then
/// calls [`finish`](Self::finish) until the output is complete.
/// [Lifecycle](https://github.com/photom/hallmark/blob/main/docs/reference/design/lifecycle.md#canonicalizer)
/// gives its states, Streaming and Ended.
///
/// | Rule | Requirements |
/// |---|---|
/// | Between calls, the canonicalizer holds counts and flags of fixed size and no input octet | STR-001, STR-007 |
/// | The output doesn't depend on where the chunks are cut | STR-005 |
/// | A bare CR or LF is an octet of the body | STR-011 |
/// | A call that returns an error changes nothing. A canonicalizer that has reported a recipe error gives that error from then on | DK2-007 |
#[derive(Debug)]
pub struct Canonicalizer {
    state: (),
}

impl Canonicalizer {
    /// Makes a canonicalizer in Streaming.
    ///
    /// # Parameters
    ///
    /// - `mode`: the canonical form to stream.
    /// - `length`: the `l=` length in octets, to cut the canonical body at.
    /// - `rebuild`: the earlier DKIM2 version to stream instead of the current
    ///   body.
    /// - `limits`: the limits that apply to the recipes: `message_instances`,
    ///   `recipe_steps`, `recipe_size` and `recipe_depth`.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-601 | `length` | `Some` only with `Simple` or `Relaxed` | [`canonicalizer.length_not_allowed`](CanonicalizerError::LengthNotAllowed) | STR-012, draft-09 §6.1 | API-601 |
    /// | VAL-602 | `rebuild` | `Some` only with `Dkim2` | [`canonicalizer.rebuild_not_allowed`](CanonicalizerError::RebuildNotAllowed) | STR-012 | API-602 |
    /// | VAL-603 | `rebuild` | Each Message-Instance value parses, at most `message_instances` of them | [`canonicalizer.instance_invalid`](CanonicalizerError::InstanceInvalid), with the parse error | DK2-006, SAF-002 | API-603 |
    /// | VAL-604 | `rebuild` | `version` is below the highest instance, and every instance from the highest down to `version` is given | [`canonicalizer.instance_missing`](CanonicalizerError::InstanceMissing) | DK2-006 | API-604 |
    /// | VAL-605 | `rebuild` | No recipe on the way is a null body recipe | [`canonicalizer.null_recipe`](CanonicalizerError::NullRecipe) | DK2-007 | API-605 |
    /// | VAL-606 | `rebuild` | Each recipe keeps its `c` steps in ascending order without overlap, and stays within `recipe_steps`, `recipe_size` and `recipe_depth` | [`canonicalizer.recipe_invalid`](CanonicalizerError::RecipeInvalid), with the step or the limit | DK2-007, SAF-002, draft-09 §5.2 | API-606 |
    ///
    /// Requirements: STR-012.
    pub fn new(
        mode: CanonicalizerMode,
        length: Option<u64>,
        rebuild: Option<Rebuild>,
        limits: &Limits,
    ) -> Result<Canonicalizer, CanonicalizerError> {
        unimplemented!()
    }

    /// Canonicalizes the next body bytes into `output`.
    ///
    /// Accepted in Streaming. It gives how many octets of `chunk` it read and
    /// how many it wrote into `output`. The caller calls again with the rest of
    /// the chunk until all of it is read.
    ///
    /// # Parameters
    ///
    /// - `chunk`: the next body bytes, with CRLF line endings, cut anywhere
    ///   (STR-011).
    /// - `output`: where the canonical bytes go, from the start of the buffer.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-610 | The call | The canonicalizer is in Streaming | [`canonicalizer.ended`](CanonicalizerError::Ended) | STR-012 | API-610 |
    /// | VAL-611 | `output` | Holds at least 1 byte | [`canonicalizer.buffer_empty`](CanonicalizerError::BufferEmpty) | STR-003 | API-611 |
    ///
    /// Requirements: STR-001, STR-005.
    pub fn canonicalize(
        &mut self,
        chunk: &[u8],
        output: &mut [u8],
    ) -> Result<Progress, CanonicalizerError> {
        unimplemented!()
    }

    /// Writes the rest of the canonical body into `output`.
    ///
    /// Accepted in Streaming and Ended. The first call moves the canonicalizer
    /// to Ended. It gives how many octets it wrote and whether the output is
    /// complete. Once the output is complete, `finish` writes nothing and gives
    /// `complete` as true.
    ///
    /// # Parameters
    ///
    /// - `output`: where the canonical bytes go, from the start of the buffer.
    ///
    /// # Validation
    ///
    /// | ID | Input | Rule | Error code | Requirement | Scenario |
    /// |---|---|---|---|---|---|
    /// | VAL-615 | `output` | Holds at least 1 byte | [`canonicalizer.buffer_empty`](CanonicalizerError::BufferEmpty) | STR-003 | API-615 |
    /// | VAL-616 | `rebuild` of [`new`](Self::new) | Every line that a recipe copies lies within the body | [`canonicalizer.recipe_invalid`](CanonicalizerError::RecipeInvalid), with the step | DK2-007 | API-616 |
    ///
    /// Requirements: STR-012.
    pub fn finish(&mut self, output: &mut [u8]) -> Result<Finish, CanonicalizerError> {
        unimplemented!()
    }
}

/// The canonical form that a [`Canonicalizer`] streams.
///
/// `Dkim2` exists only with the `unstable-dkim2` feature, so the enum is
/// `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CanonicalizerMode {
    /// The body of RFC 6376 §3.4.3, cut at `length` if given (STR-012).
    Simple,
    /// The body of RFC 6376 §3.4.4, cut at `length` if given (STR-012).
    Relaxed,
    /// The DKIM2 body hash input of draft-09 §6.1, of the current body or of
    /// the version that `rebuild` names (STR-012, DK2-006). Available with the
    /// `unstable-dkim2` feature.
    #[cfg(feature = "unstable-dkim2")]
    Dkim2,
}

/// The earlier DKIM2 version that a [`Canonicalizer`] rebuilds and streams
/// (DK2-006).
///
/// The caller builds it with `Rebuild::new`, which exists only with the
/// `unstable-dkim2` feature. Without the feature, the caller passes `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Rebuild {
    /// The `m=` of the Message-Instance whose body to rebuild.
    pub version: u64,
    /// Message-Instance field values, each without the field name and the
    /// final CRLF, from the highest down to `version`.
    pub instances: Vec<Vec<u8>>,
}

impl Rebuild {
    /// Names the earlier version to rebuild. Available with the
    /// `unstable-dkim2` feature.
    ///
    /// # Parameters
    ///
    /// - `version`: the `m=` of the Message-Instance whose body to rebuild.
    /// - `instances`: Message-Instance field values, each without the field
    ///   name and the final CRLF, from the highest down to `version`.
    ///
    /// Building a rebuild checks nothing. [`Canonicalizer::new`] checks it.
    ///
    /// Requirements: DK2-006.
    #[cfg(feature = "unstable-dkim2")]
    pub fn new(version: u64, instances: Vec<Vec<u8>>) -> Rebuild {
        unimplemented!()
    }
}

/// What one call of [`Canonicalizer::canonicalize`] read and wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Progress {
    /// Octets of `chunk` read, from its start (STR-001).
    pub read: usize,
    /// Octets written into `output`, from its start (STR-003).
    pub written: usize,
}

/// What one call of [`Canonicalizer::finish`] wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Finish {
    /// Octets written into `output`, from its start (STR-003).
    pub written: usize,
    /// The output is complete, and later calls write nothing (STR-012).
    pub complete: bool,
}

/// The error of the methods of [`Canonicalizer`].
///
/// | Code | Variant | Given by | Rules | Requirements |
/// |---|---|---|---|---|
/// | `canonicalizer.length_not_allowed` | [`LengthNotAllowed`](Self::LengthNotAllowed) | [`Canonicalizer::new`] | VAL-601 | STR-012 |
/// | `canonicalizer.rebuild_not_allowed` | [`RebuildNotAllowed`](Self::RebuildNotAllowed) | [`Canonicalizer::new`] | VAL-602 | STR-012 |
/// | `canonicalizer.instance_invalid` | [`InstanceInvalid`](Self::InstanceInvalid) | [`Canonicalizer::new`] | VAL-603 | DK2-006 |
/// | `canonicalizer.instance_missing` | [`InstanceMissing`](Self::InstanceMissing) | [`Canonicalizer::new`] | VAL-604 | DK2-006 |
/// | `canonicalizer.null_recipe` | [`NullRecipe`](Self::NullRecipe) | [`Canonicalizer::new`] | VAL-605 | DK2-007 |
/// | `canonicalizer.recipe_invalid` | [`RecipeInvalid`](Self::RecipeInvalid) | [`Canonicalizer::new`], [`Canonicalizer::finish`] | VAL-606, VAL-616 | DK2-007 |
/// | `canonicalizer.ended` | [`Ended`](Self::Ended) | [`Canonicalizer::canonicalize`] | VAL-610 | STR-012 |
/// | `canonicalizer.buffer_empty` | [`BufferEmpty`](Self::BufferEmpty) | [`Canonicalizer::canonicalize`], [`Canonicalizer::finish`] | VAL-611, VAL-615 | STR-003 |
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CanonicalizerError {
    /// `canonicalizer.length_not_allowed`: an `l=` length in `Dkim2` mode.
    LengthNotAllowed,
    /// `canonicalizer.rebuild_not_allowed`: a rebuild outside `Dkim2` mode.
    RebuildNotAllowed,
    /// `canonicalizer.instance_invalid`: a Message-Instance value that doesn't
    /// parse, or too many.
    InstanceInvalid(ParseError),
    /// `canonicalizer.instance_missing`: the version asked for, or an instance
    /// on the way to it, is missing.
    InstanceMissing,
    /// `canonicalizer.null_recipe`: a null body recipe on the way to the
    /// version asked for.
    NullRecipe,
    /// `canonicalizer.recipe_invalid`: a recipe that breaks the order of
    /// draft-09 §5.2, a recipe limit, or the end of the body.
    RecipeInvalid {
        /// The step that broke the rule, counted from 1.
        step: Option<u64>,
        /// The limit the recipe reached.
        limit: Option<Limit>,
    },
    /// `canonicalizer.ended`: a chunk after `finish`.
    Ended,
    /// `canonicalizer.buffer_empty`: an output buffer of 0 bytes.
    BufferEmpty,
}

impl CanonicalizerError {
    /// The error's code, such as `canonicalizer.ended` (STB-001).
    pub fn code(&self) -> &'static str {
        unimplemented!()
    }
}

impl core::fmt::Display for CanonicalizerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for CanonicalizerError {}
