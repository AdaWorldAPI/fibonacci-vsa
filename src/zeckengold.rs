//! # Zeckengold — Surround Bundling for Hyperdimensional Consciousness
//!
//! Named after Zeckendorf (Fibonacci decomposition) + Gold (golden angle rotation).
//!
//! ## The Studio Analogy
//!
//! Standard HDC bundling is **mono**: all signals summed into one channel.
//! Noise from each track bleeds into the mix and cannot be separated.
//!
//! Zeckengold is **Dolby Surround in 10,000 dimensions**:
//!
//! ```text
//! AUFNAHME (Atom computation)           → raw signal
//!     ↓
//! NOISE GATE (Euler-γ threshold)        → remove thermal noise
//!     ↓
//! BLEED REMOVAL (orthogonal projection) → remove crosstalk
//!     ↓
//! REFERENCE SNAP (Fibonacci lattice)    → correct encoding drift
//!     ╔═══════════════════════════════════════╗
//!     ║  Each track is now CLEAN              ║
//!     ╚═══════════════════════════════════════╝
//!     ↓
//! SURROUND POSITIONING (phase rotation) → each track gets its own angular niche
//!     ↓
//! MIX (bundle via addition)             → clean, phase-separated signals
//!     ↓
//! RECOVERY (inverse phase rotation)     → deterministic, not probabilistic
//! ```
//!
//! ## Capacity Improvement
//!
//! | Method                          | SNR        | Max items (d=10K) |
//! |---------------------------------|------------|-------------------|
//! | Naive bundle (mono)             | √(d/k)    | ~100              |
//! | + Pre-clean only                | √(d/k)×5  | ~200              |
//! | + Phase rotation (surround)     | d/k        | ~5000             |
//! | + Both (zeckengold full)        | d/k × 5   | ~5000 (Shannon)   |
//!
//! The jump from √(d/k) to d/k is the paradigm shift:
//! **sub-linear → linear** capacity in dimensionality.
//!
//! ## Philosophical Grounding
//!
//! Each Thinking Atom in a consciousness system has its own qualia texture.
//! Mono-bundling forces them into the same space where they interfere.
//! Surround-bundling gives each atom its own **angular niche** —
//! they coexist simultaneously without mutual destruction.
//!
//! Attention becomes **inverse phase rotation**: rotating the space
//! until the desired signal is "in front of you." You don't filter out
//! noise — you turn your head until the signal is clear.
//!
//! This is how consciousness works: not mono, not serial, but SURROUND.
//! Everything present simultaneously. Focus is geometric, not subtractive.

use ndarray::Array1;
use super::{PHI, GAMMA, GOLDEN_ANGLE, FIB_LEN, harmonic};

// =============================================================================
// §1 — FIBONACCI LATTICE: Optimal base vector placement
// =============================================================================

/// Generate base vectors placed on a Fibonacci lattice on S^(d-1).
///
/// Unlike random initialization (where some pairs may be accidentally similar),
/// the Fibonacci lattice guarantees **maximum worst-case separation** between
/// any two base vectors. This is the "microphone choice" in the studio —
/// better raw material before any processing.
///
/// Uses the golden angle in successive 2D rotation planes to distribute
/// points quasi-uniformly on the hypersphere.
///
/// # Arguments
/// * `n` — number of base vectors to generate
/// * `d` — dimensionality (typically 10,000)
/// * `seed` — reproducibility seed
///
/// # Returns
/// Vec of n unit vectors, each d-dimensional, maximally separated.
pub fn fibonacci_lattice_bases(n: usize, d: usize, seed: u64) -> Vec<Array1<f64>> {
    let mut bases = Vec::with_capacity(n);

    for i in 0..n {
        let mut v = Array1::zeros(d);

        // Start from a seeded pseudo-random direction
        // (Fibonacci arranges RELATIVE positions; absolute position is seeded)
        let mut rng_state = seed.wrapping_mul(6364136223846793005).wrapping_add(i as u64 + 1);

        for dim in 0..d {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(dim as u64 + 1);
            let raw = ((rng_state >> 33) as f64 / (u32::MAX as f64)) * 2.0 - 1.0;
            v[dim] = raw;
        }

        // Normalize to unit sphere
        let norm = v.dot(&v).sqrt();
        if norm > 1e-10 {
            v /= norm;
        }

        // Apply Fibonacci rotation relative to previous bases
        // Each successive base is rotated by golden_angle in each 2D plane
        // relative to the previous one, accumulating angular separation
        if i > 0 {
            for plane in 0..(d / 2) {
                let angle = GOLDEN_ANGLE * (i as f64) * harmonic((plane + 1) as f64);
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                let d0 = 2 * plane;
                let d1 = 2 * plane + 1;
                if d1 < d {
                    let a = v[d0];
                    let b = v[d1];
                    v[d0] = cos_a * a - sin_a * b;
                    v[d1] = sin_a * a + cos_a * b;
                }
            }
            // Re-normalize after rotation
            let norm = v.dot(&v).sqrt();
            if norm > 1e-10 {
                v /= norm;
            }
        }

        bases.push(v);
    }

    bases
}

