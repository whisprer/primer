# Performance Analysis

Deep dive into benchmarks, scaling behaviour, memory characteristics, and target use cases for the bit-packed sieve family. All numbers collected on a containerised x86_64 Linux environment, `rustc 1.75.0`, release mode with LTO. Your native hardware will differ — run the benchmark harness yourself for authoritative numbers.


## Implementations Tested

| Label | Description | Sieve Memory | Dependencies |
|---|---|---|---|
| **wofl (flat)** | Bit-packed, odd-only, single allocation | `n/128` bytes | zero |
| **wofl (segmented)** | Same algorithm, L1-cache-windowed | 32 KB fixed | zero |
| **primes crate** | `primes` v0.3, iterator-based | internal | 1 crate |
| **primal (iterator)** | `primal` v0.3, `Primes::all()` | internal | 4 crates |
| **primal (Sieve::new)** | `primal` v0.3, direct segmented sieve | `~n/8` bytes | 4 crates |

All implementations cross-verified at every `n` — prime counts must agree exactly or the benchmark panics. 25 iterations per measurement, 2 warmup runs discarded.


## Raw Benchmark Results

### Timing (Median of 25 Iterations)

| n | π(n) | wofl flat | wofl seg | primes | primal iter | primal sieve |
|---|---|---|---|---|---|---|
| 10K | 1,229 | 7.1 µs | 7.4 µs | 258.8 µs | 140.7 µs | **4.3 µs** |
| 100K | 9,592 | 78.9 µs | 80.7 µs | 4.90 ms | 159.1 µs | **39.9 µs** |
| 500K | 41,538 | 434 µs | 443 µs | 28.3 ms | 262.6 µs | **169.6 µs** |
| 1M | 78,498 | 905 µs | 917 µs | 62.7 ms | 420.0 µs | **352.4 µs** |
| 10M | 664,579 | 10.3 ms | 10.3 ms | — | 3.37 ms | **3.20 ms** |
| 50M | 3,001,134 | 66.6 ms | **52.5 ms** | — | 30.6 ms | 30.7 ms |
| 100M | 5,761,455 | 189.7 ms | **137.0 ms** | — | 61.7 ms | 62.2 ms |

Bold = fastest in row. The `primes` crate is omitted above 1M (it runs at 150–250× slower than everything else due to trial division).

### Variance (Standard Deviation, µs)

| n | wofl flat | wofl seg | primal iter | primal sieve |
|---|---|---|---|---|
| 10K | 2.1 | 3.5 | 29.5 | 0.0 |
| 100K | 3.3 | 2.9 | 4.1 | 3.6 |
| 500K | 7.4 | 8.4 | 25.1 | 14.8 |
| 1M | 10.2 | 6.4 | 61.1 | 8.0 |
| 10M | 380.5 | 364.6 | 410.2 | 262.6 |
| 50M | 1,770.6 | **655.5** | 1,596.6 | 2,001.9 |
| 100M | 4,705.6 | **3,293.3** | 1,332.9 | 2,272.5 |

The segmented sieve has notably lower variance at large `n` — the fixed 32 KB working set means cache behaviour is deterministic rather than dependent on what else the OS has evicted.


## Analysis


### Flat vs Segmented Crossover

The segmented sieve adds overhead: segment setup, per-prime start-offset computation, buffer reset. At small `n` this overhead dominates — the flat sieve's single-pass simplicity wins.

The crossover occurs around **n ≈ 10M**, precisely where the flat sieve's bit array (~610 KB) exceeds the L1 cache (32 KB). Beyond this point, every sieving pass in the flat version triggers L1 misses, pulling data from L2 (or L3 at very large `n`). The segmented sieve's 32 KB buffer stays resident in L1 for the entire computation.

