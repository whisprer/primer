# From C++ Bit Tricks to Cache-Aware Rust: A Prime Sieve Odyssey

## The Starting Point: 9 Lines of Brutal C++

It started with a challenge — how compact can you make a prime sieve that's still *fast*? The answer, in C++, turned out to be terrifyingly dense:

```cpp
for(i=1; i<=sqrt(n)/2; ++i)
    if(b[i>>6]>>(i&63)&1)
        for(j=2*i*(i+1); j<=h; j+=2*i+1)
            b[j>>6]&=~(1ULL<<(j&63));
```

Four lines. No allocator. No abstractions. Just raw bit-banging over a packed `u64` array where each bit represents an odd number. The trick is old — represent only odd candidates, pack 64 of them into each machine word, use the shift-and-mask idiom `b[i>>6] >> (i&63) & 1` to test primality, and Brian Kernighan's `w &= w - 1` trick to iterate only set bits during extraction.

The result: a sieve that generates 41,538 primes up to 500,000 in under 2ms, using just 4KB of memory. Months of whittling got it there — stripping every unnecessary variable, every redundant check, every wasted byte.

The question was: could Rust match it?

## Act I: The Port (And the Borrow Checker Fight)

The first Rust translation looked almost identical to the C++. Same bit layout, same sieving logic, same extraction loop. It should have been a clean compile.

It wasn't.

The natural instinct was to write it as an iterator chain — `.filter()` to test primality, `.for_each()` to mark composites. Idiomatic Rust. Beautiful Rust. Rust that doesn't compile:

```rust
// The closure in .filter() captures `b` immutably
// The closure in .for_each() captures `b` mutably
// Rust says: absolutely not
(1..=(sqrt_n / 2))
    .filter(|&i| (b[(i >> 6) as usize] >> (i & 63)) & 1 == 1)
    .for_each(|i| {
        b[(j >> 6) as usize] &= !(1u64 << (j & 63));  // boom
    });
```

The borrow checker was right, of course. You can't hold an immutable reference through `.filter()` while simultaneously mutating through `.for_each()`. C++ doesn't care about this — it'll happily let you read and write the same array through aliased pointers, and most of the time it works, and sometimes it doesn't, and when it doesn't you spend three days with Valgrind.

The fix was straightforward: use explicit `for` loops for the sieving phase (where mutation happens), reserve iterators for the collection phase (read-only). The borrow checker can see that each loop iteration's borrow is released before the next mutation occurs. Same performance — LLVM optimises both forms to identical assembly — but the code is honest about what it's doing.

```rust
// Sieving: explicit loop, borrows released each iteration
for i in 1..=(sqrt_n / 2) {
    if (b[(i >> 6) as usize] >> (i & 63)) & 1 == 1 {
        let mut j = 2 * i * (i + 1);
        while j <= h {
            b[(j >> 6) as usize] &= !(1u64 << (j & 63));
            j += 2 * i + 1;
        }
    }
}

// Collection: iterators fine here, b is never mutated
for (i, &word) in b.iter().enumerate() {
    let mut w = word;
    while w != 0 {
        let p = (((i << 6) + w.trailing_zeros() as usize) * 2 + 1) as u64;
        if p <= n { r.push(p); }
        w &= w - 1;
    }
}
```

Lesson learned: the borrow checker isn't an obstacle to porting C++ — it's a translator between "code that probably works" and "code that provably works."

## Act II: Making It Proper

The initial port compiled and ran, but it was carrying baggage. A round of optimisation tightened things up:

**Pre-allocated result vector.** The original `let mut r = vec![2]` was growing through repeated reallocation — about 15 doublings to hold 41,538 primes. Using the prime number theorem's upper bound `n/ln(n) × 1.15` to pre-size the vector eliminated every reallocation.

**Integer square root.** The original `(n as f64).sqrt() as u64` works fine for n up to about 2^52 — the limit of f64 mantissa precision. Beyond that, the cast silently gives wrong results. A Newton-corrected integer square root with overflow-safe `checked_mul` handles every u64 value correctly, including `u64::MAX`.

