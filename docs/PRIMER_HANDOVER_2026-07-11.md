# Primer / primercrate.rs Handover
**Authoritative project state at the end of the 11 July 2026 session**

## 1. Purpose of this document

This document freezes the known-good state of the Primer project so work can continue in a fresh conversation without reconstructing decisions from a long, error-prone transcript.

The most important rule is:

> **Do not rerun any old migration or release-review ZIP/script. The repository has already reached the validated package state. Begin by inspecting Git, not by applying another patch.**

The user reported that the final standalone script, `FIX-PRIMER-NOW.ps1`, completed successfully. It performed the exact package-content gate, crates.io name check, package build, and `cargo publish --dry-run` without publishing anything.

No actual crates.io publication, Git commit, Git tag, or Git push was performed by that script.

---

## 2. Project identity

| Item | Authoritative value |
|---|---|
| Product/brand | **Primer** |
| Website | **https://primercrate.rs/** |
| Git repository | **https://github.com/whisprer/primer** |
| Local repository | **C:\github\primer** |
| crates.io package name | **primer-sieve** |
| Rust library crate name | **primer** |
| CLI executable name | **primer** |
| Release-candidate version | **0.3.0** |
| Main public function | `primer::sieve_primes(limit)` |
| Compatibility function | `primer::segmented_sieve(limit)` |
| Current feature branch | **feature/cargo-crate-v0.3.0** |

The exact registry package name `primer` is occupied by an unrelated crate. The correct solution is therefore:

```toml
[package]
name = "primer-sieve"

[lib]
name = "primer"
path = "src/lib.rs"

[[bin]]
name = "primer"
path = "src/bin/primer.rs"
```

This preserves the public Rust API:

```rust
use primer::sieve_primes;
```

while publication and installation use the registry package `primer-sieve`.

Expected dependency installation:

```powershell
cargo add primer-sieve --rename primer
```

Expected CLI installation:

```powershell
cargo install primer-sieve
primer 1_000_000
```

---

## 3. Current milestone

The project has successfully crossed from “standalone Rust source plus benchmark archive” into a conventional Cargo package.

The validated state includes:

- root Cargo package;
- canonical library source;
- command-line binary;
- public integration tests;
- runnable example;
- Cargo metadata for the website and repository;
- exact crates.io package filtering;
- successful package construction;
- successful publication dry run;
- no actual publication.

This is the first solid release-candidate milestone.

---

## 4. Domain and website state

`primercrate.rs` is registered and delegated to Cloudflare.

Verified DNS and HTTP state:

- authoritative nameservers:
  - `ruth.ns.cloudflare.com`
  - `cris.ns.cloudflare.com`
- apex resolves through Cloudflare;
- `https://primercrate.rs/` returns `200 OK`;
- `https://www.primercrate.rs/...` returns `301 Moved Permanently`;
- the redirect preserves both path and query string;
- the redirected apex request returns `200 OK`;
- HTTPS is working;
- the temporary Primer landing page is deployed through Cloudflare Pages.

Canonical URL:

```text
https://primercrate.rs/
```

`www` is intentionally redirected to the apex domain.

The temporary website bundle/folder may still physically exist inside `C:\github\primer`, but the corrected `.gitignore` is intended to ignore:

```text
primercrate-placeholder/
primercrate-placeholder.zip
```

Do not add those temporary deployment artifacts to the crate commit.

---

## 5. Repository history and safety state

Original audit state before Cargo migration:

```text
Repository: C:\github\primer
Branch: main
Remote: https://github.com/whisprer/primer.git
```

The migration created:

- a timestamped safety branch matching:

```text
safety/pre-cargo-v0.3.0-*
```

- the working feature branch:

```text
feature/cargo-crate-v0.3.0
```

The exact timestamped safety-branch name was not captured in the final chat. Discover it with:

```powershell
cd C:\github\primer

git branch --list "safety/pre-cargo-v0.3.0-*"
git branch --show-current
git status --short --branch
```

