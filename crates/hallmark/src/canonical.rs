//! The canonical forms that DKIM and DKIM2 hash (RFC 6376 §3.4, draft-09 §6). A check and a
//! signing hold the same code, so signing hashes a message exactly as verification does
//! (STR-001).

pub(crate) mod body;
pub(crate) mod header;
