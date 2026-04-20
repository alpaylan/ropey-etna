# ropey — ETNA Tasks

Total tasks: 24

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task. The `<PropertyKey>` token in the command column uses the PascalCase key recognised by `src/bin/etna.rs`; passing `All` runs every property for the named framework in a single invocation.

## Property keys

| Property | PropertyKey |
|----------|-------------|
| `property_lines_match_model` | `LinesMatchModel` |
| `property_rope_eq_chunk_invariant` | `RopeEqChunkInvariant` |
| `property_rope_hash_chunk_invariant` | `RopeHashChunkInvariant` |
| `property_utf16_char_roundtrip` | `Utf16CharRoundtrip` |
| `property_rope_builder_default_build` | `RopeBuilderDefaultBuild` |
| `property_slice_crlf_len_lines` | `SliceCrlfLenLines` |

## Task Index

| Task | Variant | Framework | Property | Witness | Command |
|------|---------|-----------|----------|---------|---------|
| 001 | `lines_empty_total_lines_faf6738_1` | proptest | `property_lines_match_model` | `witness_lines_match_model_case_empty` | `cargo run --release --bin etna -- proptest LinesMatchModel` |
| 002 | `lines_empty_total_lines_faf6738_1` | quickcheck | `property_lines_match_model` | `witness_lines_match_model_case_empty` | `cargo run --release --bin etna -- quickcheck LinesMatchModel` |
| 003 | `lines_empty_total_lines_faf6738_1` | crabcheck | `property_lines_match_model` | `witness_lines_match_model_case_empty` | `cargo run --release --bin etna -- crabcheck LinesMatchModel` |
| 004 | `lines_empty_total_lines_faf6738_1` | hegel | `property_lines_match_model` | `witness_lines_match_model_case_empty` | `cargo run --release --bin etna -- hegel LinesMatchModel` |
| 005 | `rope_eq_utf8_boundary_cc516d5_1` | proptest | `property_rope_eq_chunk_invariant` | `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` | `cargo run --release --bin etna -- proptest RopeEqChunkInvariant` |
| 006 | `rope_eq_utf8_boundary_cc516d5_1` | quickcheck | `property_rope_eq_chunk_invariant` | `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` | `cargo run --release --bin etna -- quickcheck RopeEqChunkInvariant` |
| 007 | `rope_eq_utf8_boundary_cc516d5_1` | crabcheck | `property_rope_eq_chunk_invariant` | `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` | `cargo run --release --bin etna -- crabcheck RopeEqChunkInvariant` |
| 008 | `rope_eq_utf8_boundary_cc516d5_1` | hegel | `property_rope_eq_chunk_invariant` | `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` | `cargo run --release --bin etna -- hegel RopeEqChunkInvariant` |
| 009 | `rope_hash_chunk_boundary_fef5be9_1` | proptest | `property_rope_hash_chunk_invariant` | `witness_rope_hash_chunk_invariant_case_ascii_split` | `cargo run --release --bin etna -- proptest RopeHashChunkInvariant` |
| 010 | `rope_hash_chunk_boundary_fef5be9_1` | quickcheck | `property_rope_hash_chunk_invariant` | `witness_rope_hash_chunk_invariant_case_ascii_split` | `cargo run --release --bin etna -- quickcheck RopeHashChunkInvariant` |
| 011 | `rope_hash_chunk_boundary_fef5be9_1` | crabcheck | `property_rope_hash_chunk_invariant` | `witness_rope_hash_chunk_invariant_case_ascii_split` | `cargo run --release --bin etna -- crabcheck RopeHashChunkInvariant` |
| 012 | `rope_hash_chunk_boundary_fef5be9_1` | hegel | `property_rope_hash_chunk_invariant` | `witness_rope_hash_chunk_invariant_case_ascii_split` | `cargo run --release --bin etna -- hegel RopeHashChunkInvariant` |
| 013 | `utf16_code_unit_conversion_c0af16b_1` | proptest | `property_utf16_char_roundtrip` | `witness_utf16_char_roundtrip_case_latin1` | `cargo run --release --bin etna -- proptest Utf16CharRoundtrip` |
| 014 | `utf16_code_unit_conversion_c0af16b_1` | quickcheck | `property_utf16_char_roundtrip` | `witness_utf16_char_roundtrip_case_latin1` | `cargo run --release --bin etna -- quickcheck Utf16CharRoundtrip` |
| 015 | `utf16_code_unit_conversion_c0af16b_1` | crabcheck | `property_utf16_char_roundtrip` | `witness_utf16_char_roundtrip_case_latin1` | `cargo run --release --bin etna -- crabcheck Utf16CharRoundtrip` |
| 016 | `utf16_code_unit_conversion_c0af16b_1` | hegel | `property_utf16_char_roundtrip` | `witness_utf16_char_roundtrip_case_latin1` | `cargo run --release --bin etna -- hegel Utf16CharRoundtrip` |
| 017 | `rope_builder_default_empty_stack_dfcac8b_1` | proptest | `property_rope_builder_default_build` | `witness_rope_builder_default_build_case_hello` | `cargo run --release --bin etna -- proptest RopeBuilderDefaultBuild` |
| 018 | `rope_builder_default_empty_stack_dfcac8b_1` | quickcheck | `property_rope_builder_default_build` | `witness_rope_builder_default_build_case_hello` | `cargo run --release --bin etna -- quickcheck RopeBuilderDefaultBuild` |
| 019 | `rope_builder_default_empty_stack_dfcac8b_1` | crabcheck | `property_rope_builder_default_build` | `witness_rope_builder_default_build_case_hello` | `cargo run --release --bin etna -- crabcheck RopeBuilderDefaultBuild` |
| 020 | `rope_builder_default_empty_stack_dfcac8b_1` | hegel | `property_rope_builder_default_build` | `witness_rope_builder_default_build_case_hello` | `cargo run --release --bin etna -- hegel RopeBuilderDefaultBuild` |
| 021 | `slice_crlf_split_end_info_8699de0_1` | proptest | `property_slice_crlf_len_lines` | `witness_slice_crlf_len_lines_case_mid_crlf` | `cargo run --release --bin etna -- proptest SliceCrlfLenLines` |
| 022 | `slice_crlf_split_end_info_8699de0_1` | quickcheck | `property_slice_crlf_len_lines` | `witness_slice_crlf_len_lines_case_mid_crlf` | `cargo run --release --bin etna -- quickcheck SliceCrlfLenLines` |
| 023 | `slice_crlf_split_end_info_8699de0_1` | crabcheck | `property_slice_crlf_len_lines` | `witness_slice_crlf_len_lines_case_mid_crlf` | `cargo run --release --bin etna -- crabcheck SliceCrlfLenLines` |
| 024 | `slice_crlf_split_end_info_8699de0_1` | hegel | `property_slice_crlf_len_lines` | `witness_slice_crlf_len_lines_case_mid_crlf` | `cargo run --release --bin etna -- hegel SliceCrlfLenLines` |

