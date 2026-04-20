# ropey — Injected Bugs

Total mutations: 6

## Bug Index

| # | Name | Variant | File | Injection | Fix Commit |
|---|------|---------|------|-----------|------------|
| 1 | `lines_empty_total_lines` | `lines_empty_total_lines_faf6738_1` | `patches/lines_empty_total_lines_faf6738_1.patch` | `patch` | `faf67387b86584a893d1af60d3a71ab0bd1deac6` |
| 2 | `rope_eq_utf8_boundary` | `rope_eq_utf8_boundary_cc516d5_1` | `patches/rope_eq_utf8_boundary_cc516d5_1.patch` | `patch` | `cc516d54037a2f98785dc8cc77d6e6a6201502c3` |
| 3 | `rope_hash_chunk_boundary` | `rope_hash_chunk_boundary_fef5be9_1` | `patches/rope_hash_chunk_boundary_fef5be9_1.patch` | `patch` | `fef5be9c52b3bab9d2e4d25fbb66e05ff5ddb8ca` |
| 4 | `utf16_code_unit_conversion` | `utf16_code_unit_conversion_c0af16b_1` | `patches/utf16_code_unit_conversion_c0af16b_1.patch` | `patch` | `c0af16bdf5fef1a8f0a29b13a5dd0d44dd16c7a7` |
| 5 | `rope_builder_default_empty_stack` | `rope_builder_default_empty_stack_dfcac8b_1` | `patches/rope_builder_default_empty_stack_dfcac8b_1.patch` | `patch` | `dfcac8b19ee571a2a399e189c12b2a10663ce464` |
| 6 | `slice_crlf_split_end_info` | `slice_crlf_split_end_info_8699de0_1` | `patches/slice_crlf_split_end_info_8699de0_1.patch` | `patch` | `8699de0908b3853431e2dcac6eb70faad7f326d0` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `lines_empty_total_lines_faf6738_1` | `property_lines_match_model` | `witness_lines_match_model_case_empty` |
| `rope_eq_utf8_boundary_cc516d5_1` | `property_rope_eq_chunk_invariant` | `witness_rope_eq_chunk_invariant_case_non_ascii_boundary` |
| `rope_hash_chunk_boundary_fef5be9_1` | `property_rope_hash_chunk_invariant` | `witness_rope_hash_chunk_invariant_case_ascii_split` |
| `utf16_code_unit_conversion_c0af16b_1` | `property_utf16_char_roundtrip` | `witness_utf16_char_roundtrip_case_latin1` |
| `rope_builder_default_empty_stack_dfcac8b_1` | `property_rope_builder_default_build` | `witness_rope_builder_default_build_case_hello` |
| `slice_crlf_split_end_info_8699de0_1` | `property_slice_crlf_len_lines` | `witness_slice_crlf_len_lines_case_mid_crlf` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `property_lines_match_model` | OK | OK | OK | OK |
| `property_rope_eq_chunk_invariant` | OK | OK | OK | OK |
| `property_rope_hash_chunk_invariant` | OK | OK | OK | OK |
| `property_utf16_char_roundtrip` | OK | OK | OK | OK |
| `property_rope_builder_default_build` | OK | OK | OK | OK |
| `property_slice_crlf_len_lines` | OK | OK | OK | OK |

## Bug Details

### 1. lines_empty_total_lines (faf6738_1)
- **Variant**: `lines_empty_total_lines_faf6738_1`
- **Location**: `src/iter.rs`, `Lines::new_with_range_at` empty-slice branch
- **Property**: `property_lines_match_model`
- **Witness**: `witness_lines_match_model_case_empty`
- **Fix commit**: `faf67387b86584a893d1af60d3a71ab0bd1deac6` — "fix integer overflow when the lines iterator is created for an empty Rope"
- **Invariant violated**: For any `text`, `Rope::from_str(text).lines().len()` equals the slow per-byte line count. An empty rope has exactly one (empty) line.
- **How the mutation triggers**: The empty-slice branch returns `total_lines: 0`. `Lines::size_hint()` computes `total_lines - line_idx`, which both reports length `0` (wrong: want `1`) and underflows once a single `.next()` advances `line_idx` — surfacing as either `.len() == 0` or a debug-assert panic.

### 2. rope_eq_utf8_boundary (cc516d5_1)
- **Variant**: `rope_eq_utf8_boundary_cc516d5_1`
- **Location**: `src/slice.rs`, `RopeSlice: PartialEq<RopeSlice>`
- **Property**: `property_rope_eq_chunk_invariant`
- **Witness**: `witness_rope_eq_chunk_invariant_case_non_ascii_boundary`
- **Fix commit**: `cc516d54037a2f98785dc8cc77d6e6a6201502c3` — "fix: panic when comparing ropes with chunks not aligned at char bounds"
- **Invariant violated**: Comparing `RopeSlice`s obtained from ropes containing non-ASCII text (with at least one modifying operation applied) never panics.
- **How the mutation triggers**: The buggy comparator holds chunks as `&str` and advances via `chunk2[..chunk1.len()]`. When the internal chunks of a `.lines()` slice start mid-scalar (common after `rope.remove(...)` against text with multi-byte chars), that slice index is not a char boundary and `&str` indexing panics. The fix switches the inner comparison to `&[u8]`.

