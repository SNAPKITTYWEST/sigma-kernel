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

// Part VI.b + VIII — Sigma Kernel ZK Counter
// Zero-knowledge circuit for verifying admissibility of spine structures
// Based on the Alloy model and the prime-indexed Sedona Spine

pragma circom 2.0;

// Helper: IsEqual template (checks if a == b)
template IsEqual() {
    signal input a;
    signal input b;
    signal output out;

    signal diff;
    diff <== a - b;

    signal inv;
    inv <== 1 / (1 + diff * diff);

    out <== 1 - diff * diff * inv;
}

// Helper: Detects if (a, b) is a bad pair
// Bad pairs from Alloy: (P2->P11), (P11->P2), (P2->P13), (P13->P2), (P3->P13), (P13->P3)
// Prime encoding: Nil=0, P2=1, P3=2, P5=3, P7=4, P11=5, P13=6
template IsBadPair() {
    signal input a;
    signal input b;
    signal output out;

    signal eq1_5, eq5_1, eq1_6, eq6_1, eq2_6, eq6_2;

    // (1,5): P2 -> P11
    eq1_5 <== IsEqual()({a: a, b: 1});
    eq1_5 <== eq1_5 * IsEqual()({a: b, b: 5});

    // (5,1): P11 -> P2
    eq5_1 <== IsEqual()({a: a, b: 5});
    eq5_1 <== eq5_1 * IsEqual()({a: b, b: 1});

    // (1,6): P2 -> P13
    eq1_6 <== IsEqual()({a: a, b: 1});
    eq1_6 <== eq1_6 * IsEqual()({a: b, b: 6});

    // (6,1): P13 -> P2
    eq6_1 <== IsEqual()({a: a, b: 6});
    eq6_1 <== eq6_1 * IsEqual()({a: b, b: 1});

    // (2,6): P3 -> P13
    eq2_6 <== IsEqual()({a: a, b: 2});
    eq2_6 <== eq2_6 * IsEqual()({a: b, b: 6});

    // (6,2): P13 -> P3
    eq6_2 <== IsEqual()({a: a, b: 6});
    eq6_2 <== eq6_2 * IsEqual()({a: b, b: 2});

    out <== eq1_5 + eq5_1 + eq1_6 + eq6_1 + eq2_6 + eq6_2;
}

// Helper: OR template
template OR(n) {
    signal input in[n];
    signal output out;

    signal prod;
    prod <== 1;
    for (var i = 0; i < n; i++) {
        prod <== prod * (1 - in[i]);
    }
    out <== 1 - prod;
}

// Main circuit: Sigma Kernel Counter
// Verifies admissibility of a spine structure via ZK proof
template SigmaKernelCounter() {
    // Parameters: N=5 nodes, L=5 levels (0..4)

    // Public inputs
    signal input labels[5];        // prime labels: 0-6
    signal input levels[5];        // execution levels: 0-4
    signal input edges[5][5];      // adjacency: boolean matrix

    // Compute level violations for each edge
    signal[5][5] levelViolation;
    for (var i = 0; i < 5; i++) {
        for (var j = 0; j < 5; j++) {
            signal eq, gt1, gt2, gt3, gt4, gt, tmp;

            // Check if levels[i] == levels[j]
            eq <== IsEqual()({a: levels[i], b: levels[j]});

            // Check if levels[i] > levels[j] (levels[i] = levels[j]+k for k in 1..4)
            gt1 <== IsEqual()({a: levels[i], b: levels[j] + 1});
            gt2 <== IsEqual()({a: levels[i], b: levels[j] + 2});
            gt3 <== IsEqual()({a: levels[i], b: levels[j] + 3});
            gt4 <== IsEqual()({a: levels[i], b: levels[j] + 4});

            gt <== gt1 + gt2 + gt3 + gt4;
            tmp <== eq + gt;

            levelViolation[i][j] <== tmp;

            // Constraint: if there is an edge, level must increase
            signal constraint;
            constraint <== edges[i][j] * levelViolation[i][j];
            constraint === 0;
        }
    }

    // Check for bad pairs in edges
    signal[5][5] isBadEdge;
    for (var i = 0; i < 5; i++) {
        for (var j = 0; j < 5; j++) {
            signal badPair;
            badPair <== IsBadPair()({a: labels[i], b: labels[j]});
            isBadEdge[i][j] <== edges[i][j] * badPair;
        }
    }

    // OR over all bad edges
    signal[25] badEdgeFlat;
    var idx = 0;
    for (var i = 0; i < 5; i++) {
        for (var j = 0; j < 5; j++) {
            badEdgeFlat[idx] <== isBadEdge[i][j];
            idx++;
        }
    }

    signal badPairFound;
    badPairFound <== OR(25)({in: badEdgeFlat});

    // Output: 1 if any bad pair exists, 0 otherwise
    signal output out;
    out <== badPairFound;
}

component main = SigmaKernelCounter();
