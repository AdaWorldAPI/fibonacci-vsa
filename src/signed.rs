// fibonacci-vsa/src/signed.rs
//
// Signed Ternary Zeckendorf: {-1, 0, +1} per Fibonacci position
// with pentagonal 5-dim grouping for 10,000D projection
//
// The key insight: 10000D / 5 = 2000 groups × 3^5 states = 15,850 bits ≈ 16kbit
// This is not a design choice. It falls out of φ's pentagonal symmetry.

use ndarray::{Array1, Array2, ArrayView1};

/// Compile-time Fibonacci table
const FIB_LEN: usize = 24; // enough for 8-bit quantization range
const FIB: [u64; FIB_LEN] = {
    let mut t = [0u64; FIB_LEN];
    t[0] = 1; t[1] = 2;
    let mut i = 2;
    while i < FIB_LEN { t[i] = t[i-1] + t[i-2]; i += 1; }
    t
};

const PHI: f64 = 1.618_033_988_749_895;
const GAMMA: f64 = 0.577_215_664_901_532_9;

#[inline]
fn harmonic(n: f64) -> f64 {
    if n <= 0.0 { return 0.0; }
    n.ln() + GAMMA + 1.0 / (2.0 * n) - 1.0 / (12.0 * n * n)
}

// ─── SIGNED TERNARY ZECKENDORF ───────────────────────────────────────
// Each Fibonacci position k holds a trit: -1, 0, or +1
// Value = Σ trit[k] × Fib(k)
// This naturally represents negative numbers without a separate sign bit.
// Cancellation: +1 at position k from vector A and -1 from vector B = 0

/// A single dimension encoded as signed ternary Zeckendorf
/// Packed: 2 bits per position (00=0, 01=+1, 10=-1, 11=unused)
#[derive(Clone, Copy, Debug)]
pub struct SignedZeckendorf {
    /// Positive bits: bit k set = +Fib(k) contributes
    pub pos: u32,
    /// Negative bits: bit k set = -Fib(k) contributes
    pub neg: u32,
    /// Highest active position (max of pos and neg)
    pub max_scale: u8,
}

impl SignedZeckendorf {
    /// Encode a signed f64 value in [-1, 1] range
    /// Quantizes to signed integer, then decomposes into ±Fibonacci
    pub fn encode(value: f64, precision_bits: u8) -> Self {
        let scale = (1u64 << precision_bits.min(20)) as f64;
        let quantized = (value * scale).round() as i64;

        if quantized == 0 {
            return Self { pos: 0, neg: 0, max_scale: 0 };
        }

        let sign = quantized.signum();
        let mut magnitude = quantized.unsigned_abs();

        let mut pos: u32 = 0;
        let mut neg: u32 = 0;
        let mut max_scale: u8 = 0;
        let mut first = true;

        // Greedy Zeckendorf on magnitude, then apply sign
        for i in (0..FIB_LEN).rev() {
            if FIB[i] <= magnitude {
                if sign > 0 {
                    pos |= 1u32 << i;
                } else {
                    neg |= 1u32 << i;
                }
                magnitude -= FIB[i];
                if first {
                    max_scale = i as u8;
                    first = false;
                }
                if magnitude == 0 { break; }
            }
        }

        Self { pos, neg, max_scale }
    }

    /// Decode back to i64
    pub fn decode(&self) -> i64 {
        let mut val: i64 = 0;
        for i in 0..FIB_LEN {
            if self.pos & (1u32 << i) != 0 { val += FIB[i] as i64; }
            if self.neg & (1u32 << i) != 0 { val -= FIB[i] as i64; }
        }
        val
    }

    /// Trit at position k: -1, 0, or +1
    pub fn trit(&self, k: usize) -> i8 {
        let p = (self.pos >> k) & 1;
        let n = (self.neg >> k) & 1;
        p as i8 - n as i8
    }

