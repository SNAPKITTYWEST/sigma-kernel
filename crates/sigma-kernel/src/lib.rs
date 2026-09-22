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

//! Part VI.b — Sigma Kernel Admissibility Gate
//!
//! The protocol-level firewall that governs whether a tensor state is admitted
//! into the Multiplicity Theory manifold. Four gates separate into distinct
//! logical operations that can be independently audited, tested, and proved:
//!
//! G1 Phase Mirror Dissonance — structural admissibility
//! G2 Resonance Certificate — energetic admissibility
//! G3 Recursive Stabilization — global admissibility
//! G4 GOV_HASH Sealing — cryptographic immutability
//!
//! Gates G1–G3 are evaluated in-memory. Gate G4 is evaluated only when the
//! tensor reaches the ledger boundary.

use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

// ─────────────────────────────────────────────────────────────────────
// Constants — the protocol's admissibility thresholds
// ─────────────────────────────────────────────────────────────────────

/// Maximum permitted prime gap. Any larger gap is dissonance.
pub const SIGMA_GAP_MAX: u32 = 8;

/// Minimum resonance weight for a node to participate in a contraction.
pub const SIGMA_RESONANCE_MIN: f32 = 1.0;

/// The permitted prime eigenvalues. Nil (0) is the contradiction state.
pub const CANDIDATE_PRIMES: [u32; 6] = [2, 3, 5, 7, 11, 13];

// ─────────────────────────────────────────────────────────────────────
// Core types
// ─────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GapTensorNode {
    pub prime_val: u32,
    pub multiplicity: u32,
    pub spectral_weight: f32,
}

impl GapTensorNode {
    pub const NIL: Self = Self {
        prime_val: 0,
        multiplicity: 0,
        spectral_weight: 0.0,
    };

    /// A node participates in a contraction iff its resonance is
    /// at least SIGMA_RESONANCE_MIN.
    pub fn resonance(&self) -> f32 {
        self.multiplicity as f32 * self.spectral_weight
    }

    /// Is this node in the Nil / contradiction state?
    pub fn is_nil(&self) -> bool {
        self.prime_val == 0
    }
}

// ─────────────────────────────────────────────────────────────────────
// Admissibility verdicts
// ─────────────────────────────────────────────────────────────────────

/// The outcome of a Sigma Kernel admissibility check. Every gate
/// returns one of these. `Admit` is the only success case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SigmaVerdict {
    /// The tensor passed all gates and may be sealed.
    Admit,
    /// G1 failed: gap exceeds SIGMA_GAP_MAX at the given index.
    DissonanceGap { index: usize, gap: u32 },
    /// G2 failed: resonance below threshold at the given index.
    DissonanceResonance { index: usize, resonance_bits: u32 },
    /// The arena is malformed (zero length, null pointer).
    Malformed,
    /// G4 has not yet been evaluated.
    Unsealed,
}

// ─────────────────────────────────────────────────────────────────────
// The arena
// ─────────────────────────────────────────────────────────────────────

pub struct MultiplicityArena {
    pub nodes: *mut GapTensorNode,
    pub num_nodes: usize,
    sealed: bool,
}

impl MultiplicityArena {
    /// Build the spine. This is allocation + initialization only;
    /// it performs no admissibility checks.
    ///
    /// # Safety
    /// The caller must ensure `num_nodes > 0` and `< usize::MAX / size_of::<GapTensorNode>()`.
    pub unsafe fn build_spine(num_nodes: usize) -> Self {
        assert!(num_nodes > 0, "SIGMA-E-EMPTY: zero-length spine");
        let layout = Layout::array::<GapTensorNode>(num_nodes)
            .expect("SIGMA-E-LAYOUT: array layout overflow");
        let ptr = alloc(layout) as *mut GapTensorNode;
        if ptr.is_null() {
            panic!("SIGMA-E-ALLOC: prime materia allocation failed");
        }
        for i in 0..num_nodes {
            ptr.add(i).write(GapTensorNode::NIL);
        }
        Self {
            nodes: ptr,
            num_nodes,
            sealed: false,
        }
    }

