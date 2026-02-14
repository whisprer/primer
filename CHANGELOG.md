# Changelog

All notable changes to this project will be documented in this file.

The format is inspired by "Keep a Changelog", and this project follows semantic versioning where practical.
## [not numbered] 2026-02-14
### Changed
- Name finalized to `Primer`.

## [0.2.0] - 2026-02-14
### Added
- Segmented sieve implementation (`seg.rs`) — L1-cache-friendly, 32 KB working memory regardless of `n`.
- Benchmark harness comparing flat sieve, segmented sieve, `primes` crate, and `primal` crate.
- Full development writeup (`writeup.md`) covering C++ origins through Rust port to segmented sieve.
- Hacker News submission draft (`hn_submission.md`).

### Changed
- Flat sieve now pre-allocates result vector via prime-counting upper bound (zero reallocs).
- Replaced `f64` square root cast with overflow-safe integer `isqrt()` (correct for all `u64`).
- Hoisted inner-loop step computation (`2 * i + 1`) for guaranteed optimisation in debug builds.
- Early termination in collection phase when word base exceeds `n`.

### Fixed
- Assertion for 10,001st prime: `primes[10_000]` is 104,743, not 104,729 (off-by-one in index).
- Assertion that 499,999 is prime: it is composite. Largest prime ≤ 500,000 is 499,979.

## [0.1.0] - 2026-02-13
### Added
- Bit-packed Sieve of Eratosthenes (`primes.rs`) — Rust port of ultra-compact C++ implementation.
- 64× memory reduction via 1-bit-per-odd-number packing.
- Hardware intrinsics: `trailing_zeros()` compiles to `tzcnt` on x86_64.
- Brian Kernighan bit iteration for prime extraction (skips zero bits entirely).
- Odd-only sieving with hardcoded 2.
- Borrow checker fix guide (`BORROW_CHECKER_FIX.md`).
- README with usage examples, performance tables, and integration patterns.
- Test suite: edge cases, known counts, compact-vs-standard equality.

### Fixed
- N/A

### Changed
- N/A

## [0.0.1] - 2025-XX-XX
### Added
- Original C++ bit-packed sieve implementation (~9 lines of core logic).
- Months of iterative optimisation: bit packing, odd-only sieving, Brian Kernighan extraction.
