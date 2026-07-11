# Changelog

All notable changes to this project are documented here.

The format is inspired by Keep a Changelog, and the project follows semantic
versioning where practical.

## [0.3.0] - 2026-07-11

### Added

- Conventional root Cargo package for the `primer` library and CLI.
- Canonical public `sieve_primes(limit)` API.
- Compatibility alias for the historical `segmented_sieve(limit)` API.
- Root-level unit tests, public integration tests, and a runnable example.
- Cross-platform GitHub Actions validation.
- Package metadata for `primercrate.rs`, GitHub, docs.rs, and crates.io.
- Explicit documentation of Primer's memory model and scope.

### Changed

- Established `src/lib.rs` as the canonical implementation.
- Moved the demonstration program into `src/bin/primer.rs`.
- Corrected half-index arithmetic to remain in `u64` until local segment
  indexing is required.
- Replaced ambiguous total-memory wording with the precise 32 KiB reusable
  segment-buffer claim.
- Replaced the legacy README with Cargo-first installation and usage guidance.
- Renamed the repository's `gitignore` file to `.gitignore`.

### Preserved

- The segmented algorithm and its 32 KiB segment geometry.
- Historical standalone implementations and benchmark experiments for
  provenance.
- The zero-dependency and no-`unsafe` design.

## [not numbered] - 2026-02-14

### Changed

- Name finalized to `Primer`.

## [0.2.0] - 2026-02-14

### Added

- Segmented sieve implementation (`seg.rs`) with a reusable 32 KiB segment.
- Benchmark harness comparing flat sieve, segmented sieve, `primes`, and
  `primal`.
- Development write-up covering the C++ origin, Rust port, and segmentation.
- Hacker News submission draft.

### Changed

- Flat sieve pre-allocates its result vector using a prime-counting upper bound.
- Replaced a direct floating-point square-root cast with a corrected,
  overflow-safe integer square root.
- Hoisted inner-loop step computation.
- Added early termination in the collection phase.

### Fixed

- Corrected the assertion for the 10,001st prime.
- Corrected the largest-prime-at-500,000 assertion.

## [0.1.0] - 2026-02-13

### Added

- Bit-packed Sieve of Eratosthenes Rust port.
- One-bit-per-odd-candidate representation.
- Trailing-zero scans and Brian Kernighan set-bit iteration.
- Odd-only sieving with `2` handled separately.
- Borrow-checker fix guide.
- Initial usage, performance, and integration documentation.
- Edge-case and known-count tests.

## [0.0.1] - 2025

### Added

- Original compact C++ bit-packed sieve implementation.
- Iterative optimisation work leading to the Rust port.
