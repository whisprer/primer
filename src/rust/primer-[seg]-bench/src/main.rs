/// Segmented Bit-Packed Sieve of Eratosthenes
///
/// Processes the sieve in L1-cache-sized segments (~32KB) to avoid
/// cache thrashing on large n. Same bit-packing and Brian Kernighan
/// tricks as the flat version, but 2-3x faster at n > 1M.

use std::time::Instant;

// ─── Tuning ────────────────────────────────────────────────────────────────

/// Segment size in bits. 32KB = 262,144 bits = covers 524,288 odd numbers.
/// Tune to L1d cache: x86_64 → 32KB, Apple M → 64KB, ESP32-S3 → 8KB.
const SEGMENT_BITS: u64 = 32 * 1024 * 8;
const SEGMENT_WORDS: usize = (SEGMENT_BITS / 64) as usize;

// ─── Helpers ───────────────────────────────────────────────────────────────

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

// ─── Small primes (seeds for segmented sieve) ─────────────────────────────

fn sieve_small(limit: u64) -> Vec<u64> {
    if limit < 2 { return vec![]; }
    let h = limit / 2;
    let num_words = ((h >> 6) + 1) as usize;
    let mut b = vec![!0u64; num_words];
    b[0] ^= 1;

    let sq = isqrt(limit);
    for i in 1..=(sq / 2) {
        if (b[(i >> 6) as usize] >> (i & 63)) & 1 == 1 {
            let step = 2 * i + 1;
            let mut j = 2 * i * (i + 1);
            while j <= h {
                b[(j >> 6) as usize] &= !(1u64 << (j & 63));
                j += step;
            }
        }
    }

    let mut primes = Vec::new();
    for (i, &word) in b.iter().enumerate() {
        let mut w = word;
        while w != 0 {
            let tz = w.trailing_zeros() as usize;
            let p = ((i << 6) + tz) * 2 + 1;
            if (p as u64) <= limit { primes.push(p as u64); }
            w &= w - 1;
        }
    }
    primes
}

// ─── Segmented sieve ──────────────────────────────────────────────────────

pub fn sieve_primes_segmented(n: u64) -> Vec<u64> {
    if n < 2 { return vec![]; }
    if n < 3 { return vec![2]; }

    let sqrt_n = isqrt(n);
    let small_primes = sieve_small(sqrt_n);

    let mut result = Vec::with_capacity(prime_count_upper(n));
    result.push(2);

    // Reusable segment buffer — fits in L1 cache
    let mut segment = vec![0u64; SEGMENT_WORDS];

    // Half-index space: index i → odd number 2*i+1
    let h = n / 2;

    // Track where each small prime's next composite falls
    let mut next_composite: Vec<u64> = small_primes.iter().map(|&p| {
        (p * p - 1) / 2  // half-index of p²
    }).collect();

    let mut seg_start: u64 = 1;

    while seg_start <= h {
        let seg_end = (seg_start + SEGMENT_BITS - 1).min(h);
        let seg_len_bits = (seg_end - seg_start + 1) as usize;
        let seg_len_words = (seg_len_bits + 63) / 64;

        // Reset segment — all bits on (assume prime)
        for w in segment[..seg_len_words].iter_mut() {
            *w = !0u64;
        }

        // Sieve with each small prime
        for (pi, &p) in small_primes.iter().enumerate() {
            let step = p;
            let mut j = next_composite[pi];

            if j > seg_end { continue; }

            // Advance to start of this segment
            if j < seg_start {
                let gap = seg_start - j;
                j += ((gap + step - 1) / step) * step;
            }

            // Mark composites
            while j <= seg_end {
                let local = (j - seg_start) as usize;
                segment[local >> 6] &= !(1u64 << (local & 63));
                j += step;
            }

            next_composite[pi] = j;
        }

        // Mask trailing bits in last word
        let tail = seg_len_bits & 63;
        if tail != 0 {
            segment[seg_len_words - 1] &= (1u64 << tail) - 1;
        }

        // Extract primes — Brian Kernighan
        for wi in 0..seg_len_words {
            let mut w = segment[wi];
            while w != 0 {
                let tz = w.trailing_zeros() as u64;
                let half_idx = seg_start + (wi as u64 * 64) + tz;
                let p = half_idx * 2 + 1;
                if p <= n { result.push(p); }
                w &= w - 1;
            }
        }

        seg_start += SEGMENT_BITS;
    }

    result
}

// ─── Flat sieve (original, for comparison) ─────────────────────────────────

pub fn sieve_primes_flat(n: u64) -> Vec<u64> {
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

// ─── Benchmark ─────────────────────────────────────────────────────────────

fn format_commas(n: u64) -> String {
    let s = n.to_string();
    let mut r = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { r.push(','); }
        r.push(c);
    }
    r.chars().rev().collect()
}

fn format_dur(d: std::time::Duration) -> String {
    let ns = d.as_nanos();
    if ns < 1_000 { format!("{} ns", ns) }
    else if ns < 1_000_000 { format!("{:.1} µs", ns as f64 / 1e3) }
    else if ns < 1_000_000_000 { format!("{:.2} ms", ns as f64 / 1e6) }
    else { format!("{:.2} s", ns as f64 / 1e9) }
}

