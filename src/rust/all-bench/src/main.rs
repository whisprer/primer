//! Prime Sieve Benchmark Harness v2
//! Compares: wofl bit-packed sieve vs wofl SEGMENTED sieve vs `primes` crate vs `primal` crate
//!
//! Usage: cargo run --release

use std::time::{Duration, Instant};
use std::fmt;

// ─── Shared utilities ──────────────────────────────────────────────────────

#[inline]
fn isqrt(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut x = (n as f64).sqrt() as u64;
    while x > 0 && x.checked_mul(x).map_or(true, |sq| sq > n) { x -= 1; }
    while (x + 1).checked_mul(x + 1).map_or(false, |sq| sq <= n) { x += 1; }
    x
}

#[inline]
fn prime_count_upper(n: u64) -> usize {
    if n < 10 { return 4; }
    let nf = n as f64;
    (nf / nf.ln() * 1.15) as usize + 1
}

// ─── wofl's bit-packed sieve (original, non-segmented) ─────────────────────

fn wofl_sieve(n: u64) -> Vec<u64> {
    if n < 2 { return vec![]; }
    let h = n / 2;
    let num_words = ((h >> 6) + 1) as usize;
    let mut b = vec![!0u64; num_words];
    b[0] ^= 1;
    let sqrt_n = isqrt(n);

    for i in 1..=(sqrt_n / 2) {
        if (b[(i >> 6) as usize] >> (i & 63)) & 1 == 1 {
            let step = 2 * i + 1;
            let mut j = 2 * i * (i + 1);
            while j <= h {
                b[(j >> 6) as usize] &= !(1u64 << (j & 63));
                j += step;
            }
        }
    }

    let mut r = Vec::with_capacity(prime_count_upper(n));
    r.push(2);
    for (i, &word) in b.iter().enumerate() {
        let base = ((i << 6) * 2 + 1) as u64;
        if base > n { break; }
        let mut w = word;
        while w != 0 {
            let tz = w.trailing_zeros() as usize;
            let p = ((i << 6) + tz) * 2 + 1;
            if (p as u64) <= n { r.push(p as u64); }
            w &= w - 1;
        }
    }
    r
}

// ─── wofl's SEGMENTED bit-packed sieve (L1-cache-friendly) ─────────────────
//
// Strategy:
//   1. Small sieve: find all primes up to √n (fits in a few KB)
//   2. Process the full range in L1-sized segments (~32KB = 262144 odd numbers)
//   3. For each segment, strike out composites using the small primes
//   4. Extract surviving primes via Brian Kernighan bit iteration
//
// The segment buffer stays hot in L1 cache the entire time, eliminating
// the cache thrashing that kills the flat sieve at large n.

/// L1 cache segment size in bytes. 32KB is safe for virtually all x86/ARM.
/// Each byte holds 8 bits → 8 odd numbers, so 32KB covers 262,144 odd numbers
/// which spans ~524,288 integers.
const SEGMENT_BYTES: usize = 32 * 1024;
const SEGMENT_BITS: u64 = (SEGMENT_BYTES * 8) as u64;
const SEGMENT_WORDS: usize = SEGMENT_BYTES / 8;