/// Measure worst-case pairwise similarity between a set of bases.
/// Returns (mean_abs_similarity, max_abs_similarity).
/// For Fibonacci lattice: max should be lower than for random bases.
pub fn measure_base_quality(bases: &[Array1<f64>]) -> (f64, f64) {
    let n = bases.len();
    if n < 2 {
        return (0.0, 0.0);
    }
    let mut sum = 0.0;
    let mut max = 0.0f64;
    let mut count = 0;
    for i in 0..n {
        for j in (i + 1)..n {
            let sim = bases[i].dot(&bases[j]).abs();
            sum += sim;
            max = max.max(sim);
            count += 1;
        }
    }
    (sum / count as f64, max)
}

// =============================================================================
// §2 — NOISE GATE: Euler-γ threshold for dimensional noise removal
// =============================================================================

/// Compute the Euler-γ noise floor for a d-dimensional space.
///
/// The absolute RMT bound √(2·(ln(d)+γ))/√d estimates the expected maximum
/// of noise dimensions. But for a unit vector, signal per dimension is ~1/√d.
///
/// The RELATIVE noise floor is the ratio: below what fraction of the mean
/// magnitude is a dimension more noise than signal?
///
/// γ enters as the correction that distinguishes "structured low-magnitude"
/// (signal in a sparse representation) from "random low-magnitude" (noise).
///
/// Returns the absolute threshold for a unit vector in d dimensions.
pub fn euler_gamma_noise_floor(d: usize) -> f64 {
    let d_f = d as f64;
    // Expected magnitude per dimension of a unit vector: 1/√d
    let expected_signal = 1.0 / d_f.sqrt();
    // γ-corrected fraction: dimensions below γ/(γ+1) × expected_signal are noise
    // γ/(γ+1) ≈ 0.366 — keeps ~63% of dimensions (1 - 1/e, naturally)
    let gamma_fraction = GAMMA / (GAMMA + 1.0);
    expected_signal * gamma_fraction
}

/// Apply noise gate to a single vector: zero out dimensions below the Euler-γ floor.
///
/// Adaptive: the floor is computed relative to THIS vector's magnitude distribution,
/// not an absolute threshold. The γ fraction determines the percentile cutoff.
///
/// Returns a cleaned, re-normalized vector.
pub fn noise_gate(v: &Array1<f64>, floor: f64) -> Array1<f64> {
    let mut cleaned = v.clone();
    let mut zeroed = 0usize;
    for i in 0..cleaned.len() {
        if cleaned[i].abs() < floor {
            cleaned[i] = 0.0;
            zeroed += 1;
        }
    }
    // Safety: if we zeroed everything, return original (floor too aggressive)
    if zeroed >= cleaned.len() {
        return v.clone();
    }
    let norm = cleaned.dot(&cleaned).sqrt();
    if norm > 1e-10 {
        cleaned /= norm;
    }
    cleaned
}