The pre-migration canonical standalone source hash was:

```text
245A61DF4A4D3185C192421F8891EED3FF507EF523DF207E044D2FC0BE068474
```

The following three historical files were verified byte-for-byte identical:

```text
primer-crate\primer.rs
src\final-package\primer.rs
src\rust\primer-[seg final]\primer.rs
```

They are retained for provenance but are not part of the published package.

---

## 6. Canonical package structure

The authoritative implementation is now expected at:

```text
C:\github\primer\
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── LICENSE.md
├── .gitignore
├── src\
│   ├── lib.rs
│   └── bin\
│       └── primer.rs
├── examples\
│   └── basic.rs
├── tests\
│   └── public_api.rs
├── docs\
│   ├── CARGO_MIGRATION_v0.3.0.md
│   └── archive\
│       ├── README-pre-cargo-v0.2.0.md
│       └── README-legacy-secondary.md
└── .github\
    └── workflows\
        └── cargo.yml
```

Historical material remains elsewhere in the repository:

```text
primer-crate/
src/final-package/
src/rust/
packages/
assets/
```

Those directories are repository provenance and experiments, not crate contents.

---

## 7. Exact crates.io package contents

The final package review was designed to accept only this package set:

```text
.cargo_vcs_info.json
CHANGELOG.md
Cargo.lock
Cargo.toml
Cargo.toml.orig
LICENSE.md
README.md
examples/basic.rs
src/bin/primer.rs
src/lib.rs
tests/public_api.rs
```

The package must not include:

```text
primer-crate/
src/final-package/
src/rust/
assets/
packages/
docs/
primercrate-placeholder/
primercrate-placeholder.zip
rustlang-banner.png
rustlang-backcard.png
README.md2.md
file_structure.md
```

The final `Cargo.toml` package include list should be exact, not globbed:

```toml
include = [
    "/Cargo.toml",
    "/README.md",
    "/CHANGELOG.md",
    "/LICENSE.md",
    "/src/lib.rs",
    "/src/bin/primer.rs",
    "/examples/basic.rs",
    "/tests/public_api.rs",
]
```

Do not restore the earlier broad pattern:

```toml
"/src/**/*.rs"
```

That pattern was the cause of the package-contamination failure.

---

## 8. Algorithm and API state

Primer is an odd-only, bit-packed, segmented Sieve of Eratosthenes.

Current public API:

```rust
pub fn sieve_primes(limit: u64) -> Vec<u64>
```

Compatibility API:

```rust
pub fn segmented_sieve(limit: u64) -> Vec<u64>
```

Published constants include:

```rust
pub const SEGMENT_BYTES: usize = 32 * 1024;
pub const SEGMENT_BITS: u64 = (SEGMENT_BYTES * 8) as u64;
```

Important claim discipline:

> **32 KiB is the reusable segment buffer, not the total process memory and not the total heap use of `sieve_primes`.**

Additional memory is used for:

- bootstrap primes up to `sqrt(limit)`;
- the returned `Vec<u64>`;
- allocator metadata;
- ordinary process/runtime overhead.

Safe promotional wording:

> Primer is a cache-aware, bit-packed segmented prime sieve with a reusable 32 KiB segment buffer, zero runtime dependencies, and competitive single-threaded throughput.

Do not currently claim:

- “Primer only uses 32 KiB total memory”;
- “Primer is always as fast as `primal`”;
- “Primer uses 64× less total memory”;
- “Primer is the fastest Rust prime crate.”

Those require the new canonical benchmark suite.

---

## 9. Validation already completed

The Cargo migration succeeded before the final package-name correction.

The final package repair then reportedly completed successfully and validated:

1. Cargo metadata package name:
   - `primer-sieve`

2. Cargo library target:
   - `primer`

3. Cargo binary target:
   - `primer`

4. Exact package file set.

5. Live crates.io name lookup:
   - `primer-sieve` was unoccupied at the time of the check.

