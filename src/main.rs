use fibonacci_vsa::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  fibonacci-vsa: Zeckendorf Vector Space with γ-Curvature   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ─── 1. ZECKENDORF ENCODING DEMO ──────────────────────────────────
    println!("═══ ZECKENDORF ENCODING ═══\n");
    for val in [1, 7, 42, 100, 255, 1000, 65535] {
        let z = ZeckendorfBits::encode(val);
        let decoded = z.decode();
        let sp = to_spiral(&z);
        println!(
            "  {} → max_scale={}, popcount={}, decoded={} | spiral: θ={:.3}rad r={:.3} z={:.3} warp={:.4}",
            val, z.max_scale, z.popcount(), decoded, sp.theta, sp.radius, sp.z, sp.warp_factor
        );
    }

    // ─── 2. γ CURVATURE PROFILE ──────────────────────────────────────
    println!("\n═══ γ SPACETIME CURVATURE ═══");
    println!("  (How Euler-Mascheroni bends the Fibonacci lattice)\n");
    println!("  {:>5} {:>15} {:>15} {:>10}", "Scale", "φ^k (flat)", "φ^H(k) (bent)", "Warp");
    println!("  {:─>5} {:─>15} {:─>15} {:─>10}", "", "", "", "");

    let profile = gamma_curvature_profile();
    for &(k, raw, warped, ratio) in &profile {
        if k <= 20 || k % 10 == 0 {
            println!("  {:>5} {:>15.2} {:>15.2} {:>10.6}", k, raw, warped, ratio);
        }
    }

    println!("\n  Key insight: warp ratio starts high (~{:.3}) at low scales",
        profile[0].3);
    println!("  and converges to ~{:.6} at high scales", profile[profile.len() - 1].3);
    println!("  γ = {:.15} is the cosmological constant that keeps", 0.577_215_664_901_532_9);
    println!("  the space curved near the origin (high resolution)");
    println!("  and flattened at infinity (abstract compression)");

    // ─── 3. WORMHOLE RESONANCE DEMO ──────────────────────────────────
    println!("\n═══ WORMHOLE RESONANCE ═══\n");

    let a = ZeckendorfBits::encode(42);     // "the answer"
    let b = ZeckendorfBits::encode(43);     // one step away
    let c = ZeckendorfBits::encode(65535);  // very different scale

    let metric_ab = wormhole_distance(&a, &b);
    let metric_ac = wormhole_distance(&a, &c);

    println!("  42 ↔ 43 (neighbors):");
    println!("    wormholes: {}, flat_dist: {:.4}, bent_dist: {:.4}, curvature: {:.4}",
        metric_ab.resonance_count, metric_ab.flat_distance, metric_ab.bent_distance, metric_ab.curvature);
    for wh in &metric_ab.wormholes {
        println!("    └─ wormhole at scale {} (warped {:.3}), strength {:.4}",
            wh.scale, wh.warped_scale, wh.shortcut_strength);
    }

    println!("\n  42 ↔ 65535 (different scale):");
    println!("    wormholes: {}, flat_dist: {:.4}, bent_dist: {:.4}, curvature: {:.4}",
        metric_ac.resonance_count, metric_ac.flat_distance, metric_ac.bent_distance, metric_ac.curvature);
    for wh in &metric_ac.wormholes {
        println!("    └─ wormhole at scale {} (warped {:.3}), strength {:.4}",
            wh.scale, wh.warped_scale, wh.shortcut_strength);
    }

    // ─── 4. FULL VECTOR DEMO (simulated 1024D) ──────────────────────
    println!("\n═══ 1024D VECTOR RESONANCE MAP ═══\n");

    // Simulate two "embeddings" — pseudo-random but deterministic
    let a_vals: Vec<f64> = (0..1024)
        .map(|i| {
            let x = (i as f64 * 0.037).sin() * 0.8;
            x
        })
        .collect();

    let b_vals: Vec<f64> = (0..1024)
        .map(|i| {
            // Similar but shifted — like two related concepts
            let x = ((i as f64 + 3.0) * 0.037).sin() * 0.8;
            x
        })
        .collect();

    let c_vals: Vec<f64> = (0..1024)
        .map(|i| {
            // Very different — unrelated concept
            let x = (i as f64 * 0.157).cos() * 0.6;
            x
        })
        .collect();

    let va = ZeckendorfVector::from_f64_slice(&a_vals, 16);
    let vb = ZeckendorfVector::from_f64_slice(&b_vals, 16);
    let vc = ZeckendorfVector::from_f64_slice(&c_vals, 16);

    let map_ab = va.resonance_map(&vb);
    let map_ac = va.resonance_map(&vc);

    println!("  A ↔ B (related concepts, slight shift):");
    println!("    Total wormholes:       {}", map_ab.total_wormholes);
    println!("    Avg curvature:         {:.6}", map_ab.avg_curvature);
    println!("    Dominant scale:        {} (γ-warped: {:.4})", map_ab.dominant_scale, map_ab.dominant_warped);
    println!("    Bent distance:         {:.6}", map_ab.bent_distance);
    println!("    Coarse distance:       {:.6}", va.coarse_distance(&vb));
    println!("    Max curvature dim:     {}", map_ab.max_curvature_dimension);

    println!("\n  A ↔ C (unrelated concepts):");
    println!("    Total wormholes:       {}", map_ac.total_wormholes);
    println!("    Avg curvature:         {:.6}", map_ac.avg_curvature);
    println!("    Dominant scale:        {} (γ-warped: {:.4})", map_ac.dominant_scale, map_ac.dominant_warped);
    println!("    Bent distance:         {:.6}", map_ac.bent_distance);
    println!("    Coarse distance:       {:.6}", va.coarse_distance(&vc));
    println!("    Max curvature dim:     {}", map_ac.max_curvature_dimension);

    // ─── 5. SCALE HISTOGRAM ─────────────────────────────────────────
    println!("\n  Scale histogram (A ↔ B) — wormhole density per Fibonacci level:");
    let max_hist = *map_ab.scale_histogram.iter().max().unwrap_or(&1);
    for (i, &count) in map_ab.scale_histogram.iter().enumerate() {
        if count > 0 {
            let bar_len = (count as f64 / max_hist as f64 * 40.0) as usize;
            let bar: String = "█".repeat(bar_len);
            println!("    scale {:>2}: {:>4} {}", i, count, bar);
        }
    }

    // ─── 6. THE γ BENDING PROOF ─────────────────────────────────────
    println!("\n═══ THE γ-BENDING PROOF ═══\n");
    println!("  Without γ, the Fibonacci lattice is a straight exponential.");
    println!("  With γ, it curves — dense near origin, flat at infinity.\n");

    println!("  {:>6} {:>12} {:>12} {:>12} {:>10}", "Scale", "H(k)", "ln(k)", "γ-effect", "% bend");
    println!("  {:─>6} {:─>12} {:─>12} {:─>12} {:─>10}", "", "", "", "", "");

    for k in [1.0f64, 2.0, 5.0, 10.0, 20.0, 50.0, 92.0] {
        let hk = k.ln() + 0.577_215_664_901_532_9 + 1.0 / (2.0 * k) - 1.0 / (12.0 * k * k);
        let lnk = k.ln();
        let gamma_contribution = hk - lnk;
        let pct = gamma_contribution / hk * 100.0;
        println!("  {:>6.0} {:>12.6} {:>12.6} {:>12.6} {:>9.2}%", k, hk, lnk, gamma_contribution, pct);
    }

    println!("\n  At scale 1: γ contributes {:.1}% of the total curvature", {
        let hk = 1.0_f64.ln() + 0.577_215_664_901_532_9 + 0.5 - 1.0/12.0;
        let gamma_part = hk - 1.0_f64.ln();
        gamma_part / hk * 100.0
    });
    println!("  At scale 92: γ contributes {:.1}% of the total curvature", {
        let k: f64 = 92.0;
        let hk = k.ln() + 0.577_215_664_901_532_9 + 1.0/(2.0*k) - 1.0/(12.0*k*k);
        let gamma_part = hk - k.ln();
        gamma_part / hk * 100.0
    });
    println!("  → γ IS the gravitational lens at human-scale semantics");
    println!("  → At cosmic scales it fades — flat spacetime returns");
    println!("  → This is EXACTLY how general relativity works:");
    println!("    mass curves space locally, but the universe is flat at large scales");
}
