[README.md]

# Prime-Shootout

<p align="center">
  <strong>Benchmarking prime number algorithms across languages and LLMs</strong>
</p>

<p align="center">
  <a href="https://github.com/whisprer/prime-shootout/releases">
    <img src="https://img.shields.io/github/v/release/whisprer/prime-shootout?color=4CAF50&label=release" alt="Release">
  </a>
  <a href="https://github.com/whisprer/prime-shootout/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/whisprer/prime-shootout/lint-and-plot.yml?label=build" alt="Build">
  </a>
  <img src="https://img.shields.io/badge/version-3.1.1-blue.svg" alt="Version">
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-lightgrey.svg" alt="Platform">
  <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License">
</p>

---

## Overview

**Prime-Shootout** is a benchmarking project comparing prime number sieve implementations across:

- **Languages:** C++, Rust, Python
- **LLM Authors:** Claude Opus 4.5, ChatGPT 5.1, Gemini 3 Pro, Grok 4.1
- **Scales:** 500,000 (Round 1) and 1,000,000,000 (Round 2)

The goal is to evaluate both **algorithmic efficiency** and **code quality** produced by different LLMs when given identical prompts for optimization challenges.

[n.b. I've removed quirks like use of emojis by LLMs and also only put final code blocks resulting from iterations through errors - it should be noted on top of exceptional speed and compactness of code, Claude made not a single error, everything compiled and ran perfectly first go and G-Petey's elegant beauty code only flawed once - otoh, both Gemini and Grok proved to be a little more flawed with multiple errors and iterations needed to produce compiling and running code...]

---

## Benchmark Results

### Round 1: n = 500,000 (41,538 primes)

| Implementation | Avg (ms) | Min (ms) | Max (ms) | StdDev |
|----------------|----------|----------|----------|--------|
| c-primes-gpetey | 21.65 | 20.66 | 24.29 | 0.87 |
| **c-primes-claude** | **21.36** | **20.48** | 24.90 | 0.70 |
| c-primes-fast | 25.23 | 23.31 | 30.71 | 2.10 |
| r-primes-fast | 27.80 | 26.12 | 31.20 | 1.57 |
| r-primes | 27.83 | 26.04 | 31.04 | 1.52 |
| c-primes | 29.38 | 27.19 | 35.33 | 2.53 |

**Winner:** Claude (fastest avg + lowest variance)

### Round 2: n = 1,000,000,000 (50,847,534 primes)

| Implementation | Avg (ms) | Strategy |
|----------------|----------|----------|
| c-primes-claude-seg | ~2,100 | Segmented + bit-packed + ctzll |
| c-primes-gpetey-seg | ~2,200 | Segmented + bit-packed |
| c-primes-gemini-seg | ~2,400 | Segmented sieve |
| c-primes-grok-seg | ~2,500 | Segmented sieve |

### Round Aux: Specialized Implementations

| Implementation | Avg (ms) | Technique |
|----------------|----------|-----------|
| c-primes-parallel | ~800 | Multi-threaded (8 cores) |
| c-primes-bitpacked | ~2,000 | Segmented + Kernighan bit-clear |
| c-primes-wheel | ~2,300 | Wheel-30 factorization |
| c-primes-segment-sieve | ~3,500 | Basic segmented |
| the-beast-reborn | ~300-800 | Auto-selecting (parallel if >= 4 cores) |

---

## Algorithm Techniques

### Sieve of Eratosthenes (Basic)

The classic O(n log log n) algorithm:

```cpp
for (int p = 2; p * p <= n; p++)
    if (is_prime[p])
        for (int i = p * p; i <= n; i += p)
            is_prime[i] = false;
```

**Memory:** O(n) bits

### Odd-Only Sieve

Skip even numbers entirely. Index `i` maps to value `2i + 1`:

