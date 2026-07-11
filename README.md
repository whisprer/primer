# Primer

**A prime sieve built to live in cache.**

Primer is a zero-dependency Rust library and command-line program for enumerating
prime numbers with an odd-only, bit-packed, segmented Sieve of Eratosthenes.

Its reusable sieve segment is 32 KiB. That segment is cleared and reused as
Primer advances through the requested range, keeping the hottest part of the
algorithm compact and cache-friendly.

## Important memory distinction

**32 KiB is the reusable segment buffer, not total process memory.**

A call also uses memory for:

- bootstrap primes up to `sqrt(limit)`;
- the returned `Vec<u64>`, which stores every generated prime;
- allocator metadata and normal process overhead.

Primer 0.3 establishes a conventional Cargo library interface around the
existing segmented algorithm. Streaming and count-only interfaces can be added
separately without obscuring this packaging release.

## Installation

After the package is published on crates.io:

```console
cargo add primer-sieve --rename primer
```

Until then, use the Git repository:

```toml
[dependencies]
primer = { package = "primer-sieve", git = "https://github.com/whisprer/primer" }
```

## Library usage

```rust
use primer::sieve_primes;

fn main() {
    let primes = sieve_primes(1_000_000);

    assert_eq!(primes.len(), 78_498);
    assert_eq!(primes.last(), Some(&999_983));
}
```

The historical `segmented_sieve(limit)` name remains available as a compatibility
alias. New code should use `sieve_primes(limit)`.

## Command-line usage

Run the included CLI without installing it:

```console
cargo run --release -- 50_000_000
```

Or install the binary from the repository:

```console
cargo install --git https://github.com/whisprer/primer
primer 1_000_000
```

The limit accepts plain digits, underscores, or commas.

## Why Primer is compact

Primer combines:

- one bit per odd candidate in the active segment;
- omission of even candidates after handling `2`;
- a reusable 32 KiB segment;
- target-appropriate trailing-zero scans through Rust's `trailing_zeros`;
- set-bit iteration using the Brian Kernighan bit-clearing technique;
- no runtime dependencies;
- no `unsafe` code.

The compiler decides the exact machine instruction used for bit scans according
to the compilation target and enabled CPU features.

## Scope

Primer currently focuses on **bulk prime enumeration**. It is not yet intended
as a complete number-theory toolkit, a cryptographic prime generator, a
factorisation library, or a constant-memory stream.

## Verification

```console
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo package --allow-dirty
```

Tests cover edge cases, known prime-counting values, segment boundaries, the
historical API alias, integer-square-root boundaries, and cross-checks against
an independent reference sieve.

## Repository layout

The root `src/lib.rs` is the canonical crate implementation.

Historical standalone implementations and benchmark experiments remain under
`src/rust/`, `src/final-package/`, and `primer-crate/` for provenance until a
later archive-only cleanup. They are not included in the crates.io package.

## Benchmarks

The repository contains historical benchmark programs, but their results use
different harnesses and measurement definitions. Primer will publish one
canonical, reproducible benchmark suite before making headline comparisons.

That suite will distinguish:

- reusable sieve-buffer memory;
- bootstrap working memory;
- returned-result storage;
- total observed process memory;
- latency and throughput on clearly identified hardware.

## Links

- Project site: <https://primercrate.rs/>
- Source: <https://github.com/whisprer/primer>
- Issues: <https://github.com/whisprer/primer/issues>

## License

Primer uses the project license in [`LICENSE.md`](LICENSE.md).