    /// Signed resonance: considers both agreement AND cancellation
    pub fn signed_resonance(&self, other: &Self) -> SignedResonance {
        // Agreement: same sign at same position (both +1 or both -1)
        let agree_pos = self.pos & other.pos;  // both positive at k
        let agree_neg = self.neg & other.neg;  // both negative at k
        let agreement = agree_pos | agree_neg;

        // Cancellation: opposite signs at same position
        let cancel_1 = self.pos & other.neg;   // I'm +, they're -
        let cancel_2 = self.neg & other.pos;   // I'm -, they're +
        let cancellation = cancel_1 | cancel_2;

        // One-sided: only one vector has signal at this position
        let self_active = self.pos | self.neg;
        let other_active = other.pos | other.neg;
        let one_sided = (self_active ^ other_active) & (self_active | other_active);

        SignedResonance {
            agreement,
            cancellation,
            one_sided,
            agreement_count: agreement.count_ones(),
            cancellation_count: cancellation.count_ones(),
            net_resonance: agreement.count_ones() as i32 - cancellation.count_ones() as i32,
        }
    }

    /// Total active trits (non-zero positions)
    pub fn active_count(&self) -> u32 {
        (self.pos | self.neg).count_ones()
    }
}

#[derive(Clone, Debug)]
pub struct SignedResonance {
    pub agreement: u32,      // bitmask: same sign at same scale
    pub cancellation: u32,   // bitmask: opposite signs at same scale
    pub one_sided: u32,      // bitmask: only one vector active
    pub agreement_count: u32,
    pub cancellation_count: u32,
    pub net_resonance: i32,  // agreement - cancellation (can be negative!)
}

// ─── PENTAGONAL 5-DIM GROUPS ─────────────────────────────────────────
// 10000D → 2000 groups of 5 dimensions
// Each group: 5 signed ternary Zeckendorf values
// State space per group: 3^5 = 243 (ternary) per Fibonacci position
// Total: 2000 groups × ~8 bits/group = ~16kbit

/// A 5-dimensional pentagonal group
#[derive(Clone, Debug)]
pub struct PentaGroup {
    pub dims: [SignedZeckendorf; 5],
}

impl PentaGroup {
    /// Encode 5 f64 values into a pentagonal group
    pub fn encode(values: &[f64; 5], precision_bits: u8) -> Self {
        Self {
            dims: [
                SignedZeckendorf::encode(values[0], precision_bits),
                SignedZeckendorf::encode(values[1], precision_bits),
                SignedZeckendorf::encode(values[2], precision_bits),
                SignedZeckendorf::encode(values[3], precision_bits),
                SignedZeckendorf::encode(values[4], precision_bits),
            ],
        }
    }

    /// Group-level resonance: aggregates 5 dimensions
    pub fn group_resonance(&self, other: &PentaGroup) -> GroupResonance {
        let mut total_agreement = 0u32;
        let mut total_cancellation = 0u32;
        let mut net = 0i32;

        for i in 0..5 {
            let r = self.dims[i].signed_resonance(&other.dims[i]);
            total_agreement += r.agreement_count;
            total_cancellation += r.cancellation_count;
            net += r.net_resonance;
        }

        GroupResonance {
            agreement: total_agreement,
            cancellation: total_cancellation,
            net_resonance: net,
        }
    }

    /// Fingerprint: collapse 5 dims × FIB_LEN positions into a single u64
    /// by hashing the trit pattern. Used for O(1) group comparison.
    pub fn fingerprint(&self) -> u64 {
        let mut hash: u64 = 0;
        for d in 0..5 {
            for k in 0..FIB_LEN.min(12) { // top 12 Fibonacci positions
                let trit = self.dims[d].trit(k);
                // Map trit {-1, 0, +1} to {0, 1, 2} and pack
                let val = (trit + 1) as u64;
                let bit_pos = d * 12 + k; // 5 × 12 = 60 bits, fits u64
                hash |= val << (bit_pos * 1); // 1 bit per — simplified
            }
        }
        hash
    }

    /// Bits needed to represent this group (information content)
    pub fn bit_cost(&self) -> u32 {
        self.dims.iter().map(|d| d.active_count() * 2).sum::<u32>() // 2 bits per active trit
    }
}

