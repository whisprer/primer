/// Segmented Bit-Packed Sieve of Eratosthenes
/// L1-cache-friendly, zero dependencies, standalone binary
///
/// Compile: rustc -C opt-level=3 -C target-cpu=native seg.rs
/// Run:     ./seg

use std::time::Instant;

// ─── Configuration ─────────────────────────────────────────────────────────

/// L1 cache segment size in bytes. 32KB is safe for virtually all x86/ARM.
/// Each byte holds 8 bits → 8 odd numbers, so 32KB covers 262,144 odd numbers
/// spanning ~524,288 integers per segment.
const SEGMENT_BYTES: usize = 32 * 1024;
const SEGMENT_BITS: u64 = (SEGMENT_BYTES * 8) as u64;
const SEGMENT_WORDS: usize = SEGMENT_BYTES / 8;

// ─── Utilities ─────────────────────────────────────────────────────────────

/// Integer square root — overflow-safe for all u64 values.
#[inline]
fn isqrt(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut x = (n as f64).sqrt() as u64;
    while x > 0 && x.checked_mul(x).map_or(true, |sq| sq > n) { x -= 1; }
    while (x + 1).checked_mul(x + 1).map_or(false, |sq| sq <= n) { x += 1; }
    x
}

/// Upper bound on π(n) for pre-allocation. Overestimates by ~15%.
#[inline]
fn prime_count_upper(n: u64) -> usize {
    if n < 10 { return 4; }
    let nf = n as f64;
    (nf / nf.ln() * 1.15) as usize + 1
}

// ─── Small flat sieve (for bootstrapping primes ≤ √n) ─────────────────────

fn small_sieve(n: u64) -> Vec<u64> {
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

// ─── Segmented sieve ──────────────────────────────────────────────────────
//
// Strategy:
//   1. Small sieve: find all primes ≤ √n (fits in a few KB)
//   2. Process the full range in L1-sized segments (32KB each)
//   3. For each segment, strike composites using the small primes
//   4. Extract surviving primes via Brian Kernighan bit iteration
//
// The segment buffer stays hot in L1 cache, eliminating the thrashing
// that kills flat sieves when the bit array exceeds ~32KB.

pub fn segmented_sieve(n: u64) -> Vec<u64> {
    if n < 2 { return vec![]; }
    if n < 3 { return vec![2]; }

    let sqrt_n = isqrt(n);
    let h = n / 2; // max half-index (odd-only)

    // Phase 1: bootstrap sieving primes ≤ √n
    let small_primes = small_sieve(sqrt_n);
    let small_odd: Vec<u64> = small_primes.into_iter().filter(|&p| p > 2).collect();

    // Phase 2: process range in L1-sized segments
    let mut result = Vec::with_capacity(prime_count_upper(n));
    result.push(2);

    let mut seg = vec![0u64; SEGMENT_WORDS];
    let mut lo: u64 = 0;

    while lo <= h {
        let hi = std::cmp::min(lo + SEGMENT_BITS - 1, h);
        let seg_len = (hi - lo + 1) as usize;
        let words_needed = (seg_len + 63) / 64;

        // Reset: all bits = 1 (assume prime)
        for w in seg[..words_needed].iter_mut() {
            *w = !0u64;
        }

        // Bit 0 of first segment = number 1, not prime
        if lo == 0 {
            seg[0] ^= 1;
        }

        // Strike composites for each sieving prime
        for &p in &small_odd {
            let start_half = (p * p - 1) / 2;

            let first = if start_half >= lo {
                start_half
            } else {
                let offset = (lo - start_half) % p;
                if offset == 0 { lo } else { lo + p - offset }
            };

            let mut j = first;
            while j <= hi {
                let local = (j - lo) as usize;
                seg[local >> 6] &= !(1u64 << (local & 63));
                j += p;
            }
        }

        // Mask trailing bits in last segment
        if hi == h && seg_len % 64 != 0 {
            let valid_bits = seg_len % 64;
            seg[words_needed - 1] &= (1u64 << valid_bits) - 1;
        }

        // Extract primes (Brian Kernighan: iterate only set bits)
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

// ─── Main ──────────────────────────────────────────────────────────────────

fn main() {
    let n = 500_000;

    println!("🦀 Segmented Bit-Packed Sieve of Eratosthenes 🦀\n");
    println!("Segment size: {} KB (L1 cache line)", SEGMENT_BYTES / 1024);

    let start = Instant::now();
    let primes = segmented_sieve(n);
    let elapsed = start.elapsed();

    println!("Generated {} primes up to {}", primes.len(), n);
    println!("Time: {:?}", elapsed);
    println!("Sieve memory: {} bytes (single reused segment)", SEGMENT_BYTES);
    println!("Result vector: {} bytes ({} × 8)",
             primes.capacity() * 8, primes.capacity());

    println!("\nFirst 10 primes: {:?}", &primes[..10]);
    println!("Last 10 primes:  {:?}", &primes[primes.len() - 10..]);

    // Verify known values
    assert_eq!(primes.len(), 41_538);
    assert_eq!(primes[9], 29);
    assert_eq!(primes[9_999], 104_729);
    assert_eq!(primes[10_000], 104_743);
    assert_eq!(*primes.last().unwrap(), 499_979);

    println!("\n✓ All assertions passed!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_primes() {
        assert_eq!(segmented_sieve(10), vec![2, 3, 5, 7]);
        assert_eq!(segmented_sieve(20), vec![2, 3, 5, 7, 11, 13, 17, 19]);
    }

    #[test]
    fn test_known_counts() {
        assert_eq!(segmented_sieve(100).len(), 25);
        assert_eq!(segmented_sieve(1_000).len(), 168);
        assert_eq!(segmented_sieve(10_000).len(), 1_229);
        assert_eq!(segmented_sieve(500_000).len(), 41_538);
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(segmented_sieve(0), vec![]);
        assert_eq!(segmented_sieve(1), vec![]);
        assert_eq!(segmented_sieve(2), vec![2]);
        assert_eq!(segmented_sieve(3), vec![2, 3]);
    }

    #[test]
    fn test_boundary_primes() {
        let p = segmented_sieve(29);
        assert_eq!(*p.last().unwrap(), 29);

        let p = segmented_sieve(500_000);
        assert_eq!(*p.last().unwrap(), 499_979);
    }

    #[test]
    fn test_isqrt_safety() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(u64::MAX), 4_294_967_295);
        assert_eq!(isqrt(1 << 52), 1 << 26);
    }

    #[test]
    fn test_matches_flat_sieve() {
        for n in [10, 100, 1_000, 10_000, 100_000, 500_000] {
            assert_eq!(small_sieve(n), segmented_sieve(n),
                "Mismatch at n={}", n);
        }
    }
}
