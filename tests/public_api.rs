use primer::{segmented_sieve, sieve_primes, SEGMENT_BITS, SEGMENT_BYTES};

#[test]
fn public_api_generates_expected_primes() {
    assert_eq!(
        sieve_primes(50),
        vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
    );
}

#[test]
fn historical_api_name_remains_compatible() {
    assert_eq!(segmented_sieve(100_000), sieve_primes(100_000));
}

#[test]
fn public_segment_constants_are_consistent() {
    assert_eq!(SEGMENT_BYTES, 32 * 1024);
    assert_eq!(SEGMENT_BITS, (SEGMENT_BYTES * 8) as u64);
}
