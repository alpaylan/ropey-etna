// ETNA workload runner for ropey.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: LinesMatchModel | RopeEqChunkInvariant |
//             RopeHashChunkInvariant | Utf16CharRoundtrip | All
//
// Every invocation prints exactly one JSON line to stdout and exits 0
// (except argv parsing, which exits 2).

use crabcheck::quickcheck as crabcheck_qc;
use crabcheck::quickcheck::Arbitrary as CcArbitrary;
use hegel::{generators as hgen, HealthCheck, Hegel, Settings as HegelSettings, TestCase};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestError, TestRunner};
use quickcheck::{Arbitrary as QcArbitrary, Gen, QuickCheck, ResultStatus, TestResult};
use rand::Rng;
use ropey::etna::{
    property_lines_match_model, property_rope_eq_chunk_invariant,
    property_rope_hash_chunk_invariant, property_utf16_char_roundtrip, PropertyResult,
};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &[
    "LinesMatchModel",
    "RopeEqChunkInvariant",
    "RopeHashChunkInvariant",
    "Utf16CharRoundtrip",
];

fn cases_budget() -> u64 {
    std::env::var("ETNA_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(40_000_000)
}

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if let Err(e) = r {
            return (Err(e), total);
        }
    }
    (Ok(()), total)
}

// ---------- Canonical witness builders ----------

fn canonical_empty_text() -> String {
    String::new()
}

fn canonical_non_ascii_text() -> String {
    "aéaéaéaéaéaéaéaéaé".to_string()
}

fn canonical_ascii_chunking_text() -> String {
    "Hello world, this is a rope that spans multiple chunks.".to_string()
}

fn canonical_latin1_text() -> String {
    "éé".to_string()
}

fn check_lines_match_model() -> Result<(), String> {
    to_err(property_lines_match_model(canonical_empty_text()))
}

fn check_rope_eq_chunk_invariant() -> Result<(), String> {
    to_err(property_rope_eq_chunk_invariant(
        canonical_non_ascii_text(),
        3,
        5,
    ))
}

fn check_rope_hash_chunk_invariant() -> Result<(), String> {
    to_err(property_rope_hash_chunk_invariant(
        canonical_ascii_chunking_text(),
        7,
        11,
    ))
}

fn check_utf16_char_roundtrip() -> Result<(), String> {
    to_err(property_utf16_char_roundtrip(canonical_latin1_text()))
}

// ---------- etna (deterministic witness replay) ----------

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "LinesMatchModel" => check_lines_match_model(),
        "RopeEqChunkInvariant" => check_rope_eq_chunk_invariant(),
        "RopeHashChunkInvariant" => check_rope_hash_chunk_invariant(),
        "Utf16CharRoundtrip" => check_utf16_char_roundtrip(),
        _ => {
            return (
                Err(format!("Unknown property for etna: {property}")),
                Metrics::default(),
            );
        }
    };
    (
        result,
        Metrics {
            inputs: 1,
            elapsed_us: t0.elapsed().as_micros(),
        },
    )
}

// ---------- shared Arbitrary-biased generators (qc + cc) ----------
//
// Three `Text*` newtype wrappers around `String`, each biased to hit
// a different class of bug:
//   * `TextAny`          — empty ~25% of the time, then short ASCII/Latin-1 mixes.
//   * `TextNonAscii`     — always contains multi-byte UTF-8 chars.
//   * `TextChunky`       — longer strings (to force multi-chunk ropes).
// `SmallChunk` draws a nonzero chunk size in 1..=32 bytes to force chunk
// boundaries to fall inside multi-byte UTF-8 scalars during generation.

#[derive(Clone)]
struct TextAny(String);
#[derive(Clone)]
struct TextNonAscii(String);
#[derive(Clone)]
struct TextChunky(String);
#[derive(Clone, Copy)]
struct SmallChunk(u16);

