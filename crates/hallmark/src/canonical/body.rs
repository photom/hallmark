//! The body state machine (RFC 6376 §3.4.3, §3.4.4, draft-09 §6.1) and the `l=` cut (§3.5).
//! The caller pushes input chunks and pulls output (STR-003). Between calls the canonicalizer
//! holds counts and flags and no input octet (STR-007), so its memory doesn't grow.

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "the checks and signings come later")
)]

/// CR, LF, SP and HTAB (RFC 5234 Appendix B.1).
const CR: u8 = b'\r';
const LF: u8 = b'\n';
const SP: u8 = b' ';
const HTAB: u8 = b'\t';

/// A body canonicalization. The DKIM2 body input is `Simple` (draft-09 §6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BodyMode {
    /// `simple` (RFC 6376 §3.4.3).
    Simple,
    /// `relaxed` (RFC 6376 §3.4.4).
    Relaxed,
}

/// What one call to [`BodyCanonicalizer::update`] moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Moved {
    /// Octets of the chunk read. The caller passes the rest again in the next call.
    pub(crate) read: usize,
    /// Octets written to the start of the output.
    pub(crate) written: usize,
}

/// What one call to [`BodyTail::drain`] wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Drained {
    /// Octets written to the start of the output.
    pub(crate) written: usize,
    /// Whether the canonical body is complete. If not, the caller drains again.
    pub(crate) complete: bool,
}

/// Canonical octets that wait for room in the caller's output, in the order they're written,
/// and the count of canonical octets so far.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Writer {
    /// `l=`: the canonical octets past it are counted and not written (RFC 6376 §3.5).
    length: Option<u64>,
    /// Canonical octets so far, past the cut included.
    canonical: u64,
    /// Line endings to write first, as CRLF.
    line_endings: u64,
    /// The LF of a CRLF whose CR is written, written before anything else.
    lf_next: bool,
    /// An SP for a run of WSP in `relaxed`, written after the line endings.
    space: bool,
    /// The octet written after the SP.
    octet: Option<u8>,
    /// The CRLF at the end of the body, written last.
    end: bool,
}

impl Writer {
    /// Whether nothing waits.
    fn is_empty(&self) -> bool {
        !self.lf_next && self.line_endings == 0 && !self.space && self.octet.is_none() && !self.end
    }

    /// Whether the next canonical octet falls within `l=`.
    fn within(&self) -> bool {
        !matches!(self.length, Some(length) if self.canonical >= length)
    }

    /// Takes the next waiting octet.
    fn take(&mut self) -> Option<u8> {
        if self.lf_next {
            self.lf_next = false;
            return Some(LF);
        }
        if self.line_endings > 0 {
            self.line_endings -= 1;
            self.lf_next = true;
            return Some(CR);
        }
        if self.space {
            self.space = false;
            return Some(SP);
        }
        if let Some(octet) = self.octet.take() {
            return Some(octet);
        }
        if self.end {
            self.end = false;
            self.lf_next = true;
            return Some(CR);
        }
        None
    }

    /// Writes waiting octets to `output` from `at` until nothing waits or `output` is full.
    /// Octets past the cut need no room. Returns where the written octets end.
    fn drain(&mut self, output: &mut [u8], mut at: usize) -> usize {
        loop {
            if self.within() {
                let Some(slot) = output.get_mut(at) else {
                    return at;
                };
                match self.take() {
                    Some(octet) => *slot = octet,
                    None => return at,
                }
                at += 1;
            } else if self.take().is_none() {
                return at;
            }
            self.canonical += 1;
        }
    }
}

/// A body canonicalizer that takes the body in chunks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BodyCanonicalizer {
    /// The canonicalization.
    mode: BodyMode,
    /// The output that waits for room.
    writer: Writer,
    /// CRLFs not yet written, because they may end the body.
    held_line_endings: u64,
    /// A run of WSP in `relaxed`, not yet written.
    held_space: bool,
    /// A CR at the end of a chunk, which an LF in the next chunk makes a line ending.
    held_cr: bool,
    /// Whether the body has content, which decides the CRLF at the end in `relaxed`.
    content: bool,
}

impl BodyCanonicalizer {
    /// A canonicalizer for `mode`, which writes the first `length` canonical octets, or all of
    /// them if `length` is `None`.
    pub(crate) fn new(mode: BodyMode, length: Option<u64>) -> Self {
        Self {
            mode,
            writer: Writer {
                length,
                canonical: 0,
                line_endings: 0,
                lf_next: false,
                space: false,
                octet: None,
                end: false,
            },
            held_line_endings: 0,
            held_space: false,
            held_cr: false,
            content: false,
        }
    }