fn wofl_segmented_sieve(n: u64) -> Vec<u64> {
    if n < 2 { return vec![]; }
    if n < 3 { return vec![2]; }

    let sqrt_n = isqrt(n);
    let h = n / 2; // max half-index (odd-only)

    // ── Phase 1: small sieve to find primes ≤ √n ──────────────────────
    // These are the "sieving primes" that we'll use to mark composites
    // in each segment. For n=50M, √n ≈ 7071, so this is tiny.
    let small_primes = wofl_sieve(sqrt_n);

    // Pre-compute sieving state for each small odd prime:
    //   - half_prime: the step size in half-index space (= prime value)
    //   - start offset gets computed per-segment
    // Skip prime=2 since we only track odd numbers
    let small_odd_primes: Vec<u64> = small_primes.iter()
        .copied()
        .filter(|&p| p > 2)
        .collect();

    // ── Phase 2: process segments ──────────────────────────────────────
    let mut result = Vec::with_capacity(prime_count_upper(n));
    result.push(2);

    // Segment buffer — reused across all segments, stays in L1
    let mut seg = vec![0u64; SEGMENT_WORDS];

    // Process in chunks of SEGMENT_BITS half-indices
    let mut lo: u64 = 0; // current segment start (in half-index space)

    while lo <= h {
        let hi = std::cmp::min(lo + SEGMENT_BITS - 1, h); // inclusive end
        let seg_len = (hi - lo + 1) as usize; // actual bits in this segment
        let words_needed = (seg_len + 63) / 64;

        // Reset segment: all bits = 1 (assume prime)
        for w in seg[..words_needed].iter_mut() {
            *w = !0u64;
        }

        // Special case: bit 0 of first segment represents 1 (not prime)
        if lo == 0 {
            seg[0] ^= 1;
        }

        // Strike composites for each sieving prime
        for &p in &small_odd_primes {
            // Find the first half-index ≥ lo that's a composite of p.
            // Number at half-index lo is: 2*lo + 1
            // We want the smallest odd multiple of p that's ≥ max(p*p, 2*lo+1)
            let start_half = (p * p - 1) / 2;

            // Find first composite half-index ≥ lo
            let first = if start_half >= lo {
                start_half
            } else {
                // Find remainder and adjust
                let offset = (lo - start_half) % p;
                if offset == 0 { lo } else { lo + p - offset }
            };

            // Strike all multiples within this segment
            let mut j = first;
            while j <= hi {
                let local = (j - lo) as usize;
                seg[local >> 6] &= !(1u64 << (local & 63));
                j += p;
            }
        }

        // Mask off any trailing bits beyond h in the last segment
        if hi == h && seg_len % 64 != 0 {
            let last_word = words_needed - 1;
            let valid_bits = seg_len % 64;
            seg[last_word] &= (1u64 << valid_bits) - 1;
        }

        // Extract primes from this segment
        for (wi, &word) in seg[..words_needed].iter().enumerate() {
            let mut w = word;
            while w != 0 {
                let tz = w.trailing_zeros() as usize;
                let half_idx = lo as usize + (wi << 6) + tz;
                let p = (half_idx * 2 + 1) as u64;
                if p <= n {
                    result.push(p);
                }
                w &= w - 1;
            }
        }

        lo += SEGMENT_BITS;
    }

    result
}

// ─── Wrappers for crate implementations ────────────────────────────────────

fn primes_crate_sieve(n: u64) -> Vec<u64> {
    use primes::{Sieve, PrimeSet};
    let mut sieve = Sieve::new();
    sieve.iter().take_while(|&p| p <= n).collect()
}

fn primal_crate_sieve(n: u64) -> Vec<u64> {
    primal::Primes::all()
        .take_while(|&p| p <= n as usize)
        .map(|p| p as u64)
        .collect()
}

fn primal_crate_sieve_direct(n: u64) -> Vec<u64> {
    let sieve = primal::Sieve::new(n as usize);
    sieve.primes_from(0)
        .take_while(|&p| p <= n as usize)
        .map(|p| p as u64)
        .collect()
}

// ─── Benchmarking machinery ────────────────────────────────────────────────

#[allow(dead_code)]
struct BenchResult {
    name: String,
    n: u64,
    prime_count: usize,
    times: Vec<Duration>,
    sieve_bytes: usize,
    result_bytes: usize,
}

impl BenchResult {
    fn median(&self) -> Duration {
        let mut sorted: Vec<Duration> = self.times.clone();
        sorted.sort();
        sorted[sorted.len() / 2]
    }

    fn min(&self) -> Duration {
        *self.times.iter().min().unwrap()
    }

    fn max(&self) -> Duration {
        *self.times.iter().max().unwrap()
    }

    fn mean(&self) -> Duration {
        let total: Duration = self.times.iter().sum();
        total / self.times.len() as u32
    }