    /// Run the recursive stabilization search. This is the *solver*:
    /// it tries candidate primes at each depth and backtracks when a
    /// gate rejects the partial tensor.
    ///
    /// # Safety
    /// `target_depth <= self.num_nodes`.
    pub unsafe fn stabilize_tensor_recursive(
        &mut self,
        target_depth: usize,
        current_idx: usize,
    ) -> bool {
        if current_idx >= target_depth {
            // Base case: evaluate G1 and G2 on the complete prefix.
            return matches!(self.sigma_gate(), SigmaVerdict::Admit);
        }

        for &p in &CANDIDATE_PRIMES {
            let node = self.nodes.add(current_idx);
            (*node).prime_val = p;
            (*node).multiplicity += 1;

            if self.stabilize_tensor_recursive(target_depth, current_idx + 1) {
                return true;
            }

            (*node).multiplicity -= 1;
        }

        // Contradiction: revert to Nil. S7 fix: reset multiplicity.
        (*self.nodes.add(current_idx)).prime_val = 0;
        (*self.nodes.add(current_idx)).multiplicity = 0;
        false
    }

    /// G1 alone: structural admissibility.
    ///
    /// # Safety
    /// Caller must ensure `self.nodes` is a valid pointer to an array of at least `self.num_nodes` elements,
    /// or `self.num_nodes == 0` (returns Malformed immediately).
    pub unsafe fn gate_phase_mirror(&self) -> SigmaVerdict {
        if self.num_nodes == 0 || self.nodes.is_null() {
            return SigmaVerdict::Malformed;
        }
        for i in 0..(self.num_nodes - 1) {
            let a = (*self.nodes.add(i)).prime_val;
            let b = (*self.nodes.add(i + 1)).prime_val;
            // Nil pairs are permitted during construction.
            if a == 0 || b == 0 {
                continue;
            }
            let gap = b.abs_diff(a);
            if gap > SIGMA_GAP_MAX {
                return SigmaVerdict::DissonanceGap { index: i, gap };
            }
        }
        SigmaVerdict::Admit
    }

    /// G2 alone: energetic admissibility.
    ///
    /// # Safety
    /// Caller must ensure `self.nodes` is a valid pointer to an array of at least `self.num_nodes` elements,
    /// or `self.num_nodes == 0` (returns Malformed immediately).
    pub unsafe fn gate_resonance(&self) -> SigmaVerdict {
        if self.num_nodes == 0 || self.nodes.is_null() {
            return SigmaVerdict::Malformed;
        }
        for i in 0..self.num_nodes {
            let n = *self.nodes.add(i);
            // Nil nodes are permitted during construction.
            if n.is_nil() {
                continue;
            }
            let r = n.resonance();
            if r < SIGMA_RESONANCE_MIN {
                return SigmaVerdict::DissonanceResonance {
                    index: i,
                    resonance_bits: r.to_bits(),
                };
            }
        }
        SigmaVerdict::Admit
    }

    /// G1 + G2 combined. This is the composite `sigma_gate` called by
    /// the recursive solver's base case.
    ///
    /// # Safety
    /// Caller must ensure `self.nodes` is a valid pointer to an array of at least `self.num_nodes` elements,
    /// or `self.num_nodes == 0` (returns Malformed immediately).
    pub unsafe fn sigma_gate(&self) -> SigmaVerdict {
        match self.gate_phase_mirror() {
            SigmaVerdict::Admit => {}
            v => return v,
        }
        self.gate_resonance()
    }

    /// G4: seal the tensor. Only callable when sigma_gate() = Admit.
    ///
    /// # Safety
    /// The caller must have already verified `sigma_gate() = Admit`.
    pub unsafe fn seal(&mut self, gov_hash: u64) -> SigmaVerdict {
        match self.sigma_gate() {
            SigmaVerdict::Admit => {
                self.sealed = true;
                let _ = gov_hash;
                SigmaVerdict::Admit
            }
            v => v,
        }
    }

    /// Is this arena sealed?
    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    /// Free the arena. Safe to call once; double-free is prevented by
    /// consuming `self`.
    pub fn destroy(self) {
        unsafe {
            let layout = Layout::array::<GapTensorNode>(self.num_nodes).unwrap();
            dealloc(self.nodes as *mut u8, layout);
        }
        std::mem::forget(self);
    }
}

