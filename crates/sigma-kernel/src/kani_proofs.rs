//! Kani bounded model checking proofs for the Sigma Kernel
//! These harnesses verify the invariants S1–S9 at compile time

#[cfg(all(test, kani))]
mod kani_proofs {
    use crate::{
        MultiplicityArena, SigmaVerdict, GapTensorNode, SIGMA_GAP_MAX, SIGMA_RESONANCE_MIN,
        CANDIDATE_PRIMES,
    };

    // ─────────────────────────────────────────────────────────────────────
    // S1 — Malformed Guard
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    fn verify_s1_malformed_guard_empty() {
        unsafe {
            let arena = MultiplicityArena {
                nodes: std::ptr::null_mut(),
                num_nodes: 0,
                sealed: false,
            };

            let verdict = arena.gate_phase_mirror();
            assert_eq!(verdict, SigmaVerdict::Malformed, "S1: empty arena should return Malformed");
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // S2 — Gap Soundness
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_s2_gap_soundness() {
        unsafe {
            let n: usize = kani::any();
            kani::assume(n > 0 && n <= 4);

            let mut arena = MultiplicityArena::build_spine(n);

            for i in 0..n {
                let p: u32 = kani::any();
                kani::assume(
                    p == 0
                        || p == 2
                        || p == 3
                        || p == 5
                        || p == 7
                        || p == 11
                        || p == 13,
                );
                (*arena.nodes.add(i)).prime_val = p;
                (*arena.nodes.add(i)).multiplicity = kani::any();
                (*arena.nodes.add(i)).spectral_weight = kani::any::<f32>();
            }

            if let SigmaVerdict::Admit = arena.gate_phase_mirror() {
                for i in 0..(n.saturating_sub(1)) {
                    let a = (*arena.nodes.add(i)).prime_val;
                    let b = (*arena.nodes.add(i + 1)).prime_val;

                    if a != 0 && b != 0 {
                        let gap = a.abs_diff(b);
                        assert!(
                            gap <= SIGMA_GAP_MAX,
                            "S2: gap {} at index {} exceeds SIGMA_GAP_MAX {}",
                            gap,
                            i,
                            SIGMA_GAP_MAX
                        );
                    }
                }
            }

            arena.destroy();
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // S3 — Resonance Soundness
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_s3_resonance_soundness() {
        unsafe {
            let n: usize = kani::any();
            kani::assume(n > 0 && n <= 4);

            let mut arena = MultiplicityArena::build_spine(n);

            for i in 0..n {
                let p: u32 = kani::any();
                kani::assume(
                    p == 0
                        || p == 2
                        || p == 3
                        || p == 5
                        || p == 7
                        || p == 11
                        || p == 13,
                );
                let mult: u32 = kani::any();
                let weight: f32 = kani::any();
                kani::assume(weight >= 0.0);

                (*arena.nodes.add(i)).prime_val = p;
                (*arena.nodes.add(i)).multiplicity = mult;
                (*arena.nodes.add(i)).spectral_weight = weight;
            }

            if let SigmaVerdict::Admit = arena.gate_resonance() {
                for i in 0..n {
                    let node = *arena.nodes.add(i);

                    if node.prime_val != 0 {
                        let r = node.resonance();
                        assert!(
                            r >= SIGMA_RESONANCE_MIN,
                            "S3: resonance {} at index {} below SIGMA_RESONANCE_MIN {}",
                            r,
                            i,
                            SIGMA_RESONANCE_MIN
                        );
                    }
                }
            }

            arena.destroy();
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // S4 — Composite Gate
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_s4_composite_gate() {
        unsafe {
            let n: usize = kani::any();
            kani::assume(n > 0 && n <= 4);

            let mut arena = MultiplicityArena::build_spine(n);

            for i in 0..n {
                let p: u32 = kani::any();
                kani::assume(
                    p == 0
                        || p == 2
                        || p == 3
                        || p == 5
                        || p == 7
                        || p == 11
                        || p == 13,
                );
                (*arena.nodes.add(i)).prime_val = p;
                (*arena.nodes.add(i)).multiplicity = kani::any();
                (*arena.nodes.add(i)).spectral_weight = kani::any::<f32>();
            }

            let g1 = arena.gate_phase_mirror();
            let g2 = arena.gate_resonance();
            let sigma = arena.sigma_gate();

            // S4: sigma_gate = Admit iff both G1 and G2 = Admit
            match (g1, g2, sigma) {
                (SigmaVerdict::Admit, SigmaVerdict::Admit, SigmaVerdict::Admit) => {
                    // OK
                }
                (SigmaVerdict::Admit, SigmaVerdict::Admit, _) => {
                    panic!("S4: both gates Admit but sigma != Admit")
                }
                (_, _, SigmaVerdict::Admit) => {
                    panic!("S4: sigma Admit but gates not both Admit")
                }
                _ => {
                    // OK
                }
            }

            arena.destroy();
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // S5 — Seal Monotonicity
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    fn verify_s5_seal_monotonicity() {
        unsafe {
            let mut arena = MultiplicityArena::build_spine(1);
            (*arena.nodes.add(0)).prime_val = 2;
            (*arena.nodes.add(0)).multiplicity = 1;
            (*arena.nodes.add(0)).spectral_weight = 1.5;

            assert!(!arena.is_sealed(), "S5: arena initially not sealed");

            let _ = arena.seal(42);

            assert!(arena.is_sealed(), "S5: arena sealed after seal()");

            arena.destroy();
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // S6 — Solver Soundness (if recursive returns true, sigma_gate = Admit)
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_s6_solver_soundness() {
        unsafe {
            let target_depth: usize = kani::any();
            kani::assume(target_depth > 0 && target_depth <= 4);

            let mut arena = MultiplicityArena::build_spine(target_depth);

            for i in 0..target_depth {
                (*arena.nodes.add(i)).spectral_weight = 1.5;
            }

            let result = arena.stabilize_tensor_recursive(target_depth, 0);

            if result {
                let sigma = arena.sigma_gate();
                assert_eq!(
                    sigma, SigmaVerdict::Admit,
                    "S6: if solver returns true, sigma_gate must be Admit"
                );
            }

            arena.destroy();
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // S7 — Backtracking Restores State
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_s7_backtracking_restores() {
        unsafe {
            let target_depth: usize = kani::any();
            kani::assume(target_depth > 0 && target_depth <= 4);

            let mut arena = MultiplicityArena::build_spine(target_depth);

            for i in 0..target_depth {
                (*arena.nodes.add(i)).spectral_weight = 1.5;
            }

            let _ = arena.stabilize_tensor_recursive(target_depth, 0);

            // S7: after backtrack, if prime is Nil, multiplicity must be 0
            for i in 0..target_depth {
                let node = *arena.nodes.add(i);
                if node.prime_val == 0 {
                    assert_eq!(
                        node.multiplicity, 0,
                        "S7: at index {}, if prime_val = Nil, multiplicity must be 0",
                        i
                    );
                }
            }

            arena.destroy();
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // S8 — Termination (structural, no timeout)
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_s8_termination() {
        unsafe {
            let target_depth: usize = kani::any();
            kani::assume(target_depth > 0 && target_depth <= 4);

            let mut arena = MultiplicityArena::build_spine(target_depth);

            for i in 0..target_depth {
                (*arena.nodes.add(i)).spectral_weight = 1.5;
            }

            // No explicit timeout; Kani's unwind bound ensures termination
            let _result = arena.stabilize_tensor_recursive(target_depth, 0);

            arena.destroy();
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // S9 — GOV_HASH Sealing
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    fn verify_s9_gov_hash_sealing() {
        unsafe {
            let mut arena = MultiplicityArena::build_spine(1);
            (*arena.nodes.add(0)).prime_val = 2;
            (*arena.nodes.add(0)).multiplicity = 1;
            (*arena.nodes.add(0)).spectral_weight = 1.5;

            let gov_hash: u64 = 0xdeadbeef_cafebabe;
            let verdict = arena.seal(gov_hash);

            assert_eq!(
                verdict, SigmaVerdict::Admit,
                "S9: seal on admissible arena returns Admit"
            );
            assert!(arena.is_sealed(), "S9: arena is sealed after seal()");

            arena.destroy();
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Index bounds checking
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    #[kani::unwind(5)]
    fn verify_no_out_of_bounds() {
        unsafe {
            let n: usize = kani::any();
            kani::assume(n > 0 && n <= 4);

            let mut arena = MultiplicityArena::build_spine(n);

            for i in 0..n {
                let p: u32 = kani::any();
                kani::assume(p == 0 || p == 2 || p == 3 || p == 5 || p == 7 || p == 11 || p == 13);
                (*arena.nodes.add(i)).prime_val = p;
                (*arena.nodes.add(i)).multiplicity = kani::any();
                (*arena.nodes.add(i)).spectral_weight = kani::any::<f32>();
            }

            // Trigger all gates; they should not panic on valid indices
            let _g1 = arena.gate_phase_mirror();
            let _g2 = arena.gate_resonance();
            let _sigma = arena.sigma_gate();

            arena.destroy();
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Null pointer check
    // ─────────────────────────────────────────────────────────────────────

    #[kani::proof]
    fn verify_null_pointer_handling() {
        unsafe {
            let arena = MultiplicityArena {
                nodes: std::ptr::null_mut(),
                num_nodes: 1,
                sealed: false,
            };

            let g1 = arena.gate_phase_mirror();
            let g2 = arena.gate_resonance();

            assert_eq!(g1, SigmaVerdict::Malformed, "null nodes should return Malformed");
            assert_eq!(g2, SigmaVerdict::Malformed, "null nodes should return Malformed");
        }
    }
}