6. Package construction:

```powershell
cargo package --allow-dirty
```

7. Publication dry run:

```powershell
cargo publish --dry-run --allow-dirty
```

8. Expected archive:

```text
C:\github\primer\target\package\primer-sieve-0.3.0.crate
```

The archive size and SHA-256 were not pasted into the conversation. Re-read them locally if desired.

Crucially:

- no real publication occurred;
- no commit occurred;
- no push occurred;
- no tag occurred.

The crates.io name check is time-sensitive and does not reserve the name. Recheck immediately before real publication.

---

## 10. Superseded files: do not rerun

The Downloads folder may contain many obsolete artifacts from the failed attempts.

Treat all of these as superseded:

```text
Audit-PrimerRepository.ps1
Collect-PrimerCore.ps1

primer-cargo-migration-v0.3.0.zip
primer-cargo-migration-v0.3.0-r1.zip
primer-cargo-migration-v0.3.0-r2.zip

primer-release-review-v0.3.0.zip
primer-release-review-v0.3.0-r1.zip

PRIMER-SIEVE-FIX.zip
PRIMER-SIEVE-FINAL.zip

any extracted directories named:
primer-cargo-migration-run*
primer-release-review-run*
PRIMER-SIEVE-FIX-RUN*
PRIMER-SIEVE-FINAL-RUN*
```

`FIX-PRIMER-NOW.ps1` was the final successful repair script. It should now be archived as evidence, not rerun routinely.

No new conversation should begin by applying another migration bundle.

---

## 11. First action in the fresh conversation

The next conversation must begin with a read-only inspection.

Run:

```powershell
cd C:\github\primer

Write-Host "`n=== Branch ===" -ForegroundColor Cyan
git branch --show-current

Write-Host "`n=== Safety branch ===" -ForegroundColor Cyan
git branch --list "safety/pre-cargo-v0.3.0-*"

Write-Host "`n=== Status ===" -ForegroundColor Cyan
git status --short --branch

Write-Host "`n=== Diff summary ===" -ForegroundColor Cyan
git diff --stat

Write-Host "`n=== Package identity ===" -ForegroundColor Cyan
cargo metadata --format-version 1 --no-deps |
    ConvertFrom-Json |
    Select-Object -ExpandProperty packages |
    Select-Object name, version, manifest_path

Write-Host "`n=== Exact package contents ===" -ForegroundColor Cyan
cargo package --allow-dirty --list

Write-Host "`n=== Existing package archive ===" -ForegroundColor Cyan
Get-ChildItem `
    -LiteralPath ".\target\package" `
    -Filter "primer-sieve-0.3.0.crate" `
    -File `
    -ErrorAction SilentlyContinue |
Select-Object Name, Length, LastWriteTime, FullName
```

Expected essentials:

```text
branch: feature/cargo-crate-v0.3.0
package: primer-sieve 0.3.0
library: primer
binary: primer
historical src files absent from cargo package list
```

Do not modify anything until this inspection agrees with the handover.

---

## 12. Recommended immediate next milestone

The next milestone is:

> **Review the uncommitted diff, commit the validated Cargo conversion, and push the feature branch.**

Do not publish to crates.io before the feature branch is committed and backed up remotely.

After reviewing `git status` and `git diff`, the likely intended commit set is:

```text
Cargo.toml
README.md
CHANGELOG.md
.gitignore
src/lib.rs
src/bin/primer.rs
examples/basic.rs
tests/public_api.rs
docs/CARGO_MIGRATION_v0.3.0.md
docs/archive/README-pre-cargo-v0.2.0.md
docs/archive/README-legacy-secondary.md
.github/workflows/cargo.yml
```

Verify the actual status before staging. Do not blindly assume every listed file exists.

Safe staging pattern after review:

```powershell
cd C:\github\primer