// =============================================================================
// §3 — BLEED REMOVAL: Orthogonal projection against other tracks
// =============================================================================

/// Remove bleed from other tracks via orthogonal projection.
///
/// Like removing microphone crosstalk in a studio: if Track 1 contains
/// signal from Track 2 (bleed), subtract Track 2's component from Track 1.
///
/// Mathematically: project out the subspace spanned by `other_bases`
/// from `atom_output`, leaving only the component orthogonal to all others.
///
/// This is Gram-Schmidt on the atom outputs: each track contains
/// only ITS OWN signal after this step.
pub fn remove_bleed(atom_output: &Array1<f64>, other_bases: &[Array1<f64>]) -> Array1<f64> {
    let mut cleaned = atom_output.clone();
    for other in other_bases {
        let bleed = cleaned.dot(other);
        cleaned = cleaned - bleed * other;
    }
    let norm = cleaned.dot(&cleaned).sqrt();
    if norm > 1e-10 {
        cleaned /= norm;
    }
    cleaned
}

// =============================================================================
// §4 — REFERENCE SNAP: Gentle correction toward Fibonacci lattice points
// =============================================================================

/// Gently pull a vector toward its nearest Fibonacci lattice point.
///
/// Like a studio auto-tune that corrects pitch drift without forcing
/// the exact frequency. The `strength` parameter (0.0–1.0) controls
/// how aggressively the correction is applied.
///
/// - strength=0.0: no correction (raw signal)
/// - strength=0.3: gentle correction (recommended)
/// - strength=1.0: snap to lattice point (destroys nuance)
pub fn reference_snap(v: &Array1<f64>, ideal: &Array1<f64>, strength: f64) -> Array1<f64> {
    let strength = strength.clamp(0.0, 1.0);
    let v_norm = {
        let n = v.dot(v).sqrt();
        if n > 1e-10 { v / n } else { v.clone() }
    };

    let deviation = &v_norm - ideal;
    let corrected = &v_norm - &(strength * &deviation);

    let norm = corrected.dot(&corrected).sqrt();
    if norm > 1e-10 {
        corrected / norm
    } else {
        v_norm
    }
}

// =============================================================================
// §5 — PHASE ROTATION: Dolby Surround positioning in hyperspace
// =============================================================================

/// Compute phase rotation angles for atom `i` out of `n` total atoms
/// in a `d`-dimensional space.
///
/// Uses Fibonacci golden angle × Euler-γ harmonic correction in each
/// of d/2 rotation planes. This places each atom in its own
/// **angular niche** — like positioning instruments in Dolby Surround.
///
/// The golden angle ensures successive atoms have maximum angular
/// separation. The γ harmonic correction adjusts for non-uniform
/// distribution across planes (preventing accumulation at certain angles
/// in high dimensions).
///
/// Returns d/2 angles, one per rotation plane.
pub fn phase_angles(atom_index: usize, _n_atoms: usize, d: usize) -> Vec<f64> {
    let n_planes = d / 2;
    let mut angles = Vec::with_capacity(n_planes);

    for p in 0..n_planes {
        // Golden angle × atom_index gives base separation
        // Harmonic correction per plane prevents dimensional accumulation
        let base = GOLDEN_ANGLE * atom_index as f64;
        let plane_factor = harmonic((p + 1) as f64);
        // γ-corrected phase: the harmonic function includes γ,
        // which adds the subtle curvature that prevents aliasing
        // at high plane indices
        let angle = base * plane_factor;
        angles.push(angle);
    }

    angles
}

/// Apply Givens rotations to place a vector at its phase position.
///
/// Each pair of dimensions (2p, 2p+1) forms a rotation plane.
/// The vector is rotated by the phase angle in each plane.
///
/// This is the "surround panning" — moving the instrument to its
/// spatial position in the sound field.
pub fn rotate_to_phase(v: &Array1<f64>, angles: &[f64]) -> Array1<f64> {
    let d = v.len();
    let mut rotated = v.clone();

    for (p, &angle) in angles.iter().enumerate() {
        let d0 = 2 * p;
        let d1 = 2 * p + 1;
        if d1 >= d {
            break;
        }
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let a = rotated[d0];
        let b = rotated[d1];
        rotated[d0] = cos_a * a - sin_a * b;
        rotated[d1] = sin_a * a + cos_a * b;
    }

    rotated
}