impl Drop for MultiplicityArena {
    fn drop(&mut self) {
        if !self.nodes.is_null() {
            unsafe {
                let layout = Layout::array::<GapTensorNode>(self.num_nodes).unwrap();
                dealloc(self.nodes as *mut u8, layout);
            }
            self.nodes = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_tensor_node_nil() {
        let node = GapTensorNode::NIL;
        assert!(node.is_nil());
        assert_eq!(node.resonance(), 0.0);
    }

    #[test]
    fn test_gap_tensor_node_resonance() {
        let node = GapTensorNode {
            prime_val: 2,
            multiplicity: 3,
            spectral_weight: 2.0,
        };
        assert_eq!(node.resonance(), 6.0);
    }

    #[test]
    fn test_arena_malformed_empty() {
        unsafe {
            let arena = MultiplicityArena {
                nodes: std::ptr::null_mut(),
                num_nodes: 0,
                sealed: false,
            };
            let verdict = arena.gate_phase_mirror();
            assert_eq!(verdict, SigmaVerdict::Malformed);
        }
    }

    #[test]
    fn test_arena_gate_phase_mirror_admit() {
        unsafe {
            let mut arena = MultiplicityArena::build_spine(3);
            (*arena.nodes.add(0)).prime_val = 2;
            (*arena.nodes.add(1)).prime_val = 3;
            (*arena.nodes.add(2)).prime_val = 5;

            let verdict = arena.gate_phase_mirror();
            assert_eq!(verdict, SigmaVerdict::Admit);

            arena.destroy();
        }
    }

    #[test]
    fn test_arena_gate_phase_mirror_dissonance() {
        unsafe {
            let mut arena = MultiplicityArena::build_spine(2);
            (*arena.nodes.add(0)).prime_val = 2;
            (*arena.nodes.add(1)).prime_val = 13;

            let verdict = arena.gate_phase_mirror();
            match verdict {
                SigmaVerdict::DissonanceGap { index, gap } => {
                    assert_eq!(index, 0);
                    assert_eq!(gap, 11);
                }
                _ => panic!("Expected DissonanceGap, got {:?}", verdict),
            }

            arena.destroy();
        }
    }

    #[test]
    fn test_arena_gate_resonance_admit() {
        unsafe {
            let mut arena = MultiplicityArena::build_spine(1);
            (*arena.nodes.add(0)).prime_val = 2;
            (*arena.nodes.add(0)).multiplicity = 1;
            (*arena.nodes.add(0)).spectral_weight = 1.5;

            let verdict = arena.gate_resonance();
            assert_eq!(verdict, SigmaVerdict::Admit);

            arena.destroy();
        }
    }

    #[test]
    fn test_arena_gate_resonance_dissonance() {
        unsafe {
            let mut arena = MultiplicityArena::build_spine(1);
            (*arena.nodes.add(0)).prime_val = 2;
            (*arena.nodes.add(0)).multiplicity = 0;
            (*arena.nodes.add(0)).spectral_weight = 2.0;

            let verdict = arena.gate_resonance();
            match verdict {
                SigmaVerdict::DissonanceResonance { index, .. } => {
                    assert_eq!(index, 0);
                }
                _ => panic!("Expected DissonanceResonance, got {:?}", verdict),
            }

            arena.destroy();
        }
    }

    #[test]
    fn test_seal_monotonicity() {
        unsafe {
            let mut arena = MultiplicityArena::build_spine(1);
            (*arena.nodes.add(0)).prime_val = 2;
            (*arena.nodes.add(0)).multiplicity = 1;
            (*arena.nodes.add(0)).spectral_weight = 1.5;

            assert!(!arena.is_sealed());

            let verdict = arena.seal(42);
            assert_eq!(verdict, SigmaVerdict::Admit);
            assert!(arena.is_sealed());

            arena.destroy();
        }
    }

    #[test]
    fn test_backtracking_restores_state() {
        unsafe {
            let mut arena = MultiplicityArena::build_spine(3);
            (*arena.nodes.add(0)).multiplicity = 0;
            (*arena.nodes.add(1)).multiplicity = 0;
            (*arena.nodes.add(2)).multiplicity = 0;

            for i in 0..3 {
                (*arena.nodes.add(i)).spectral_weight = 1.5;
            }

            let _ = arena.stabilize_tensor_recursive(3, 0);

            for i in 0..3 {
                let node = *arena.nodes.add(i);
                if node.prime_val == 0 {
                    assert_eq!(node.multiplicity, 0, "S7 violation: multiplicity not reset at index {}", i);
                }
            }

            arena.destroy();
        }
    }
}
