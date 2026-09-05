# Primer

**A prime sieve built to live in cache.**

Primer is a zero-dependency Rust library and command-line program for
enumerating every prime up to an inclusive `u64` limit. It uses an odd-only,
bit-packed, segmented Sieve of Eratosthenes and reuses a 32 KiB segment.

## Install

The crates.io package is named `primer-sieve`; its Rust library crate and CLI
are both named `primer`.

```console
cargo add primer-sieve
```

```rust
use primer::sieve_primes;

fn main() {
    let primes = sieve_primes(1_000_000);

    assert_eq!(primes.len(), 78_498);
    assert_eq!(primes.last(), Some(&999_983));
}
```

To install the command-line program:

```console
cargo install primer-sieve
primer 1_000_000
```

Before the first crates.io release, a Git dependency can be used instead:

```toml
[dependencies]
primer-sieve = { git = "https://github.com/whisprer/primer" }
```

The code still imports the library as `primer`.

## Library API

```rust
use primer::{segmented_sieve, sieve_primes, SEGMENT_BYTES};

assert_eq!(SEGMENT_BYTES, 32 * 1024);
assert_eq!(sieve_primes(30), vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
assert_eq!(segmented_sieve(30), sieve_primes(30));
```

`sieve_primes(limit)` is the primary API. `segmented_sieve(limit)` is retained
as a compatibility name for the historical standalone implementation.

## Command-line usage

```console
primer [LIMIT]
```

`LIMIT` defaults to `500000` and may contain commas or underscores:

```console
primer 50,000,000
primer 50_000_000
primer --help
```

## Memory model

The **32 KiB** figure describes the reusable segment buffer, not total memory.
A call also allocates:

- bootstrap primes up to `sqrt(limit)`;
- the returned `Vec<u64>`, containing every generated prime;
- ordinary allocator and process overhead.

Consequently, very large limits can exhaust memory even though the active sieve
segment remains fixed at 32 KiB. Primer 0.3 is a bulk enumerator, not a
constant-memory stream.

## Design

- one bit per odd candidate in the active segment;
- even candidates omitted after handling `2`;
- fixed, reusable 32 KiB segment;
- set-bit scans using `u64::trailing_zeros`;
- set-bit iteration using `word &= word - 1`;
- zero runtime dependencies;
- no `unsafe` code.

The compiler selects the actual machine instructions according to the target
and enabled CPU features. The release automation does not force
`target-cpu=native`; consumers remain in control of target-specific tuning.

## Scope and safety

Primer is not a primality-proof system, factorisation toolkit, or
cryptographically secure prime generator. Do not use its deterministic output
as secret key material.

The minimum supported Rust version is **1.70.0**.

## Verify

```console
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo doc --no-deps
cargo package --locked
```

Tests cover edge cases, known prime counts, segment crossings, integer square
root boundaries, the public compatibility API, and CLI behaviour.

## Links

- Project site: <https://primercrate.rs/>
- Source and issues: <https://github.com/whisprer/primer>
- API documentation: <https://docs.rs/primer-sieve>

## License

Primer is distributed under the terms in [`LICENSE.md`](LICENSE.md).