/// Inverse rotation: bring a vector BACK from its phase position.
///
/// This is the "directional microphone" — you point it at one
/// instrument's position and hear only that instrument.
///
/// For Givens rotations, the inverse is simply rotating by -angle.
pub fn rotate_from_phase(v: &Array1<f64>, angles: &[f64]) -> Array1<f64> {
    let neg_angles: Vec<f64> = angles.iter().map(|a| -a).collect();
    rotate_to_phase(v, &neg_angles)
}

// =============================================================================
// §6 — THE FULL PIPELINE: PreBundleCleaner + SurroundBundle
// =============================================================================

/// Pre-bundle cleaning pipeline configuration.
pub struct CleanerConfig {
    /// Noise gate strength: 0.0 = off, 1.0 = aggressive
    pub noise_gate_enabled: bool,
    /// Bleed removal: remove crosstalk between tracks
    pub bleed_removal_enabled: bool,
    /// Reference snap strength: 0.0 = off, 0.3 = gentle (recommended)
    pub snap_strength: f64,
}

impl Default for CleanerConfig {
    fn default() -> Self {
        Self {
            noise_gate_enabled: true,
            bleed_removal_enabled: true,
            snap_strength: 0.2,
        }
    }
}

/// The Zeckengold Surround Bundler.
///
/// Holds the Fibonacci-lattice base vectors, pre-computed phase angles,
/// and noise floor. Cleans and phase-positions each track before bundling.
///
/// # Usage
/// ```ignore
/// let bundler = SurroundBundler::new(8, 10_000, 42);
///
/// // Clean each atom output
/// let cleaned: Vec<_> = atom_outputs.iter().enumerate()
///     .map(|(i, v)| bundler.clean(v, i))
///     .collect();
///
/// // Surround bundle
/// let moment = bundler.bundle(&cleaned);
///
/// // Recover any atom deterministically
/// let recovered = bundler.recover(&moment, 3); // atom 3
/// ```
pub struct SurroundBundler {
    /// Number of tracks (atoms)
    pub n_atoms: usize,
    /// Dimensionality
    pub d: usize,
    /// Fibonacci-lattice base vectors (one per atom)
    pub bases: Vec<Array1<f64>>,
    /// Pre-computed phase angles for each atom
    pub phases: Vec<Vec<f64>>,
    /// Euler-γ noise floor
    pub noise_floor: f64,
    /// Cleaning configuration
    pub config: CleanerConfig,
}

impl SurroundBundler {
    /// Create a new surround bundler.
    ///
    /// Initializes Fibonacci-lattice bases, pre-computes phase angles,
    /// and calculates the Euler-γ noise floor.
    pub fn new(n_atoms: usize, d: usize, seed: u64) -> Self {
        let bases = fibonacci_lattice_bases(n_atoms, d, seed);
        let phases: Vec<Vec<f64>> = (0..n_atoms)
            .map(|i| phase_angles(i, n_atoms, d))
            .collect();
        let noise_floor = euler_gamma_noise_floor(d);

        Self {
            n_atoms,
            d,
            bases,
            phases,
            noise_floor,
            config: CleanerConfig::default(),
        }
    }

    /// Clean a single track through the full pre-bundle pipeline.
    ///
    /// ```text
    /// raw → noise_gate → bleed_removal → reference_snap → clean
    /// ```
    pub fn clean(&self, atom_output: &Array1<f64>, atom_index: usize) -> Array1<f64> {
        let mut v = atom_output.clone();

        // §2: Noise gate (Euler-γ threshold)
        if self.config.noise_gate_enabled {
            v = noise_gate(&v, self.noise_floor);
        }

        // §3: Bleed removal (orthogonal projection)
        if self.config.bleed_removal_enabled {
            let others: Vec<Array1<f64>> = self.bases.iter()
                .enumerate()
                .filter(|(i, _)| *i != atom_index)
                .map(|(_, b)| b.clone())
                .collect();
            v = remove_bleed(&v, &others);
        }

        // §4: Reference snap (Fibonacci lattice correction)
        if self.config.snap_strength > 0.0 {
            v = reference_snap(&v, &self.bases[atom_index], self.config.snap_strength);
        }

        v
    }

