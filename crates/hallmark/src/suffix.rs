//! Public suffix questions and answers.

use crate::{Check, QuestionId, Request};

/// A public suffix question that a check hands the verifying caller (DMC-010).
///
/// In RFC 7489 mode, the check asks for the public suffix of each domain whose
/// organizational domain it may need. RFC 9989 mode asks none
/// ([ADR 0038](https://github.com/photom/hallmark/blob/main/docs/adr/0038-in-rfc-7489-mode-the-caller-answers-public-suffix-questions.md)).
/// [`Check::take_requests`] gives it inside [`Request::Suffix`], and the
/// verifying caller answers it with [`Check::answer_suffix`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SuffixQuestion {
    /// The question's ID, from the ID space of DNS questions (DMC-010).
    pub id: QuestionId,
    /// The domain in lookup form, as ASCII without a final dot (DMC-010).
    pub domain: Vec<u8>,
}

/// The answer to a public suffix question, which the verifying caller builds
/// (DMC-010).
///
/// [`Check::answer_suffix`] checks it when the verifying caller hands it back
/// (VAL-130 to VAL-134).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuffixAnswer {
    /// The public suffix (DMC-010, DMC-011).
    Suffix {
        /// The number of labels in the public suffix, the x of RFC 7489 §3.2
        /// step 3.
        labels: u8,
        /// The name of the list, such as its date or a hash.
        list: Option<String>,
    },
    /// The verifying caller has no suffix for the domain. It gives `temperror`
    /// if the result depends on that organizational domain (DMC-011). It
    /// carries the verifying caller's description, which the trace records as
    /// given (DMC-010).
    Unavailable(Option<Vec<u8>>),
}
