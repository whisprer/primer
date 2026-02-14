# Contributing

Thanks for your interest in improving **bit-packed-sieve**!


## Code style
- Follow Rust 2021 edition conventions.
- Run `cargo fmt` before commits (if using Cargo), or ensure consistent formatting for single-file builds.
- Resolve all warnings: `rustc -C opt-level=3 seg.rs` should produce zero warnings.
- Keep logic minimal — this project is deliberately dependency-free and single-file where possible.
- Prefer explicit loops over iterator chains when mutation is involved (see `BORROW_CHECKER_FIX.md`).


## Development workflow
1. Fork the repository.
2. Create a feature branch:
```bash
git checkout -b feature/your-feature
```

3. Commit changes with clear messages (examples):
```
feat: add wheel-30 factorisation to segmented sieve
fix: handle n=2 edge case in isqrt
perf: reduce collection phase allocations
docs: add ESP32 integration example
```

4. Push and open a Pull Request against `main`.


## Building & testing

### Standalone (no Cargo required)
```bash
# Compile with full optimisations
rustc -C opt-level=3 -C target-cpu=native seg.rs -o seg

# Run
./seg

# Run tests
rustc --test seg.rs -o test_seg && ./test_seg
```

### With Cargo (for benchmarks)
```bash
cargo build --release
cargo run --release
cargo test
```

### Manual verification
- Cross-check prime counts against known values: π(100) = 25, π(1,000) = 168, π(10,000) = 1,229, π(500,000) = 41,538.
- Verify the `test_matches_flat_sieve` test passes — this confirms segmented and flat implementations produce identical output.
- When changing sieve logic, run the benchmark harness to confirm no performance regression.


## Performance contributions
If you're submitting a performance improvement:
- Include before/after benchmark numbers (25+ iterations, release mode, `target-cpu=native`).
- Specify the hardware and `rustc --version` used.
- Ensure all existing tests still pass.
- Keep the single-file, zero-dependency constraint unless there's a compelling reason to break it.


## Documentation
If you add or change functionality, please update:
- `README.md` — usage examples and performance tables.
- `CHANGELOG.md` — under an `[Unreleased]` section.
- Inline doc comments (`///`) on public functions.
- `--help` output if a CLI interface is added.


## Communication
Open a GitHub Issue for:
- Bug reports (include `n`, expected vs actual prime count, `rustc --version`)
- Feature requests
- Questions / clarifications
