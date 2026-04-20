//! ETNA framework-neutral property functions for ropey.
//!
//! Each `property_<name>` is a pure function taking concrete, owned inputs
//! and returning `PropertyResult`. Framework adapters (proptest / quickcheck
//! / crabcheck / hegel) in `src/bin/etna.rs` and witness tests in
//! `tests/etna_witnesses.rs` all call these functions directly — the
//! invariants are never re-implemented inside an adapter.

#![allow(missing_docs)]

use std::hash::{Hash, Hasher};

use crate::{Rope, RopeBuilder};

pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

// --------------------------------------------------------------------
// Helpers.

/// Build a `Rope` from a `&str` by pushing contiguous grapheme-free chunks of
/// `chunk_size` bytes each using `_append_chunk`. If `chunk_size == 0` we fall
/// back to `Rope::from_str`.
///
/// We step forward at least `chunk_size` bytes, then extend the step to the
/// next char boundary, so each chunk is valid UTF-8. Using `_finish_no_fix`
/// means the btree invariants are NOT restored at finish — any chunk layout
/// the caller requests is preserved in the resulting rope, which is exactly
/// what the chunk-boundary invariance properties need.
pub fn rope_from_str_chunked(text: &str, chunk_size: usize) -> Rope {
    if chunk_size == 0 || text.is_empty() {
        return Rope::from_str(text);
    }
    let mut b = RopeBuilder::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let mut end = (i + chunk_size).min(bytes.len());
        while end < bytes.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        b._append_chunk(&text[i..end]);
        i = end;
    }
    b._finish_no_fix()
}

/// Slow but obviously-correct count of the number of logical lines in a text
/// under ropey's default (unicode) line-break recognizer. Every line break
/// terminates a line, and the trailing (possibly empty) line after the last
/// break is also counted — so the line count is always at least 1.
fn slow_line_count(text: &str) -> usize {
    let mut count = 1usize;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            0x0A => {
                count += 1;
                i += 1;
            }
            0x0D => {
                count += 1;
                // Swallow an immediately-following LF as part of a CRLF pair.
                if i + 1 < bytes.len() && bytes[i + 1] == 0x0A {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            0x0B | 0x0C => {
                count += 1;
                i += 1;
            }
            // NEL U+0085 (two bytes: C2 85).
            0xC2 if i + 1 < bytes.len() && bytes[i + 1] == 0x85 => {
                count += 1;
                i += 2;
            }
            // LS U+2028 (E2 80 A8) / PS U+2029 (E2 80 A9).
            0xE2 if i + 2 < bytes.len()
                && bytes[i + 1] == 0x80
                && (bytes[i + 2] == 0xA8 || bytes[i + 2] == 0xA9) =>
            {
                count += 1;
                i += 3;
            }
            _ => {
                i += 1;
            }
        }
    }
    count
}

/// An `Hasher` that inserts a boundary byte after every `write()` call, so
/// hashers that would normally coalesce calls are forced to observe them.
/// The standard library's hash contract allows this, and the ropey fix for
/// fef5be9c exists precisely because the real-world hashers (fnv, fxhash,
/// ahash) do not coalesce.
#[derive(Default)]
struct ChunkBoundarySensitiveHasher(fnv::FnvHasher);

impl Hasher for ChunkBoundarySensitiveHasher {
    fn finish(&self) -> u64 {
        self.0.finish()
    }
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
        self.0.write_u8(0xFF);
    }
}

fn hash_rope(r: &Rope) -> u64 {
    let mut h = ChunkBoundarySensitiveHasher::default();
    r.hash(&mut h);
    h.finish()
}

// --------------------------------------------------------------------
// Properties.