impl fmt::Debug for TextAny {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl fmt::Debug for TextNonAscii {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl fmt::Debug for TextChunky {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl fmt::Debug for SmallChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl fmt::Display for TextAny {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
impl fmt::Display for TextNonAscii {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
impl fmt::Display for TextChunky {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
impl fmt::Display for SmallChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

// Character pool biased to provoke each family of bug:
//   * ASCII printable for baseline chunking.
//   * Line-ending codepoints (\n, \r, \u{0085}, \u{2028}) for the lines iter.
//   * Latin-1 `é` (2 bytes) and CJK `あ` (3 bytes) / surrogate-pair `𝄞` (4 bytes)
//     for non-ASCII / utf16 paths.
const GEN_CHARS: &[char] = &[
    'a', 'b', 'c', 'x', 'y', 'z', ' ', '.', ',', '0', '9', '\n', '\r', '\u{000B}', '\u{000C}',
    '\u{0085}', '\u{2028}', 'é', 'ñ', 'あ', 'が', '字', '𝄞', '🎉',
];

const NON_ASCII_CHARS: &[char] = &[
    'a', 'b', ' ', 'é', 'ñ', 'ü', 'あ', 'が', '字', '国', '𝄞', '🎉', '\n', '\r',
];

fn random_text_any<R: Rng>(rng: &mut R) -> String {
    // 20% chance of empty string.
    if rng.random_range(0u8..5) == 0 {
        return String::new();
    }
    let len = rng.random_range(1usize..=96);
    (0..len)
        .map(|_| GEN_CHARS[rng.random_range(0..GEN_CHARS.len())])
        .collect()
}

fn random_text_non_ascii<R: Rng>(rng: &mut R) -> String {
    let len = rng.random_range(1usize..=48);
    let mut saw_non_ascii = false;
    let mut s: String = (0..len)
        .map(|_| {
            let c = NON_ASCII_CHARS[rng.random_range(0..NON_ASCII_CHARS.len())];
            if !c.is_ascii() {
                saw_non_ascii = true;
            }
            c
        })
        .collect();
    if !saw_non_ascii {
        s.push('é');
    }
    s
}

fn random_text_chunky<R: Rng>(rng: &mut R) -> String {
    let len = rng.random_range(1usize..=256);
    (0..len)
        .map(|_| GEN_CHARS[rng.random_range(0..GEN_CHARS.len())])
        .collect()
}

fn random_small_chunk<R: Rng>(rng: &mut R) -> u16 {
    // 1..=32 bytes is small enough to land a boundary inside a multi-byte
    // UTF-8 scalar, large enough to exercise multi-chunk ropes.
    rng.random_range(1u16..=32)
}

impl QcArbitrary for TextAny {
    fn arbitrary(g: &mut Gen) -> Self {
        if g.random_range(0u8..5) == 0 {
            return TextAny(String::new());
        }
        let len = g.random_range(1usize..=96);
        let s: String = (0..len)
            .map(|_| GEN_CHARS[g.random_range(0..GEN_CHARS.len())])
            .collect();
        TextAny(s)
    }
}

impl QcArbitrary for TextNonAscii {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = g.random_range(1usize..=48);
        let mut saw_non_ascii = false;
        let mut s: String = (0..len)
            .map(|_| {
                let c = NON_ASCII_CHARS[g.random_range(0..NON_ASCII_CHARS.len())];
                if !c.is_ascii() {
                    saw_non_ascii = true;
                }
                c
            })
            .collect();
        if !saw_non_ascii {
            s.push('é');
        }
        TextNonAscii(s)
    }
}

impl QcArbitrary for TextChunky {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = g.random_range(1usize..=256);
        let s: String = (0..len)
            .map(|_| GEN_CHARS[g.random_range(0..GEN_CHARS.len())])
            .collect();
        TextChunky(s)
    }
}

impl QcArbitrary for SmallChunk {
    fn arbitrary(g: &mut Gen) -> Self {
        SmallChunk(g.random_range(1u16..=32))
    }
}

impl<R: Rng> CcArbitrary<R> for TextAny {
    fn generate(rng: &mut R, _n: usize) -> Self {
        TextAny(random_text_any(rng))
    }
}
impl<R: Rng> CcArbitrary<R> for TextNonAscii {
    fn generate(rng: &mut R, _n: usize) -> Self {
        TextNonAscii(random_text_non_ascii(rng))
    }
}
impl<R: Rng> CcArbitrary<R> for TextChunky {
    fn generate(rng: &mut R, _n: usize) -> Self {
        TextChunky(random_text_chunky(rng))
    }
}
impl<R: Rng> CcArbitrary<R> for SmallChunk {
    fn generate(rng: &mut R, _n: usize) -> Self {
        SmallChunk(random_small_chunk(rng))
    }
}

// ---------- proptest ----------

fn text_any_strategy() -> BoxedStrategy<String> {
    prop_oneof![
        1 => Just(String::new()),
        4 => prop::collection::vec(prop::sample::select(GEN_CHARS.to_vec()), 1..=96)
            .prop_map(|cs| cs.into_iter().collect()),
    ]
    .boxed()
}

fn text_non_ascii_strategy() -> BoxedStrategy<String> {
    prop::collection::vec(prop::sample::select(NON_ASCII_CHARS.to_vec()), 1..=48)
        .prop_map(|cs: Vec<char>| {
            let mut s: String = cs.iter().collect();
            if s.chars().all(|c| c.is_ascii()) {
                s.push('é');
            }
            s
        })
        .boxed()
}

fn text_chunky_strategy() -> BoxedStrategy<String> {
    prop::collection::vec(prop::sample::select(GEN_CHARS.to_vec()), 1..=256)
        .prop_map(|cs: Vec<char>| cs.into_iter().collect())
        .boxed()
}

fn small_chunk_strategy() -> BoxedStrategy<u16> {
    (1u16..=32u16).boxed()
}

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let cfg = ProptestConfig {
        cases: cases_budget() as u32,
        max_shrink_iters: 32,
        failure_persistence: None,
        ..ProptestConfig::default()
    };
    let mut runner = TestRunner::new(cfg);
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "LinesMatchModel" => runner
            .run(&text_any_strategy(), move |text| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = text.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_lines_match_model(text)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({:?})", cex)))
                    }
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "RopeEqChunkInvariant" => runner
            .run(
                &(
                    text_non_ascii_strategy(),
                    small_chunk_strategy(),
                    small_chunk_strategy(),
                ),
                move |(text, a, b)| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let cex_text = text.clone();
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        property_rope_eq_chunk_invariant(text, a, b)
                    }));
                    match outcome {
                        Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                        Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(format!(
                            "({:?} {} {})",
                            cex_text, a, b
                        ))),
                    }
                },
            )
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "RopeHashChunkInvariant" => runner
            .run(
                &(
                    text_chunky_strategy(),
                    small_chunk_strategy(),
                    small_chunk_strategy(),
                ),
                move |(text, a, b)| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let cex_text = text.clone();
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        property_rope_hash_chunk_invariant(text, a, b)
                    }));
                    match outcome {
                        Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                        Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(format!(
                            "({:?} {} {})",
                            cex_text, a, b
                        ))),
                    }
                },
            )
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        "Utf16CharRoundtrip" => runner
            .run(&text_non_ascii_strategy(), move |text| {
                c.fetch_add(1, Ordering::Relaxed);
                let cex = text.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_utf16_char_roundtrip(text)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        Err(TestCaseError::fail(format!("({:?})", cex)))
                    }
                }
            })
            .map_err(|e| match e {
                TestError::Fail(reason, _) => reason.to_string(),
                other => other.to_string(),
            }),
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ---------- quickcheck (forked crate with `etna` feature) ----------

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_lines_match_model(TextAny(text): TextAny) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_lines_match_model(text) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_rope_eq_chunk_invariant(
    TextNonAscii(text): TextNonAscii,
    SmallChunk(a): SmallChunk,
    SmallChunk(b): SmallChunk,
) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_rope_eq_chunk_invariant(text, a, b) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_rope_hash_chunk_invariant(
    TextChunky(text): TextChunky,
    SmallChunk(a): SmallChunk,
    SmallChunk(b): SmallChunk,
) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_rope_hash_chunk_invariant(text, a, b) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_utf16_char_roundtrip(TextNonAscii(text): TextNonAscii) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_utf16_char_roundtrip(text) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let budget = cases_budget();
    let mut qc = QuickCheck::new()
        .tests(budget)
        .max_tests(budget.saturating_mul(2))
        .max_time(Duration::from_secs(86_400));
    let result = match property {
        "LinesMatchModel" => qc.quicktest(qc_lines_match_model as fn(TextAny) -> TestResult),
        "RopeEqChunkInvariant" => qc.quicktest(
            qc_rope_eq_chunk_invariant as fn(TextNonAscii, SmallChunk, SmallChunk) -> TestResult,
        ),
        "RopeHashChunkInvariant" => qc.quicktest(
            qc_rope_hash_chunk_invariant as fn(TextChunky, SmallChunk, SmallChunk) -> TestResult,
        ),
        "Utf16CharRoundtrip" => {
            qc.quicktest(qc_utf16_char_roundtrip as fn(TextNonAscii) -> TestResult)
        }
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!("({})", arguments.join(" "))),
        ResultStatus::Aborted { err } => Err(format!("quickcheck aborted: {err:?}")),
        ResultStatus::TimedOut => Err("quickcheck timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "quickcheck gave up after {} tests",
            result.n_tests_passed
        )),
    };
    (status, Metrics { inputs, elapsed_us })
}

