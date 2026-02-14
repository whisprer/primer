//! Prime Sieve Benchmark Harness
//! Compares: wofl bit-packed sieve vs `primes` crate vs `primal` crate
//!
//! Usage: cargo run --release

use std::time::{Duration, Instant};
use std::fmt;

// ─── wofl's bit-packed sieve ───────────────────────────────────────────────

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
    // primal also has a direct sieve that's more optimised
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
    sieve_bytes: usize,   // memory for the sieve structure
    result_bytes: usize,  // memory for the result vec
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
        variance.sqrt() / 1000.0 // convert to µs
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
        // Prevent optimiser from eliding the computation
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
    println!("🦀 Prime Sieve Benchmark 🦀");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let test_sizes: Vec<u64> = vec![
        10_000,
        100_000,
        500_000,
        1_000_000,
        10_000_000,
        50_000_000,
    ];

    let iterations = 25;

    for &n in &test_sizes {
        println!("┌─ n = {} ({} iterations) ─────────────────────────────────────────────────────",
            format_with_commas(n), iterations);
        println!("│");

        print_header();

        // wofl sieve
        let wofl_sieve_mem = ((n / 2 / 64 + 1) * 8) as usize;
        let wofl = bench("wofl (bit-packed)", n, iterations, wofl_sieve_mem, wofl_sieve);
        println!("{}", wofl);

        // primes crate
        let primes_res = bench("primes crate (iter)", n, iterations, 0, primes_crate_sieve);
        println!("{}", primes_res);

        // primal crate (iterator)
        let primal_iter = bench("primal (iterator)", n, iterations, 0, primal_crate_sieve);
        println!("{}", primal_iter);

        // primal crate (direct sieve)
        let primal_sieve_mem = n as usize / 8; // approximate
        let primal_direct = bench("primal (Sieve::new)", n, iterations, primal_sieve_mem, primal_crate_sieve_direct);
        println!("{}", primal_direct);

        // Verify all implementations agree on count
        assert_eq!(wofl.prime_count, primes_res.prime_count,
            "MISMATCH at n={}: wofl={} vs primes={}", n, wofl.prime_count, primes_res.prime_count);
        assert_eq!(wofl.prime_count, primal_iter.prime_count,
            "MISMATCH at n={}: wofl={} vs primal={}", n, wofl.prime_count, primal_iter.prime_count);
        assert_eq!(wofl.prime_count, primal_direct.prime_count,
            "MISMATCH at n={}: wofl={} vs primal_direct={}", n, wofl.prime_count, primal_direct.prime_count);

        // Summary
        println!("│");
        let fastest = wofl.median().min(primes_res.median()).min(primal_iter.median()).min(primal_direct.median());
        println!("│  π({}) = {}   │  All implementations agree ✓", 
            format_with_commas(n), format_with_commas(wofl.prime_count as u64));

        let speedups = [
            ("wofl", wofl.median()),
            ("primes", primes_res.median()),
            ("primal iter", primal_iter.median()),
            ("primal sieve", primal_direct.median()),
        ];
        for (name, time) in &speedups {
            let ratio = time.as_nanos() as f64 / fastest.as_nanos() as f64;
            if ratio <= 1.01 {
                println!("│  {} : fastest 🏆", name);
            } else {
                println!("│  {} : {:.2}x slower", name, ratio);
            }
        }

        println!("│");
        println!("└──────────────────────────────────────────────────────────────────────────────────");
        println!();
    }

    // Memory efficiency comparison at n=10M
    println!("📊 Memory Efficiency @ n=10,000,000");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let n = 10_000_000u64;
    let wofl_sieve_bytes = ((n / 2 / 64 + 1) * 8) as usize;
    let wofl_result = wofl_sieve(n);
    let wofl_result_bytes = wofl_result.capacity() * 8;
    println!("  wofl sieve array:   {:>10}", format_bytes(wofl_sieve_bytes));
    println!("  wofl result vec:    {:>10}", format_bytes(wofl_result_bytes));
    println!("  wofl total:         {:>10}", format_bytes(wofl_sieve_bytes + wofl_result_bytes));
    println!("  naive bool array:   {:>10} (comparison)", format_bytes(n as usize));
    println!("  compression ratio:  {:>10.0}x vs naive", n as f64 / wofl_sieve_bytes as f64);
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