// This Source Code Form is governed by the
// Node-Locked Network Public License v1.0 (NLNPL-1.0).
//
// File-level copyleft applies to this Covered File.
//
// Network-service use may trigger source-disclosure obligations.
//
// Execution may require a valid Licensor-issued Node Key.
//
// See LICENSE for complete terms.
//
// PRIOR ART BADGE: April 14, 2026 — Project inception.
// SPDX-License-Identifier: LicenseRef-NLNPL-1.0

use sigma_kernel_lib::{
    MultiplicityArena, CANDIDATE_PRIMES, SIGMA_GAP_MAX, SIGMA_RESONANCE_MIN,
};

fn main() {
    println!("Sigma Kernel — Admissibility Gate");
    println!("═════════════════════════════════════════════════");
    println!();
    println!("Configuration:");
    println!("  SIGMA_GAP_MAX:        {}", SIGMA_GAP_MAX);
    println!("  SIGMA_RESONANCE_MIN:  {}", SIGMA_RESONANCE_MIN);
    println!("  CANDIDATE_PRIMES:     {:?}", CANDIDATE_PRIMES);
    println!();

    unsafe {
        println!("Demo 1: Admissible spine [2, 3, 5, 7]");
        let arena = MultiplicityArena::build_spine(4);
        (*arena.nodes.add(0)).prime_val = 2;
        (*arena.nodes.add(0)).multiplicity = 1;
        (*arena.nodes.add(0)).spectral_weight = 1.5;
        (*arena.nodes.add(1)).prime_val = 3;
        (*arena.nodes.add(1)).multiplicity = 1;
        (*arena.nodes.add(1)).spectral_weight = 1.2;
        (*arena.nodes.add(2)).prime_val = 5;
        (*arena.nodes.add(2)).multiplicity = 1;
        (*arena.nodes.add(2)).spectral_weight = 1.0;
        (*arena.nodes.add(3)).prime_val = 7;
        (*arena.nodes.add(3)).multiplicity = 1;
        (*arena.nodes.add(3)).spectral_weight = 1.3;
        println!("  G1 (Phase Mirror):  {:?}", arena.gate_phase_mirror());
        println!("  G2 (Resonance):     {:?}", arena.gate_resonance());
        println!("  G3 (Sigma Gate):    {:?}", arena.sigma_gate());
        arena.destroy();
    }

    println!();

    unsafe {
        println!("Demo 2: Dissonant spine [2, 13] (gap = 11 > 8)");
        let arena = MultiplicityArena::build_spine(2);
        (*arena.nodes.add(0)).prime_val = 2;
        (*arena.nodes.add(0)).multiplicity = 1;
        (*arena.nodes.add(0)).spectral_weight = 1.5;
        (*arena.nodes.add(1)).prime_val = 13;
        (*arena.nodes.add(1)).multiplicity = 1;
        (*arena.nodes.add(1)).spectral_weight = 1.5;
        println!("  G1 (Phase Mirror):  {:?}", arena.gate_phase_mirror());
        println!("  G2 (Resonance):     {:?}", arena.gate_resonance());
        println!("  G3 (Sigma Gate):    {:?}", arena.sigma_gate());
        arena.destroy();
    }

    println!();

    unsafe {
        println!("Demo 3: Low-resonance spine [2] with weak spectral weight");
        let arena = MultiplicityArena::build_spine(1);
        (*arena.nodes.add(0)).prime_val = 2;
        (*arena.nodes.add(0)).multiplicity = 0;
        (*arena.nodes.add(0)).spectral_weight = 2.0;
        println!("  G1 (Phase Mirror):  {:?}", arena.gate_phase_mirror());
        println!("  G2 (Resonance):     {:?}", arena.gate_resonance());
        println!("  G3 (Sigma Gate):    {:?}", arena.sigma_gate());
        arena.destroy();
    }

    println!();

    unsafe {
        println!("Demo 4: Seal a valid spine");
        let mut arena = MultiplicityArena::build_spine(1);
        (*arena.nodes.add(0)).prime_val = 3;
        (*arena.nodes.add(0)).multiplicity = 2;
        (*arena.nodes.add(0)).spectral_weight = 0.6;
        println!("  Before seal: Sealed={}, Sigma={:?}", arena.is_sealed(), arena.sigma_gate());
        let seal_result = arena.seal(0xdeadbeef_cafebabe);
        println!("  After seal:  Result={:?}, Sealed={}", seal_result, arena.is_sealed());
        arena.destroy();
    }

    println!();
    println!("═════════════════════════════════════════════════");
    println!("All gates operational. Sigma Kernel active.");
}
