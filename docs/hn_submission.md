# Hacker News Submission — Final Drafts

---

## TITLE (78 chars):

Show HN: Bit-packed segmented prime sieve in Rust, 32KB working memory, 0 deps

---

## BODY — Option A: Full Story (1,964 chars):

Started with a hyper-compact C++ bit-packed sieve — months of whittling it down to ~9 lines of raw bit manipulation. One bit per odd number, Brian Kernighan's bit trick for extraction, hardware tzcnt intrinsics. Then I ported it to Rust.

The borrow checker had opinions. You can't .filter() over a bit array (immutable borrow) and .for_each() mutate it simultaneously. Fair enough — explicit loops for the sieve phase, iterators for collection. Same assembly output, provably safe. Along the way I caught two assertion bugs the sieve was too correct to trigger (off-by-one on the 10,001st prime; 499,999 is composite, not prime).

The flat sieve hit a wall at ~10M — the bit array exceeds L1 cache (32KB) and every sieving pass thrashes. So I built a segmented version: bootstrap sieving primes via flat sieve up to √n, then process the full range in 32KB L1-sized segments. One buffer, reused, never leaves cache.

Results (single-threaded, 25 iterations, cross-verified against primal crate):

  n=500K:   ~440µs, 30KB sieve mem
  n=10M:    ~10ms, 32KB sieve mem
  n=50M:    ~52ms, 32KB sieve mem (flat: ~67ms)
  n=100M:   ~137ms, 32KB sieve mem (flat: ~190ms)

Compared to the primal crate (Rust's best-in-class), raw speed is ~2x behind — primal uses wheel-30 factorisation which skips multiples of 2, 3, and 5. This sieve only skips evens. But memory is where it wins: 187x less sieve working memory than primal at n=50M (32KB vs 6MB), and a tighter result vector from prime-counting pre-allocation.

The target use case is embedded/constrained: ESP32 nodes, Raspberry Pi Zeros, distributed timing networks where every KB matters. It's also a clean reference implementation — one file, zero dependencies, compiles with a single rustc invocation.

Single file. No Cargo. No crates. Just:

  rustc -C opt-level=3 -C target-cpu=native seg.rs && ./seg

Code, benchmarks, borrow checker writeup, and a full development narrative are all in the repo.


---

## BODY — Option B: Punchier (1,459 chars):

Months of whittling a C++ prime sieve down to 9 lines of bit manipulation. Then I ported it to Rust, fought the borrow checker (it was right), caught two hidden test bugs, and ended up with something I'm pretty happy with.

It's a segmented bit-packed Sieve of Eratosthenes. One bit per odd number, processed in 32KB L1-cache-sized segments. Brian Kernighan's bit trick for extraction, hardware tzcnt intrinsics, pre-allocated result vector via prime-counting bound. Zero dependencies. Single file. Compiles with rustc alone.

Key numbers (single-threaded, 25 iterations):

  n=100K:   ~80µs
  n=1M:     ~920µs
  n=10M:    ~10ms
  n=100M:   ~137ms

Working sieve memory is always 32KB regardless of n. The flat (non-segmented) version uses n/128 bytes — at n=100M that's 6MB blowing L1 cache on every pass. Segmentation gives 1.4x speedup at that scale.

Vs primal (Rust's best sieve crate): ~2x slower on raw speed (no wheel factorisation), but 187x less sieve memory. Built for embedded targets — ESP32, Pi Zero, distributed beacon nodes — where memory is the constraint, not clock cycles.

The borrow checker story is worth reading if you're porting C++ bit manipulation code to Rust. The natural iterator-chain approach doesn't compile (simultaneous immutable + mutable borrow). Explicit loops fix it at zero performance cost.

  rustc -C opt-level=3 -C target-cpu=native seg.rs && ./seg

Full writeup, benchmarks, and C++ comparison in the repo.
