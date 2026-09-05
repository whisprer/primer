# Contributing to Primer

Thank you for helping improve Primer.

## Development setup

Primer supports Rust 1.70.0 and current stable Rust. Install both with rustup:

```console
rustup toolchain install stable --component clippy --component rustfmt
rustup toolchain install 1.70.0
```

Clone the repository and create a focused branch:

```console
git clone https://github.com/whisprer/primer.git
cd primer
git switch -c feature/short-description
```

## Required checks

Run these before opening a pull request:

```console
cargo +stable fmt --all -- --check
cargo +stable clippy --all-targets --locked -- -D warnings
cargo +stable test --all-targets --locked
cargo +stable doc --no-deps
cargo +1.70.0 test --all-targets --locked
cargo +stable package --locked
```

Keep the package-content gate exact:

```console
cargo +stable package --list --locked
```

Only canonical package files belong in the `.crate` archive. Historical source,
benchmark output, executables, website assets, and development notes belong in
the private development archive, not this repository.

## Changes to the sieve

- Preserve correct inclusive-limit behaviour for `0..=u64::MAX` wherever the
  requested result can be allocated.
- Add or update tests before changing index arithmetic.
- Do not introduce `unsafe` code.
- Keep runtime dependencies at zero unless a measured, documented benefit
  justifies changing that contract.
- Distinguish segment-buffer memory from bootstrap, result-vector, allocator,
  and process memory.

Performance pull requests should include reproducible commands, at least 25
samples per case, compiler version, target flags, operating system, and hardware
details. Do not publish comparisons gathered with different workloads or
measurement definitions.

## Documentation

Update the relevant public surface when behaviour changes:

- `README.md` for user-facing behaviour;
- `CHANGELOG.md` under `[Unreleased]`;
- Rustdoc on every changed public item;
- `primer --help` when CLI behaviour changes.

Use a clear commit message such as `fix: handle segment boundary` or
`docs: clarify result-vector memory`.