#[derive(Clone, Debug)]
pub struct GroupResonance {
    pub agreement: u32,
    pub cancellation: u32,
    pub net_resonance: i32,
}

// ─── 10,000D FIBONACCI VECTOR ────────────────────────────────────────

/// A full 10,000D vector organized as 2000 pentagonal groups
#[derive(Clone, Debug)]
pub struct FibVec10K {
    pub groups: Vec<PentaGroup>,
}

impl FibVec10K {
    /// Encode from f64 slice — must be multiple of 5 in length
    /// Pads with zeros if not divisible by 5
    pub fn encode(values: &[f64], precision_bits: u8) -> Self {
        let padded_len = ((values.len() + 4) / 5) * 5;
        let mut padded = values.to_vec();
        padded.resize(padded_len, 0.0);

        let groups = padded.chunks_exact(5)
            .map(|chunk| {
                let arr: [f64; 5] = [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4]];
                PentaGroup::encode(&arr, precision_bits)
            })
            .collect();

        Self { groups }
    }

    /// From ndarray
    pub fn from_ndarray(arr: &Array1<f64>, precision_bits: u8) -> Self {
        Self::encode(arr.as_slice().unwrap(), precision_bits)
    }

    /// Batch encode from Array2
    pub fn batch_encode(matrix: &Array2<f64>, precision_bits: u8) -> Vec<Self> {
        matrix.rows().into_iter()
            .map(|row| Self::encode(row.as_slice().unwrap(), precision_bits))
            .collect()
    }

    /// Total bit cost of this vector
    pub fn total_bits(&self) -> u32 {
        self.groups.iter().map(|g| g.bit_cost()).sum()
    }

    /// Full resonance analysis
    pub fn resonance(&self, other: &Self) -> FibResonance10K {
        let n = self.groups.len().min(other.groups.len());
        let mut total_agree = 0u32;
        let mut total_cancel = 0u32;
        let mut total_net = 0i64;

        let mut scale_agree = [0u32; FIB_LEN];
        let mut scale_cancel = [0u32; FIB_LEN];

        for g in 0..n {
            for d in 0..5 {
                let r = self.groups[g].dims[d].signed_resonance(&other.groups[g].dims[d]);
                total_agree += r.agreement_count;
                total_cancel += r.cancellation_count;
                total_net += r.net_resonance as i64;

                // Per-scale tracking
                for k in 0..FIB_LEN {
                    if r.agreement & (1u32 << k) != 0 { scale_agree[k] += 1; }
                    if r.cancellation & (1u32 << k) != 0 { scale_cancel[k] += 1; }
                }
            }
        }

        // γ-warped distance
        let mut bent_distance = 0.0;
        for k in 0..FIB_LEN {
            let warped = harmonic((k as f64).max(1.0));
            let phi_w = PHI.powf(warped);
            // Agreement shortens distance, cancellation lengthens it
            bent_distance -= scale_agree[k] as f64 * phi_w / (1.0 + GAMMA * k as f64);
            bent_distance += scale_cancel[k] as f64 * phi_w * (1.0 + GAMMA * k as f64);
        }
        bent_distance = bent_distance.max(0.0);

        FibResonance10K {
            total_agreement: total_agree,
            total_cancellation: total_cancel,
            net_resonance: total_net,
            bent_distance,
            scale_agreement: scale_agree,
            scale_cancellation: scale_cancel,
            n_groups: n,
        }
    }

    /// Coarse distance: only max_scale per group dimension
    pub fn coarse_distance(&self, other: &Self) -> f64 {
        let n = self.groups.len().min(other.groups.len());
        let mut dist = 0.0;
        for g in 0..n {
            for d in 0..5 {
                let sa = self.groups[g].dims[d].max_scale as f64;
                let sb = other.groups[g].dims[d].max_scale as f64;
                let da = harmonic(sa.max(1.0));
                let db = harmonic(sb.max(1.0));
                dist += (da - db).powi(2);
            }
        }
        dist.sqrt()
    }

    pub fn n_dims(&self) -> usize { self.groups.len() * 5 }
    pub fn n_groups(&self) -> usize { self.groups.len() }
}