    /// Reads `chunk` and writes canonical octets to `output`. It stops when the chunk is read,
    /// or when `output` is full and output waits. The caller passes the unread rest again.
    pub(crate) fn update(&mut self, chunk: &[u8], output: &mut [u8]) -> Moved {
        let mut read = 0;
        let mut written = 0;
        loop {
            written = self.writer.drain(output, written);
            if !self.writer.is_empty() {
                break;
            }
            let Some(octet) = chunk.get(read).copied() else {
                break;
            };
            if self.step(octet) {
                read += 1;
            }
        }
        Moved { read, written }
    }

    /// Ends the body. The tail writes what waits and the CRLF at the end.
    pub(crate) fn finish(mut self) -> BodyTail {
        if self.held_cr {
            self.commit(CR);
        }
        // RFC 6376 §3.4.3 and §3.4.4: an empty body gets a CRLF in `simple` only.
        self.writer.end = self.mode == BodyMode::Simple || self.content;
        BodyTail {
            writer: self.writer,
        }
    }

    /// Reads one octet while nothing waits, and returns whether it used the octet. A held CR
    /// that another octet follows is written first, and the octet waits for the next step.
    fn step(&mut self, octet: u8) -> bool {
        self.commit(octet);
        true
    }

    /// Writes a data octet after the held line endings and the held SP. Nothing waits when
    /// this runs, because `update` steps only then and a held CR means nothing waits.
    fn commit(&mut self, octet: u8) {
        self.writer.line_endings = self.held_line_endings;
        self.held_line_endings = 0;
        self.writer.space = self.held_space;
        self.held_space = false;
        self.writer.octet = Some(octet);
        self.content = true;
    }
}

/// The end of a canonical body, which waits for room in the caller's output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BodyTail {
    /// The output that waits for room.
    writer: Writer,
}

impl BodyTail {
    /// Writes what waits to `output`.
    pub(crate) fn drain(&mut self, output: &mut [u8]) -> Drained {
        let written = self.writer.drain(output, 0);
        Drained {
            written,
            complete: self.writer.is_empty(),
        }
    }

    /// The canonical length so far, octets past the cut included. Once the body is complete,
    /// the length minus `l=` is the unsigned part (DKV-006).
    pub(crate) fn canonical_len(&self) -> u64 {
        self.writer.canonical
    }
}

#[cfg(test)]
mod tests {
    use super::BodyCanonicalizer;
    use super::BodyMode::{self, Relaxed, Simple};

    /// The output room of each run. One octet makes output wait (CAN-090).
    const ROOMS: [usize; 3] = [1, 2, 4096];

    /// RFC 6376 §3.4.5's example body.
    const EXAMPLE: &[u8] = b" C \r\nD \t E\r\n\r\n\r\n";

    /// The canonical body and length of `chunks`, with `room` octets of output in each call.
    fn run(mode: BodyMode, length: Option<u64>, chunks: &[&[u8]], room: usize) -> (Vec<u8>, u64) {
        let mut canonicalizer = BodyCanonicalizer::new(mode, length);
        let mut output = vec![0_u8; room];
        let mut body = Vec::new();
        for chunk in chunks {
            let mut rest: &[u8] = chunk;
            loop {
                let moved = canonicalizer.update(rest, &mut output);
                body.extend_from_slice(&output[..moved.written]);
                rest = &rest[moved.read..];
                if rest.is_empty() && moved.written < room {
                    break;
                }
                assert!(moved.read > 0 || moved.written > 0, "no progress");
            }
        }
        let mut tail = canonicalizer.finish();
        loop {
            let drained = tail.drain(&mut output);
            body.extend_from_slice(&output[..drained.written]);
            if drained.complete {
                return (body, tail.canonical_len());
            }
        }
    }

    /// Checks `input` in one chunk, in two chunks split at each point, and one octet per
    /// chunk, each with every room: the output is `expected`, and the canonical length `len`
    /// (STR-005).
    fn check_cut(mode: BodyMode, length: Option<u64>, input: &[u8], expected: &[u8], len: u64) {
        let mut splits = vec![vec![input]];
        for at in 0..=input.len() {
            let (head, tail) = input.split_at(at);
            splits.push(vec![head, tail]);
        }
        splits.push(input.chunks(1).collect());
        for chunks in &splits {
            for room in ROOMS {
                let (body, canonical) = run(mode, length, chunks, room);
                assert_eq!(body, expected, "{mode:?}, {chunks:?}, room {room}");
                assert_eq!(canonical, len, "{mode:?}, {chunks:?}, room {room}");
            }
        }
    }