    fn stddev_us(&self) -> f64 {
        let mean = self.mean().as_nanos() as f64;
        let variance = self.times.iter()
            .map(|t| {
                let diff = t.as_nanos() as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / self.times.len() as f64;
        variance.sqrt() / 1000.0
    }
}

impl fmt::Display for BenchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<24} │ {:>10} │ {:>10} │ {:>10} │ {:>10} │ {:>8.1} │ {:>8} │ {:>8}",
            self.name,
            format_duration(self.min()),
            format_duration(self.median()),
            format_duration(self.mean()),
            format_duration(self.max()),
            self.stddev_us(),
            format_bytes(self.sieve_bytes),
            format_bytes(self.result_bytes),
        )
    }
}

fn format_duration(d: Duration) -> String {
    let nanos = d.as_nanos();
    if nanos < 1_000 {
        format!("{} ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.1} µs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2} ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2} s", nanos as f64 / 1_000_000_000.0)
    }
}

fn format_bytes(b: usize) -> String {
    if b < 1024 {
        format!("{} B", b)
    } else if b < 1024 * 1024 {
        format!("{:.1} KB", b as f64 / 1024.0)
    } else {
        format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
    }
}

fn bench<F>(name: &str, n: u64, iterations: usize, sieve_bytes: usize, f: F) -> BenchResult
where
    F: Fn(u64) -> Vec<u64>,
{
    // Warmup
    let _ = f(n);
    let _ = f(n);

    let mut times = Vec::with_capacity(iterations);
    let mut prime_count = 0;
    let mut result_bytes = 0;

    for _ in 0..iterations {
        let start = Instant::now();
        let result = f(n);
        let elapsed = start.elapsed();
        prime_count = result.len();
        result_bytes = result.capacity() * std::mem::size_of::<u64>();
        times.push(elapsed);
        std::hint::black_box(&result);
    }

    BenchResult {
        name: name.to_string(),
        n,
        prime_count,
        times,
        sieve_bytes,
        result_bytes,
    }
}

fn print_header() {
    println!("{:<24} │ {:>10} │ {:>10} │ {:>10} │ {:>10} │ {:>8} │ {:>8} │ {:>8}",
        "Implementation", "Min", "Median", "Mean", "Max", "σ (µs)", "Sieve", "Result");
    println!("{}", "─".repeat(115));
}