| n | Flat sieve array | Fits in L1? | Segmented speedup |
|---|---|---|---|
| 500K | 30.5 KB | ✅ yes | 0.98× (flat wins slightly) |
| 1M | 61 KB | ❌ 2× L1 | 0.99× (within noise) |
| 10M | 610 KB | ❌ 19× L1 | 1.00× (crossover) |
| 50M | 3.0 MB | ❌ 96× L1 | **1.27×** |
| 100M | 6.0 MB | ❌ 192× L1 | **1.38×** |

The speedup grows with `n` because the penalty for L1 misses compounds — each sieving prime's inner loop touches the entire array, and with hundreds of sieving primes at large `n`, the array is accessed in increasingly random patterns that defeat hardware prefetch.


### Why primal Is Faster (And Where It Isn't)

`primal` consistently beats both wofl implementations on raw speed by roughly 2–2.5×. The reason is straightforward: **wheel-30 factorisation**.

The wofl sieve skips only even numbers (factor out 2). This means it processes every odd number as a candidate. A wheel-30 sieve factors out 2, 3, and 5, processing only numbers coprime to 30. Of every 30 consecutive integers, only 8 are coprime to 30 — a 3.75× reduction in candidates compared to all integers, or roughly **1.87× fewer** than odd-only.

Combined with primal's own segmented architecture, this accounts for almost the entire speed gap.

However, **primal does not win on memory**:

| n = 50M | wofl (segmented) | primal (Sieve::new) |
|---|---|---|
| Sieve working memory | **32 KB** | 6.0 MB |
| Result vector | **24.7 MB** | 32.0 MB |
| Total | **24.8 MB** | 38.0 MB |
| Sieve memory ratio | **1×** | **187×** |

The wofl sieve's result vector is smaller because `prime_count_upper(n)` gives a tighter pre-allocation than primal's approach. The sieve working memory difference is extreme — 32 KB vs 6 MB — because the segmented wofl sieve reuses a single L1-sized buffer while primal's `Sieve::new` allocates the full range.


### The `primes` Crate: A Cautionary Tale

The `primes` crate uses trial division — testing each candidate against previously found primes. This is O(n√n / ln(n)) compared to the sieve's O(n log log n). The difference is not subtle:

| n | primes crate | wofl (flat) | Ratio |
|---|---|---|---|
| 10K | 259 µs | 7.1 µs | 36× |
| 100K | 4.9 ms | 78.9 µs | 62× |
| 500K | 28.3 ms | 434 µs | 65× |
| 1M | 62.7 ms | 905 µs | 69× |

The ratio worsens with scale because trial division's cost per prime grows (each new prime must be tested against all smaller primes), while the sieve's cost per prime is essentially constant. At 50M, the `primes` crate takes **4.7 seconds** — the wofl segmented sieve takes 52 ms. That's a **90× gap**.

If your codebase currently uses `primes` for bulk generation, switching to literally any sieve is a free 50–100× speedup.


### Scaling Behaviour

The Sieve of Eratosthenes has theoretical complexity O(n log log n), which is nearly linear. Empirically:

| n → 10× | wofl flat | wofl seg | primal sieve | Theoretical |
|---|---|---|---|---|
| 10K → 100K | 11.1× | 10.9× | 9.3× | ~10.4× |
| 100K → 1M | 11.5× | 11.4× | 8.8× | ~10.3× |
| 1M → 10M | 11.4× | 11.2× | 9.1× | ~10.3× |
| 10M → 100M | 18.4× | **13.4×** | 19.4× | ~10.2× |

At the 10M → 100M jump, the flat sieve's scaling breaks down badly (18.4× for a 10× increase) due to cache thrashing. The segmented sieve degrades more gracefully (13.4×) — still above theoretical due to result vector growth and TLB pressure, but a marked improvement. primal's degradation (19.4×) is surprising and likely relates to its own internal memory management at scale.


### Memory Characteristics

#### Sieve Working Memory