```cpp
// Index 0 = 1, Index 1 = 3, Index 2 = 5, ...
for (u32 i = 1; i <= sqrt_n/2; ++i)
    if (bits[i])
        for (u32 j = 2*i*(i+1); j < half; j += 2*i+1)
            bits[j] = false;
```

**Memory:** O(n/2) bits — 50% reduction

### Bit-Packed Storage

Pack 64 candidates per `uint64_t` word:

```cpp
#define SET_BIT(arr, i)   (arr[(i) >> 6] |=  (1ULL << ((i) & 63)))
#define CLR_BIT(arr, i)   (arr[(i) >> 6] &= ~(1ULL << ((i) & 63)))
#define TST_BIT(arr, i)   (arr[(i) >> 6] &   (1ULL << ((i) & 63)))
```

**Benefit:** Cache-friendly, SIMD-compatible

### Kernighan Bit-Clear Extraction

Extract primes by iterating only set bits:

```cpp
for (auto w = bits[i]; w; w &= w - 1) {
    int pos = __builtin_ctzll(w);  // Count trailing zeros
    primes.push_back(base + pos * 2 + 1);
}
```

**Complexity:** O(number of primes) vs O(n) for linear scan

### Segmented Sieve

Process in L1-cache-sized chunks (~16-32KB):

```
[Base primes: 2..sqrt(n)]
     |
     v
[Segment 0] -> [Segment 1] -> [Segment 2] -> ...
  0..S          S..2S          2S..3S
```

**Memory:** O(sqrt(n)) for base primes + O(S) for segment buffer

**Why it matters:** At n = 1e9, naive sieve needs 125MB. Segmented needs ~30KB.

### Wheel Factorization (Wheel-30)

Skip multiples of 2, 3, 5. Only 8 out of every 30 numbers can be prime:

```
Wheel positions: 1, 7, 11, 13, 17, 19, 23, 29 (mod 30)
```

**Memory:** O(n * 8/30) = 73% reduction vs odd-only

### Parallel Segmented Sieve

Distribute segments across threads:

```cpp
std::atomic<u64> next_segment{0};
auto worker = [&](int tid) {
    while (true) {
        u64 seg = next_segment.fetch_add(1);
        if (seg >= max_segments) break;
        sieve_segment(seg * S, (seg+1) * S - 1);
    }
};
```

**Speedup:** Near-linear with core count (sieve phase only)

---

## Code Quality Comparison

### Lines of Code (Round 1, n = 500K)

| Author | Lines | Style |
|--------|-------|-------|
| Claude | 18 | Dense, intrinsic-heavy |
| GPetey | 45 | Clean, readable |
| Gemini | 52 | Documented |
| Grok | 48 | Balanced |

### Key Differentiators

**Claude's approach:**
- `__builtin_ctzll` for O(1) bit position
- `w &= w - 1` Kernighan trick
- Minimal variable declarations
- Aggressive expression compression

**GPetey's approach:**
- Named helper macros
- Explicit loop counters
- Clear variable naming
- Human-readable structure

---

## Building

### Prerequisites

- GCC 10+ or Clang 12+ (C++17/20)
- Rust 1.70+
- Python 3.8+ with NumPy, Numba (optional)

### C++ (all implementations)

```bash
# Basic optimization
g++ -O3 -march=native -std=c++17 src/cpp/c-primes-claude.cpp -o c-primes-claude.exe

# With LTO
g++ -O3 -march=native -flto -std=c++17 src/cpp/c-primes-claude.cpp -o c-primes-claude.exe

# Parallel (requires pthread)
g++ -O3 -march=native -flto -std=c++17 -pthread src/cpp-alt/c-primes-parallel-1e9.cpp -o c-primes-parallel.exe
```

### Rust

```bash
rustc -O src/rust/main-5e5.rs -o r-primes.exe
rustc -O src/rust/main-1e9.rs -o r-primes-1e9.exe
```

### Python

```bash
pip install numpy numba pyinstaller
pyinstaller --onefile src/python/smart_primes_cli.py
```

