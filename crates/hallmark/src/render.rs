//! The English renderer of trace events. It reads the English templates that
//! each context gives for its own codes, so it sits in the application layer
//! ([ADR 0061](https://github.com/photom/hallmark/blob/main/docs/adr/0061-each-context-owns-its-trace-codes-and-the-english-renderer-sits-in-the-application-layer.md)).

use crate::Event;

impl Event {
    /// The English text of the event, as
    /// [Trace codes](https://github.com/photom/hallmark/blob/main/docs/reference/trace-codes.md)
    /// gives it.
    ///
    /// A code the renderer doesn't know renders as the code and its
    /// parameters. An `Event` is valid once parsed, so this method has no rule
    /// to check (STEP-056).
    ///
    /// Requirements: TRC-011.
    pub fn render_english(&self) -> String {
        unimplemented!()
    }
}