fn main() {
    println!("🦀 Prime Sieve Benchmark v2 — now with segmented sieve! 🦀");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Segment size: {} KB (L1 cache line)", SEGMENT_BYTES / 1024);
    println!();

    let test_sizes: Vec<u64> = vec![
        10_000,
        100_000,
        500_000,
        1_000_000,
        10_000_000,
        50_000_000,
        100_000_000,
    ];

    let iterations = 25;

    for &n in &test_sizes {
        println!("┌─ n = {} ({} iterations) ─────────────────────────────────────────────────────",
            format_with_commas(n), iterations);
        println!("│");

        print_header();

        // wofl flat sieve
        let wofl_flat_mem = ((n / 2 / 64 + 1) * 8) as usize;
        let wofl_flat = bench("wofl (flat)", n, iterations, wofl_flat_mem, wofl_sieve);
        println!("{}", wofl_flat);

        // wofl segmented sieve
        let wofl_seg_mem = SEGMENT_BYTES; // only ever uses one segment buffer
        let wofl_seg = bench("wofl (segmented)", n, iterations, wofl_seg_mem, wofl_segmented_sieve);
        println!("{}", wofl_seg);

        // primes crate (skip for large n — it's painfully slow)
        let primes_res = if n <= 1_000_000 {
            Some(bench("primes crate (iter)", n, iterations, 0, primes_crate_sieve))
        } else {
            None
        };
        if let Some(ref r) = primes_res {
            println!("{}", r);
        } else {
            println!("{:<24} │ {:>10} │ {:>10} │ {:>10} │ {:>10} │ {:>8} │ {:>8} │ {:>8}",
                "primes crate (iter)", "—", "skipped", "(too slow", "for n>1M)", "—", "—", "—");
        }

        // primal iterator
        let primal_iter = bench("primal (iterator)", n, iterations, 0, primal_crate_sieve);
        println!("{}", primal_iter);

        // primal direct sieve
        let primal_sieve_mem = n as usize / 8;
        let primal_direct = bench("primal (Sieve::new)", n, iterations, primal_sieve_mem, primal_crate_sieve_direct);
        println!("{}", primal_direct);

        // Verify all implementations agree
        assert_eq!(wofl_flat.prime_count, wofl_seg.prime_count,
            "MISMATCH at n={}: flat={} vs segmented={}", n, wofl_flat.prime_count, wofl_seg.prime_count);
        if let Some(ref r) = primes_res {
            assert_eq!(wofl_flat.prime_count, r.prime_count,
                "MISMATCH at n={}: wofl={} vs primes={}", n, wofl_flat.prime_count, r.prime_count);
        }
        assert_eq!(wofl_flat.prime_count, primal_iter.prime_count,
            "MISMATCH at n={}: wofl={} vs primal_iter={}", n, wofl_flat.prime_count, primal_iter.prime_count);
        assert_eq!(wofl_flat.prime_count, primal_direct.prime_count,
            "MISMATCH at n={}: wofl={} vs primal_direct={}", n, wofl_flat.prime_count, primal_direct.prime_count);

        // Summary — find fastest
        let mut all: Vec<(&str, Duration)> = vec![
            ("wofl flat", wofl_flat.median()),
            ("wofl segmented", wofl_seg.median()),
            ("primal iter", primal_iter.median()),
            ("primal sieve", primal_direct.median()),
        ];
        if let Some(ref r) = primes_res {
            all.push(("primes crate", r.median()));
        }
        let fastest = all.iter().map(|(_, d)| *d).min().unwrap();

        println!("│");
        println!("│  π({}) = {}   │  All implementations agree ✓",
            format_with_commas(n), format_with_commas(wofl_flat.prime_count as u64));

        for (name, time) in &all {
            let ratio = time.as_nanos() as f64 / fastest.as_nanos() as f64;
            if ratio <= 1.01 {
                println!("│  {:16} : fastest 🏆", name);
            } else {
                println!("│  {:16} : {:.2}x slower", name, ratio);
            }
        }

        // Segmented vs flat speedup
        let seg_speedup = wofl_flat.median().as_nanos() as f64 / wofl_seg.median().as_nanos() as f64;
        if seg_speedup > 1.05 {
            println!("│  ⚡ segmented is {:.1}x faster than flat", seg_speedup);
        } else if seg_speedup < 0.95 {
            println!("│  ℹ️  flat is {:.1}x faster than segmented (overhead dominates at small n)", 1.0/seg_speedup);
        } else {
            println!("│  ≈ segmented ≈ flat (within noise)");
        }

        println!("│");
        println!("└──────────────────────────────────────────────────────────────────────────────────");
        println!();
    }

    // Memory comparison
    println!("📊 Memory Efficiency @ n=50,000,000");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let n = 50_000_000u64;
    let flat_sieve_bytes = ((n / 2 / 64 + 1) * 8) as usize;
    let seg_sieve_bytes = SEGMENT_BYTES;
    let result = wofl_sieve(n);
    let result_bytes = result.capacity() * 8;
    println!("  flat sieve array:     {:>10}  (entire range in memory)", format_bytes(flat_sieve_bytes));
    println!("  segmented buffer:     {:>10}  (single L1 segment, reused) 🏆", format_bytes(seg_sieve_bytes));
    println!("  result vec (shared):  {:>10}", format_bytes(result_bytes));
    println!("  flat total:           {:>10}", format_bytes(flat_sieve_bytes + result_bytes));
    println!("  segmented total:      {:>10}", format_bytes(seg_sieve_bytes + result_bytes));
    println!("  sieve memory saving:  {:>10.0}x", flat_sieve_bytes as f64 / seg_sieve_bytes as f64);
    println!("  naive bool array:     {:>10}  (comparison)", format_bytes(n as usize));
    println!();
    println!("✓ Benchmark complete!");
}

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
