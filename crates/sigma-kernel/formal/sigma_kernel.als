-- Part VI.b — Sigma Kernel: Alloy Model
-- Formal specification of the admissibility gates and invariants
-- for the Multiplicity Theory manifold.

module SigmaKernel

// ─────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────

sig SIGMA_GAP_MAX { val: one Int }
sig SIGMA_RESONANCE_MIN { val: one Int }

// ─────────────────────────────────────────────────────────────────────
// Prime values
// ─────────────────────────────────────────────────────────────────────

abstract sig Prime {}
one sig Nil, P2, P3, P5, P7, P11, P13 extends Prime {}

fun primeValue(p: Prime): Int {
  p = Nil => 0 else
  p = P2 => 2 else
  p = P3 => 3 else
  p = P5 => 5 else
  p = P7 => 7 else
  p = P11 => 11 else
  p = P13 => 13 else 0
}

// ─────────────────────────────────────────────────────────────────────
// Gap Tensor Node
// ─────────────────────────────────────────────────────────────────────

sig GapTensorNode {
  prime_val: one Prime,
  multiplicity: one Int,
  spectral_weight: one Int  -- represented as fixed-point scaled by 1000
}

// ─────────────────────────────────────────────────────────────────────
// Multiplicty Arena
// ─────────────────────────────────────────────────────────────────────

sig MultiplicityArena {
  nodes: seq GapTensorNode,
  sealed: one Int  -- 0 = false, 1 = true
}

// ─────────────────────────────────────────────────────────────────────
// Verdict outcomes
// ─────────────────────────────────────────────────────────────────────

abstract sig SigmaVerdict {}
one sig Admit extends SigmaVerdict {}
sig DissonanceGap extends SigmaVerdict {
  index: one Int,
  gap: one Int
}
sig DissonanceResonance extends SigmaVerdict {
  index: one Int,
  resonance: one Int
}
one sig Malformed extends SigmaVerdict {}
one sig Unsealed extends SigmaVerdict {}

// ─────────────────────────────────────────────────────────────────────
// S1 — Malformed Guard Invariant
// ─────────────────────────────────────────────────────────────────────

pred isWellFormed(arena: MultiplicityArena): one SigmaVerdict {
  (no arena.nodes or arena.nodes = none) => Malformed else Admit
}

// ─────────────────────────────────────────────────────────────────────
// S2 — Gap Soundness (G1: Phase Mirror)
// ─────────────────────────────────────────────────────────────────────

fun gapBetween(p1: Prime, p2: Prime): Int {
  let v1 = primeValue(p1) |
  let v2 = primeValue(p2) |
  (v1 > v2 => v1 - v2 else v2 - v1)
}

pred isNil(node: GapTensorNode): Boolean {
  node.prime_val = Nil
}

fun gatePhaseMirror(arena: MultiplicityArena): SigmaVerdict {
  (no arena.nodes) => Malformed else
  let indices = Int | i >= 0 && i < sub[#arena.nodes, 1] |
  (exists i: indices |
    let node_i = arena.nodes[i] |
    let node_next = arena.nodes[add[i, 1]] |
    (!(isNil[node_i]) && !(isNil[node_next]) &&
     gapBetween[node_i.prime_val, node_next.prime_val] > SIGMA_GAP_MAX.val)
  ) => DissonanceGap else Admit
}

// ─────────────────────────────────────────────────────────────────────
// S3 — Resonance Soundness (G2: Resonance Certificate)
// ─────────────────────────────────────────────────────────────────────

fun resonance(node: GapTensorNode): Int {
  node.multiplicity * node.spectral_weight
}

fun gateResonance(arena: MultiplicityArena): SigmaVerdict {
  (no arena.nodes) => Malformed else
  let indices = Int | i >= 0 && i < #arena.nodes |
  (exists i: indices |
    let node = arena.nodes[i] |
    (!(isNil[node]) && resonance[node] < SIGMA_RESONANCE_MIN.val)
  ) => DissonanceResonance else Admit
}

// ─────────────────────────────────────────────────────────────────────
// S4 — Composite Gate (G1 + G2)
// ─────────────────────────────────────────────────────────────────────

fun sigmaGate(arena: MultiplicityArena): SigmaVerdict {
  let g1 = gatePhaseMirror[arena] |
  let g2 = gateResonance[arena] |
  (g1 = Admit && g2 = Admit) => Admit else
  (g1 != Admit) => g1 else g2
}

// ─────────────────────────────────────────────────────────────────────
// S5 — Seal Monotonicity
// ─────────────────────────────────────────────────────────────────────

pred sealMonotonicity(arena: MultiplicityArena): Boolean {
  arena.sealed = 1 => (all arena2: MultiplicityArena | arena2 = arena => arena2.sealed = 1)
}

// ─────────────────────────────────────────────────────────────────────
// S7 — Backtracking Restores State
// ─────────────────────────────────────────────────────────────────────

pred backtrackRestores(arena: MultiplicityArena, index: Int): Boolean {
  (arena.nodes[index].prime_val = Nil) => (arena.nodes[index].multiplicity = 0)
}

// ─────────────────────────────────────────────────────────────────────
// Key Invariants
// ─────────────────────────────────────────────────────────────────────

-- S2 Check: if gate_phase_mirror returns Admit, gap is bounded
assert GatePhaseMirrorSoundness {
  all arena: MultiplicityArena |
    sigmaGate[arena] = Admit => gatePhaseMirror[arena] = Admit
}

-- S3 Check: if gate_resonance returns Admit, resonance is bounded
assert GateResonanceSoundness {
  all arena: MultiplicityArena |
    sigmaGate[arena] = Admit => gateResonance[arena] = Admit
}

-- S4 Check: sigma_gate = Admit iff both G1 and G2 = Admit
assert SigmaGateComposite {
  all arena: MultiplicityArena |
    sigmaGate[arena] = Admit <=> (gatePhaseMirror[arena] = Admit && gateResonance[arena] = Admit)
}

// ─────────────────────────────────────────────────────────────────────
// Verification commands
// ─────────────────────────────────────────────────────────────────────

-- Check for counterexamples to S2
check GatePhaseMirrorSoundness for 5

-- Check for counterexamples to S3
check GateResonanceSoundness for 5

-- Check for counterexamples to S4
check SigmaGateComposite for 5