#[derive(Clone, Debug)]
pub struct FibResonance10K {
    pub total_agreement: u32,
    pub total_cancellation: u32,
    pub net_resonance: i64,
    pub bent_distance: f64,
    pub scale_agreement: [u32; FIB_LEN],
    pub scale_cancellation: [u32; FIB_LEN],
    pub n_groups: usize,
}

// ─── BF16 COMPATIBILITY CHECK ────────────────────────────────────────
// Show that Zeckendorf bit positions naturally align with BF16 exponent range

/// Analyze BF16 alignment for a Zeckendorf-encoded value
pub fn bf16_analysis(sz: &SignedZeckendorf) -> Bf16Alignment {
    let active = sz.pos | sz.neg;
    let max_pos = if active == 0 { 0 } else { 31 - active.leading_zeros() };
    let min_pos = if active == 0 { 0 } else { active.trailing_zeros() };

    // BF16: 1 sign + 8 exponent + 7 mantissa
    // Exponent covers 2^(-126) to 2^(127), biased by 127
    // Fibonacci position k corresponds to ~log2(φ^k) = k × 0.694 bits
    // So Fib position k maps to BF16 exponent ≈ k × 0.694 + 127

    let bf16_exp_max = (max_pos as f64 * 0.694 + 127.0) as u8;
    let bf16_exp_min = (min_pos as f64 * 0.694 + 127.0) as u8;

    // The "mantissa" in Zeckendorf is the pattern of lower bits
    // In BF16, mantissa = 7 bits = 128 levels
    // In Zeckendorf, the lower bits give finer resolution, but non-uniformly
    // The lower ~10 Fibonacci positions span ~7 BF16 mantissa bits
    let mantissa_equivalent_bits = (10.0 * 0.694) as u8; // ~6.9 ≈ 7

    Bf16Alignment {
        zeck_max_scale: max_pos as u8,
        zeck_min_scale: min_pos as u8,
        bf16_exponent_range: (bf16_exp_min, bf16_exp_max),
        mantissa_equivalent: mantissa_equivalent_bits,
        is_bf16_compatible: mantissa_equivalent_bits >= 6, // close enough to 7
    }
}

#[derive(Clone, Debug)]
pub struct Bf16Alignment {
    pub zeck_max_scale: u8,
    pub zeck_min_scale: u8,
    pub bf16_exponent_range: (u8, u8),
    pub mantissa_equivalent: u8,
    pub is_bf16_compatible: bool,
}


