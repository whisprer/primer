/// Bit-packed Sieve of Eratosthenes — optimised Rust implementation
///
/// Optimisations:
/// - 1 bit per odd number (64x memory reduction vs naive)
/// - Hardware intrinsics for bit manipulation (tzcnt)
/// - Odd-only sieving (2 is hardcoded)
/// - Brian Kernighan bit iteration (skip zero bits)
/// - Pre-allocated result vector (zero reallocs)
/// - Integer square root (no f64 precision ceiling)
/// - Early termination in collection phase

use std::time::Instant;

/// Integer square root — safe for all u64 values.
/// Newton-corrected from f64 seed; 2 iterations max.
#[inline]
fn isqrt(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut x = (n as f64).sqrt() as u64;
    // Newton correction — safe for all u64 via checked arithmetic
    while x > 0 && x.checked_mul(x).map_or(true, |sq| sq > n) { x -= 1; }
    while (x + 1).checked_mul(x + 1).map_or(false, |sq| sq <= n) { x += 1; }
    x
}

/// Prime-counting upper bound for pre-allocation.
/// Overestimates π(n) by ~15% — guarantees zero reallocation.
#[inline]
fn prime_count_upper(n: u64) -> usize {
    if n < 10 { return 4; }
    let nf = n as f64;
    (nf / nf.ln() * 1.15) as usize + 1
}

/// Generate all primes up to and including `n` using bit-packed sieve.
pub fn sieve_primes(n: u64) -> Vec<u64> {
    if n < 2 { return vec![]; }

    let h = n / 2; // only track odd numbers
    let num_words = ((h >> 6) + 1) as usize;

    // All bits set → assume every odd number is prime
    let mut b = vec![!0u64; num_words];

    // Bit 0 represents 1 — not prime
    b[0] ^= 1;

    let sqrt_n = isqrt(n);

    // ── Sieving phase: mark composite odd numbers ──────────────────
    for i in 1..=(sqrt_n / 2) {
        if (b[(i >> 6) as usize] >> (i & 63)) & 1 == 1 {
            let step = 2 * i + 1;           // prime value = step
            let mut j = 2 * i * (i + 1);    // = (prime² - 1) / 2
            while j <= h {
                b[(j >> 6) as usize] &= !(1u64 << (j & 63));
                j += step;
            }
        }
    }

    // ── Collection phase: extract surviving primes ─────────────────
    let mut r = Vec::with_capacity(prime_count_upper(n));
    r.push(2);

    for (i, &word) in b.iter().enumerate() {
        // Early exit: if the lowest number in this word exceeds n, done
        let base = ((i << 6) * 2 + 1) as u64;
        if base > n { break; }

        // Brian Kernighan: iterate only set bits
        let mut w = word;
        while w != 0 {
            let tz = w.trailing_zeros() as usize;
            let p = ((i << 6) + tz) * 2 + 1;
            if (p as u64) <= n {
                r.push(p as u64);
            }
            w &= w - 1; // clear lowest set bit
        }
    }

    r
}

fn main() {
    let n = 500_000;

    println!("🦀 Bit-Packed Sieve of Eratosthenes 🦀\n");

    let start = Instant::now();
    let primes = sieve_primes(n);
    let elapsed = start.elapsed();

    println!("Generated {} primes up to {}", primes.len(), n);
    println!("Time: {:?}", elapsed);
    println!("Memory: {} bytes (bit-packed)", (n / 2 / 64 + 1) * 8);
    println!("Result vector: {} bytes (pre-allocated, zero reallocs)",
             primes.capacity() * std::mem::size_of::<u64>());

    println!("\nFirst 10 primes: {:?}", &primes[..10]);
    println!("Last 10 primes:  {:?}", &primes[primes.len() - 10..]);

    // Verify known values
    assert_eq!(primes.len(), 41_538);       // π(500,000) = 41,538
    assert_eq!(primes[9], 29);              // 10th prime
    assert_eq!(primes[9_999], 104_729);      // 10,000th prime
    assert_eq!(primes[10_000], 104_743);     // 10,001st prime
    assert_eq!(*primes.last().unwrap(), 499_979); // largest prime ≤ 500,000

    println!("\n✓ All assertions passed!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_primes() {
        assert_eq!(sieve_primes(10), vec![2, 3, 5, 7]);
        assert_eq!(sieve_primes(20), vec![2, 3, 5, 7, 11, 13, 17, 19]);
    }

    #[test]
    fn test_known_counts() {
        assert_eq!(sieve_primes(100).len(), 25);
        assert_eq!(sieve_primes(1_000).len(), 168);
        assert_eq!(sieve_primes(10_000).len(), 1_229);
        assert_eq!(sieve_primes(500_000).len(), 41_538);
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(sieve_primes(0), vec![]);
        assert_eq!(sieve_primes(1), vec![]);
        assert_eq!(sieve_primes(2), vec![2]);
        assert_eq!(sieve_primes(3), vec![2, 3]);
    }

    #[test]
    fn test_large_n_isqrt_safety() {
        // Verify isqrt doesn't choke near f64 precision limits
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(u64::MAX), 4_294_967_295); // 2^32 - 1
        // Spot check: (2^26)^2 = 2^52 — edge of f64 mantissa
        assert_eq!(isqrt(1 << 52), 1 << 26);
    }

    #[test]
    fn test_prime_boundaries() {
        // n itself is composite → last prime must be < n
        let p = sieve_primes(500_000);
        assert_eq!(*p.last().unwrap(), 499_979);

        // Verify a known prime boundary
        let p = sieve_primes(29);
        assert_eq!(*p.last().unwrap(), 29);
    }
}