// ---------- crabcheck ----------

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_lines_match_model(TextAny(text): TextAny) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_lines_match_model(text) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_rope_eq_chunk_invariant(
    (TextNonAscii(text), SmallChunk(a), SmallChunk(b)): (TextNonAscii, SmallChunk, SmallChunk),
) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_rope_eq_chunk_invariant(text, a, b) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_rope_hash_chunk_invariant(
    (TextChunky(text), SmallChunk(a), SmallChunk(b)): (TextChunky, SmallChunk, SmallChunk),
) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_rope_hash_chunk_invariant(text, a, b) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_utf16_char_roundtrip(TextNonAscii(text): TextNonAscii) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_utf16_char_roundtrip(text) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let cc_config = crabcheck_qc::Config {
        tests: cases_budget(),
    };
    let result = match property {
        "LinesMatchModel" => crabcheck_qc::quickcheck_with_config(cc_config, cc_lines_match_model),
        "RopeEqChunkInvariant" => {
            crabcheck_qc::quickcheck_with_config(cc_config, cc_rope_eq_chunk_invariant)
        }
        "RopeHashChunkInvariant" => {
            crabcheck_qc::quickcheck_with_config(cc_config, cc_rope_hash_chunk_invariant)
        }
        "Utf16CharRoundtrip" => {
            crabcheck_qc::quickcheck_with_config(cc_config, cc_utf16_char_roundtrip)
        }
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            )
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        }
        crabcheck_qc::ResultStatus::TimedOut => Err("crabcheck timed out".to_string()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "crabcheck gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => {
            Err(format!("crabcheck aborted: {error}"))
        }
    };
    (status, Metrics { inputs, elapsed_us })
}

