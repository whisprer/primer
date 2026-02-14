# Prime Sieve Techniques: A Complete Guide

From ancient algorithms to modern SIMD-accelerated parallel implementations, this guide covers every major technique for generating prime numbers efficiently.

---

## Table of Contents

1. [The Fundamental Problem](#the-fundamental-problem)
2. [Trial Division (Naive Approach)](#trial-division)
3. [Sieve of Eratosthenes (The Classic)](#sieve-of-eratosthenes)
4. [Odd-Only Sieve](#odd-only-sieve)
5. [Bit-Packing](#bit-packing)
6. [Wheel Factorization](#wheel-factorization)
7. [Segmented Sieve](#segmented-sieve)
8. [Kernighan Bit-Clear Extraction](#kernighan-bit-clear-extraction)
9. [Loop Unrolling](#loop-unrolling)
10. [SIMD Vectorization](#simd-vectorization)
11. [Parallel Sieving](#parallel-sieving)
12. [Sieve of Atkin](#sieve-of-atkin)
13. [Sieve of Sundaram](#sieve-of-sundaram)
14. [Comparison Table](#comparison-table)

---

## The Fundamental Problem

**Goal:** Find all prime numbers up to a given limit n.

**Definition:** A prime number is a natural number greater than 1 that has no positive divisors other than 1 and itself.

**The Challenge:** As n grows, the computational and memory requirements explode. For n = 1 billion:
- There are 50,847,534 primes to find
- Naive approaches would take hours
- Memory-naive approaches need 1GB+ RAM

**The Solution:** Clever algorithmic techniques that trade different resources (time, memory, complexity) against each other.

---

## Trial Division

**The simplest approach — and the slowest.**

### How It Works

For each candidate number, test if any smaller number divides it evenly:

```cpp
bool is_prime(int n) {
    if (n < 2) return false;
    for (int i = 2; i * i <= n; i++) {
        if (n % i == 0) return false;
    }
    return true;
}
```

### Complexity

- **Time:** O(n√n) — for each of n numbers, test up to √n divisors
- **Space:** O(1) — no extra storage needed

### When to Use

- Educational purposes
- Checking if a single number is prime
- Never for bulk generation

### The Problem

At n = 1,000,000, this takes seconds. At n = 1,000,000,000, it would take hours.

---

## Sieve of Eratosthenes

**The 2,300-year-old algorithm that still forms the basis of modern implementations.**

### History

Invented by Eratosthenes of Cyrene (~276-194 BCE), a Greek mathematician who also calculated the Earth's circumference with remarkable accuracy.

### How It Works

1. Create a list of all numbers from 2 to n
2. Start with the first unmarked number (2)
3. Mark all its multiples as composite
4. Move to the next unmarked number
5. Repeat until you've processed all numbers up to √n

```cpp
vector<bool> sieve(int n) {
    vector<bool> is_prime(n + 1, true);
    is_prime[0] = is_prime[1] = false;
    
    for (int p = 2; p * p <= n; p++) {
        if (is_prime[p]) {
            for (int i = p * p; i <= n; i += p) {
                is_prime[i] = false;
            }
        }
    }
    return is_prime;
}
```

### Key Insight: Start at p²

When marking multiples of prime p, we start at p² rather than 2p. Why?

All multiples of p smaller than p² have already been marked by smaller primes:
- 2p was marked when processing prime 2
- 3p was marked when processing prime 3
- etc.

### Complexity

- **Time:** O(n log log n) — remarkably efficient!
- **Space:** O(n) — one boolean per number

### Why O(n log log n)?

The sum of reciprocals of primes up to n is approximately ln(ln(n)). Each prime p marks n/p composites. The total work is:

```
n/2 + n/3 + n/5 + n/7 + ... ≈ n × ln(ln(n))
```

This is almost linear — only slightly worse than O(n).

---

## Odd-Only Sieve

**First optimization: skip even numbers entirely.**

### The Insight

Except for 2, all primes are odd. Why store or process even numbers at all?

### Implementation

Map index i to odd number 2i + 1:
- Index 0 → 1 (not prime)
- Index 1 → 3 (prime)
- Index 2 → 5 (prime)
- Index 3 → 7 (prime)
- Index 4 → 9 (composite: 3²)

```cpp
// Only odd numbers: index i represents number 2i + 1
vector<bool> sieve_odd(int n) {
    int size = n / 2 + 1;
    vector<bool> is_prime(size, true);
    is_prime[0] = false;  // 1 is not prime
    
    for (int i = 1; 2*i*(i+1) < size; i++) {
        if (is_prime[i]) {
            int p = 2*i + 1;
            // Mark odd multiples of p starting at p²
            for (int j = 2*i*(i+1); j < size; j += p) {
                is_prime[j] = false;
            }
        }
    }
    return is_prime;
}
```

### Benefits

- **Memory:** 50% reduction (n/2 bits instead of n)
- **Time:** ~50% faster (half the iterations)
- **Cache:** Better utilization

### The Math Behind j = 2*i*(i+1)

If index i represents prime p = 2i + 1, then p² = (2i + 1)² = 4i² + 4i + 1.

The index of p² in our odd-only array is (p² - 1) / 2 = (4i² + 4i) / 2 = 2i(i + 1).

---

## Bit-Packing

**Pack 64 candidates into a single 64-bit word.**

### The Problem with vector<bool>

C++ `vector<bool>` is already bit-packed, but:
- Access patterns are not cache-optimal
- No control over memory layout
- Overhead from bounds checking

### Manual Bit-Packing

Store 64 numbers per `uint64_t`:

```cpp
// Bit manipulation macros
#define SET_BIT(arr, i)   (arr[(i) >> 6] |=  (1ULL << ((i) & 63)))
#define CLR_BIT(arr, i)   (arr[(i) >> 6] &= ~(1ULL << ((i) & 63)))
#define TST_BIT(arr, i)   (arr[(i) >> 6] &   (1ULL << ((i) & 63)))
```

### Understanding the Bit Math

- `(i) >> 6` — Divide by 64 to get the word index
- `(i) & 63` — Modulo 64 to get the bit position within the word
- `1ULL << bit` — Create a mask with only that bit set

### Combined with Odd-Only

When combining bit-packing with odd-only representation:
- Index i represents odd number 2i + 1
- Word w contains indices 64w through 64w + 63
- These represent odd numbers 128w + 1 through 128w + 127

### Benefits

- **Memory:** 8 bytes stores 64 candidates (vs. 64 bytes for bool array)
- **Cache:** Entire L1 cache can hold 256K candidates
- **Vectorization:** Natural 64-bit operations

---

## Wheel Factorization

**Skip multiples of small primes beyond just 2.**

### The Concept

If we skip multiples of 2, we eliminate 50% of candidates. What if we also skip multiples of 3? And 5?

### Wheel-6 (Skip 2, 3)

Only numbers of the form 6k ± 1 can be prime (except 2 and 3):
- 6k + 0 = divisible by 6
- 6k + 1 = potential prime ✓
- 6k + 2 = divisible by 2
- 6k + 3 = divisible by 3
- 6k + 4 = divisible by 2
- 6k + 5 = potential prime ✓ (same as 6k - 1)

**Density:** 2/6 = 33% of numbers are candidates (vs. 50% for odd-only)

### Wheel-30 (Skip 2, 3, 5)

Only 8 residues modulo 30 can be prime:
```
1, 7, 11, 13, 17, 19, 23, 29
```

**Density:** 8/30 = 26.7% of numbers are candidates

### Wheel-210 (Skip 2, 3, 5, 7)

48 residues modulo 210.

**Density:** 48/210 = 22.9% of numbers are candidates

### Diminishing Returns

| Wheel | Period | Candidates | Density | Improvement |
|-------|--------|------------|---------|-------------|
| 2 | 2 | 1 | 50.0% | baseline |
| 6 | 6 | 2 | 33.3% | 1.5x |
| 30 | 30 | 8 | 26.7% | 1.25x |
| 210 | 210 | 48 | 22.9% | 1.17x |
| 2310 | 2310 | 480 | 20.8% | 1.10x |

The complexity of implementing larger wheels grows faster than the benefit.

### Implementation Challenge

Wheel factorization requires:
- Mapping between wheel positions and actual numbers
- Handling the irregular gaps between candidates
- More complex loop structures

Often the overhead exceeds the benefit, especially at smaller n.

---

## Segmented Sieve

**The key to sieving billions: process in cache-sized chunks.**

### The Memory Problem

For n = 1,000,000,000:
- Basic sieve: 1 billion booleans = 1 GB
- Odd-only: 500 million bits = 62.5 MB
- Still too large for CPU cache!

### The Cache Hierarchy

| Cache | Size | Latency |
|-------|------|---------|
| L1 | 32 KB | ~4 cycles |
| L2 | 256 KB | ~12 cycles |
| L3 | 8 MB | ~40 cycles |
| RAM | 16+ GB | ~200 cycles |

Every cache miss costs 50x the time of a cache hit!

### The Solution: Segmentation

1. Find all primes up to √n (the "base primes")
2. Process the range [√n, n] in segments that fit in L1 cache
3. For each segment, mark composites using base primes
4. Extract primes from the segment

```
[2, √n] → Base primes (small sieve)
    ↓
[√n+1, √n+S] → Segment 1
[√n+S+1, √n+2S] → Segment 2
    ...
[n-S+1, n] → Final segment
```

### Segment Size Selection

**Optimal segment size:** Fits in L1 cache with room for base primes.

- L1 = 32 KB
- Base primes for n=10⁹: ~3,400 primes × 4 bytes = 13.6 KB
- Remaining: ~18 KB for segment
- Segment size: ~16 KB = 128K odd numbers

### Memory Complexity

- **Base primes:** O(√n / ln(√n)) ≈ O(√n)
- **Segment buffer:** O(segment_size) — constant!
- **Total:** O(√n) instead of O(n)

For n = 10⁹: ~30 KB instead of 62.5 MB — a 2000x reduction!

---

## Kernighan Bit-Clear Extraction

**Extract primes by visiting only set bits.**

### The Naive Approach

```cpp
// O(n) - visits every bit
for (int i = 0; i < n; i++) {
    if (is_prime[i]) {
        primes.push_back(i);
    }
}
```

### The Kernighan Trick

Brian Kernighan (co-creator of C) discovered a beautiful identity:

```cpp
x & (x - 1)  // Clears the lowest set bit of x
```

**Why it works:**
- Subtracting 1 flips all bits from the lowest set bit rightward
- AND-ing with the original clears the lowest set bit

Example: x = 01011000
- x - 1 = 01010111
- x & (x-1) = 01010000

### Applied to Prime Extraction

```cpp
for (size_t i = 0; i < words; i++) {
    uint64_t w = bits[i];
    while (w) {
        int pos = __builtin_ctzll(w);  // Count trailing zeros
        int prime = (i * 128) + (pos * 2) + 1;
        primes.push_back(prime);
        w &= w - 1;  // Clear lowest bit, move to next
    }
}
```

### Complexity

- **Naive:** O(n) — visit every candidate
- **Kernighan:** O(π(n)) — visit only primes

For n = 10⁹: 50 million iterations instead of 500 million — 10x fewer!

### The ctzll Intrinsic

`__builtin_ctzll(x)` counts trailing zeros in x (GCC/Clang).

On x86, this compiles to a single `TZCNT` or `BSF` instruction — O(1) time.

Windows equivalent: `_BitScanForward64()`.

---

## Loop Unrolling

**Reduce loop overhead by processing multiple elements per iteration.**

### The Problem

Loops have overhead:
- Increment counter
- Compare to limit
- Branch prediction

For tight inner loops, this overhead is significant.

### Manual Unrolling

```cpp
// Before: 1 operation per iteration
for (int j = start; j < limit; j += step) {
    clear_bit(j);
}

// After: 4 operations per iteration
for (; j + 3*step < limit; j += 4*step) {
    clear_bit(j);
    clear_bit(j + step);
    clear_bit(j + 2*step);
    clear_bit(j + 3*step);
}
// Handle remainder
for (; j < limit; j += step) {
    clear_bit(j);
}
```

### Benefits

- **Fewer branches:** 4x fewer loop iterations
- **Better pipelining:** CPU can execute independent operations in parallel
- **Instruction-level parallelism:** Multiple operations in flight

### Optimal Unroll Factor

- Too little: Overhead remains
- Too much: Instruction cache pressure, code bloat

Typical sweet spot: 4-8x unrolling for simple operations.

---

## SIMD Vectorization

**Process multiple data elements with single instructions.**

### What is SIMD?

**S**ingle **I**nstruction, **M**ultiple **D**ata.

Modern CPUs have vector registers that can process multiple values simultaneously:

| Technology | Register Size | 64-bit Values |
|------------|---------------|---------------|
| SSE2 | 128 bits | 2 |
| AVX2 | 256 bits | 4 |
| AVX-512 | 512 bits | 8 |

### Where SIMD Helps in Sieving

**Memory initialization:**
```cpp
// Scalar: 1 word per instruction
for (int i = 0; i < words; i++)
    arr[i] = ~0ULL;

// AVX2: 4 words per instruction
__m256i ones = _mm256_set1_epi64x(-1LL);
for (int i = 0; i < words; i += 4)
    _mm256_store_si256((__m256i*)&arr[i], ones);
```

**Population counting:**
```cpp
// AVX-512 has native VPOPCNTDQ
// AVX2 requires lookup tables or multiple instructions
```

### Where SIMD Doesn't Help

**Random bit clearing:** Sieve marking accesses memory at irregular intervals determined by each prime. SIMD works best on contiguous, predictable access patterns.

The irregular pattern p, 2p, 3p, 4p... doesn't vectorize well because:
- Different primes have different strides
- Accesses span different cache lines
- Dependencies between iterations

### Practical SIMD Strategy

1. Use SIMD for bulk memory operations (fill, copy)
2. Use scalar code for marking (irregular access)
3. Use SIMD-friendly popcount for counting
4. Let the compiler auto-vectorize where it can

---

## Parallel Sieving

**Distribute work across multiple CPU cores.**

### The Challenge

The basic sieve has dependencies — we need prime p before marking its multiples.

But segmented sieving is embarrassingly parallel! Each segment can be processed independently once we have base primes.

### Work Distribution Strategies

**Static partitioning:**
- Divide range into equal chunks per thread
- Simple, but load imbalance if chunks have different prime densities

**Dynamic work-stealing:**
```cpp
std::atomic<int> next_segment{0};

auto worker = [&]() {
    while (true) {
        int seg = next_segment.fetch_add(1);
        if (seg >= total_segments) break;
        process_segment(seg);
    }
};
```

This automatically balances load — fast threads take more segments.

### What Parallelizes

✅ Segment processing — each segment is independent
✅ Prime counting — sum per-thread counts
✅ Memory initialization — parallel fills

### What Doesn't Parallelize

❌ Base prime generation — small enough to not matter
❌ Ordered prime collection — need synchronization
❌ Finding "last N primes" — requires final pass

### Speedup Expectations

**Ideal:** Linear with core count (8 cores = 8x faster)

**Reality:** 
- Memory bandwidth limits scaling
- Cache contention between threads
- Synchronization overhead

Typical achieved speedup: 3-6x on 8 cores.

---

## Sieve of Atkin

**A modern alternative with better theoretical complexity.**

### Background

Developed by A.O.L. Atkin and Daniel J. Bernstein in 2003.

### How It Works

Instead of marking composites, it uses quadratic forms to identify primes:

1. Numbers with odd count of solutions to 4x² + y² = n (mod 12 ∈ {1, 5})
2. Numbers with odd count of solutions to 3x² + y² = n (mod 12 = 7)
3. Numbers with odd count of solutions to 3x² - y² = n (mod 12 = 11, x > y)

Then eliminate squares of primes.

### Complexity

- **Time:** O(n / log log n) — theoretically better than Eratosthenes!
- **Space:** O(n)

### In Practice

Despite better theoretical complexity:
- Constant factors are higher
- More complex to implement
- Only faster for very large n (>10¹⁰)
- Harder to optimize with SIMD/parallelism

For most practical purposes, optimized Eratosthenes wins.

---

## Sieve of Sundaram

**An elegant algorithm that naturally produces odd primes.**

### How It Works

1. Create array for numbers 1 to n
2. Mark all numbers of form i + j + 2ij where 1 ≤ i ≤ j and i + j + 2ij ≤ n
3. Remaining numbers k give primes 2k + 1

### The Math

If k is not marked, then 2k + 1 is prime.

The marked numbers represent exactly those k where 2k + 1 is an odd composite.

### Complexity

- **Time:** O(n log n) — worse than Eratosthenes
- **Space:** O(n)

### When to Use

Mostly of theoretical interest. The natural odd-number output is elegant, but performance doesn't match optimized Eratosthenes.

---

## Comparison Table

| Technique | Time | Space | Practical Speed | Complexity |
|-----------|------|-------|-----------------|------------|
| Trial Division | O(n√n) | O(1) | Very Slow | Trivial |
| Basic Eratosthenes | O(n log log n) | O(n) | Slow | Simple |
| Odd-Only | O(n log log n) | O(n/2) | Medium | Simple |
| Bit-Packed | O(n log log n) | O(n/16) | Fast | Medium |
| Wheel-30 | O(n log log n) | O(n/30) | Fast | Complex |
| Segmented | O(n log log n) | O(√n) | Very Fast | Medium |
| + Kernighan | O(n log log n) | O(√n) | Very Fast | Medium |
| + SIMD | O(n log log n) | O(√n) | Faster | Complex |
| + Parallel | O(n log log n / p) | O(√n × p) | Fastest | Complex |

---

## Putting It All Together

The ultimate modern prime sieve combines:

1. **Segmented approach** — O(√n) memory, cache-friendly
2. **Odd-only representation** — 50% memory/time savings
3. **64-bit packed storage** — Cache-optimal, SIMD-friendly
4. **Kernighan extraction** — O(primes) not O(candidates)
5. **Loop unrolling** — Reduced branch overhead
6. **SIMD initialization** — Fast segment setup
7. **Parallel processing** — Linear scaling with cores

This combination achieves:
- **1 billion primes in ~250ms** on modern hardware
- **30 KB memory** instead of 125 MB
- **4+ billion integers/second** throughput

The 2,300-year-old algorithm of Eratosthenes, enhanced with 50 years of computer science optimization techniques, remains unbeaten for practical prime generation.

---

## Further Reading

- "The Genuine Sieve of Eratosthenes" — Melissa E. O'Neill
- "Segmented Sieve of Eratosthenes" — primesieve.org
- "Prime Sieves using Binary Quadratic Forms" — Atkin & Bernstein

---

*"In mathematics, simplicity is the ultimate sophistication. The sieve of Eratosthenes proves that a 2,300-year-old idea, properly implemented, can outperform 'clever' modern alternatives."*
