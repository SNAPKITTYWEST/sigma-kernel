# Sigma Kernel — Admissibility Gate for the Multiplicity Theory Manifold

A complete verification pipeline for the Sigma Kernel, the protocol-level firewall that governs tensor state admission into the Multiplicity Theory (MT) manifold. Implements a four-stage gate system with formal proofs across Rust, Alloy, Kani bounded model checking, LiquidHaskell refinement types, and Circom zero-knowledge circuits.

## Architecture

The Sigma Kernel separates four distinct logical operations:

### G1 — Phase Mirror Dissonance (Structural)
Rejects any tensor with consecutive prime gaps > 8. Ensures structural admissibility of the gap tensor.

### G2 — Resonance Certificate (Energetic)
Rejects any node whose multiplicity × spectral_weight < 1.0. Ensures nodes carry enough weight for contraction.

### G3 — Recursive Stabilization (Global)
Evaluates G1 and G2 at every base case of the recursive solver. Returns true only when the entire prefix passes both gates.

### G4 — GOV_HASH Sealing (Cryptographic)
Hashes the tensor's canonical form and binds it to an append-only ledger. Makes the tensor state immutable.

## Files

```
sigma-kernel/
├── src/
│   ├── lib.rs                 # Rust implementation: MultiplicityArena, gates, tests
│   ├── main.rs                # Demo: four admissibility scenarios
│   ├── kani_proofs.rs         # Kani BMC harnesses: verify invariants S1–S9
│   └── SedonaSpineExample.hs  # Haskell/LiquidHaskell: type-level DAG + borrow checker
├── formal/
│   ├── sigma_kernel.als       # Alloy: basic invariant model
│   ├── sigma_kernel_alloy.als # Alloy: level-synchronous barriers + topology
│   └── sigma_kernel_zk.circom # Circom: ZK counter circuit
├── Cargo.toml
└── README.md
```

## The Three-Stage Verification Pipeline

### Stage 1 — Alloy: Counterexample Discovery

```bash
alloy sigma_kernel_alloy.als
```

Alloy searches for counterexamples to invariants S1–S9:
- **S1**: Malformed guard (empty arena)
- **S2**: Gap soundness (if G1 admits, gap ≤ 8)
- **S3**: Resonance soundness (if G2 admits, resonance ≥ 1.0)
- **S4**: Composite gate (sigma = admit iff both G1 and G2 admit)
- **S5**: Seal monotonicity (sealed state is permanent)
- **S6**: Solver soundness (if solver returns true, gates admit)
- **S7**: Backtracking restores state (multiplicity reset to 0)
- **S8**: Termination (recursive solver halts)
- **S9**: GOV_HASH sealing (cryptographic immutability)

If Alloy finds a counterexample, that witness becomes the test case for Stage 2.

### Stage 2 — Kani: Bit-Precise Bounded Model Checking

```bash
cargo kani
```

Kani checks the actual Rust implementation against each invariant using bit-precise reasoning:

- **verify_s1_malformed_guard_empty**: Empty arena returns Malformed
- **verify_s2_gap_soundness**: Gap ≤ 8 when gate admits
- **verify_s3_resonance_soundness**: Resonance ≥ 1.0 when gate admits
- **verify_s4_composite_gate**: Sigma gate correctly combines G1 and G2
- **verify_s5_seal_monotonicity**: Seal is permanent
- **verify_s6_solver_soundness**: Solver returns true only when gates admit
- **verify_s7_backtracking_restores**: Backtrack resets multiplicity to 0
- **verify_s8_termination**: Recursive solver terminates
- **verify_s9_gov_hash_sealing**: Seal operation succeeds on admissible arenas

Kani produces concrete failure traces via `--concrete-playback`, which become regression tests.

### Stage 3 — LiquidHaskell / GHC: Type-Level Hardening

```bash
ghc SedonaSpineExample.hs
```

Converts Alloy/Kani discoveries into permanent compiler-enforced invariants:

- **Admissible xs**: Type-level constraint ensuring all consecutive prime gaps ≤ 8
- **Resonant xs**: Type-level constraint ensuring all non-Nil levels participate
- **DeltaAdmissible Δ xs**: Tolerance parameter Δ for fine-grained gap control
- **SigmaAdmissible xs**: Composite constraint (Admissible + Resonant)

The GHC type checker discharges these constraints at every spine construction site. Invalid spines are rejected at compile time, not runtime.

## The Prime-Indexed Sedona Spine (Part VIII)

### Type-Level DAG

The Spine is a heterogeneous list of prime-indexed levels:

```haskell
type ValidSpine = '[ 'P2, 'P3, 'P5, 'P7, 'P11, 'P13 ]

validSpine :: Spine ValidSpine
validSpine = SCons SP2 $ SCons SP3 $ SCons SP5 $ SCons SP7 $ SCons SP11 $ SCons SP13 $ SEnd
```

Each consecutive pair `(p_i, p_{i+1})` must satisfy `GapBounded p_i p_{i+1}`, which the compiler verifies automatically.

### Borrow Checking

The spine is managed via phantom borrow states:

```haskell
borrow :: OwnedSpine xs -> BorrowedSpine xs        -- consume owned, return borrowed
release :: BorrowedSpine xs -> OwnedSpine xs       -- consume borrowed, return owned
splitAt :: BorrowedSpine xs -> (BorrowedSpine ys, BorrowedSpine zs)  -- prove xs ~ (ys ++ zs)
```

The type checker prevents:
- Use-after-free
- Double-free
- Out-of-bounds access
- Dangling borrows