/// Invariant: the number of lines yielded by `Rope::lines()` matches the
/// slow, byte-by-byte reference count over the same text. In particular,
/// an empty rope has exactly one (empty) line, not zero.
///
/// Detects:
///   - `lines_empty_total_lines_faf6738_1` — empty-rope special case returns
///     `total_lines: 0` (underflowed size_hint / count instead of the single
///     empty line).
pub fn property_lines_match_model(text: String) -> PropertyResult {
    let rope = Rope::from_str(&text);
    let want = slow_line_count(&text);
    // `Lines` is an ExactSizeIterator; `.len()` is the canonical way to ask
    // "how many lines does this rope have?" and is the surface the bug fix
    // `faf6738` restored — `total_lines: 0` for an empty rope makes
    // `len()` return 0 (or underflow in `size_hint`), while `.count()` would
    // still drain and report 1. Use `.len()` so the witness is discriminating.
    let got_len = rope.lines().len();
    if got_len != want {
        return PropertyResult::Fail(format!(
            "rope.lines().len() = {got_len}, slow model = {want} (len_bytes={})",
            text.len()
        ));
    }
    // Draining must also match — and must not underflow after the first step.
    let drained = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut it = rope.lines();
        let _first = it.next();
        // Querying size_hint after advancing is where the underflow panics
        // under `total_lines: 0`.
        let _ = it.size_hint();
        rope.lines().count()
    }));
    match drained {
        Ok(n) if n == want => PropertyResult::Pass,
        Ok(n) => PropertyResult::Fail(format!(
            "rope.lines().count() = {n}, slow model = {want} (len_bytes={})",
            text.len()
        )),
        Err(_) => PropertyResult::Fail(format!(
            "rope.lines() panicked (likely `total_lines - line_idx` underflow); len_bytes={}",
            text.len()
        )),
    }
}

/// Invariant: `PartialEq` between two equal `RopeSlice`s holds without
/// panicking, even when the slice's internal chunk iterator yields chunks
/// that start or end mid-UTF-8-scalar (as happens after `rope.remove(...)`
/// or `rope.slice(a..b)` against non-ASCII text).
///
/// The comparator in `cc516d5` compared chunks via `&str` slicing, so
/// `chunk2[..chunk1.len()]` could land inside a multi-byte scalar and panic
/// with "byte index X is not a char boundary". The fix switched to
/// `&[u8]` slicing.
///
/// Detects:
///   - `rope_eq_utf8_boundary_cc516d5_1` — naïve `&str` slicing in the
///     chunk-wise comparator panics on a non-char-boundary index.
pub fn property_rope_eq_chunk_invariant(
    text: String,
    remove_start: u16,
    remove_len: u16,
) -> PropertyResult {
    // Need at least one non-ASCII scalar for the bug to have anywhere to
    // land, and enough text to force multiple internal chunks.
    if !text.chars().any(|c| !c.is_ascii()) {
        return PropertyResult::Discard;
    }
    let rope1 = Rope::from_str(&text);
    let n_chars = rope1.len_chars();
    if n_chars < 8 {
        return PropertyResult::Discard;
    }
    let start = (remove_start as usize) % n_chars;
    let max_len = n_chars - start;
    let len = ((remove_len as usize) % max_len).max(1);

    let mut rope2 = Rope::from_str(&text);
    rope2.remove(start..(start + len));

    // Cross-compare every line in rope1 with every line in rope2 (the
    // reproduction pattern from the cc516d5 fix). The comparator's inner
    // slice indices are what panic under the buggy impl — equality of the
    // result itself is irrelevant; only panics count as a failure.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let l2: Vec<_> = rope2.lines().collect();
        for line1 in rope1.lines() {
            for line2 in &l2 {
                // Evaluate both RopeSlice==RopeSlice directions.
                let _ = line1 == *line2;
                let _ = *line2 == line1;
            }
        }
    }));
    match result {
        Ok(()) => PropertyResult::Pass,
        Err(_) => PropertyResult::Fail(format!(
            "PartialEq panicked while line-by-line comparing rope1 vs rope2 (text_len={}, remove={start}..{})",
            text.len(),
            start + len
        )),
    }
}

/// Invariant: two ropes built from the same text but with different chunk
/// boundaries produce the same hash, when hashed with a `Hasher` that does
/// not coalesce `write()` calls.
///
/// Detects:
///   - `rope_hash_chunk_boundary_fef5be9_1` — the naïve `Hash` impl writes
///     each chunk as-is, so ropes with different chunk layouts disagree
///     under a boundary-sensitive hasher.
pub fn property_rope_hash_chunk_invariant(
    text: String,
    chunk_a: u16,
    chunk_b: u16,
) -> PropertyResult {
    if chunk_a == 0 || chunk_b == 0 || chunk_a == chunk_b {
        return PropertyResult::Discard;
    }
    let a = rope_from_str_chunked(&text, chunk_a as usize);
    let b = rope_from_str_chunked(&text, chunk_b as usize);
    let ha = hash_rope(&a);
    let hb = hash_rope(&b);
    if ha != hb {
        return PropertyResult::Fail(format!(
            "hash disagreement: chunk={chunk_a} -> {ha:x}, chunk={chunk_b} -> {hb:x}, text_len={}",
            text.len()
        ));
    }
    PropertyResult::Pass
}

