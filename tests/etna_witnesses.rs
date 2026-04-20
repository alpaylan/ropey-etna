//! ETNA witness tests for ropey.
//!
//! Each `witness_<name>_case_<tag>` is a deterministic `#[test]` that calls
//! the framework-neutral `property_<name>` function with frozen inputs.
//! On the base HEAD every witness must PASS; on the corresponding
//! `etna/<variant>` branch the witness for that variant must FAIL.

use ropey::etna::{
    property_lines_match_model, property_rope_builder_default_build,
    property_rope_eq_chunk_invariant, property_rope_hash_chunk_invariant,
    property_slice_crlf_len_lines, property_utf16_char_roundtrip, PropertyResult,
};

fn must_pass(r: PropertyResult, label: &str) {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => {}
        PropertyResult::Fail(m) => panic!("{label}: {m}"),
    }
}

// --- lines_empty_total_lines_faf6738_1 ---

#[test]
fn witness_lines_match_model_case_empty() {
    must_pass(
        property_lines_match_model(String::new()),
        "lines_match_model empty",
    );
}

// --- rope_eq_utf8_boundary_cc516d5_1 ---

#[test]
fn witness_rope_eq_chunk_invariant_case_non_ascii_boundary() {
    // Exact reproduction from the cc516d5 fix commit's regression test
    // `tests/non_ascii_comparison.rs`. Under the buggy PartialEq, the
    // cross-product line comparison panics at `chunk2[..chunk1.len()]`
    // landing inside a 3-byte `ㅇ` scalar.
    let text = include_str!("non_ascii.txt").to_string();
    must_pass(
        property_rope_eq_chunk_invariant(text, 1467, 360),
        "rope_eq_chunk_invariant non-ascii boundary",
    );
}

// --- rope_hash_chunk_boundary_fef5be9_1 ---

#[test]
fn witness_rope_hash_chunk_invariant_case_ascii_split() {
    // Same text, different chunk sizes. With a boundary-sensitive hasher,
    // the naïve per-chunk Hash impl disagrees; the correct fixed-block
    // impl agrees.
    let text = "Hello world, this is a rope that spans multiple chunks.".to_string();
    must_pass(
        property_rope_hash_chunk_invariant(text, 7, 11),
        "rope_hash_chunk_invariant ascii split",
    );
}

// --- utf16_code_unit_conversion_c0af16b_1 ---

#[test]
fn witness_utf16_char_roundtrip_case_latin1() {
    // "éé" — each `é` is one char, one utf16 cu, but two bytes. The buggy
    // version treats the utf16 cu index as a byte index, giving wrong results
    // from char index 1 onwards.
    let text = "éé".to_string();
    must_pass(
        property_utf16_char_roundtrip(text),
        "utf16_char_roundtrip latin1",
    );
}

// --- rope_builder_default_empty_stack_dfcac8b_1 ---

#[test]
fn witness_rope_builder_default_build_case_hello() {
    // Default()-constructed builder must accept an append() and finish()
    // without panicking. Under the buggy `#[derive(Default)]` the stack is
    // empty, `append_leaf_node`'s `self.stack.pop().unwrap()` panics.
    must_pass(
        property_rope_builder_default_build("Hello, world!".to_string()),
        "rope_builder_default_build hello",
    );
}

// --- slice_crlf_split_end_info_8699de0_1 ---

#[test]
fn witness_slice_crlf_len_lines_case_mid_crlf() {
    // 512 CRLF pairs = 1024 bytes, 1024 chars. `Rope::from_str` produces a
    // two-leaf rope (no CRLF ever split across leaves), so `rope.slice(..769)`
    // reaches the `RSEnum::Full` branch — 769 is past the first leaf
    // boundary. The slice endpoint lands at an '\n' whose preceding '\r'
    // sits in the same leaf, so the fixed `is_crlf_split(769)` returns true
    // and bumps `end_info.line_breaks` by one; the buggy end_info omits it.
    let text = "\r\n".repeat(512);
    must_pass(
        property_slice_crlf_len_lines(text, 769),
        "slice_crlf_len_lines mid crlf",
    );
}
