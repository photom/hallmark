//! Email sender authentication: SPF, DKIM, DKIM2 and DMARC.
//!
//! hallmark verifies SPF, DKIM, DKIM2 and DMARC, signs messages with DKIM and
//! DKIM2, and records a trace that explains every step of every check. The
//! core does no I/O of its own: the caller supplies DNS answers, the time, and
//! the message as a stream. The same code runs natively and in WebAssembly.
//!
//! The requirements are in `docs/reference/requirements.md` in the
//! repository.