/// Invariant: `char_to_utf16_cu` and `utf16_cu_to_char` round-trip on every
/// char boundary index of a rope, for arbitrary UTF-8 text. The char index
/// `i` is mapped to a utf16 code unit index, and mapping back must yield
/// `i` again.
///
/// Detects:
///   - `utf16_code_unit_conversion_c0af16b_1` — the buggy version imports
///     `str_indices::utf16::from_byte_idx` directly as
///     `utf16_code_unit_to_char_idx`, so the "char index" it returns is
///     actually a utf16 cu count, off for any text with non-ASCII chars.
pub fn property_utf16_char_roundtrip(text: String) -> PropertyResult {
    let rope = Rope::from_str(&text);
    let n_chars = rope.len_chars();
    for i in 0..=n_chars {
        let u = rope.char_to_utf16_cu(i);
        let round = rope.utf16_cu_to_char(u);
        if round != i {
            return PropertyResult::Fail(format!(
                "char_to_utf16_cu({i})={u}, utf16_cu_to_char({u})={round} (len_chars={n_chars})"
            ));
        }
    }
    PropertyResult::Pass
}

/// Invariant: `RopeBuilder::default()` produces a builder that is functionally
/// equivalent to `RopeBuilder::new()` — appending `text` and finishing must
/// succeed and yield a rope whose contents equal `text`.
///
/// Detects:
///   - `rope_builder_default_empty_stack_dfcac8b_1` — the buggy
///     `#[derive(Default)]` gives `RopeBuilder { stack: SmallVec::new(), .. }`
///     (empty stack). `append_leaf_node` immediately `self.stack.pop().unwrap()`s
///     and panics; `finish` underflows `self.stack.len() - 1`. Either way,
///     appending or finishing the default builder panics.
pub fn property_rope_builder_default_build(text: String) -> PropertyResult {
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut b = RopeBuilder::default();
        b.append(&text);
        b.finish()
    }));
    match outcome {
        Ok(rope) => {
            let got: String = rope.chars().collect();
            if got == text {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!(
                    "RopeBuilder::default() built rope content mismatch: got {got:?}, want {text:?}"
                ))
            }
        }
        Err(_) => PropertyResult::Fail(format!(
            "RopeBuilder::default().append({:?}).finish() panicked",
            text
        )),
    }
}

/// Invariant: for every char index `i` in a rope, the sliced rope
/// `r.slice(..i)` reports the same `len_lines()` as the slow per-byte line
/// count over `&text[..byte_idx_of(i)]`. This must hold even when `i` lands
/// between a `\r` and a `\n`, which is the exact trigger for the `8699de0`
/// bug.
///
/// Detects:
///   - `slice_crlf_split_end_info_8699de0_1` — the buggy slice builder sets
///     `end_info = node.char_to_text_info(n_end)` without accounting for a
///     `\r\n` pair split by the slice boundary. When the slice cuts between
///     `\r` and `\n`, the terminating `\r` of the slice no longer has a
///     following `\n` inside the slice, so CRLF no longer pairs and the
///     count of line breaks inside the slice is under-reported by one.
pub fn property_slice_crlf_len_lines(text: String, prefix: u16) -> PropertyResult {
    // Force an internal-node rope layout with small chunks so the slice
    // endpoint falls inside `RSEnum::Full`'s `end_info` computation — that is
    // the code path the patch corrupts. A single-leaf rope takes the
    // `Node::Leaf` early-return in `RopeSlice::new_with_range`, which
    // recomputes line breaks from the slice text directly and therefore hides
    // the bug.
    let rope = rope_from_str_chunked(&text, 4);
    let n_chars = rope.len_chars();
    if n_chars == 0 {
        return PropertyResult::Discard;
    }
    let at_char = (prefix as usize) % (n_chars + 1);
    let at_byte = rope.char_to_byte(at_char);
    let prefix_text = &text[..at_byte];
    let want = slow_line_count(prefix_text);
    let slice = rope.slice(..at_char);
    let got = slice.len_lines();
    if got == want {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(format!(
            "slice(..{at_char}).len_lines() = {got}, slow model = {want} (prefix_text={:?})",
            prefix_text
        ))
    }
}