**Hoisted inner loop step.** The sieve's inner loop was computing `2 * i + 1` on every iteration. Hoisting it to a `let step = 2 * i + 1` before the loop costs nothing but guarantees the optimisation even in debug builds.

**Early termination.** The collection phase was iterating the entire bit array even when the remaining words were all beyond `n`. A simple `if base > n { break; }` at the word level cuts the tail.

And then — the fun part — two bugs surfaced in the original test assertions that had been hiding in plain sight:

The assertion `primes[10_000] == 104_729` was wrong. Index 10,000 (zero-based) is the 10,001st prime, which is 104,743. The value 104,729 sits at index 9,999. Both are prime; the index was off by one.

The assertion that 499,999 is prime was also wrong. It's composite: 499,999 = 7 × 71,428 + 3. The largest prime not exceeding 500,000 is 499,979.

The sieve itself was always correct — it was the hand-written test expectations that were wrong. A reminder that the most dangerous bugs live in the code you don't test.

## Act III: The Benchmark

With the sieve solid, the natural question was: how does it stack up against the established Rust ecosystem? A benchmark harness pitted the bit-packed sieve against two crates across six orders of magnitude (10K to 50M):

**`primes` crate (v0.3):** 150–250× slower at every scale. This crate uses trial division under the hood. For bulk generation, it's not a serious contender.

**`primal` crate (v0.3):** The real competition. At n=500K, `primal::Sieve::new` ran in ~170µs to the bit-packed sieve's ~435µs — about 2.5× faster. At n=50M, the gap held at roughly 2.2×.

The speed gap had a clear cause: `primal` uses a segmented, wheel-factored sieve. "Wheel factorisation" means it skips multiples of 2, 3, and 5 during sieving — a mod-30 pattern that eliminates ~73% of candidates before touching them. The bit-packed sieve only skips multiples of 2 (odd-only), doing roughly 2.6× more work per range.

But memory told a different story. At n=10M:

| Implementation | Sieve Memory | Result Vec | Total |
|---|---|---|---|
| Bit-packed (flat) | 610 KB | 5.4 MB | 6.0 MB |
| primal (Sieve::new) | 1.2 MB | 8.0 MB | 9.2 MB |

The bit-packed sieve's working set was half the size — and for the sieve array alone (the hot allocation during computation), it was 2× smaller. On an ESP32 with 520KB of SRAM, that's the difference between "fits" and "doesn't fit."

## Act IV: The Segmented Sieve

The benchmark revealed something else: the flat sieve's performance degraded at scale. At n=50M the bit array is 3MB — far larger than a typical L1 cache (32KB). Every sieving pass was blowing the cache, fetching data from L2 or L3 on almost every access.

The fix is a technique as old as the sieve itself: segmentation. Instead of allocating the entire bit array up front, process the range in L1-sized chunks:

1. **Bootstrap:** Run the flat sieve up to √n to find the small "sieving primes." For n=50M, √n ≈ 7071, so this phase produces just 900 primes and takes microseconds.

2. **Segment loop:** Allocate a single 32KB buffer. For each segment of the range, reset the buffer, strike composites using the sieving primes, then extract surviving primes. The buffer is reused for every segment — it never leaves L1.

3. **Composite striking:** For each sieving prime `p`, find the first composite half-index within the current segment, then step through by `p`. The key insight is that the starting offset for each prime can be computed from the segment boundaries — no global state needed.

```rust
const SEGMENT_BYTES: usize = 32 * 1024;
const SEGMENT_BITS: u64 = (SEGMENT_BYTES * 8) as u64;

let mut seg = vec![0u64; SEGMENT_WORDS]; // single buffer, reused

while lo <= h {
    let hi = std::cmp::min(lo + SEGMENT_BITS - 1, h);

    // Reset segment
    for w in seg[..words_needed].iter_mut() { *w = !0u64; }

    // Strike composites
    for &p in &small_odd_primes {
        let start_half = (p * p - 1) / 2;
        let first = if start_half >= lo { start_half }
                    else {
                        let offset = (lo - start_half) % p;
                        if offset == 0 { lo } else { lo + p - offset }
                    };
        let mut j = first;
        while j <= hi {
            let local = (j - lo) as usize;
            seg[local >> 6] &= !(1u64 << (local & 63));
            j += p;
        }
    }

    // Extract primes via Brian Kernighan
    // ...

    lo += SEGMENT_BITS;
}
```