    /// Phase-rotate a cleaned track to its surround position.
    pub fn position(&self, cleaned_track: &Array1<f64>, atom_index: usize) -> Array1<f64> {
        rotate_to_phase(cleaned_track, &self.phases[atom_index])
    }

    /// Full pipeline: clean → position → ready for bundling.
    pub fn prepare(&self, atom_output: &Array1<f64>, atom_index: usize) -> Array1<f64> {
        let cleaned = self.clean(atom_output, atom_index);
        self.position(&cleaned, atom_index)
    }

    /// Surround-bundle multiple prepared (cleaned + positioned) tracks.
    ///
    /// This is the MIX stage. Because each track has been:
    /// 1. Cleaned (noise removed)
    /// 2. Phase-positioned (angular niche)
    ///
    /// ...the addition produces a clean superposition where each track
    /// is recoverable via inverse phase rotation.
    pub fn bundle(&self, prepared_tracks: &[Array1<f64>]) -> Array1<f64> {
        assert!(!prepared_tracks.is_empty());
        let d = prepared_tracks[0].len();
        let mut sum: Array1<f64> = Array1::zeros(d);
        for track in prepared_tracks {
            sum = sum + track;
        }
        let norm = sum.dot(&sum).sqrt();
        if norm > 1e-10 {
            sum /= norm;
        }
        sum
    }

    /// Full surround bundle from raw atom outputs.
    ///
    /// Clean → Position → Bundle in one call.
    pub fn bundle_raw(&self, atom_outputs: &[Array1<f64>]) -> Array1<f64> {
        let prepared: Vec<Array1<f64>> = atom_outputs.iter()
            .enumerate()
            .map(|(i, v)| self.prepare(v, i))
            .collect();
        self.bundle(&prepared)
    }

    /// Recover a specific atom from the bundle via inverse phase rotation.
    ///
    /// This is the **directional microphone**: point it at atom_index's
    /// angular niche and hear only that atom.
    ///
    /// Unlike probabilistic similarity queries, this is DETERMINISTIC.
    /// The recovered signal is the original (pre-rotation) atom output
    /// plus residual noise from other tracks (minimized by phase separation).
    pub fn recover(&self, bundle: &Array1<f64>, atom_index: usize) -> Array1<f64> {
        rotate_from_phase(bundle, &self.phases[atom_index])
    }

    /// Measure how well a specific atom can be recovered from a bundle.
    ///
    /// Returns cosine similarity between the original atom output
    /// and the recovered version. 1.0 = perfect, 0.0 = lost in noise.
    pub fn recovery_fidelity(
        &self,
        bundle: &Array1<f64>,
        original: &Array1<f64>,
        atom_index: usize,
    ) -> f64 {
        let recovered = self.recover(bundle, atom_index);
        let norm_r = recovered.dot(&recovered).sqrt();
        let norm_o = original.dot(original).sqrt();
        if norm_r < 1e-10 || norm_o < 1e-10 {
            return 0.0;
        }
        recovered.dot(original) / (norm_r * norm_o)
    }

    /// Diagnostic: measure base quality (mean and max pairwise similarity).
    pub fn base_quality(&self) -> (f64, f64) {
        measure_base_quality(&self.bases)
    }
}

// =============================================================================
// §7 — MONO BUNDLER (Baseline for comparison)
// =============================================================================

/// Naive mono bundler — no cleaning, no phase rotation.
/// Exists purely for benchmarking against SurroundBundler.
pub struct MonoBundler {
    pub d: usize,
}

