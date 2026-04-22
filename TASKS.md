# ropey — ETNA Tasks

Total tasks: 24

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `lines_empty_total_lines_faf6738_1` | proptest | `LinesMatchModel` | `witness_lines_match_model_case_empty` |
| 002 | `lines_empty_total_lines_faf6738_1` | quickcheck | `LinesMatchModel` | `witness_lines_match_model_case_empty` |
| 003 | `lines_empty_total_lines_faf6738_1` | crabcheck | `LinesMatchModel` | `witness_lines_match_model_case_empty` |
| 004 | `lines_empty_total_lines_faf6738_1` | hegel | `LinesMatchModel` | `witness_lines_match_model_case_empty` |
| 005 | `rope_builder_default_empty_stack_dfcac8b_1` | proptest | `RopeBuilderDefaultBuild` | `witness_rope_builder_default_build_case_hello` |
| 006 | `rope_builder_default_empty_stack_dfcac8b_1` | quickcheck | `RopeBuilderDefaultBuild` | `witness_rope_builder_default_build_case_hello` |
| 007 | `rope_builder_default_empty_stack_dfcac8b_1` | crabcheck | `RopeBuilderDefaultBuild` | `witness_rope_builder_default_build_case_hello` |
| 008 | `rope_builder_default_empty_stack_dfcac8b_1` | hegel | `RopeBuilderDefaultBuild` | `witness_rope_builder_default_build_case_hello` |
| 009 | `rope_eq_utf8_boundary_cc516d5_1` | proptest | `RopeEqChunkInvariant` | `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` |
| 010 | `rope_eq_utf8_boundary_cc516d5_1` | quickcheck | `RopeEqChunkInvariant` | `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` |
| 011 | `rope_eq_utf8_boundary_cc516d5_1` | crabcheck | `RopeEqChunkInvariant` | `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` |
| 012 | `rope_eq_utf8_boundary_cc516d5_1` | hegel | `RopeEqChunkInvariant` | `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` |
| 013 | `rope_hash_chunk_boundary_fef5be9_1` | proptest | `RopeHashChunkInvariant` | `witness_rope_hash_chunk_invariant_case_ascii_split` |
| 014 | `rope_hash_chunk_boundary_fef5be9_1` | quickcheck | `RopeHashChunkInvariant` | `witness_rope_hash_chunk_invariant_case_ascii_split` |
| 015 | `rope_hash_chunk_boundary_fef5be9_1` | crabcheck | `RopeHashChunkInvariant` | `witness_rope_hash_chunk_invariant_case_ascii_split` |
| 016 | `rope_hash_chunk_boundary_fef5be9_1` | hegel | `RopeHashChunkInvariant` | `witness_rope_hash_chunk_invariant_case_ascii_split` |
| 017 | `slice_crlf_split_end_info_8699de0_1` | proptest | `SliceCrlfLenLines` | `witness_slice_crlf_len_lines_case_mid_crlf` |
| 018 | `slice_crlf_split_end_info_8699de0_1` | quickcheck | `SliceCrlfLenLines` | `witness_slice_crlf_len_lines_case_mid_crlf` |
| 019 | `slice_crlf_split_end_info_8699de0_1` | crabcheck | `SliceCrlfLenLines` | `witness_slice_crlf_len_lines_case_mid_crlf` |
| 020 | `slice_crlf_split_end_info_8699de0_1` | hegel | `SliceCrlfLenLines` | `witness_slice_crlf_len_lines_case_mid_crlf` |
| 021 | `utf16_code_unit_conversion_c0af16b_1` | proptest | `Utf16CharRoundtrip` | `witness_utf16_char_roundtrip_case_latin1` |
| 022 | `utf16_code_unit_conversion_c0af16b_1` | quickcheck | `Utf16CharRoundtrip` | `witness_utf16_char_roundtrip_case_latin1` |
| 023 | `utf16_code_unit_conversion_c0af16b_1` | crabcheck | `Utf16CharRoundtrip` | `witness_utf16_char_roundtrip_case_latin1` |
| 024 | `utf16_code_unit_conversion_c0af16b_1` | hegel | `Utf16CharRoundtrip` | `witness_utf16_char_roundtrip_case_latin1` |

## Witness Catalog

- `witness_lines_match_model_case_empty` — base passes, variant fails
- `witness_rope_builder_default_build_case_hello` — base passes, variant fails
- `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` — base passes, variant fails
- `witness_rope_hash_chunk_invariant_case_ascii_split` — base passes, variant fails
- `witness_slice_crlf_len_lines_case_mid_crlf` — base passes, variant fails
- `witness_utf16_char_roundtrip_case_latin1` — base passes, variant fails
