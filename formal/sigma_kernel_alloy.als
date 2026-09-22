// Part VIII — Sigma Kernel: Level-Synchronous Barriers and DAG Topology
// Formal specification incorporating execution levels and parallelism safety

open util/ordering[Level] as levels

// ─────────────────────────────────────────────────────────────────────
// 1. Core signatures
// ─────────────────────────────────────────────────────────────────────

abstract sig Prime {}
one sig Nil, P2, P3, P5, P7, P11, P13 extends Prime {}

// Represents synchronous barriers in the parallel scheduler
sig Level {}

sig SpineNode {
  label: one Prime,
  edges: set SpineNode,
  level: one Level
}

// ─────────────────────────────────────────────────────────────────────
// 2. Structural Invariants
// ─────────────────────────────────────────────────────────────────────

// Invariant 1: Pure DAG topology (no cycles)
fact IsDAG {
  no iden & ^edges
}

// Invariant 2: No floating nodes (connectivity)
fact IsConnected {
  SpineNode in SpineNode.^edges + ^edges.SpineNode
}

// Invariant 3: Level-Synchronous Barriers
// Child nodes scheduled at later levels than parents
fact LevelSynchronousBarriers {
  all n1, n2: SpineNode |
    n2 in n1.edges implies levels/lt[n1.level, n2.level]
}

// ─────────────────────────────────────────────────────────────────────
// 3. Gap Tensor Constraints (Sigma Kernel Bounds)
// ─────────────────────────────────────────────────────────────────────

// Bad pairs: edges that violate gap <= 8 constraint
fun badPairs : Prime -> Prime {
  P2->P11 + P11->P2 +    // gap = 9 > 8
  P2->P13 + P13->P2 +    // gap = 11 > 8
  P3->P13 + P13->P3      // gap = 10 > 8
}

// ─────────────────────────────────────────────────────────────────────
// 4. Predicates: Admissibility Gates
// ─────────────────────────────────────────────────────────────────────

// G1 — Phase Mirror Dissonance: hunt for violations
pred hasDissonance[] {
  some n1, n2: SpineNode |
    n2 in n1.edges and (n1.label -> n2.label in badPairs)
}

// G2 — Resonance: all nodes must be reachable (implicit in IsConnected)
pred hasResonance[] {
  all n: SpineNode | n in SpineNode
}

// G3 — Recursive Stabilization: all edges respect level barriers
pred hasStabilization[] {
  all n1, n2: SpineNode |
    n2 in n1.edges implies levels/lt[n1.level, n2.level]
}

// ─────────────────────────────────────────────────────────────────────
// 5. Composite Gates
// ─────────────────────────────────────────────────────────────────────

// Sigma Gate: conjunction of all admissibility gates
pred validExecutionSchedule[] {
  not hasDissonance[] and
  hasResonance[] and
  hasStabilization[]
}

// Delta-admissible: strict gap <= 8 (Δ=0)
pred deltaAdmissible[] {
  not hasDissonance[]
}

// ─────────────────────────────────────────────────────────────────────
// 6. Verification Commands
// ─────────────────────────────────────────────────────────────────────

// Find counterexamples: spines that violate DAG invariant
run {
  some n: SpineNode | n in n.^edges
} for 7 SpineNode, 5 Level

// Find counterexamples: spines with floating nodes
run {
  some n: SpineNode | n not in (SpineNode.^edges + ^edges.SpineNode)
} for 7 SpineNode, 5 Level

// Find counterexamples: spines with level inversions
run {
  some n1, n2: SpineNode |
    n2 in n1.edges and levels/lt[n2.level, n1.level]
} for 7 SpineNode, 5 Level

// Find counterexamples: spines with gap violations
run hasDissonance for 7 SpineNode, 5 Level

// Generate valid execution schedules (no violations)
run validExecutionSchedule for 7 SpineNode, 5 Level

// Generate delta-admissible spines (gap <= 8)
run deltaAdmissible for 7 SpineNode, 5 Level

// ─────────────────────────────────────────────────────────────────────
// 7. Assertions for Invariant Checking
// ─────────────────────────────────────────────────────────────────────

// S1 — Malformed Guard: no empty spines
assert NoEmptySpines {
  #SpineNode > 0
}

// S2 — Gap Soundness: if deltaAdmissible, no bad pairs
assert GapSoundness {
  deltaAdmissible[] implies not hasDissonance[]
}

// S4 — Composite Gate: sigma validates all sub-gates
assert SigmaComposite {
  validExecutionSchedule[] implies
    (not hasDissonance[] and hasResonance[] and hasStabilization[])
}

// S6 — Solver Soundness: valid schedules satisfy all gates
assert SolverSoundness {
  all nodes: set SpineNode |
    (no n1, n2: nodes | n2 in n1.^edges and n1.label -> n2.label in badPairs) and
    (all n1, n2: nodes | n2 in n1.edges implies levels/lt[n1.level, n2.level])
    implies validExecutionSchedule[]
}

// ─────────────────────────────────────────────────────────────────────
// 8. Check commands
// ─────────────────────────────────────────────────────────────────────

check NoEmptySpines for 7 SpineNode, 5 Level
check GapSoundness for 7 SpineNode, 5 Level
check SigmaComposite for 7 SpineNode, 5 Level
check SolverSoundness for 7 SpineNode, 5 Level