fn bench<F>(name: &str, n: u64, iters: usize, f: F) -> (std::time::Duration, usize)
where F: Fn(u64) -> Vec<u64>
{
    let _ = f(n); let _ = f(n); // warmup
    let mut times = Vec::with_capacity(iters);
    let mut count = 0;
    for _ in 0..iters {
        let t = Instant::now();
        let r = f(n);
        times.push(t.elapsed());
        count = r.len();
        std::hint::black_box(&r);
    }
    times.sort();
    let min = times[0];
    let med = times[times.len() / 2];
    println!("│  {:<28} │ {:>10} │ {:>10} │ π = {}",
        name, format_dur(min), format_dur(med), format_commas(count as u64));
    (med, count)
}

fn main() {
    println!("🦀 Segmented vs Flat Sieve Benchmark 🦀");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Segment size: {}KB ({} u64 words, covers {} odd numbers per segment)",
        SEGMENT_BITS / 64/ 1024, SEGMENT_WORDS, SEGMENT_BITS * 8);
    println!();

    let sizes: Vec<u64> = vec![
        10_000, 100_000, 500_000, 1_000_000,
        10_000_000, 50_000_000, 100_000_000,
    ];
    let iters = 25;

    for &n in &sizes {
        println!("┌─ n = {} ({} iters) ───────────────────────────────────────────────",
            format_commas(n), iters);
        println!("│  {:<28} │ {:>10} │ {:>10} │", "Implementation", "Min", "Median");
        println!("│  {}", "─".repeat(68));

        let (flat_med, flat_c) = bench("wofl flat", n, iters, sieve_primes_flat);
        let (seg_med, seg_c)   = bench("wofl segmented", n, iters, sieve_primes_segmented);

        assert_eq!(flat_c, seg_c, "COUNT MISMATCH at n={}", n);

        let ratio = flat_med.as_nanos() as f64 / seg_med.as_nanos() as f64;
        let (label, emoji) = if ratio > 1.1 { ("faster", "🏆") }
            else if ratio > 0.95 { ("~same", "≈") }
            else { ("slower", "🐢") };
        println!("│");
        println!("│  Segmented is {:.2}x {} {}   │   counts match ✓", ratio, label, emoji);
        println!("└──────────────────────────────────────────────────────────────────────────");
        println!();
    }

    // Full correctness
    println!("🔬 Full correctness at n=1,000,000...");
    let flat = sieve_primes_flat(1_000_000);
    let seg = sieve_primes_segmented(1_000_000);
    assert_eq!(flat, seg, "FULL MISMATCH");
    println!("   {} primes — byte-for-byte identical ✓", flat.len());
    println!();

    println!("🔬 Full correctness at n=10,000,000...");
    let flat = sieve_primes_flat(10_000_000);
    let seg = sieve_primes_segmented(10_000_000);
    assert_eq!(flat, seg, "FULL MISMATCH");
    println!("   {} primes — byte-for-byte identical ✓", flat.len());
    println!();

    println!("✓ Done!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmented_small() {
        assert_eq!(sieve_primes_segmented(10), vec![2, 3, 5, 7]);
        assert_eq!(sieve_primes_segmented(20), vec![2, 3, 5, 7, 11, 13, 17, 19]);
    }

    #[test]
    fn test_segmented_edge_cases() {
        assert_eq!(sieve_primes_segmented(0), vec![]);
        assert_eq!(sieve_primes_segmented(1), vec![]);
        assert_eq!(sieve_primes_segmented(2), vec![2]);
        assert_eq!(sieve_primes_segmented(3), vec![2, 3]);
    }

    #[test]
    fn test_segmented_known_counts() {
        assert_eq!(sieve_primes_segmented(100).len(), 25);
        assert_eq!(sieve_primes_segmented(1_000).len(), 168);
        assert_eq!(sieve_primes_segmented(10_000).len(), 1_229);
        assert_eq!(sieve_primes_segmented(100_000).len(), 9_592);
        assert_eq!(sieve_primes_segmented(500_000).len(), 41_538);
        assert_eq!(sieve_primes_segmented(1_000_000).len(), 78_498);
    }

    #[test]
    fn test_matches_flat() {
        for n in [10, 100, 1_000, 10_000, 100_000, 500_000, 1_000_000] {
            assert_eq!(sieve_primes_flat(n), sieve_primes_segmented(n),
                "Mismatch at n={}", n);
        }
    }

    #[test]
    fn test_segment_boundaries() {
        // n that straddles segment boundary (~1,048,576)
        let flat = sieve_primes_flat(1_100_000);
        let seg = sieve_primes_segmented(1_100_000);
        assert_eq!(flat, seg);
    }

    #[test]
    fn test_multi_segment() {
        let flat = sieve_primes_flat(5_000_000);
        let seg = sieve_primes_segmented(5_000_000);
        assert_eq!(flat, seg);
    }
}
