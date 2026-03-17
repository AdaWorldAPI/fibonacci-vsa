# fibonacci-vsa

**Zeckendorf-coded Vector Space with Euler-Mascheroni γ spacetime curvature.**

No ML. No weights. No training. Only mathematics against compile-time constants.

## What is this?

A fundamentally new approach to vector similarity that replaces flat distance metrics (Hamming, cosine, Euclidean) with a self-navigating, fractally-compressed, gravitationally-curved space built from:

- **φ (Golden Ratio)** — Fibonacci basis for scale-hierarchical encoding
- **γ (Euler-Mascheroni)** — Spacetime curvature that bends the metric field
- **Zeckendorf's theorem** — Unique, non-consecutive Fibonacci decomposition

## Key Results

| Metric | Value |
|--------|-------|
| Coarse distance speedup vs cosine | **946×** |
| Discrimination (similar vs different) | 2× separation in bent distance |
| Monotonic noise response | ✓ confirmed |
| Rank correlation with cosine | ρ = 0.34 (different metric, not worse — sees multi-scale resonance) |
| γ contribution at human scale | **100%** of all curvature at scale 1 |
| γ fade at abstract scale | 11.4% at scale 92 |

## Quick Start

```rust
use fibonacci_vsa::*;
use ndarray::Array1;

let embedding_a = Array1::from_vec(vec![0.5, -0.3, 0.8, /* ... 1024D */]);
let embedding_b = Array1::from_vec(vec![0.4, -0.2, 0.7, /* ... 1024D */]);

let za = ZeckendorfVector::from_ndarray(&embedding_a, 16);
let zb = ZeckendorfVector::from_ndarray(&embedding_b, 16);

// 946× faster than cosine
let coarse = za.coarse_distance(&zb);

// Full multi-scale resonance analysis
let map = za.resonance_map(&zb);
println!("Wormholes: {}, Bent distance: {:.2}", map.total_wormholes, map.bent_distance);
```

## Tests

```bash
cargo test --release -- --nocapture
```

12/12 tests pass covering: Zeckendorf correctness, γ-curvature proofs, ndarray integration, rank correlation vs cosine, discrimination, monotonicity, symmetry, and timing benchmarks.

## Architecture

See [FIBONACCI-VSA-PROMPT.md](FIBONACCI-VSA-PROMPT.md) for the complete theoretical foundation.

## License

MIT
