# Security Policy


## Supported versions
| Version | Supported |
|--------:|:---------:|
| 0.2.x   | ✅ Active |
| 0.1.x   | ⚠️ Fixes only |


## Reporting a vulnerability
If you discover a potential vulnerability:
1. **Do not** open a public GitHub issue immediately.
2. Use GitHub's **"Report a vulnerability"** feature if enabled, or contact the maintainer(s) directly at security@whispr.dev.

Response target: within **72 hours**.


## Security scope
This repository is a local CLI tool / library that generates prime number tables in memory and optionally prints them to stdout.

Security concerns are primarily:
- **Integer overflow in `isqrt()`** — mitigated via `checked_mul` for all u64 values including `u64::MAX`.
- **Denial-of-service via large `n`** — very large values of `n` will consume proportional memory for the result vector (≈ 8 bytes × π(n)). The sieve working memory is capped at 32 KB (segmented) regardless of `n`.
- **Index out-of-bounds in bit array access** — the sieve uses computed indices into `Vec<u64>`; bounds are enforced by Rust's runtime checks in debug mode and verified by test coverage.
- **`unsafe` usage** — this project uses zero `unsafe` blocks. All memory safety is enforced by the compiler.


## Known limitations
- The project does not attempt to provide sandboxing guarantees; it runs with the permissions of the user executing it.
- No cryptographic guarantees — this is a deterministic sieve, not a cryptographically secure prime generator. Do not use for key generation.
- The `f64` seed in `isqrt()` has precision limits above 2^52, corrected by Newton iteration. This is tested but worth noting for adversarial inputs.


## Verification
Users are encouraged to:
- Run the built-in test suite: `rustc --test seg.rs -o test_seg && ./test_seg`
- Cross-check prime counts against known values (OEIS A000720).
- Inspect, audit, and rebuild from source before use in sensitive environments.