### Homological Layer

The spine carries homological structure:

```haskell
data ProjectiveResolution (r :: Type) (a :: Type) (n :: Nat) where
  ProjRes ::
    { prComplex :: Spine xs                  -- the chain complex
    , prProjective :: AllProjective xs       -- all levels are projective
    , prQuasiIso :: QuasiIso xs a           -- H_0 of complex is a
    , prExact :: ExactAtEveryLevel xs       -- exactness everywhere
    } -> ProjectiveResolution r a (Depth xs)
```

This mirrors the Lean formalization and ensures the spine is a valid complex.

## Constant Configuration

All admissibility thresholds are constants defined in `src/lib.rs`:

```rust
pub const SIGMA_GAP_MAX: u32 = 8;              // Phase Mirror threshold
pub const SIGMA_RESONANCE_MIN: f32 = 1.0;      // Resonance threshold
pub const CANDIDATE_PRIMES: [u32; 6] = [2, 3, 5, 7, 11, 13];
```

These values are reflected into the type system via Alloy and LiquidHaskell.

## Zero-Knowledge Counter (Circom)

The `sigma_kernel_zk.circom` circuit implements a ZK proof of spine validity:

**Public Inputs:**
- `labels[5]`: Prime labels (0–6)
- `levels[5]`: Execution levels (0–4)
- `edges[5][5]`: Adjacency matrix

**Constraints:**
1. **LevelSynchronousBarriers**: If `edges[i][j] == 1`, then `levels[i] < levels[j]`
2. **NoBadPairs**: No edge connects bad pairs `(P2→P11, P11→P2, P2→P13, P13→P2, P3→P13, P13→P3)`

**Output:**
- `out == 1` if any bad pair exists (invalid spine)
- `out == 0` if spine is admissible

## Quick Start

### Build the Rust implementation

```bash
cd sigma-kernel
cargo build
```

### Run the demo

```bash
cargo run --bin sigma_kernel
```

Output:
```
Sigma Kernel — Admissibility Gate
═════════════════════════════════════════════════

Configuration:
  SIGMA_GAP_MAX:        8
  SIGMA_RESONANCE_MIN:  1.0
  CANDIDATE_PRIMES:     [2, 3, 5, 7, 11, 13]

Demo 1: Admissible spine [2, 3, 5, 7]
  G1 (Phase Mirror):  Admit
  G2 (Resonance):     Admit
  G3 (Sigma Gate):    Admit

Demo 2: Dissonant spine [2, 13] (gap = 11 > 8)
  G1 (Phase Mirror):  DissonanceGap { index: 0, gap: 11 }
  G2 (Resonance):     Admit
  G3 (Sigma Gate):    DissonanceGap { index: 0, gap: 11 }

Demo 3: Low-resonance spine [2] with weak spectral weight
  G1 (Phase Mirror):  Admit
  G2 (Resonance):     DissonanceResonance { index: 0, resonance_bits: 0 }
  G3 (Sigma Gate):    DissonanceResonance { index: 0, resonance_bits: 0 }

Demo 4: Seal a valid spine
  Before seal:
    Sealed:  false
    Sigma:   Admit
  After seal:
    Result:  Admit
    Sealed:  true

═════════════════════════════════════════════════
All gates operational. Sigma Kernel active.
```

### Run Kani proofs

```bash
cargo kani
```

Kani runs all proof harnesses and reports verification results.

### Compile Haskell spine

```bash
ghc -O2 SedonaSpineExample.hs -o sedona
./sedona
```

The GHC type checker verifies all admissibility constraints. If any constraint is unsatisfiable, compilation fails.

### Run Alloy model checker

```bash
java -jar alloy.jar
# Load sigma_kernel_alloy.als
# Run the verification commands
```

## The S7 Bug Fix

The original implementation had a critical bug in backtracking:

```rust
// BROKEN: multiplicity leaks
(*self.nodes.add(current_idx)).prime_val = 0;
```

When a branch failed, `prime_val` was reset to Nil but `multiplicity` was not. On the next forward pass at the same depth, multiplicity accumulated across failed branches, corrupting the state.

**The fix:**

```rust
// FIXED: both fields reset
(*self.nodes.add(current_idx)).prime_val = 0;
(*self.nodes.add(current_idx)).multiplicity = 0;
```

This ensures backtracking restores the exact pre-branch state (Invariant S7).

## References

- **Part VI.b**: Sigma Kernel Admissibility Gate (Rust + Invariant Extraction)
- **Part VII**: Truncation and Resonance (SMT + LiquidHaskell)
- **Part VIII**: Prime-Indexed Sedona Spine (Haskell + Borrow Checking + Homology)

## Stack Integration

The Sigma Kernel sits at the top of the Sedona Stack:

```
I.      AVX-512 kernels (compute candidate primes)
II.     ColorForth (tag words as green if prime structure passes G1)
III.    GA144 mesh (node routing respects gap constraints)
IV.     q/kdb+ ledger (store sealed tensors keyed by GOV_HASH)
V.      Thermodynamic sink (rejected tensors → heat)
V.b     Truncation (dissonance = physical amalgamation)
VI.b    Sigma Kernel (admissibility gate: LIVE HERE)
VIII.   Prime-Indexed Sedona Spine (type-level DAG + borrow checker)
```

The Sigma Kernel bridges abstract MT theory and operational reality: every tensor state that enters the ledger is certified admissible by four independent gates, each enforced at a different layer (structural, energetic, algorithmic, cryptographic).