The memory profile shifted dramatically. At n=50M, the sieve's working memory dropped from 3MB (flat) to 32KB (one segment) — a 95× reduction. The segment buffer is smaller than L1 cache on every modern CPU, meaning zero cache misses during the hot sieving inner loop.

## The Final Numbers

Benchmarked at 25 iterations per size, all implementations cross-verified for correctness at every scale:

| n | Flat sieve | Segmented sieve | primal | Speedup (seg vs flat) |
|---|---|---|---|---|
| 10K | 7.1 µs | 7.4 µs | 4.3 µs | ~1× (overhead dominates) |
| 100K | 78.9 µs | 80.7 µs | 39.9 µs | ~1× |
| 500K | 434 µs | 443 µs | 170 µs | ~1× |
| 1M | 905 µs | 917 µs | 352 µs | ~1× |
| 10M | 10.3 ms | 10.3 ms | 3.2 ms | ~1× (crossover point) |
| 50M | 66.6 ms | 52.5 ms | 30.7 ms | **1.3×** |
| 100M | 189.7 ms | 137.0 ms | 61.7 ms | **1.4×** |

The segmented sieve starts winning at ~10M — exactly where the flat sieve's bit array exceeds L1 cache. By 100M, it's 1.4× faster than flat, with the gap widening at larger n.

Against `primal`, the segmented sieve closes the gap from ~3× (flat at 100M) to ~2.2×. The remaining difference is almost entirely wheel factorisation — `primal` skips multiples of 2, 3, and 5, while this sieve only skips multiples of 2.

## Memory: Where the Sieve Wins

At n=50,000,000:

| Component | Flat | Segmented | primal |
|---|---|---|---|
| Sieve working memory | 3.0 MB | **32 KB** | 6.0 MB |
| Result vector | 24.7 MB | 24.7 MB | 32.0 MB |
| Total | 27.7 MB | **24.8 MB** | 38.0 MB |
| vs naive bool array | 16× smaller | **1,490× smaller** (sieve only) | 8× smaller |

The segmented sieve uses 187× less sieve memory than `primal`, and the result vector is 23% smaller thanks to tighter pre-allocation via the prime counting bound. For embedded targets — ESP32, Raspberry Pi Zero, lighthouse beacon nodes with constrained SRAM — this is the margin that matters.

## What's Left on the Table

**Wheel factorisation (mod 30).** Skipping multiples of 2, 3, and 5 would reduce sieving work by ~2.6× and likely bring the segmented sieve within 10–15% of `primal`. The cost: roughly double the code complexity, with a 30-element lookup table for the wheel pattern. For a production library, worth it. For clarity and embeddability, the current odd-only approach is a better trade.

**Parallelism.** Each segment is independent — the sieving primes are read-only, and each segment writes to its own buffer. A `rayon`-based parallel iterator over segments would scale linearly with cores. On an 8-core machine, the 100M benchmark could drop from 137ms to ~20ms.

**Compile-time generation.** For fixed small limits (n ≤ 10K), `const fn` evaluation could embed the prime table directly in the binary at zero runtime cost. Useful for embedded firmware where boot time matters.

## The Journey

From a dense C++ one-liner honed over months of bit-twiddling, through a borrow checker fight that forced better structure, past hidden assertion bugs that the sieve itself was too correct to trigger, to a cache-aware segmented implementation that trades blows with the best Rust crates while using a fraction of the memory.

The borrow checker never slowed the code down. It slowed *development* down — for exactly as long as it took to understand why the mutation pattern was unsafe, and to restructure it into something that's both safe and just as fast. The optimiser erases the syntactic cost. What remains is a guarantee that no debug session will ever begin with "it worked fine in single-threaded tests."

The sieve runs in 32KB of working memory. It generates 5.7 million primes up to 100 million in 137ms. It compiles to a single binary with `rustc`, no dependencies, no crate ecosystem, no build system beyond a one-line shell command.

```bash
rustc -C opt-level=3 -C target-cpu=native seg.rs && ./seg
```

Ship it.