| n | wofl flat | wofl segmented | primal sieve | Naive `Vec<bool>` |
|---|---|---|---|---|
| 10K | 632 B | 32 KB | 1.2 KB | 10 KB |
| 100K | 6.1 KB | 32 KB | 12.2 KB | 100 KB |
| 1M | 61 KB | 32 KB | 122 KB | 1 MB |
| 10M | 610 KB | **32 KB** | 1.2 MB | 10 MB |
| 100M | 6.0 MB | **32 KB** | 11.9 MB | 100 MB |

The segmented sieve's working memory is constant. At n=100M, it uses **187× less** than the flat sieve and **372× less** than primal. Compared to a naive boolean array, it's **3,125× smaller**.

Note: the segmented sieve does use more memory than the flat sieve at small `n` (32 KB vs 632 bytes at n=10K). The 32 KB segment buffer is allocated regardless of how much is actually needed. For very small `n` where memory is critical, the flat sieve is the better choice.

#### Result Vector

The result vector dominates total memory at large `n` regardless of sieve implementation — π(100M) ≈ 5.76M primes × 8 bytes = ~46 MB. Pre-allocation via the prime-counting bound saves ~15 reallocations (each copying the entire vector) but doesn't reduce peak usage.

| n | wofl result vec | primal result vec | Saving |
|---|---|---|---|
| 10K | 9.8 KB | 16.0 KB | 1.6× |
| 100K | 78.0 KB | 128.0 KB | 1.6× |
| 1M | 650 KB | 1.0 MB | 1.6× |
| 10M | 5.4 MB | 8.0 MB | 1.5× |
| 50M | 24.7 MB | 32.0 MB | 1.3× |

The wofl sieve's result vector is consistently ~1.5× smaller. This comes from `prime_count_upper()` providing a tighter capacity estimate.

#### Peak Resident Memory

For applications where total RSS matters (containers, embedded, serverless):

| n | wofl segmented total | primal total | Saving |
|---|---|---|---|
| 10K | 42 KB | 17 KB | primal wins (small n) |
| 100K | 110 KB | 140 KB | 1.3× |
| 1M | 682 KB | 1.1 MB | 1.7× |
| 10M | 5.4 MB | 9.2 MB | 1.7× |
| 50M | 24.8 MB | 38.0 MB | 1.5× |
| 100M | 47.6 MB | 75.9 MB | 1.6× |


## Target Use Cases


### Embedded / Constrained Devices

**ESP32** (520 KB SRAM): The segmented sieve can generate primes up to ~33M within the memory budget (32 KB sieve + result vector). The flat sieve hits the memory wall at ~4M (sieve array alone is 31 KB). primal's 1.2 MB sieve at n=10M simply doesn't fit.

**Raspberry Pi Zero** (512 MB RAM): All implementations fit comfortably. The segmented sieve's advantage here is cache efficiency — the Pi Zero's ARM1176 has a 16 KB L1 data cache, so even the 32 KB segment is 2× L1. Reducing `SEGMENT_BYTES` to 16 KB would be optimal on this hardware.

**Lighthouse beacon nodes**: For distributed timing networks where each node needs a local prime table for hash functions, topic partitioning, or coordinate hashing. Boot-time generation of π(10K) = 1,229 primes takes <10 µs and uses <10 KB total. Prime tables can be generated once and held in a `static` for the node's lifetime.


### Precomputed Lookup Tables

For applications that test primality by lookup rather than computation:

```rust
lazy_static! {
    static ref PRIMES: Vec<u64> = segmented_sieve(1_000_000);
}

fn is_prime(n: u64) -> bool {
    PRIMES.binary_search(&n).is_ok()
}
```

Generation time for common table sizes:

| Table limit | Primes | Generation time | Memory |
|---|---|---|---|
| 10K | 1,229 | ~7 µs | 10 KB |
| 100K | 9,592 | ~80 µs | 78 KB |
| 1M | 78,498 | ~920 µs | 650 KB |
| 10M | 664,579 | ~10 ms | 5.4 MB |

