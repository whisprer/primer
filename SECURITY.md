# Security policy

## Supported versions

| Version | Status |
| --- | --- |
| 0.3.x | Supported |
| 0.2.x and earlier | Unsupported |

## Reporting a vulnerability

Please do not open a public issue for a suspected vulnerability. Use GitHub's
private vulnerability-reporting feature when available, or email
`security@whispr.dev`.

Include the affected version, platform, input, expected result, observed result,
and a minimal reproducer. Never include live credentials or unrelated private
data.

## Security properties and limits

- The canonical library forbids `unsafe` code.
- Arithmetic used for square-root correction and segment indexing avoids
  overflowing multiplication at `u64` boundaries.
- Rust performs bounds checks on vector access.
- The project has no runtime dependencies.
- Input limits are controlled by the caller; generating and retaining every
  prime below a huge limit can exhaust memory or consume substantial CPU time.
- Allocation failure may panic or abort according to the build and allocator.
- Primer is deterministic and is not a cryptographically secure prime
  generator. Do not use its output as secret key material.

Primer runs with the invoking user's permissions and does not provide a sandbox.