## Witness catalog

Each witness is a deterministic concrete test. Base build: passes. Variant-active build: fails. Witnesses live in `tests/etna_witnesses.rs`.

| Witness | Property | Detects | Input shape |
|---------|----------|---------|-------------|
| `witness_lines_match_model_case_empty` | `property_lines_match_model` | `lines_empty_total_lines_faf6738_1` | empty string — hits the `total_lines: 1` empty-rope special case |
| `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` | `property_rope_eq_chunk_invariant` | `rope_eq_utf8_boundary_cc516d5_1` | `include_str!("tests/non_ascii.txt")` + `remove(1467..1827)` (the fix commit's own reproduction) |
| `witness_rope_hash_chunk_invariant_case_ascii_split` | `property_rope_hash_chunk_invariant` | `rope_hash_chunk_boundary_fef5be9_1` | 55-byte ASCII sentence hashed via chunk sizes 7 vs 11 through a boundary-sensitive FNV hasher |
| `witness_utf16_char_roundtrip_case_latin1` | `property_utf16_char_roundtrip` | `utf16_code_unit_conversion_c0af16b_1` | `"éé"` — first non-ASCII char triggers the byte-index vs utf16-index confusion |
| `witness_rope_builder_default_build_case_hello` | `property_rope_builder_default_build` | `rope_builder_default_empty_stack_dfcac8b_1` | `"Hello, world!"` appended to a `RopeBuilder::default()`; buggy derive panics in `append_leaf_node` |
| `witness_slice_crlf_len_lines_case_mid_crlf` | `property_slice_crlf_len_lines` | `slice_crlf_split_end_info_8699de0_1` | 15 CRLF pairs chunked at 4 bytes; slice ends at char 5 — a `\r`/`\n` pair split across two children, forcing the `RSEnum::Full` `end_info` path |
