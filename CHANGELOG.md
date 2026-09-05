# Changelog

All notable changes to Primer are documented here. The project follows semantic
versioning where practical.

## [Unreleased]

## [0.3.0] - 2026-09-04

### Added

- Conventional Cargo package `primer-sieve` with library crate and CLI named
  `primer`.
- Public `sieve_primes(limit)` API.
- Compatibility `segmented_sieve(limit)` API.
- Unit, integration, CLI, documentation, and example coverage.
- CI on Linux, Windows, and macOS, plus a Rust 1.70.0 MSRV check.
- Tag-driven GitHub Release packaging of the canonical `.crate` archive and
  its SHA-256 checksum.

### Changed

- Established `src/lib.rs` as the sole canonical implementation.
- Established `src/bin/primer.rs` as the sole canonical CLI.
- Corrected half-index arithmetic to stay in `u64` until local segment indexing
  is required.
- Replaced ambiguous memory claims with the precise 32 KiB reusable segment
  buffer claim.
- Restricted crates.io contents to the files needed to build, test, document,
  and license the package.
- Removed historical implementations, benchmark experiments, compiled output,
  website material, and conversation notes from the public repository head.

### Security

- Forbid `unsafe` code in the canonical library.
- Build release artifacts without host-specific `target-cpu=native` flags.

## [0.2.0] - 2026-02-14

### Added

- Segmented sieve implementation with a reusable 32 KiB segment.
- Benchmark harness comparing flat, segmented, `primes`, and `primal` sieves.
- Development notes covering the C++ origin, Rust port, and segmentation.

### Fixed

- Corrected the assertion for the 10,001st prime.
- Corrected the largest-prime-at-500,000 assertion.

## [0.1.0] - 2026-02-13

### Added

- Initial odd-only, bit-packed Rust port of the Sieve of Eratosthenes.
- Trailing-zero scans and Brian Kernighan set-bit iteration.
- Edge-case and known-count tests.
