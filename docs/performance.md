# Boundary Performance

Boundra performance claims use a reproducible synthetic workspace rather than
informal observations from small examples.

## Run

```bash
cargo build --release -p boundra-cli
pnpm benchmark:boundaries
pnpm benchmark:boundaries:large
```

The benchmark creates a temporary app with 1,000 or 10,000 TypeScript source
files. Every file imports a declared domain public API, so the run measures
project loading, import scanning, alias resolution, and boundary evaluation
without producing violations.

Reported values:

- first-run wall time (`cold_ms`)
- median subsequent wall time (`warm_median_ms`)
- maximum observed resident set size (`max_rss_mb`) where `/usr/bin/time`
  exposes it
- operating system, architecture, Node version, file count, and iterations

## Regression Gate

CI runs a 1,000-file smoke benchmark with deliberately broad ceilings. The
gate is intended to catch catastrophic regressions, not normal runner variance.
Release decisions should compare repeated results on the same environment.

## Current Baseline

Local baseline captured on 2026-07-03 (`darwin-arm64`, Node 24.14.0):

| Files | Cold | Warm median | Peak RSS |
| ---: | ---: | ---: | ---: |
| 1,000 | 20.74 ms | 18.92 ms | 2.55 MB |
| 10,000 | 163.32 ms | 164.20 ms | unavailable in sandbox |

Run the benchmark again on each release candidate and record materially changed
results in the matching release note. Do not compare different hardware as if
the results were equivalent. Linux CI reports peak RSS through `/usr/bin/time`.
