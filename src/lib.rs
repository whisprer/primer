//! Cache-aware prime enumeration with a bit-packed segmented sieve.
//!
//! Primer generates every prime number up to and including a supplied `u64`
//! limit. It tracks only odd candidates, stores one candidate per bit, and
//! reuses a fixed 32 KiB segment while crossing out composites.
//!
//! # Memory model
//!
//! [`SEGMENT_BYTES`] is the size of Primer's reusable segment buffer. It is not
//! the total memory used by a call:
//!
//! - bootstrap primes up to `sqrt(limit)` require additional memory;
//! - the returned `Vec<u64>` stores every generated prime;
//! - normal allocator and process overhead also apply.
//!
//! Applications that do not want to retain every prime will eventually benefit
//! from a streaming API; version 0.3 keeps the original vector-returning
//! behaviour while establishing a conventional Cargo library interface.
//!
//! # Example
//!
//! ```
//! use primer::sieve_primes;
//!
//! let primes = sieve_primes(30);
//! assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
//! ```

#![forbid(unsafe_code)]

use std::mem::size_of;

/// Size, in bytes, of Primer's reusable segmented-sieve buffer.
///
/// The buffer holds one bit for each odd candidate in the current segment.
/// Additional memory is used for bootstrap primes and the returned result
/// vector.
pub const SEGMENT_BYTES: usize = 32 * 1024;

/// Number of odd candidates represented by one segment.
pub const SEGMENT_BITS: u64 = (SEGMENT_BYTES * 8) as u64;

const SEGMENT_WORDS: usize = SEGMENT_BYTES / size_of::<u64>();

/// Generate all primes up to and including `limit`.
///
/// Primer uses an odd-only, bit-packed segmented Sieve of Eratosthenes. The
/// reusable segment buffer is [`SEGMENT_BYTES`] bytes. The returned vector uses
/// additional memory proportional to the number of primes produced.
///
/// # Examples
///
/// ```
/// use primer::sieve_primes;
///
/// assert_eq!(sieve_primes(1), Vec::<u64>::new());
/// assert_eq!(sieve_primes(2), vec![2]);
/// assert_eq!(sieve_primes(20), vec![2, 3, 5, 7, 11, 13, 17, 19]);
/// ```
#[must_use]
pub fn sieve_primes(limit: u64) -> Vec<u64> {
    if limit < 2 {
        return Vec::new();
    }

    if limit < 3 {
        return vec![2];
    }

    let square_root = integer_square_root(limit);
    let highest_half_index = limit / 2;

    let small_primes = small_sieve(square_root);
    let small_odd_primes: Vec<u64> = small_primes
        .into_iter()
        .filter(|&prime| prime > 2)
        .collect();

    let mut result = Vec::with_capacity(prime_count_capacity(limit));
    result.push(2);

    let mut segment = vec![0_u64; SEGMENT_WORDS];
    let mut low_half_index = 0_u64;

    while low_half_index <= highest_half_index {
        let high_half_index = low_half_index
            .saturating_add(SEGMENT_BITS - 1)
            .min(highest_half_index);

        let segment_length = (high_half_index - low_half_index + 1) as usize;
        let words_needed = (segment_length + 63) / 64;

        segment[..words_needed].fill(u64::MAX);

        if low_half_index == 0 {
            segment[0] &= !1;
        }

        for &prime in &small_odd_primes {
            let first_from_square = (prime * prime - 1) / 2;

            let first_in_segment = if first_from_square >= low_half_index {
                first_from_square
            } else {
                let offset = (low_half_index - first_from_square) % prime;
                if offset == 0 {
                    low_half_index
                } else {
                    low_half_index + prime - offset
                }
            };

            let mut composite_half_index = first_in_segment;

            while composite_half_index <= high_half_index {
                let local_index = (composite_half_index - low_half_index) as usize;
                segment[local_index >> 6] &= !(1_u64 << (local_index & 63));
                composite_half_index += prime;
            }
        }

        let trailing_bits = segment_length % 64;
        if high_half_index == highest_half_index && trailing_bits != 0 {
            segment[words_needed - 1] &= (1_u64 << trailing_bits) - 1;
        }

        for (word_index, &word) in segment[..words_needed].iter().enumerate() {
            let mut remaining = word;

            while remaining != 0 {
                let trailing_zeros = u64::from(remaining.trailing_zeros());
                let half_index = low_half_index + ((word_index as u64) << 6) + trailing_zeros;
                let prime = half_index * 2 + 1;

                if prime <= limit {
                    result.push(prime);
                }

                remaining &= remaining - 1;
            }
        }

        if high_half_index == highest_half_index {
            break;
        }

        low_half_index = high_half_index + 1;
    }

    result
}

/// Compatibility name for [`sieve_primes`].
///
/// The historical standalone implementation exported `segmented_sieve`.
/// Keeping this alias makes migration straightforward while `sieve_primes`
/// serves as the primary crate API.
#[inline]
#[must_use]
pub fn segmented_sieve(limit: u64) -> Vec<u64> {
    sieve_primes(limit)
}