// ---------- hegel ----------

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new()
        .test_cases(cases_budget())
        .suppress_health_check(HealthCheck::all())
}

fn hg_draw_char<'a>(tc: &TestCase, pool: &'a [char]) -> char {
    let idx = tc.draw(
        hgen::integers::<usize>()
            .min_value(0)
            .max_value(pool.len() - 1),
    );
    pool[idx]
}

fn hg_draw_text_any(tc: &TestCase) -> String {
    // 20% empty, else 1..=96 random chars.
    let empty = tc.draw(hgen::integers::<u8>().min_value(0).max_value(4));
    if empty == 0 {
        return String::new();
    }
    let len = tc.draw(hgen::integers::<usize>().min_value(1).max_value(96));
    (0..len).map(|_| hg_draw_char(tc, GEN_CHARS)).collect()
}

fn hg_draw_text_non_ascii(tc: &TestCase) -> String {
    let len = tc.draw(hgen::integers::<usize>().min_value(1).max_value(48));
    let mut s: String = (0..len).map(|_| hg_draw_char(tc, NON_ASCII_CHARS)).collect();
    if s.chars().all(|c| c.is_ascii()) {
        s.push('é');
    }
    s
}

fn hg_draw_text_chunky(tc: &TestCase) -> String {
    let len = tc.draw(hgen::integers::<usize>().min_value(1).max_value(256));
    (0..len).map(|_| hg_draw_char(tc, GEN_CHARS)).collect()
}

fn hg_draw_small_chunk(tc: &TestCase) -> u16 {
    tc.draw(hgen::integers::<u16>().min_value(1).max_value(32))
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "LinesMatchModel" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let text = hg_draw_text_any(&tc);
                let cex = text.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_lines_match_model(text)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("({:?})", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "RopeEqChunkInvariant" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let text = hg_draw_text_non_ascii(&tc);
                let a = hg_draw_small_chunk(&tc);
                let b = hg_draw_small_chunk(&tc);
                let cex_text = text.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_rope_eq_chunk_invariant(text, a, b)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        panic!("({:?} {} {})", cex_text, a, b)
                    }
                }
            })
            .settings(settings.clone())
            .run();
        }
        "RopeHashChunkInvariant" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let text = hg_draw_text_chunky(&tc);
                let a = hg_draw_small_chunk(&tc);
                let b = hg_draw_small_chunk(&tc);
                let cex_text = text.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_rope_hash_chunk_invariant(text, a, b)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => {
                        panic!("({:?} {} {})", cex_text, a, b)
                    }
                }
            })
            .settings(settings.clone())
            .run();
        }
        "Utf16CharRoundtrip" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let text = hg_draw_text_non_ascii(&tc);
                let cex = text.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_utf16_char_roundtrip(text)
                }));
                match outcome {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("({:?})", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("__unknown_property:{}", property),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "hegel panicked with non-string payload".to_string()
            };
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {rest}")),
                    Metrics::default(),
                );
            }
            Err(msg
                .strip_prefix("Property test failed: ")
                .unwrap_or(&msg)
                .to_string())
        }
    };
    (status, metrics)
}

// ---------- dispatch ----------

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (
            Err(format!("Unknown tool: {tool}")),
            Metrics::default(),
        ),
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!(
            "Properties: LinesMatchModel | RopeEqChunkInvariant | RopeHashChunkInvariant | Utf16CharRoundtrip | All"
        );
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "panic with non-string payload".to_string()
            };
            emit_json(tool, property, "aborted", Metrics::default(), None, Some(&msg));
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(e) => emit_json(tool, property, "failed", metrics, Some(&e), None),
    }
}