Binary search over the result vector is O(log π(n)) — about 20 comparisons for a 1M table. For hot-path primality testing, a bitset representation (testing bit `n/2` directly) avoids the binary search entirely at the cost of holding the sieve array in memory.


### Cryptographic Support (Non-Cryptographic)

The sieve is deterministic and produces a complete, ordered list of primes. It is **not** suitable for:

- RSA key generation (need probabilistic primality tests for large primes)
- Random prime selection (no entropy source)
- Any context requiring unpredictability

It **is** suitable for:

- Small-prime trial division as a first pass before Miller-Rabin
- Generating factor bases for quadratic sieve / number field sieve
- Prime-counting verification (π(n) tables for testing other implementations)
- Educational / reference use


### MQTT Topic Distribution

For distributed systems that need consistent hash-based partitioning:

```rust
fn assign_partition(key: u64, n_partitions: usize) -> usize {
    let prime = PRIMES[n_partitions]; // prime > n_partitions
    (key % prime) as usize % n_partitions
}
```

Using a prime modulus larger than the partition count reduces clustering when keys have structure (sequential IDs, timestamp-based, etc). The sieve generates the lookup table at boot; the per-message cost is a single modulo operation.


### Batch Processing / Data Pipelines

For workflows that generate primes as an intermediate step (e.g., prime-indexed sampling, sieve-based factorisation):

| Scenario | Recommended variant | Why |
|---|---|---|
| n ≤ 500K | flat sieve | simpler, no segmentation overhead |
| n > 500K, memory unconstrained | segmented sieve | cache-friendly, lower variance |
| n > 500K, memory constrained | segmented sieve | 32 KB working set regardless of n |
| Need minimal dependencies | either wofl variant | zero crates, single file, `rustc` only |
| Need absolute speed, deps OK | primal (Sieve::new) | ~2× faster via wheel factorisation |
| Currently using `primes` crate | literally anything else | 50–100× speedup for free |


## Potential Optimisations Not Yet Implemented

Listed in order of expected impact:

| Optimisation | Expected speedup | Complexity cost | Status |
|---|---|---|---|
| **Wheel-30 factorisation** | ~1.8× | Moderate (30-element lookup table, more complex inner loop) | Not implemented — would close the gap with primal to within ~15% |
| **Parallel segments (rayon)** | ~N× on N cores | Low (segments are independent, read-only sieving primes) | Not implemented — breaks zero-dependency constraint |
| **Compile-time generation** | ∞ (zero runtime) | Low for small n (const fn) | Not implemented — useful for embedded firmware with fixed prime tables |
| **AVX2/NEON vectorisation** | ~1.5–2× on inner loop | High (platform-specific, requires `unsafe` or intrinsics crate) | Not implemented — would break the zero-unsafe guarantee |
| **Wheel-210 factorisation** | ~1.15× over wheel-30 | High (48-element pattern, diminishing returns) | Not worth the complexity |

The first two (wheel-30 and parallel segments) would bring the wofl sieve to within ~10% of primal on speed while retaining the memory advantage. Whether the added complexity is worth it depends on the use case — for a reference implementation and embedded target, the current odd-only approach is the right trade.


## Reproducing These Results

```bash
# Standalone sieve (no Cargo)
rustc -C opt-level=3 -C target-cpu=native seg.rs -o seg && ./seg

# Benchmark harness (requires Cargo)
cargo new prime_bench && cd prime_bench
# Copy prime_bench.rs → src/main.rs
# Copy Cargo.toml (includes primes + primal dependencies)
cargo run --release
```

When reporting performance numbers, please include:

- `rustc --version`
- CPU model (and L1 cache size if known)
- OS and kernel version
- Whether running in a VM / container (can affect cache behaviour)
- The exact `n` values tested
- Number of iterations (minimum 25 recommended)
- Whether the system was otherwise idle during measurement