### Benchmarker

```bash
g++ -O3 -std=c++20 benchmark/cs_exe_bm_mkviii.cpp -o cs_exe_bm.exe
./cs_exe_bm.exe
```

---

## Project Structure

```
prime-shootout/
├── benchmark/
│   ├── cs_exe_bm_mkv.cpp             # Round 1 benchmarker
│   ├── cs_exe_bm_mkvii.cpp       # Round 2 benchmarker
│   ├── cs_exe_bm_mkviii.cpp    #Aux 1 benchmarker
│   └── cs_exe_bm_mkix.cpp          # Aux 2 benchmarker
├── build/dist/
│   ├── round-1/               # 500K executables
│   ├── round-2/               # 1e9 executables
│   └── round-aux/         # Specialized implementations
├── docs/
│   ├── 5e5-Results.md                       # Round 1 detailed results
│   ├── 1e9-Results.md                       # Round 2 detailed results
│   ├── 1e9-Aux-Results.md           # Auxiliary 1 results
│   ├── 1e9-Aux-2-Results.md     # Auxiliary 2 results
│   └── *-Answer.md                                # LLM responses
├── src/
│   ├── cpp/                           # Round 1 C++ sources
│   ├── cpp-1e9/               # Round 2 C++ sources
│   ├── cpp-aux/               # Experimental implementations
│   ├── python/                  # Python implementations
│   └── rust/                        # Rust implementations
├── file-structure.md # [this tree]
├── CONTRIBUTING.md       # [help for prospective contributors]
├── SECURITY.md                   #[code safety issues] 
├── CHANGELOG.md ]            #[history]
├── LICENSE.md                      # [licensing info]
└── README.md                         # [this document]
```

---

## Scoring System

Each round awards 30 points across three categories:

| Category | Points | Criteria |
|----------|--------|----------|
| Speed | 10 | Lowest average runtime |
| Compactness | 10 | Fewest lines of code |
| Elegance | 10 | Readability, idiom usage, aesthetic |

### Round 1 Final Scores

| LLM | Speed | Compact | Elegant | Total |
|-----|-------|---------|---------|-------|
| **Claude** | 10 | 10 | 0 | **20** |
| GPetey | 0 | 0 | 10 | 10 |
| Gemini | 0 | 0 | 0 | 0 |
| Grok | 0 | 0 | 0 | 0 |

---

## Mathematical Notes

### Prime Counting Function

The number of primes up to n is approximated by:

```
π(n) ≈ n / ln(n)
```

More accurate (prime number theorem):

```
π(n) ≈ Li(n) = ∫₂ⁿ dt/ln(t)
```

**Verification values:**
- π(500,000) = 41,538
- π(1,000,000,000) = 50,847,534

### Sieve Complexity

| Algorithm | Time | Space |
|-----------|------|-------|
| Trial division | O(n√n) | O(1) |
| Basic sieve | O(n log log n) | O(n) |
| Segmented sieve | O(n log log n) | O(√n) |
| Wheel-30 sieve | O(n log log n) | O(n/3.75) |

### Cache Considerations

| Cache Level | Size | Optimal Segment |
|-------------|------|-----------------|
| L1 | 32KB | 16KB (128K odds) |
| L2 | 256KB | 128KB (1M odds) |
| L3 | 8MB+ | 4MB (32M odds) |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch
3. Run benchmarks to verify no regressions
4. Submit a pull request

---

## License

MIT License. See [LICENSE.md](LICENSE.md).

---

## Acknowledgements

- **Claude Opus 4.5** (Anthropic) — Speed demon
- **ChatGPT 5.1** (OpenAI) — Elegance champion
- **Gemini 3 Pro** (Google) — Solid contender
- **Grok 4.1** (xAI) — Reliable performer

Built by [whisprer](https://github.com/whisprer) with algorithmic obsession.

*"May your sieves be cache-friendly and your bits be packed."*
