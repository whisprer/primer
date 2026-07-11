# Primer Cargo migration: v0.3.0

This migration establishes a conventional Cargo package at the repository root
without deleting the historical source and benchmark material.

## Canonical targets

- Library: `src/lib.rs`
- CLI: `src/bin/primer.rs`
- Example: `examples/basic.rs`
- Public integration tests: `tests/public_api.rs`

## Compatibility

The primary public API is:

```rust
primer::sieve_primes(limit)
```

The historical name remains available:

```rust
primer::segmented_sieve(limit)
```

Both return `Vec<u64>` containing every prime up to and including `limit`.

## Memory terminology

The algorithm reuses a 32 KiB segmented bitset. That is not a claim of 32 KiB
total process memory. Bootstrap primes, the returned vector, allocator metadata,
and process overhead are additional.

## Historical sources

The following are intentionally preserved but excluded from the published
package:

- `primer-crate/`
- `src/final-package/`
- `src/rust/`

A later cleanup may move them into an explicit archive after the new package has
been benchmarked and released.

## Validation performed by the migration script

1. Verify the repository and canonical historical source hash.
2. Rehearse the migration in a temporary local clone.
3. Run formatting checks.
4. Compile all targets.
5. Run all unit, integration, binary, and example tests.
6. Run Clippy with warnings denied.
7. Build and verify the crates.io package archive.
8. Run the release CLI at a limit of 500,000.
9. Create a safety branch.
10. Apply the validated migration on a feature branch.
11. Re-run the core checks in the real repository.

No commit or push is performed automatically.

## Validation compatibility correction

The integer square-root boundary correction uses division rather than checked
multiplication plus `Option::map_or`. This is overflow-safe for all `u64`
inputs, remains compatible with the declared Rust 1.70 MSRV, and avoids
version-dependent Clippy suggestions.
