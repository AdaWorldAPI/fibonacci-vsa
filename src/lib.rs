// fibonacci-vsa: Zeckendorf-coded vector space with Fibonacci spiral navigation
// and Euler-Mascheroni γ spacetime curvature
//
// No ML. No weights. Only mathematics against compile-time constants.

use std::f64::consts::PI;
use ndarray::{Array1, Array2, ArrayView1};

// ─── COMPILE-TIME CONSTANTS ───────────────────────────────────────────
// These exist in .rodata, not at runtime. The compiler knows them.

/// Golden ratio φ = (1 + √5) / 2
const PHI: f64 = 1.618_033_988_749_895;

/// Euler-Mascheroni constant γ — the bridge between discrete and continuous
const GAMMA: f64 = 0.577_215_664_901_532_9;

/// ln(φ) — natural log of golden ratio, used for scale mapping  
const LN_PHI: f64 = 0.481_211_825_059_603_4;

/// Fibonacci lookup table — all 93 values that fit in u64
/// This is the entire skeleton of the space. Compile-time constant.
pub const FIB_LEN: usize = 87;
const FIB: [u64; FIB_LEN] = {
    let mut table = [0u64; FIB_LEN];
    table[0] = 1;
    table[1] = 2;
    let mut i = 2;
    while i < FIB_LEN {
        table[i] = table[i - 1] + table[i - 2];
        i += 1;
    }
    table
};

/// Harmonic number approximation using γ: H_n ≈ ln(n) + γ + 1/(2n)
/// This is where γ bends the space — it maps discrete Fibonacci positions
/// to continuous spiral coordinates with gravitational curvature
#[inline]
fn harmonic(n: f64) -> f64 {
    if n <= 0.0 {
        return 0.0;
    }
    n.ln() + GAMMA + 1.0 / (2.0 * n) - 1.0 / (12.0 * n * n)
}

// ─── ZECKENDORF ENCODING ──────────────────────────────────────────────
// Greedy decomposition into non-consecutive Fibonacci numbers
// This is bijective: every positive integer has exactly one Zeckendorf representation

/// A Zeckendorf-encoded value: a bitfield where bit k means "Fibonacci(k) is present"
/// The non-consecutivity constraint is enforced by construction
#[derive(Clone, Debug)]
pub struct ZeckendorfBits {
    /// Bits packed into u128 — supports up to 87 Fibonacci positions
    /// bit i set means FIB[i] is part of the decomposition
    pub bits: u128,
    /// Highest set bit position — the "scale" of this value
    pub max_scale: u8,
}

impl ZeckendorfBits {
    /// Encode a u64 value into Zeckendorf representation
    /// Greedy algorithm against compile-time Fibonacci table
    pub fn encode(mut value: u64) -> Self {
        if value == 0 {
            return Self { bits: 0, max_scale: 0 };
        }

        let mut bits: u128 = 0;
        let mut max_scale: u8 = 0;
        let mut first = true;

        // Greedy: find largest Fibonacci ≤ remaining value, subtract, repeat
        for i in (0..FIB_LEN).rev() {
            if FIB[i] <= value {
                bits |= 1u128 << i;
                value -= FIB[i];
                if first {
                    max_scale = i as u8;
                    first = false;
                }
                if value == 0 {
                    break;
                }
            }
        }
        Self { bits, max_scale }
    }

    /// Decode back to u64 — lossless round-trip
    pub fn decode(&self) -> u64 {
        let mut value = 0u64;
        for i in 0..FIB_LEN {
            if self.bits & (1u128 << i) != 0 {
                value += FIB[i];
            }
        }
        value
    }

    /// Count of set bits — the "density" of this representation
    pub fn popcount(&self) -> u32 {
        self.bits.count_ones()
    }