git add -- `
    Cargo.toml `
    README.md `
    CHANGELOG.md `
    .gitignore `
    src/lib.rs `
    src/bin/primer.rs `
    examples/basic.rs `
    tests/public_api.rs `
    docs/CARGO_MIGRATION_v0.3.0.md `
    docs/archive/README-pre-cargo-v0.2.0.md `
    docs/archive/README-legacy-secondary.md `
    .github/workflows/cargo.yml

git status --short
git diff --cached --stat
git diff --cached
```

Only after the staged diff has been inspected:

```powershell
git commit -m "Package Primer as primer-sieve 0.3.0"
git push -u origin feature/cargo-crate-v0.3.0
```

Then verify:

```powershell
git status --short --branch
git log --oneline --decorate -5
```

Expected final status after push:

```text
feature/cargo-crate-v0.3.0...origin/feature/cargo-crate-v0.3.0
```

with no unintended working-tree changes.

---

## 13. Publication is deliberately not next

Do not immediately run:

```powershell
cargo publish
```

Before publication, complete these gates:

1. Commit and push the feature branch.
2. Confirm GitHub Actions passes.
3. Review the public README as rendered on GitHub.
4. Confirm the package list one more time.
5. Confirm `primer-sieve` is still available.
6. Decide whether to merge to `main` before publishing.
7. Create a clean release commit/tag plan.
8. Preserve the exact `.crate` SHA-256.
9. Decide whether the current project licence is suitable for crates.io adoption.
10. Build the canonical benchmark harness before making comparative headline claims.

Publication is irreversible in important ways. A published crate version cannot simply be overwritten.

---

## 14. Canonical benchmark plan

The next engineering project after the Cargo commit is a new benchmark package under something like:

```text
benchmarks/canonical/
```

Compare pinned versions:

```text
Primer 0.3.0 release candidate
primes 0.4.0
primal 0.3.3
```

Canonical retained-enumeration limits:

```text
500,000
1,000,000
10,000,000
50,000,000
100,000,000
```

Equal-work rule:

- enumerate all primes `<= n`;
- retain the complete ordered result;
- use equivalent result representation where possible;
- validate count, final prime, and deterministic checksum;
- do not compare one crate’s counting operation with another crate’s full materialisation.

Separate metrics:

1. algorithm elapsed time;
2. whole-process elapsed time;
3. reusable sieve-buffer size;
4. bootstrap-prime storage;
5. result-vector logical bytes;
6. result-vector capacity bytes;
7. peak observed process memory;
8. allocation profile, if later instrumented.

Separate build tracks:

```text
portable release
native release using -C target-cpu=native
```

Never mix the two in one ranking.

Dataset requirements:

- exact hardware;
- OS build;
- Rust version;
- Cargo version;
- target triple;
- compiler flags;
- Git commit;
- Cargo.lock hash;
- benchmark binary hash;
- raw JSON Lines;
- CSV summary;
- reproducible report-generation command.

Headline claims are not allowed until raw results from at least two machines reproduce the direction of the result.

---

## 15. Website roadmap after package and benchmarks

The current site is a temporary placeholder.

The finished `primercrate.rs` should become a public performance laboratory, not merely a brochure.

Planned major sections:

1. Hero:
   - “A prime sieve built to live in cache.”
   - install command;
   - GitHub link;
   - benchmark link.

2. Interactive WebAssembly demonstration:
   - enter a limit;
   - generate primes locally;
   - show count and elapsed time;
   - label browser timings as illustrative.

3. Benchmark laboratory:
   - input limit selector;
   - implementation selector;
   - machine selector;
   - portable/native build selector;
   - raw JSON/CSV download;
   - exact reproduction commands.

4. Algorithm visualisation:
   - odd-only representation;
   - one candidate per bit;
   - 32 KiB reusable segment;
   - bootstrap primes;
   - segment clear/reuse cycle;
   - trailing-zero scanning.

5. Honest performance page:
   - where Primer wins;
   - where competitors win;
   - precise memory definitions;
   - known limitations.

6. Documentation:
   - library API;
   - CLI;
   - examples;
   - MSRV;
   - licence;
   - docs.rs link after publication.

7. Development story:
   - C++ origin;
   - Rust port;
   - borrow-checker lessons;
   - segmentation;
   - cache behaviour;
   - future streaming/count-only APIs.

---

## 16. Known unresolved decisions

These remain open and must not be silently invented in a fresh conversation:

1. **Root `Cargo.lock` policy**
   - Cargo generates a lockfile in the package archive.
   - The repository’s `.gitignore` may ignore the root lockfile.
   - Decide deliberately whether the mixed library/CLI project should track it.

2. **MSRV**
   - The package currently declares Rust `1.70`.
   - A prior release review identified that Cargo manifest `[lints]` tables require Cargo 1.74.
   - The final successful script focused on package identity and filtering.
   - Before publication, explicitly test with `cargo +1.70.0 check --all-targets` and `cargo +1.70.0 test --all-targets`, or raise the declared MSRV honestly.

3. **CI workflow**
   - The base migration installed a stable cross-platform Cargo workflow.
   - The later enhanced MSRV workflow may have been rolled back during failed release-review attempts.
   - Inspect `.github/workflows/cargo.yml`; do not assume the Rust 1.70 CI job exists.

4. **Benchmark protocol document**
   - A detailed protocol was designed in conversation.
   - It may not exist in the repository because failed release-review scripts restored their managed files.
   - Recreate it deliberately after the Cargo commit.

5. **Licence suitability**
   - Existing `LICENSE.md` was retained.
   - Its compatibility with broad crate adoption has not yet been reviewed in this handover.

6. **Actual commit state**
   - The scripts did not commit.
   - The next conversation must inspect and commit only after reviewing the diff.

7. **Crates.io name**
   - `primer-sieve` appeared free during the successful final check.
   - It is not reserved until actual publication.

---

## 17. Anti-error rules for the next conversation

The following rules are mandatory:

- Do not generate another migration ZIP.
- Do not rerun old scripts.
- Do not rewrite the repository from memory.
- Do not claim files exist without checking `git status` or the filesystem.
- Do not run long validation before a cheap package-list check.
- Do not use broad Cargo package globs under `src`.
- Do not publish before commit, push, CI, and final package inspection.
- Do not call 32 KiB “total memory.”
- Do not invent benchmark results.
- Do not replace working source merely to make it look more idiomatic.
- Prefer one transparent PowerShell command set over nested generated wrappers.
- After every write operation, inspect Git status and diff immediately.
- If a check fails, state whether the real repository changed before proposing another command.

---

## 18. Fresh-conversation opening prompt

Paste this into the new conversation:

> We are resuming the Primer Rust crate project from a validated handover.
>
> Read the attached handover completely before suggesting any action.
>
> Current intended state:
> - repo: `C:\github\primer`
> - branch: `feature/cargo-crate-v0.3.0`
> - registry package: `primer-sieve`
> - Rust library: `primer`
> - CLI: `primer`
> - version: `0.3.0`
> - domain: `https://primercrate.rs`
> - Cargo package/dry-run reportedly passed
> - nothing has been committed, pushed, tagged, or published by the final script
>
> Do not rerun or recreate any migration bundles.
>
> First give me one read-only PowerShell inspection command set for branch, Git status, diff summary, Cargo metadata, exact package contents, package archive, and safety branch. After I return its output, help me review the diff, commit, and push the feature branch safely. Do not proceed to crates.io publication yet.

---

## 19. Bottom line

The project is on solid ground.

The hard-won stable facts are:

- domain and canonical redirect work;
- the standalone algorithm has been packaged as a real Cargo library and CLI;
- the registry package is `primer-sieve`;
- source code still imports as `primer`;
- the package file set is exact;
- package construction and publication dry-run passed;
- historical source was excluded from the crate;
- no real publication occurred;
- the next step is inspection, commit, and push—not another migration.