fn small_sieve(limit: u64) -> Vec<u64> {
    if limit < 2 {
        return Vec::new();
    }

    let highest_half_index = limit / 2;
    let word_count = ((highest_half_index >> 6) + 1) as usize;
    let mut bits = vec![u64::MAX; word_count];

    bits[0] &= !1;

    let square_root = integer_square_root(limit);

    for half_index in 1..=(square_root / 2) {
        if ((bits[(half_index >> 6) as usize] >> (half_index & 63)) & 1) == 1 {
            let prime = 2 * half_index + 1;
            let mut composite_half_index = 2 * half_index * (half_index + 1);

            while composite_half_index <= highest_half_index {
                bits[(composite_half_index >> 6) as usize] &=
                    !(1_u64 << (composite_half_index & 63));
                composite_half_index += prime;
            }
        }
    }

    let mut result = Vec::with_capacity(prime_count_capacity(limit));
    result.push(2);

    for (word_index, &word) in bits.iter().enumerate() {
        let lowest_value = ((word_index as u64) << 7) + 1;
        if lowest_value > limit {
            break;
        }

        let mut remaining = word;

        while remaining != 0 {
            let trailing_zeros = u64::from(remaining.trailing_zeros());
            let half_index = ((word_index as u64) << 6) + trailing_zeros;
            let prime = half_index * 2 + 1;

            if prime <= limit {
                result.push(prime);
            }

            remaining &= remaining - 1;
        }
    }

    result
}

#[inline]
fn integer_square_root(value: u64) -> u64 {
    if value < 2 {
        return value;
    }

    let mut estimate = (value as f64).sqrt() as u64;

    while estimate > value / estimate {
        estimate -= 1;
    }

    loop {
        let next = estimate + 1;

        if next > value / next {
            break;
        }

        estimate = next;
    }

    estimate
}

#[inline]
fn prime_count_capacity(limit: u64) -> usize {
    if limit < 10 {
        return 4;
    }

    let floating_limit = limit as f64;
    let estimate = floating_limit / floating_limit.ln() * 1.15 + 1.0;
    let maximum_capacity = (isize::MAX as usize) / size_of::<u64>();

    if !estimate.is_finite() || estimate >= maximum_capacity as f64 {
        maximum_capacity
    } else {
        estimate as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_sieve(limit: u64) -> Vec<u64> {
        if limit < 2 {
            return Vec::new();
        }

        let length = usize::try_from(limit + 1).expect("test limit must fit usize");
        let mut prime = vec![true; length];
        prime[0] = false;
        prime[1] = false;

        let square_root = integer_square_root(limit) as usize;
        for candidate in 2..=square_root {
            if prime[candidate] {
                let mut multiple = candidate * candidate;
                while multiple < length {
                    prime[multiple] = false;
                    multiple += candidate;
                }
            }
        }

        prime
            .iter()
            .enumerate()
            .filter_map(|(value, &is_prime)| is_prime.then_some(value as u64))
            .collect()
    }

    #[test]
    fn handles_edge_cases() {
        assert!(sieve_primes(0).is_empty());
        assert!(sieve_primes(1).is_empty());
        assert_eq!(sieve_primes(2), vec![2]);
        assert_eq!(sieve_primes(3), vec![2, 3]);
    }

    #[test]
    fn produces_known_small_sets() {
        assert_eq!(sieve_primes(10), vec![2, 3, 5, 7]);
        assert_eq!(sieve_primes(30), vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    }

    #[test]
    fn produces_known_prime_counts() {
        for (limit, expected_count) in [
            (100, 25),
            (1_000, 168),
            (10_000, 1_229),
            (100_000, 9_592),
            (500_000, 41_538),
            (1_000_000, 78_498),
        ] {
            assert_eq!(
                sieve_primes(limit).len(),
                expected_count,
                "wrong prime count for limit {limit}"
            );
        }
    }

    #[test]
    fn matches_reference_across_small_limits() {
        for limit in 0..=512 {
            assert_eq!(
                sieve_primes(limit),
                reference_sieve(limit),
                "mismatch for limit {limit}"
            );
        }
    }

    #[test]
    fn matches_reference_at_representative_limits() {
        for limit in [1_000, 10_000, 100_000, 500_000, 1_100_000] {
            assert_eq!(
                sieve_primes(limit),
                reference_sieve(limit),
                "mismatch for limit {limit}"
            );
        }
    }

    #[test]
    fn preserves_known_boundaries() {
        let primes = sieve_primes(500_000);
        assert_eq!(primes[9], 29);
        assert_eq!(primes[9_999], 104_729);
        assert_eq!(primes[10_000], 104_743);
        assert_eq!(primes.last(), Some(&499_979));

        assert_eq!(sieve_primes(29).last(), Some(&29));
    }

    #[test]
    fn compatibility_alias_matches_primary_api() {
        for limit in [0, 2, 3, 10, 1_000, 500_000] {
            assert_eq!(segmented_sieve(limit), sieve_primes(limit));
        }
    }

    #[test]
    fn integer_square_root_is_safe_at_boundaries() {
        assert_eq!(integer_square_root(0), 0);
        assert_eq!(integer_square_root(1), 1);
        assert_eq!(integer_square_root(4), 2);
        assert_eq!(integer_square_root(15), 3);
        assert_eq!(integer_square_root(16), 4);
        assert_eq!(integer_square_root(17), 4);
        assert_eq!(integer_square_root(1_u64 << 52), 1_u64 << 26);
        assert_eq!(integer_square_root(u64::MAX), 4_294_967_295);
    }

    #[test]
    fn segment_geometry_is_consistent() {
        assert_eq!(SEGMENT_BYTES, 32 * 1024);
        assert_eq!(SEGMENT_BITS, 262_144);
        assert_eq!(SEGMENT_WORDS * size_of::<u64>(), SEGMENT_BYTES);
    }
}