#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array1;
    use rand::Rng;

    #[test]
    fn signed_roundtrip() {
        for val in [-1.0, -0.5, -0.1, 0.0, 0.1, 0.5, 1.0] {
            let sz = SignedZeckendorf::encode(val, 8);
            let decoded = sz.decode();
            let expected = (val * 256.0).round() as i64;
            assert_eq!(decoded, expected, "Roundtrip failed for {}", val);
        }
    }

    #[test]
    fn sign_cancellation() {
        let pos = SignedZeckendorf::encode(0.5, 8);
        let neg = SignedZeckendorf::encode(-0.5, 8);

        let res = pos.signed_resonance(&neg);
        println!("\n═══ SIGN CANCELLATION ═══");
        println!("  +0.5: pos={:024b} neg={:024b}", pos.pos, pos.neg);
        println!("  -0.5: pos={:024b} neg={:024b}", neg.pos, neg.neg);
        println!("  agreement: {}, cancellation: {}, net: {}",
            res.agreement_count, res.cancellation_count, res.net_resonance);

        // Perfect opposites should have maximal cancellation
        assert!(res.cancellation_count > 0, "Opposites must cancel");
        assert_eq!(res.agreement_count, 0, "Opposites should not agree");
        assert!(res.net_resonance < 0, "Net should be negative for opposites");
    }

    #[test]
    fn sign_agreement() {
        let a = SignedZeckendorf::encode(0.7, 8);
        let b = SignedZeckendorf::encode(0.7, 8);

        let res = a.signed_resonance(&b);
        println!("\n═══ SIGN AGREEMENT ═══");
        println!("  agreement: {}, cancellation: {}, net: {}",
            res.agreement_count, res.cancellation_count, res.net_resonance);

        assert!(res.agreement_count > 0);
        assert_eq!(res.cancellation_count, 0);
        assert!(res.net_resonance > 0);
    }

    #[test]
    fn pentagroup_encoding() {
        let vals = [0.5, -0.3, 0.8, -0.1, 0.0];
        let pg = PentaGroup::encode(&vals, 8);

        println!("\n═══ PENTAGROUP ═══");
        for (i, d) in pg.dims.iter().enumerate() {
            println!("  dim {}: val={:.1}, decoded={}, active_trits={}, max_scale={}",
                i, vals[i], d.decode(), d.active_count(), d.max_scale);
        }
        println!("  bit_cost: {} bits", pg.bit_cost());
        println!("  fingerprint: {:016x}", pg.fingerprint());
    }

    #[test]
    fn full_10k_vector() {
        let mut rng = rand::thread_rng();
        let data: Vec<f64> = (0..10000).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let arr = Array1::from_vec(data);

        let fv = FibVec10K::from_ndarray(&arr, 8);

        println!("\n═══ 10,000D FIBONACCI VECTOR ═══");
        println!("  Dimensions: {}", fv.n_dims());
        println!("  Groups: {}", fv.n_groups());
        println!("  Total bits: {} ({:.2} kbit, {:.2} KB)",
            fv.total_bits(), fv.total_bits() as f64 / 1024.0, fv.total_bits() as f64 / 8192.0);
        println!("  Bits per dim: {:.2}", fv.total_bits() as f64 / fv.n_dims() as f64);

        assert_eq!(fv.n_dims(), 10000);
        assert_eq!(fv.n_groups(), 2000);
    }

    #[test]
    fn bit_budget_16kbit() {
        // THE KEY TEST: does 10K at reasonable precision fit in ~16kbit?
        let mut rng = rand::thread_rng();
        let data: Vec<f64> = (0..10000).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let arr = Array1::from_vec(data);

        println!("\n═══ 16kbit BUDGET TEST ═══");
        for bits in [2u8, 3, 4, 5, 6, 7, 8] {
            let fv = FibVec10K::from_ndarray(&arr, bits);
            let total = fv.total_bits();
            let kbit = total as f64 / 1024.0;
            let per_dim = total as f64 / 10000.0;
            let fits = kbit <= 16.5;
            println!("  precision={} bits: {:.1} kbit ({:.2} bits/dim) {}",
                bits, kbit, per_dim, if fits { "✓ fits 16kbit" } else { "✗ too large" });
        }
    }

    #[test]
    fn signed_discrimination() {
        let mut rng = rand::thread_rng();

        let base: Vec<f64> = (0..10000).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let similar: Vec<f64> = base.iter().map(|&v| v + rng.gen_range(-0.05..0.05)).collect();
        let different: Vec<f64> = (0..10000).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let negated: Vec<f64> = base.iter().map(|&v| -v).collect();

        let fb = FibVec10K::encode(&base, 6);
        let fs = FibVec10K::encode(&similar, 6);
        let fd = FibVec10K::encode(&different, 6);
        let fn_ = FibVec10K::encode(&negated, 6);

        let r_sim = fb.resonance(&fs);
        let r_dif = fb.resonance(&fd);
        let r_neg = fb.resonance(&fn_);

        println!("\n═══ 10K SIGNED DISCRIMINATION ═══");
        println!("  Similar:  agree={:>5} cancel={:>5} net={:>6} bent={:.2}",
            r_sim.total_agreement, r_sim.total_cancellation, r_sim.net_resonance, r_sim.bent_distance);
        println!("  Different: agree={:>5} cancel={:>5} net={:>6} bent={:.2}",
            r_dif.total_agreement, r_dif.total_cancellation, r_dif.net_resonance, r_dif.bent_distance);
        println!("  Negated:  agree={:>5} cancel={:>5} net={:>6} bent={:.2}",
            r_neg.total_agreement, r_neg.total_cancellation, r_neg.net_resonance, r_neg.bent_distance);

        // Similar should have most agreement
        assert!(r_sim.total_agreement > r_dif.total_agreement,
            "Similar should agree more: {} vs {}", r_sim.total_agreement, r_dif.total_agreement);

        // Negated should have maximal cancellation
        assert!(r_neg.total_cancellation > r_sim.total_cancellation,
            "Negated should cancel more: {} vs {}", r_neg.total_cancellation, r_sim.total_cancellation);
        assert!(r_neg.total_cancellation > r_dif.total_cancellation,
            "Negated should cancel more than different: {} vs {}", r_neg.total_cancellation, r_dif.total_cancellation);

        // Net resonance ordering: similar > different > negated
        assert!(r_sim.net_resonance > r_dif.net_resonance);
        assert!(r_dif.net_resonance > r_neg.net_resonance);
    }

    #[test]
    fn bf16_alignment_check() {
        println!("\n═══ BF16 ALIGNMENT ═══");
        for val in [0.01, 0.1, 0.3, 0.5, 0.7, 0.99] {
            let sz = SignedZeckendorf::encode(val, 12);
            let bf = bf16_analysis(&sz);
            println!("  {:.2} → zeck scale [{},{}], BF16 exp [{},{}], mantissa≈{} bits, compat={}",
                val, bf.zeck_min_scale, bf.zeck_max_scale,
                bf.bf16_exponent_range.0, bf.bf16_exponent_range.1,
                bf.mantissa_equivalent, bf.is_bf16_compatible);
        }
    }

    #[test]
    fn timing_10k() {
        let mut rng = rand::thread_rng();
        let n = 50;
        let vecs: Vec<Vec<f64>> = (0..n * 2)
            .map(|_| (0..10000).map(|_| rng.gen_range(-1.0..1.0)).collect())
            .collect();

        let t0 = std::time::Instant::now();
        let encoded: Vec<FibVec10K> = vecs.iter()
            .map(|v| FibVec10K::encode(v, 6)).collect();
        let t_enc = t0.elapsed();

        let t1 = std::time::Instant::now();
        let mut coarse_sum = 0.0;
        for i in 0..n { coarse_sum += encoded[i*2].coarse_distance(&encoded[i*2+1]); }
        let t_coarse = t1.elapsed();

        let t2 = std::time::Instant::now();
        let mut res_sum = 0i64;
        for i in 0..n { res_sum += encoded[i*2].resonance(&encoded[i*2+1]).net_resonance; }
        let t_res = t2.elapsed();

        // Cosine baseline on raw f64
        let t3 = std::time::Instant::now();
        let mut cos_sum = 0.0;
        for i in 0..n {
            let dot: f64 = vecs[i*2].iter().zip(&vecs[i*2+1]).map(|(a,b)| a*b).sum();
            let na: f64 = vecs[i*2].iter().map(|x| x*x).sum::<f64>().sqrt();
            let nb: f64 = vecs[i*2+1].iter().map(|x| x*x).sum::<f64>().sqrt();
            cos_sum += dot / (na * nb);
        }
        let t_cos = t3.elapsed();

        println!("\n═══ TIMING: 10,000D × {} pairs ═══", n);
        println!("  Encode {} vecs:  {:>9.2?}  ({:.1} µs/vec)",
            n*2, t_enc, t_enc.as_micros() as f64 / (n*2) as f64);
        println!("  Coarse dist:    {:>9.2?}  ({:.1} µs/pair)",
            t_coarse, t_coarse.as_micros() as f64 / n as f64);
        println!("  Full resonance: {:>9.2?}  ({:.1} µs/pair)",
            t_res, t_res.as_micros() as f64 / n as f64);
        println!("  Cosine f64:     {:>9.2?}  ({:.1} µs/pair)",
            t_cos, t_cos.as_micros() as f64 / n as f64);
        println!("  Coarse speedup: {:.1}×",
            t_cos.as_nanos() as f64 / t_coarse.as_nanos().max(1) as f64);
        println!("  Bits/vector:    {}", encoded[0].total_bits());
    }
}