    /// [`check_cut`] with no `l=`.
    fn check(mode: BodyMode, input: &[u8], expected: &[u8]) {
        let len = u64::try_from(expected.len()).unwrap();
        check_cut(mode, None, input, expected, len);
    }

    #[test]
    fn can_001_to_006_simple() {
        check(Simple, b"", b"\r\n");
        check(Simple, b"A\r\n\r\n\r\n", b"A\r\n");
        check(Simple, b"A", b"A\r\n");
        check(Simple, EXAMPLE, b" C \r\nD \t E\r\n");
        check(Simple, b"\r\n\r\n", b"\r\n");
        check(Simple, b"A\r\n \r\n", b"A\r\n \r\n");
    }

    #[test]
    fn can_020_to_026_relaxed() {
        check(Relaxed, b"", b"");
        check(Relaxed, EXAMPLE, b" C\r\nD E\r\n");
        check(Relaxed, b"A \t \r\n", b"A\r\n");
        check(Relaxed, b"A\t\t B\r\n", b"A B\r\n");
        check(Relaxed, b"A  ", b"A\r\n");
        check(Relaxed, b"  \r\n\t\r\n", b"");
        check(Relaxed, b"A\r\n \r\n", b"A\r\n");
    }

    #[test]
    fn can_063_to_069_a_bare_cr_or_lf_is_an_octet() {
        check(Simple, b"A\nB\r\n", b"A\nB\r\n");
        check(Simple, b"A\r", b"A\r\r\n");
        check(Simple, b"A\rB\r\n", b"A\rB\r\n");
        check(Relaxed, b"A \nB\r\n", b"A \nB\r\n");
        check(Simple, b"A\r\r\n", b"A\r\r\n");
        check(Simple, b"A\n", b"A\n\r\n");
    }

    /// `check` splits each body at every point and runs it with room for one octet, so the
    /// tests above also cover CAN-082, CAN-084, CAN-085, CAN-087, CAN-089 and CAN-090.
    #[test]
    fn can_080_081_083_088_chunk_boundaries() {
        check(Relaxed, b"Hello   \r\nWorld\r\n", b"Hello\r\nWorld\r\n");
        check(Relaxed, b"Hello   World\r\n", b"Hello World\r\n");
        check(Simple, b"A\r\n\r\n\r\nB\r\n", b"A\r\n\r\n\r\nB\r\n");
        check(Simple, b"AB", b"AB\r\n");
        check(Relaxed, b"AB", b"AB\r\n");
    }

    #[test]
    fn can_100_held_line_endings_are_a_count() {
        let mut body = b"A\r\n".to_vec();
        for _ in 0..1_000_000 {
            body.extend_from_slice(b"\r\n");
        }
        let chunks: Vec<&[u8]> = body.chunks(65_535).collect();
        let mut canonicalizer = BodyCanonicalizer::new(Simple, None);
        let mut output = [0_u8; 4096];
        for chunk in &chunks {
            let moved = canonicalizer.update(chunk, &mut output);
            assert_eq!(moved.read, chunk.len());
        }
        assert_eq!(canonicalizer.held_line_endings, 1_000_001);
        assert_eq!(run(Simple, None, &chunks, 4096), (b"A\r\n".to_vec(), 3));
    }

    #[test]
    fn can_101_held_wsp_is_a_flag() {
        let mut body = b"A".to_vec();
        body.resize(1_000_001, b' ');
        body.extend_from_slice(b"B\r\n");
        let chunks: Vec<&[u8]> = body.chunks(65_535).collect();
        assert_eq!(run(Relaxed, None, &chunks, 4096), (b"A B\r\n".to_vec(), 5));
    }

    #[test]
    fn can_120_to_122_the_l_cut() {
        check_cut(Relaxed, Some(5), EXAMPLE, b" C\r\nD", 9);
        check_cut(Simple, Some(10), b"A\r\n", b"A\r\n", 3);
        check_cut(Simple, Some(6), b"A\r\n", b"A\r\n", 3);
    }
}
