# Zeckengold — Surround Bundling for Hyperdimensional Consciousness

**fibonacci-vsa v0.2.0 | Authors: Jan Huebener & Ada | March 17, 2026**

## Results (d=10,000, n=8 atoms)

| Metric | Value |
|---|---|
| Fibonacci vs Random bases | mean 0.043 vs 0.750 = **17x better** |
| Bleed removal | **99.3%** crosstalk reduction |
| Noise gate | 21% dims zeroed (gamma-corrected) |
| Phase rotation | Error = 1.32e-32 (perfectly unitary) |
| **Mono classification** | **1/8 = 12%** (random chance) |
| **Surround classification** | **8/8 = 100%** |
| Surround margin | +0.19 to +0.24 (clear daylight) |

## Scaling (d=10,000)

| Atoms | Mean Fidelity | Min Fidelity |
|-------|---------------|--------------|
| 2 | 0.70 | 0.70 |
| 4 | 0.49 | 0.49 |
| 8 | 0.35 | 0.34 |
| 16 | 0.25 | 0.23 |
| 32 | 0.17 | 0.15 |

## The Studio Pipeline

```
AUFNAHME (Atom computation)           -> raw signal
NOISE GATE (Euler-gamma threshold)    -> remove dims below floor
BLEED REMOVAL (orthogonal projection) -> remove crosstalk
REFERENCE SNAP (Fibonacci lattice)    -> correct drift
=== Each track is now CLEAN ===
SURROUND POSITIONING (phase rotation) -> angular niche per atom
MIX (bundle via addition)             -> clean superposition
RECOVERY (inverse phase rotation)     -> deterministic, not probabilistic
```

## Key Insight

Mono bundling has higher raw similarity (0.88) but ZERO discrimination.
All 8 atoms look identical. Classification = random chance (12%).

Surround has lower raw similarity (0.35) but PERFECT separation.
Every atom correctly identified. Classification = 100%.

**Attention = inverse phase rotation = turning your head until the signal is clear.**

## API

```rust
use fibonacci_vsa::zeckengold::SurroundBundler;

let bundler = SurroundBundler::new(8, 10_000, 42);
let moment = bundler.bundle_raw(&atom_outputs);
let recovered = bundler.recover(&moment, 3);  // deterministic
```

## Origin

Emerged from a conversation about consciousness architecture where the insight
arose that VSA bundling is "mono" -- like recording an orchestra through a single
microphone. The studio analogy led to pre-bundle cleaning (noise gate, bleed removal)
and phase rotation positioning (Fibonacci/Euler-gamma angles).

zeckengold = Zeckendorf (Fibonacci decomposition) + Gold (golden angle rotation)