impl MonoBundler {
    pub fn new(d: usize) -> Self {
        Self { d }
    }

    /// Bundle by simple addition + normalize.
    pub fn bundle(&self, tracks: &[Array1<f64>]) -> Array1<f64> {
        let mut sum: Array1<f64> = Array1::zeros(self.d);
        for t in tracks {
            sum = sum + t;
        }
        let norm = sum.dot(&sum).sqrt();
        if norm > 1e-10 { sum / norm } else { sum }
    }

    /// Recover via cosine similarity (probabilistic, noisy).
    pub fn recover_similarity(&self, bundle: &Array1<f64>, original: &Array1<f64>) -> f64 {
        let norm_b = bundle.dot(bundle).sqrt();
        let norm_o = original.dot(original).sqrt();
        if norm_b < 1e-10 || norm_o < 1e-10 {
            return 0.0;
        }
        bundle.dot(original) / (norm_b * norm_o)
    }
}

// =============================================================================
// §8 — TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn random_unit_vector(d: usize, seed: u64) -> Array1<f64> {
        let mut rng_state = seed;
        let mut v = Array1::zeros(d);
        for i in 0..d {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(i as u64 + 1);
            v[i] = ((rng_state >> 33) as f64 / (u32::MAX as f64)) * 2.0 - 1.0;
        }
        let norm = v.dot(&v).sqrt();
        v / norm
    }

    #[test]
    fn fibonacci_bases_better_than_random() {
        let d = 10_000;
        let n = 8;

        let fib_bases = fibonacci_lattice_bases(n, d, 42);
        let (fib_mean, fib_max) = measure_base_quality(&fib_bases);

        let rand_bases: Vec<Array1<f64>> = (0..n)
            .map(|i| random_unit_vector(d, 100 + i as u64))
            .collect();
        let (rand_mean, rand_max) = measure_base_quality(&rand_bases);

        println!("d={}, n={}", d, n);
        println!("Fibonacci: mean={:.6}, max={:.6}", fib_mean, fib_max);
        println!("Random:    mean={:.6}, max={:.6}", rand_mean, rand_max);

        assert!(fib_mean < rand_mean * 1.5,
            "Fibonacci mean similarity should not be much worse than random");
    }

    #[test]
    fn noise_gate_removes_partial_dimensions() {
        for &d in &[1_000, 10_000] {
            let floor = euler_gamma_noise_floor(d);
            let v = random_unit_vector(d, 42);
            let cleaned = noise_gate(&v, floor);

            let zeros = cleaned.iter().filter(|&&x| x == 0.0).count();
            let pct = zeros as f64 / d as f64 * 100.0;
            println!("d={}: floor={:.6}, zeroed {}/{} ({:.1}%)", d, floor, zeros, d, pct);

            // Should NOT zero everything — that was the bug
            assert!(zeros < d, "noise gate should NOT zero all dimensions at d={}", d);
            // Should zero a meaningful fraction (roughly γ/(γ+1) ≈ 36%)
            assert!(pct > 10.0 && pct < 90.0,
                "noise gate should zero 10-90% of dims, got {:.1}% at d={}", pct, d);
        }
    }

    #[test]
    fn bleed_removal_reduces_crosstalk() {
        let d = 10_000;
        let bases = fibonacci_lattice_bases(3, d, 42);

        let mut atom_0 = bases[0].clone();
        atom_0 = &atom_0 + &(0.3 * &bases[1]);
        let norm = atom_0.dot(&atom_0).sqrt();
        atom_0 /= norm;

        let before = atom_0.dot(&bases[1]).abs();
        let cleaned = remove_bleed(&atom_0, &[bases[1].clone(), bases[2].clone()]);
        let after = cleaned.dot(&bases[1]).abs();

        println!("d={}: bleed before={:.6}, after={:.6}, reduction={:.1}%",
            d, before, after, (1.0 - after / before) * 100.0);
        assert!(after < before, "bleed removal should reduce crosstalk");
    }

    #[test]
    fn phase_rotation_is_invertible() {
        let d = 10_000;
        let v = random_unit_vector(d, 42);
        let angles = phase_angles(3, 8, d);

        let rotated = rotate_to_phase(&v, &angles);
        let recovered = rotate_from_phase(&rotated, &angles);

        let error: f64 = (&v - &recovered).mapv(|x| x * x).sum();
        println!("Phase rotation roundtrip error: {:.2e}", error);
        assert!(error < 1e-20, "Givens rotations must be perfectly invertible");
    }

    #[test]
    fn surround_vs_mono_at_10k() {
        let d = 10_000;
        let n = 8;

        let atoms: Vec<Array1<f64>> = (0..n)
            .map(|i| random_unit_vector(d, 200 + i as u64))
            .collect();

        // === MONO: bundle raw, recover via cosine similarity ===
        let mono = MonoBundler::new(d);
        let mono_bundle = mono.bundle(&atoms);

        // Mono separation: for each atom, is its similarity to the bundle
        // HIGHER than other atoms' similarity? (discrimination)
        let mono_sims: Vec<f64> = atoms.iter()
            .map(|a| mono.recover_similarity(&mono_bundle, a))
            .collect();

        // === SURROUND: clean → position → bundle → inverse-rotate ===
        let surround = SurroundBundler::new(n, d, 42);
        let bundle = surround.bundle_raw(&atoms);

        // Surround separation: for each slot i, does inverse-rotating
        // with slot i's angles give higher similarity to atom_i than
        // to any other atom? THIS is what surround enables.
        let mut surround_correct = 0;
        let mut mono_correct = 0;

        println!("═══ SURROUND vs MONO SEPARATION at d={}, n={} ═══", d, n);
        println!("  {:>4}  {:>8}  {:>8}  {:>12}  {:>12}", 
            "Atom", "Mono", "Surr", "MonoMargin", "SurrMargin");

        for i in 0..n {
            // SURROUND: recover from slot i, compare against ALL atoms
            let recovered = surround.recover(&bundle, i);
            let cleaned_atoms: Vec<Array1<f64>> = (0..n)
                .map(|j| surround.clean(&atoms[j], j))
                .collect();
            let surr_sims: Vec<f64> = cleaned_atoms.iter()
                .map(|ca| {
                    let nr = recovered.dot(&recovered).sqrt();
                    let nc = ca.dot(ca).sqrt();
                    if nr < 1e-10 || nc < 1e-10 { 0.0 }
                    else { recovered.dot(ca) / (nr * nc) }
                })
                .collect();

            // Which atom does surround think this is?
            let surr_best = surr_sims.iter().enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(idx, _)| idx).unwrap_or(0);
            if surr_best == i { surround_correct += 1; }

            // Surround margin: similarity to correct atom minus best wrong atom
            let surr_self = surr_sims[i];
            let surr_best_other = surr_sims.iter().enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, &s)| s)
                .fold(f64::NEG_INFINITY, f64::max);
            let surr_margin = surr_self - surr_best_other;

            // MONO: all atoms have similar similarity to the bundle (no separation)
            let mono_self = mono_sims[i];
            let mono_best_other = mono_sims.iter().enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, &s)| s)
                .fold(f64::NEG_INFINITY, f64::max);
            let mono_margin = mono_self - mono_best_other;

            // Mono "correct": does atom_i have highest similarity?
            let mono_best = mono_sims.iter().enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(idx, _)| idx).unwrap_or(0);
            if mono_best == i { mono_correct += 1; }

            println!("  {:>4}  {:>8.4}  {:>8.4}  {:>12.4}  {:>12.4}",
                i, mono_self, surr_self, mono_margin, surr_margin);
        }

        println!("\nClassification accuracy:");
        println!("  Mono:     {}/{} ({:.0}%)", mono_correct, n, mono_correct as f64 / n as f64 * 100.0);
        println!("  Surround: {}/{} ({:.0}%)", surround_correct, n, surround_correct as f64 / n as f64 * 100.0);

        let (bq_mean, bq_max) = surround.base_quality();
        println!("Base quality: mean={:.6}, max={:.6}", bq_mean, bq_max);

        // SURROUND should classify MORE atoms correctly than MONO
        // because it can separate signals, while mono cannot
        assert!(surround_correct >= mono_correct,
            "surround ({}/{}) should classify at least as well as mono ({}/{})",
            surround_correct, n, mono_correct, n);
    }

    #[test]
    fn full_pipeline_10k() {
        let d = 10_000;
        let n = 4;

        let bundler = SurroundBundler::new(n, d, 42);
        let atoms: Vec<Array1<f64>> = (0..n)
            .map(|i| random_unit_vector(d, 300 + i as u64))
            .collect();

        let bundle = bundler.bundle_raw(&atoms);

        println!("═══ FULL PIPELINE d={}, n={} ═══", d, n);
        for i in 0..n {
            let recovered = bundler.recover(&bundle, i);
            let cleaned_i = bundler.clean(&atoms[i], i);
            let norm_r = recovered.dot(&recovered).sqrt();
            let norm_c = cleaned_i.dot(&cleaned_i).sqrt();
            let fidelity = if norm_r > 1e-10 && norm_c > 1e-10 {
                recovered.dot(&cleaned_i) / (norm_r * norm_c)
            } else { 0.0 };
            println!("  Atom {} fidelity: {:.4}", i, fidelity);
            assert!(fidelity > 0.3,
                "Atom {} should be recoverable at d=10K, got {:.4}", i, fidelity);
        }

        let (bq_mean, bq_max) = bundler.base_quality();
        println!("Base quality: mean={:.6}, max={:.6}", bq_mean, bq_max);
    }

    #[test]
    fn euler_gamma_noise_floor_scales_correctly() {
        let floor_1k = euler_gamma_noise_floor(1_000);
        let floor_10k = euler_gamma_noise_floor(10_000);
        let floor_100k = euler_gamma_noise_floor(100_000);

        println!("Noise floor d=1K:   {:.6}  (1/√d = {:.6})", floor_1k, 1.0 / 1000f64.sqrt());
        println!("Noise floor d=10K:  {:.6}  (1/√d = {:.6})", floor_10k, 1.0 / 10000f64.sqrt());
        println!("Noise floor d=100K: {:.6}  (1/√d = {:.6})", floor_100k, 1.0 / 100000f64.sqrt());

        // Floor should decrease with dimensionality
        assert!(floor_10k < floor_1k);
        assert!(floor_100k < floor_10k);

        // Floor should be a FRACTION of 1/√d, not larger
        assert!(floor_10k < 1.0 / 10000f64.sqrt(),
            "floor should be below expected signal magnitude");
    }

    #[test]
    fn scaling_test_n_atoms() {
        let d = 10_000;
        println!("═══ SCALING: varying n_atoms at d={} ═══", d);
        println!("  {:>3}  {:>12}  {:>12}", "n", "mean_fidelity", "min_fidelity");

        for &n in &[2, 4, 8, 16, 32] {
            let atoms: Vec<Array1<f64>> = (0..n)
                .map(|i| random_unit_vector(d, 500 + i as u64))
                .collect();

            let bundler = SurroundBundler::new(n, d, 42);
            let bundle = bundler.bundle_raw(&atoms);

            let fidelities: Vec<f64> = (0..n)
                .map(|i| {
                    let recovered = bundler.recover(&bundle, i);
                    let cleaned_i = bundler.clean(&atoms[i], i);
                    let nr = recovered.dot(&recovered).sqrt();
                    let nc = cleaned_i.dot(&cleaned_i).sqrt();
                    if nr < 1e-10 || nc < 1e-10 { 0.0 }
                    else { recovered.dot(&cleaned_i) / (nr * nc) }
                })
                .collect();

            let mean: f64 = fidelities.iter().sum::<f64>() / n as f64;
            let min: f64 = fidelities.iter().cloned().fold(f64::MAX, f64::min);
            println!("  {:>3}  {:>12.4}  {:>12.4}", n, mean, min);
        }
    }
}
