use sigma_kernel::{
    MultiplicityArena, SigmaVerdict, CANDIDATE_PRIMES, SIGMA_GAP_MAX, SIGMA_RESONANCE_MIN,
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
        let mut arena = MultiplicityArena::build_spine(4);

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
        let mut arena = MultiplicityArena::build_spine(2);

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
        let mut arena = MultiplicityArena::build_spine(1);

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

        println!("  Before seal:");
        println!("    Sealed:  {}", arena.is_sealed());
        println!("    Sigma:   {:?}", arena.sigma_gate());

        let gov_hash = 0xdeadbeef_cafebabe;
        let seal_result = arena.seal(gov_hash);

        println!("  After seal:");
        println!("    Result:  {:?}", seal_result);
        println!("    Sealed:  {}", arena.is_sealed());

        arena.destroy();
    }

    println!();
    println!("═════════════════════════════════════════════════");
    println!("All gates operational. Sigma Kernel active.");
}