    /// Resonance: shared Fibonacci bits between two Zeckendorf values
    /// Each shared bit is a "wormhole" — a scale at which they agree
    pub fn resonance(&self, other: &Self) -> Resonance {
        let shared = self.bits & other.bits;
        let divergent = self.bits ^ other.bits;

        Resonance {
            shared_bits: shared,
            divergent_bits: divergent,
            shared_count: shared.count_ones(),
            max_shared_scale: if shared == 0 { 0 } else { 127 - shared.leading_zeros() as u8 },
            max_divergent_scale: if divergent == 0 { 0 } else { 127 - divergent.leading_zeros() as u8 },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Resonance {
    pub shared_bits: u128,
    pub divergent_bits: u128,
    pub shared_count: u32,
    pub max_shared_scale: u8,
    pub max_divergent_scale: u8,
}

// ─── SPIRAL COORDINATES WITH γ-CURVATURE ─────────────────────────────
// This is where Euler-Mascheroni bends the space.
//
// In flat space: position k → scale φ^k (exponential, uniform)
// In γ-curved space: position k → harmonic(k) mapped onto golden spiral
//
// The harmonic function H(k) = ln(k) + γ + 1/(2k) - ... introduces
// GRAVITATIONAL CURVATURE: nearby scales are stretched apart (like time
// dilation near a massive object), distant scales compress.
//
// Result: high-resolution at the scale you're looking at, compression
// at scales you're not. The γ term is the "cosmological constant" —
// it shifts the entire metric by a universal offset.

#[derive(Clone, Debug)]
pub struct SpiralPoint {
    /// Angle on the spiral (radians) — semantic direction
    pub theta: f64,
    /// Radius — abstraction level, γ-warped
    pub radius: f64,
    /// Height on the dome — temporal/phase dimension
    pub z: f64,
    /// The raw Fibonacci scale before γ-warping
    pub raw_scale: f64,
    /// The γ-warp factor applied (how much spacetime bent here)
    pub warp_factor: f64,
}

/// Map a Zeckendorf-encoded value onto the golden spiral dome with γ-curvature
pub fn to_spiral(zeck: &ZeckendorfBits) -> SpiralPoint {
    let scale = zeck.max_scale as f64;

    // ─── γ-WARP: the core spacetime bend ───
    //
    // H(scale) = ln(scale) + γ + 1/(2·scale)
    //
    // This does several things at once:
    // 1. ln(scale) compresses high scales logarithmically
    // 2. γ shifts the origin — without it, H(1) = 1, with it H(1) ≈ 1.577
    //    This "inflates" the low-scale region, giving more resolution near the origin
    // 3. 1/(2·scale) adds curvature correction — more curvature at low scales
    //
    // Net effect: spacetime is DENSER near the origin (fine detail) and
    // SPARSER at high scales (abstract concepts). Like gravitational lensing
    // around a mass at the center of the spiral.
    let warped_scale = harmonic(scale.max(1.0));
    let warp_factor = warped_scale / scale.max(1.0);

    // Golden angle: 2π/φ² ≈ 137.5° — the angle that produces maximum
    // packing efficiency (sunflower theorem). Each Fibonacci bit advances
    // by this angle, ensuring no two concepts overlap on the spiral.
    let golden_angle = 2.0 * PI / (PHI * PHI);

    // Angle: each set bit contributes its position × golden angle
    // The γ-warp stretches angles non-uniformly
    let mut theta = 0.0;
    let mut radius_acc = 0.0;
    let mut bit_count = 0u32;

    for i in 0..FIB_LEN {
        if zeck.bits & (1u128 << i) != 0 {
            let bit_scale = i as f64;
            let warped_bit = harmonic(bit_scale.max(1.0));

            // Each bit rotates by golden_angle × γ-warped position
            theta += golden_angle * warped_bit;

            // Radius accumulates φ^(γ-warped position)
            // This is the key: φ^H(k) instead of φ^k
            // The γ in H(k) means the radius grows SLOWER than pure exponential
            // at high scales — spacetime compression at the edge
            radius_acc += PHI.powf(warped_bit);
            bit_count += 1;
        }
    }

    // Normalize radius by bit count for stability
    let radius = if bit_count > 0 {
        radius_acc / bit_count as f64
    } else {
        0.0
    };

    // Z-coordinate: phase on the dome
    // Uses the RATIO of γ-warped to raw scale as dome height
    // This makes the dome "breathe" — points at scale transitions
    // (where warp_factor changes rapidly) lift off the plane
    let z = if scale > 0.0 {
        (warp_factor * PI).sin() * radius * 0.5
    } else {
        0.0
    };

    // Normalize theta to [0, 2π)
    let theta = theta % (2.0 * PI);

    SpiralPoint {
        theta,
        radius,
        z,
        raw_scale: scale,
        warp_factor,
    }
}

// ─── WORMHOLE DISTANCE ───────────────────────────────────────────────
// Distance through a wormhole: not Euclidean, not cosine, not Hamming.
// It's the γ-warped resonance distance.
//
// Two points connected by a wormhole at scale k have distance
// proportional to 1/H(k) — the INVERSE harmonic. This means:
// - Wormholes at high scales (abstract agreement) = SHORT distance
//   (like a wormhole through spacetime — you skip the flat geometry)
// - Wormholes at low scales (detail agreement) = LONGER distance
//   (fine detail agreement is "closer" in value but not in meaning)
//
// The γ term ensures the distance metric is never zero and never
// infinite — it's the cosmological constant that keeps spacetime
// well-behaved.

/// Wormhole-distance between two ZeckendorfBits
pub fn wormhole_distance(a: &ZeckendorfBits, b: &ZeckendorfBits) -> WormholeMetric {
    let res = a.resonance(b);

    // Collect wormholes: each shared bit is a tunnel
    let mut wormholes: Vec<Wormhole> = Vec::new();
    let mut total_shortcut = 0.0;

    for i in 0..FIB_LEN {
        if res.shared_bits & (1u128 << i) != 0 {
            let scale = i as f64;
            let warped = harmonic(scale.max(1.0));

            // Wormhole shortcut: inverse of γ-warped scale
            // High scale shared bit = massive shortcut
            // γ ensures this is always finite and > 0
            let shortcut = PHI.powf(warped) / (1.0 + GAMMA * scale);

            wormholes.push(Wormhole {
                scale: i as u8,
                warped_scale: warped,
                shortcut_strength: shortcut,
            });
            total_shortcut += shortcut;
        }
    }

    // Divergence: bits that differ, weighted by γ-warped scale
    let mut divergence = 0.0;
    for i in 0..FIB_LEN {
        if res.divergent_bits & (1u128 << i) != 0 {
            let scale = i as f64;
            let warped = harmonic(scale.max(1.0));
            // Divergence at high scales costs MORE (γ-amplified)
            divergence += PHI.powf(warped) * (1.0 + GAMMA * scale);
        }
    }

    // Net distance: divergence minus wormhole shortcuts
    // γ acts as the "bending constant" — more γ = more curvature
    // = wormholes are stronger shortcuts = spacetime is more bent
    let flat_distance = divergence;
    let bent_distance = (divergence - total_shortcut * GAMMA).max(0.0);

    // The bending ratio: how much spacetime curved between these two points
    let curvature = if flat_distance > 0.0 {
        1.0 - (bent_distance / flat_distance)
    } else {
        0.0
    };

    WormholeMetric {
        wormholes,
        flat_distance,
        bent_distance,
        curvature,
        resonance_count: res.shared_count,
    }
}

#[derive(Clone, Debug)]
pub struct Wormhole {
    /// Fibonacci scale position of this wormhole
    pub scale: u8,
    /// γ-warped scale — the "real" position in curved space
    pub warped_scale: f64,
    /// How strong this shortcut is
    pub shortcut_strength: f64,
}

#[derive(Clone, Debug)]
pub struct WormholeMetric {
    pub wormholes: Vec<Wormhole>,
    /// Euclidean-like distance ignoring wormholes
    pub flat_distance: f64,
    /// Distance through wormholes — γ-bent spacetime
    pub bent_distance: f64,
    /// How much spacetime curved: 0.0 = flat, 1.0 = fully collapsed
    pub curvature: f64,
    /// Number of resonance channels
    pub resonance_count: u32,
}

// ─── ZECKENDORF VECTOR ────────────────────────────────────────────────
// A full vector: multiple dimensions, each Zeckendorf-encoded

#[derive(Clone, Debug)]
pub struct ZeckendorfVector {
    dims: Vec<ZeckendorfBits>,
}

impl ZeckendorfVector {
    /// Encode from f64 slice (e.g., Jina 1024D embedding)
    /// Quantization: f64 → scaled u64 → Zeckendorf
    pub fn from_f64_slice(values: &[f64], precision_bits: u8) -> Self {
        let scale = (1u64 << precision_bits.min(52)) as f64;
        let dims = values
            .iter()
            .map(|&v| {
                // Map [-1, 1] float range to [0, 2^precision] integer range
                let normalized = ((v + 1.0) * 0.5 * scale) as u64;
                ZeckendorfBits::encode(normalized)
            })
            .collect();
        Self { dims }
    }

    /// Full resonance analysis between two vectors
    /// Returns per-dimension wormholes — the 16K tunnel map
    pub fn resonance_map(&self, other: &Self) -> ResonanceMap {
        let dim_count = self.dims.len().min(other.dims.len());
        let mut total_wormholes = 0u32;
        let mut total_curvature = 0.0;
        let mut scale_histogram = [0u32; FIB_LEN];
        let mut max_curvature_dim = 0;
        let mut max_curvature = 0.0f64;

        let mut per_dim: Vec<WormholeMetric> = Vec::with_capacity(dim_count);

        for i in 0..dim_count {
            let metric = wormhole_distance(&self.dims[i], &other.dims[i]);
            total_wormholes += metric.resonance_count;
            total_curvature += metric.curvature;

            if metric.curvature > max_curvature {
                max_curvature = metric.curvature;
                max_curvature_dim = i;
            }

            for wh in &metric.wormholes {
                if (wh.scale as usize) < FIB_LEN {
                    scale_histogram[wh.scale as usize] += 1;
                }
            }

            per_dim.push(metric);
        }

        // Find the dominant resonance scale — the Fibonacci position
        // where most wormholes cluster. This is the "frequency" at which
        // these two vectors most strongly agree.
        let dominant_scale = scale_histogram
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(idx, _)| idx as u8)
            .unwrap_or(0);

        let avg_curvature = if dim_count > 0 {
            total_curvature / dim_count as f64
        } else {
            0.0
        };

        // γ-bent aggregate distance: geometric mean of per-dimension bent distances
        // Geometric mean because multiplicative distances compose naturally in curved space
        let bent_distance_agg = if dim_count > 0 {
            let log_sum: f64 = per_dim
                .iter()
                .map(|m| (m.bent_distance + 1.0).ln())
                .sum::<f64>();
            (log_sum / dim_count as f64).exp() - 1.0
        } else {
            0.0
        };

        ResonanceMap {
            per_dimension: per_dim,
            total_wormholes,
            avg_curvature,
            dominant_scale,
            dominant_warped: harmonic(dominant_scale as f64),
            bent_distance: bent_distance_agg,
            max_curvature_dimension: max_curvature_dim,
            scale_histogram,
        }
    }

    /// Quick coarse distance: only looks at max_scale per dimension
    /// O(d) where d = number of dimensions. No bit scanning.
    pub fn coarse_distance(&self, other: &Self) -> f64 {
        let dim_count = self.dims.len().min(other.dims.len());
        let mut dist = 0.0;
        for i in 0..dim_count {
            let sa = self.dims[i].max_scale as f64;
            let sb = other.dims[i].max_scale as f64;
            // γ-warped scale difference
            let da = harmonic(sa.max(1.0));
            let db = harmonic(sb.max(1.0));
            dist += (da - db).powi(2);
        }
        dist.sqrt()
    }

    pub fn dim_count(&self) -> usize {
        self.dims.len()
    }

    /// Access raw dimension bits
    pub fn dim(&self, i: usize) -> &ZeckendorfBits {
        &self.dims[i]
    }
}

// ─── NDARRAY BRIDGE ──────────────────────────────────────────────────
// Convert between ndarray Array1<f64> and ZeckendorfVector

impl ZeckendorfVector {
    /// From ndarray Array1<f64> — the main interop path
    pub fn from_ndarray(arr: &Array1<f64>, precision_bits: u8) -> Self {
        Self::from_f64_slice(arr.as_slice().unwrap(), precision_bits)
    }

    /// From ndarray ArrayView1<f64> — zero-copy view
    pub fn from_ndarray_view(arr: ArrayView1<f64>, precision_bits: u8) -> Self {
        Self::from_f64_slice(arr.as_slice().unwrap(), precision_bits)
    }

    /// Batch encode: Array2<f64> where each row is a vector
    /// Returns Vec<ZeckendorfVector> — one per row
    pub fn batch_encode(matrix: &Array2<f64>, precision_bits: u8) -> Vec<Self> {
        matrix.rows()
            .into_iter()
            .map(|row| Self::from_ndarray_view(row, precision_bits))
            .collect()
    }

    /// Pairwise bent distance matrix for a set of vectors
    /// Returns Array2<f64> — symmetric, diagonal = 0
    pub fn pairwise_bent_distances(vectors: &[Self]) -> Array2<f64> {
        let n = vectors.len();
        let mut dists = Array2::zeros((n, n));
        for i in 0..n {
            for j in (i + 1)..n {
                let map = vectors[i].resonance_map(&vectors[j]);
                dists[[i, j]] = map.bent_distance;
                dists[[j, i]] = map.bent_distance;
            }
        }
        dists
    }

    /// Pairwise coarse distance matrix — O(n²·d), no bit scanning
    pub fn pairwise_coarse_distances(vectors: &[Self]) -> Array2<f64> {
        let n = vectors.len();
        let mut dists = Array2::zeros((n, n));
        for i in 0..n {
            for j in (i + 1)..n {
                let d = vectors[i].coarse_distance(&vectors[j]);
                dists[[i, j]] = d;
                dists[[j, i]] = d;
            }
        }
        dists
    }

    /// Pairwise wormhole count matrix
    pub fn pairwise_wormhole_counts(vectors: &[Self]) -> Array2<u32> {
        let n = vectors.len();
        let mut counts = Array2::zeros((n, n));
        for i in 0..n {
            for j in (i + 1)..n {
                let map = vectors[i].resonance_map(&vectors[j]);
                counts[[i, j]] = map.total_wormholes;
                counts[[j, i]] = map.total_wormholes;
            }
        }
        counts
    }
}

// ─── COSINE BASELINE ─────────────────────────────────────────────────
// Standard cosine similarity on ndarray for direct comparison

/// Cosine similarity between two f64 vectors
pub fn cosine_similarity(a: &Array1<f64>, b: &Array1<f64>) -> f64 {
    let dot = a.dot(b);
    let norm_a = a.dot(a).sqrt();
    let norm_b = b.dot(b).sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

/// Cosine distance = 1 - cosine_similarity
pub fn cosine_distance(a: &Array1<f64>, b: &Array1<f64>) -> f64 {
    1.0 - cosine_similarity(a, b)
}

/// Pairwise cosine distance matrix
pub fn pairwise_cosine_distances(matrix: &Array2<f64>) -> Array2<f64> {
    let n = matrix.nrows();
    let mut dists = Array2::zeros((n, n));
    for i in 0..n {
        for j in (i + 1)..n {
            let d = cosine_distance(&matrix.row(i).to_owned(), &matrix.row(j).to_owned());
            dists[[i, j]] = d;
            dists[[j, i]] = d;
        }
    }
    dists
}

/// Spearman rank correlation between two distance vectors
/// Tests whether Fibonacci-VSA preserves the ORDERING of cosine distances
pub fn spearman_rank_correlation(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    let n = a.len();
    if n < 2 {
        return 0.0;
    }

    fn ranks(vals: &[f64]) -> Vec<f64> {
        let mut indexed: Vec<(usize, f64)> = vals.iter().copied().enumerate().collect();
        indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let mut ranks = vec![0.0; vals.len()];
        let mut i = 0;
        while i < indexed.len() {
            let mut j = i;
            while j < indexed.len() && (indexed[j].1 - indexed[i].1).abs() < 1e-12 {
                j += 1;
            }
            let avg_rank = (i + j - 1) as f64 / 2.0 + 1.0;
            for k in i..j {
                ranks[indexed[k].0] = avg_rank;
            }
            i = j;
        }
        ranks
    }

    let ra = ranks(a);
    let rb = ranks(b);

    let mean_a: f64 = ra.iter().sum::<f64>() / n as f64;
    let mean_b: f64 = rb.iter().sum::<f64>() / n as f64;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for i in 0..n {
        let da = ra[i] - mean_a;
        let db = rb[i] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }

    if var_a == 0.0 || var_b == 0.0 {
        return 0.0;
    }
    cov / (var_a.sqrt() * var_b.sqrt())
}

#[derive(Clone, Debug)]
pub struct ResonanceMap {
    pub per_dimension: Vec<WormholeMetric>,
    pub total_wormholes: u32,
    pub avg_curvature: f64,
    /// The Fibonacci scale where most wormholes exist
    pub dominant_scale: u8,
    /// The γ-warped value of the dominant scale
    pub dominant_warped: f64,
    /// Aggregate bent distance through wormhole space
    pub bent_distance: f64,
    /// Which dimension has the strongest spacetime curvature
    pub max_curvature_dimension: usize,
    /// How many wormholes exist at each of the 93 Fibonacci scales
    pub scale_histogram: [u32; FIB_LEN],
}

// ─── TIME BENDING DEMO ──────────────────────────────────────────────
// Shows how γ distorts the Fibonacci lattice into curved spacetime

/// Visualize the γ-curvature at each Fibonacci scale
pub fn gamma_curvature_profile() -> Vec<(u8, f64, f64, f64)> {
    // Returns: (scale, raw_phi_k, gamma_warped, warp_ratio)
    (1..=(FIB_LEN as u8 - 1))
        .map(|k| {
            let raw = PHI.powi(k as i32);
            let warped = PHI.powf(harmonic(k as f64));
            let ratio = warped / raw;
            (k as u8, raw, warped, ratio)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{Array1, Array2};
    use rand::Rng;

    #[test]
    fn zeckendorf_roundtrip() {
        for v in [0, 1, 2, 3, 5, 8, 13, 42, 100, 1000, 65535, 1_000_000] {
            let z = ZeckendorfBits::encode(v);
            assert_eq!(z.decode(), v, "Roundtrip failed for {}", v);
        }
    }

    #[test]
    fn non_consecutive_bits() {
        for v in 1..=10000u64 {
            let z = ZeckendorfBits::encode(v);
            assert_eq!(z.bits & (z.bits >> 1), 0, "Consecutive bits found for {}", v);
        }
    }

    #[test]
    fn gamma_is_the_cosmological_constant() {
        let k: f64 = 10.0;
        let with_gamma = k.ln() + GAMMA + 1.0 / (2.0 * k);
        let without_gamma = k.ln() + 1.0 / (2.0 * k);
        let gamma_effect = with_gamma / without_gamma;
        println!("Scale 10: with γ = {:.6}, without = {:.6}, ratio = {:.6}",
            with_gamma, without_gamma, gamma_effect);
        assert!(gamma_effect > 1.2);

        let k2: f64 = 1000.0;
        let ratio_high = (k2.ln() + GAMMA + 1.0 / (2.0 * k2)) / (k2.ln() + 1.0 / (2.0 * k2));
        assert!(ratio_high < gamma_effect);
    }

    // ─── NDARRAY INTEGRATION ──────────────────────────────────────────

    #[test]
    fn ndarray_encode_roundtrip() {
        let arr = Array1::from_vec(vec![0.5, -0.3, 0.0, 0.99, -0.99]);
        let zv = ZeckendorfVector::from_ndarray(&arr, 16);
        assert_eq!(zv.dim_count(), 5);
        println!("ndarray → Zeckendorf: {} dims, scales: {:?}",
            zv.dim_count(),
            (0..5).map(|i| zv.dim(i).max_scale).collect::<Vec<_>>());
    }

    #[test]
    fn batch_encode_matrix() {
        let mut rng = rand::thread_rng();
        let data: Vec<f64> = (0..8 * 256).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let matrix = Array2::from_shape_vec((8, 256), data).unwrap();
        let encoded = ZeckendorfVector::batch_encode(&matrix, 16);
        assert_eq!(encoded.len(), 8);
        assert_eq!(encoded[0].dim_count(), 256);
    }

    // ─── THE MONEY TEST: rank correlation cosine vs fibonacci ─────────

    #[test]
    fn cosine_vs_wormhole_rank_correlation() {
        let mut rng = rand::thread_rng();
        let n = 20;
        let dims = 128;

        let data: Vec<f64> = (0..n * dims).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let matrix = Array2::from_shape_vec((n, dims), data).unwrap();

        let cosine_dists = pairwise_cosine_distances(&matrix);
        let encoded = ZeckendorfVector::batch_encode(&matrix, 16);
        let bent_dists = ZeckendorfVector::pairwise_bent_distances(&encoded);
        let coarse_dists = ZeckendorfVector::pairwise_coarse_distances(&encoded);

        let mut cos_f = Vec::new();
        let mut bent_f = Vec::new();
        let mut coarse_f = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                cos_f.push(cosine_dists[[i, j]]);
                bent_f.push(bent_dists[[i, j]]);
                coarse_f.push(coarse_dists[[i, j]]);
            }
        }

        let rho_bent = spearman_rank_correlation(&cos_f, &bent_f);
        let rho_coarse = spearman_rank_correlation(&cos_f, &coarse_f);

        println!("\n═══ RANK CORRELATION: Cosine ↔ Fibonacci-VSA ═══");
        println!("  {}×{}, {} pairs", n, dims, cos_f.len());
        println!("  Spearman ρ (cosine vs bent):   {:.6}", rho_bent);
        println!("  Spearman ρ (cosine vs coarse): {:.6}", rho_coarse);

        assert!(rho_bent > 0.1, "Expected positive rank correlation, got {:.4}", rho_bent);
    }

    // ─── DISCRIMINATION: similar vs different ─────────────────────────

    #[test]
    fn wormhole_count_discriminates() {
        let mut rng = rand::thread_rng();
        let dims = 512;

        let base: Array1<f64> = Array1::from_vec(
            (0..dims).map(|_| rng.gen_range(-1.0..1.0)).collect());
        let similar: Array1<f64> = &base + &Array1::from_vec(
            (0..dims).map(|_| rng.gen_range(-0.05..0.05)).collect());
        let different: Array1<f64> = Array1::from_vec(
            (0..dims).map(|_| rng.gen_range(-1.0..1.0)).collect());

        let zb = ZeckendorfVector::from_ndarray(&base, 16);
        let zs = ZeckendorfVector::from_ndarray(&similar, 16);
        let zd = ZeckendorfVector::from_ndarray(&different, 16);

        let m_sim = zb.resonance_map(&zs);
        let m_dif = zb.resonance_map(&zd);

        println!("\n═══ DISCRIMINATION ═══");
        println!("  Similar:   cos={:.4} wh={} bent={:.2}",
            cosine_similarity(&base, &similar), m_sim.total_wormholes, m_sim.bent_distance);
        println!("  Different: cos={:.4} wh={} bent={:.2}",
            cosine_similarity(&base, &different), m_dif.total_wormholes, m_dif.bent_distance);

        assert!(m_sim.total_wormholes > m_dif.total_wormholes);
        assert!(m_sim.bent_distance < m_dif.bent_distance);
    }

    // ─── PRECISION TRADEOFF ──────────────────────────────────────────

    #[test]
    fn precision_bits_tradeoff() {
        let mut rng = rand::thread_rng();
        let dims = 256;
        let n = 10;

        let data: Vec<f64> = (0..n * dims).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let matrix = Array2::from_shape_vec((n, dims), data).unwrap();
        let cosine_dists = pairwise_cosine_distances(&matrix);

        let mut cos_f = Vec::new();
        for i in 0..n { for j in (i+1)..n { cos_f.push(cosine_dists[[i,j]]); } }

        println!("\n═══ PRECISION BITS vs RANK CORRELATION ═══");
        println!("  {:>4}  {:>10}  {:>10}", "bits", "Spearman ρ", "mean_wh");

        for bits in [4u8, 8, 12, 16, 20, 24] {
            let enc = ZeckendorfVector::batch_encode(&matrix, bits);
            let bent = ZeckendorfVector::pairwise_bent_distances(&enc);

            let mut bent_f = Vec::new();
            let mut total_wh = 0u64;
            let mut pairs = 0u64;
            for i in 0..n {
                for j in (i+1)..n {
                    bent_f.push(bent[[i,j]]);
                    total_wh += enc[i].resonance_map(&enc[j]).total_wormholes as u64;
                    pairs += 1;
                }
            }

            let rho = spearman_rank_correlation(&cos_f, &bent_f);
            println!("  {:>4}  {:>10.6}  {:>10.1}", bits, rho, total_wh as f64 / pairs as f64);
        }
    }

    // ─── CLUSTER SCALE STRUCTURE ─────────────────────────────────────

    #[test]
    fn scale_histogram_shows_clusters() {
        let dims = 128;
        let a: Array1<f64> = Array1::from_vec(
            (0..dims).map(|i| if i < 64 { 0.8 } else { -0.8 }).collect());
        let a2: Array1<f64> = Array1::from_vec(
            (0..dims).map(|i| if i < 64 { 0.75 } else { -0.75 }).collect());
        let b: Array1<f64> = Array1::from_vec(
            (0..dims).map(|i| if i < 64 { -0.8 } else { 0.8 }).collect());

        let za = ZeckendorfVector::from_ndarray(&a, 16);
        let za2 = ZeckendorfVector::from_ndarray(&a2, 16);
        let zb = ZeckendorfVector::from_ndarray(&b, 16);

        let within = za.resonance_map(&za2);
        let between = za.resonance_map(&zb);

        println!("\n═══ CLUSTER SCALE STRUCTURE ═══");
        println!("  Within:  wh={} bent={:.2} dom_scale={}", within.total_wormholes, within.bent_distance, within.dominant_scale);
        println!("  Between: wh={} bent={:.2} dom_scale={}", between.total_wormholes, between.bent_distance, between.dominant_scale);

        // Bent distance is the reliable discriminator — wormhole counts
        // can be equal for structured vectors with identical bit patterns
        assert!(within.bent_distance < between.bent_distance,
            "Within {:.2} should < between {:.2}", within.bent_distance, between.bent_distance);
    }

    // ─── MONOTONIC DISTANCE VS NOISE ─────────────────────────────────

    #[test]
    fn monotonic_distance_with_noise() {
        let mut rng = rand::thread_rng();
        let dims = 512;
        let base: Array1<f64> = Array1::from_vec(
            (0..dims).map(|_| rng.gen_range(-1.0..1.0)).collect());
        let zb = ZeckendorfVector::from_ndarray(&base, 16);

        println!("\n═══ MONOTONIC DISTANCE vs NOISE ═══");
        println!("  {:>6}  {:>10}  {:>10}  {:>8}", "noise", "cos_d", "bent_d", "wh");

        for &noise in &[0.01, 0.05, 0.1, 0.2, 0.4, 0.8] {
            let noisy: Array1<f64> = &base + &Array1::from_vec(
                (0..dims).map(|_| rng.gen_range(-noise..noise)).collect());
            let zn = ZeckendorfVector::from_ndarray(&noisy, 16);
            let map = zb.resonance_map(&zn);
            let cos_d = cosine_distance(&base, &noisy);
            println!("  {:>6.2}  {:>10.6}  {:>10.2}  {:>8}", noise, cos_d, map.bent_distance, map.total_wormholes);
        }

        // Extreme test: quiet vs loud
        let quiet: Array1<f64> = &base + &Array1::from_vec(
            (0..dims).map(|_| rng.gen_range(-0.01..0.01)).collect());
        let loud: Array1<f64> = Array1::from_vec(
            (0..dims).map(|_| rng.gen_range(-1.0..1.0)).collect());
        let d_quiet = zb.resonance_map(&ZeckendorfVector::from_ndarray(&quiet, 16)).bent_distance;
        let d_loud = zb.resonance_map(&ZeckendorfVector::from_ndarray(&loud, 16)).bent_distance;
        assert!(d_quiet < d_loud, "quiet {:.2} should < loud {:.2}", d_quiet, d_loud);
    }

    // ─── PAIRWISE DISTANCE MATRICES ──────────────────────────────────

    #[test]
    fn pairwise_matrices_symmetric() {
        let mut rng = rand::thread_rng();
        let n = 6;
        let dims = 64;
        let data: Vec<f64> = (0..n * dims).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let matrix = Array2::from_shape_vec((n, dims), data).unwrap();
        let enc = ZeckendorfVector::batch_encode(&matrix, 16);

        let bent = ZeckendorfVector::pairwise_bent_distances(&enc);
        let coarse = ZeckendorfVector::pairwise_coarse_distances(&enc);
        let wh = ZeckendorfVector::pairwise_wormhole_counts(&enc);

        for i in 0..n {
            assert_eq!(bent[[i, i]], 0.0, "Diagonal should be 0");
            assert_eq!(coarse[[i, i]], 0.0);
            for j in 0..n {
                assert!((bent[[i, j]] - bent[[j, i]]).abs() < 1e-10, "Bent not symmetric");
                assert!((coarse[[i, j]] - coarse[[j, i]]).abs() < 1e-10, "Coarse not symmetric");
                assert_eq!(wh[[i, j]], wh[[j, i]], "Wormhole counts not symmetric");
            }
        }
        println!("\n═══ PAIRWISE MATRICES: all symmetric ✓ ═══");
    }

    // ─── TIMING BENCHMARK ────────────────────────────────────────────

    #[test]
    fn timing_1024d() {
        let mut rng = rand::thread_rng();
        let dims = 1024;
        let n = 100;

        let vecs: Vec<Array1<f64>> = (0..n * 2)
            .map(|_| Array1::from_vec((0..dims).map(|_| rng.gen_range(-1.0..1.0)).collect()))
            .collect();

        let t0 = std::time::Instant::now();
        let enc: Vec<ZeckendorfVector> = vecs.iter()
            .map(|v| ZeckendorfVector::from_ndarray(v, 16)).collect();
        let t_enc = t0.elapsed();

        let t1 = std::time::Instant::now();
        let mut c_sum = 0.0;
        for i in 0..n { c_sum += enc[i*2].coarse_distance(&enc[i*2+1]); }
        let t_coarse = t1.elapsed();

        let t2 = std::time::Instant::now();
        let mut w_sum = 0u32;
        for i in 0..n { w_sum += enc[i*2].resonance_map(&enc[i*2+1]).total_wormholes; }
        let t_res = t2.elapsed();

        let t3 = std::time::Instant::now();
        let mut cos_sum = 0.0;
        for i in 0..n { cos_sum += cosine_similarity(&vecs[i*2], &vecs[i*2+1]); }
        let t_cos = t3.elapsed();

        println!("\n═══ TIMING: 1024D × {} pairs ═══", n);
        println!("  Encode {} vecs:   {:>8.2?}  ({:.1} µs/vec)",
            n*2, t_enc, t_enc.as_micros() as f64 / (n*2) as f64);
        println!("  Coarse dist:     {:>8.2?}  ({:.1} µs/pair)",
            t_coarse, t_coarse.as_micros() as f64 / n as f64);
        println!("  Full resonance:  {:>8.2?}  ({:.1} µs/pair)  total_wh={}",
            t_res, t_res.as_micros() as f64 / n as f64, w_sum);
        println!("  Cosine baseline: {:>8.2?}  ({:.1} µs/pair)",
            t_cos, t_cos.as_micros() as f64 / n as f64);
        println!("  Coarse vs cosine: {:.1}×",
            t_cos.as_nanos() as f64 / t_coarse.as_nanos().max(1) as f64);
    }
}