### 3. rope_hash_chunk_boundary (fef5be9_1)
- **Variant**: `rope_hash_chunk_boundary_fef5be9_1`
- **Location**: `src/slice.rs`, `RopeSlice: Hash`
- **Property**: `property_rope_hash_chunk_invariant`
- **Witness**: `witness_rope_hash_chunk_invariant_case_ascii_split`
- **Fix commit**: `fef5be9c52b3bab9d2e4d25fbb66e05ff5ddb8ca` — "Hash slice in chunks of a fixed size to prevent chunk-boundary-dependent hashing"
- **Invariant violated**: Two ropes built from the same text (but with different chunk layouts) produce the same hash under any `Hasher`. `std::hash::Hash` requires identical `Hasher::write` call sequences for equal values.
- **How the mutation triggers**: The buggy implementation calls `state.write(chunk.as_bytes())` once per chunk, so two ropes with the same text but different chunk sizes feed different `write` sequences to the hasher. Under a boundary-sensitive hasher (e.g. `fnv`, which is what `HashMap`s actually use via `hashbrown`), the resulting hashes differ — breaking the `a == b => hash(a) == hash(b)` contract. The fix always hashes in fixed 256-byte blocks regardless of chunk layout.

### 4. utf16_code_unit_conversion (c0af16b_1)
- **Variant**: `utf16_code_unit_conversion_c0af16b_1`
- **Location**: `src/str_utils.rs`, `utf16_code_unit_to_char_idx`
- **Property**: `property_utf16_char_roundtrip`
- **Witness**: `witness_utf16_char_roundtrip_case_latin1`
- **Fix commit**: `c0af16bdf5fef1a8f0a29b13a5dd0d44dd16c7a7` — "Fix utf16_cu_to_char_idx using wrong conversion function"
- **Invariant violated**: For every char index `i` in a rope, `utf16_cu_to_char(char_to_utf16_cu(i)) == i`.
- **How the mutation triggers**: The buggy import aliases `str_indices::utf16::from_byte_idx` as `utf16_code_unit_to_char_idx`, so the function that should map a UTF-16 code unit count back to a char index actually maps a *byte index* to a char index. For any text with non-ASCII chars, the round trip diverges starting at char index 1. The fix uses `str_indices::utf16::to_char_idx`.

### 5. rope_builder_default_empty_stack (dfcac8b_1)
- **Variant**: `rope_builder_default_empty_stack_dfcac8b_1`
- **Location**: `src/rope_builder.rs`, `impl Default for RopeBuilder`
- **Property**: `property_rope_builder_default_build`
- **Witness**: `witness_rope_builder_default_build_case_hello`
- **Fix commit**: `dfcac8b19ee571a2a399e189c12b2a10663ce464` — "Fix broken Default impl for RopeBuilder"
- **Invariant violated**: `RopeBuilder::default()` is functionally equivalent to `RopeBuilder::new()` — appending text and finishing must not panic and must produce a rope whose contents equal the appended text.
- **How the mutation triggers**: The buggy `#[derive(Default)]` constructs `RopeBuilder { stack: SmallVec::new(), .. }` with an empty stack. `RopeBuilder::new()` instead pushes a single empty-leaf node onto the stack; every `append` / `finish` path relies on that initial leaf. With the derived default, `append_leaf_node` panics at `self.stack.pop().unwrap()`, and `finish` underflows on `self.stack.len() - 1`. The fix replaces the derive with a hand-written `impl Default` delegating to `Self::new()`.

### 6. slice_crlf_split_end_info (8699de0_1)
- **Variant**: `slice_crlf_split_end_info_8699de0_1`
- **Location**: `src/slice.rs`, `RopeSlice::new_with_range` `end_info` computation
- **Property**: `property_slice_crlf_len_lines`
- **Witness**: `witness_slice_crlf_len_lines_case_mid_crlf`
- **Fix commit**: `8699de0908b3853431e2dcac6eb70faad7f326d0` — "Fix bug when a slice splits a CRLF pair"
- **Invariant violated**: For every char index `i` in a rope, `rope.slice(..i).len_lines()` matches the slow per-byte line count of `&text[..byte_idx(i)]` — including when `i` lands between a `\r` and a `\n`.
- **How the mutation triggers**: The buggy `end_info` computes `node.char_to_text_info(n_end)` without detecting the slice-boundary CRLF split. When the slice cuts between `\r` and `\n`, the terminating `\r` of the slice no longer has a following `\n` inside the slice; the correct line-break count is one higher than what `char_to_text_info` returns from the unsliced tree. The fix adds `if node.is_crlf_split(n_end) { info.line_breaks += 1; }`. The property forces an internal-node rope layout (via `rope_from_str_chunked` with small chunks) so the slice reaches the `RSEnum::Full` branch where the bug lives; a single-leaf rope bypasses it through the `Node::Leaf` early-return.
