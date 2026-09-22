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

//! TensorCore — lightweight emergent signal engine.
//!
//! Works without the `candle` feature using deterministic heuristics.
//! When compiled with `--features candle`, CUDA-backed tensors replace heuristics.

use super::{AgentDecision, AgentRequest, SovereignAgent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::Instant;

/// Emergent signal from a single tensor head
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HeadSignal {
    pub name: String,
    pub score: f64,
    pub label: String,
}

/// Composite emergent tensor output across all heads
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TensorSignal {
    pub budget_head: HeadSignal,
    pub vendor_trust_head: HeadSignal,
    pub risk_head: HeadSignal,
    pub composite_signal: f64,
    pub recommendation: String,
}

/// TensorCore — lightweight emergent signal engine.
pub struct TensorCore;

impl TensorCore {
    pub fn new() -> Self {
        Self
    }

    /// Budget head: score based on amount vs threshold bands
    fn budget_head(amount: f64) -> HeadSignal {
        let score = if amount <= 1_000.0 {
            0.95
        } else if amount <= 10_000.0 {
            0.75
        } else if amount <= 50_000.0 {
            0.50
        } else {
            0.20
        };
        HeadSignal {
            name: "budget_head".to_string(),
            score,
            label: if score > 0.7 {
                "GREEN".to_string()
            } else if score > 0.4 {
                "AMBER".to_string()
            } else {
                "RED".to_string()
            },
        }
    }

    /// Vendor trust head: score based on vendor reputation signals in payload
    fn vendor_trust_head(payload: &Value) -> HeadSignal {
        let vendor_known = payload["vendor_id"].as_str().is_some()
            || payload["vendor"].as_str().is_some();
        let has_po = payload["po_id"].as_str().is_some() || payload["poId"].as_str().is_some();
        let score = match (vendor_known, has_po) {
            (true, true) => 0.92,
            (true, false) => 0.65,
            (false, true) => 0.55,
            (false, false) => 0.35,
        };
        HeadSignal {
            name: "vendor_trust_head".to_string(),
            score,
            label: if score > 0.7 {
                "TRUSTED".to_string()
            } else if score > 0.5 {
                "UNVERIFIED".to_string()
            } else {
                "UNKNOWN".to_string()
            },
        }
    }

    /// Risk head: derived from existing risk_score / score fields
    fn risk_head(payload: &Value) -> HeadSignal {
        let raw_risk = payload["risk_score"]
            .as_f64()
            .or_else(|| payload["score"].as_f64())
            .unwrap_or(0.0);
        let score = 1.0 - raw_risk.clamp(0.0, 1.0);
        HeadSignal {
            name: "risk_head".to_string(),
            score,
            label: if score > 0.7 {
                "LOW_RISK".to_string()
            } else if score > 0.4 {
                "MED_RISK".to_string()
            } else {
                "HIGH_RISK".to_string()
            },
        }
    }

    /// Run all heads and produce composite emergent signal
    pub fn run(&self, amount: f64, payload: &Value) -> TensorSignal {
        let budget = Self::budget_head(amount);
        let vendor = Self::vendor_trust_head(payload);
        let risk = Self::risk_head(payload);

        // Weighted composite: budget 40%, vendor 30%, risk 30%
        let composite = budget.score * 0.40 + vendor.score * 0.30 + risk.score * 0.30;

        let recommendation = if composite >= 0.75 {
            "PROCEED".to_string()
        } else if composite >= 0.50 {
            "REVIEW".to_string()
        } else {
            "ESCALATE".to_string()
        };

        TensorSignal {
            budget_head: budget,
            vendor_trust_head: vendor,
            risk_head: risk,
            composite_signal: (composite * 1000.0).round() / 1000.0,
            recommendation,
        }
    }
}

impl Default for TensorCore {
    fn default() -> Self {
        Self::new()
    }
}

/// Sovereign tensor agent: runs [`TensorCore`] and seals the verdict.
pub struct TensorAgent {
    core: TensorCore,
}

impl TensorAgent {
    pub fn new() -> Self {
        Self {
            core: TensorCore::new(),
        }
    }

    pub fn evaluate(&self, amount: f64, payload: &Value) -> TensorSignal {
        self.core.run(amount, payload)
    }

    pub fn decide(&self, amount: f64, payload: &Value) -> (TensorSignal, AgentDecision) {
        let started = Instant::now();
        let signal = self.core.run(amount, payload);
        let mut decision = decision_from_signal(&signal);
        decision.duration_ms = started.elapsed().as_millis() as u64;
        (signal, decision)
    }
}

impl Default for TensorAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl SovereignAgent for TensorAgent {
    fn name(&self) -> &'static str {
        "tensor"
    }

    fn process(&self, req: &AgentRequest) -> AgentDecision {
        let amount = req.amount.unwrap_or(0.0);
        self.decide(amount, &req.payload).1
    }
}

fn decision_from_signal(signal: &TensorSignal) -> AgentDecision {
    let approved = signal.recommendation == "PROCEED";
    let reasoning = format!(
        "composite={:.3} budget={}({}) vendor={}({}) risk={}({}) → {}",
        signal.composite_signal,
        signal.budget_head.label,
        format_score(signal.budget_head.score),
        signal.vendor_trust_head.label,
        format_score(signal.vendor_trust_head.score),
        signal.risk_head.label,
        format_score(signal.risk_head.score),
        signal.recommendation
    );
    AgentDecision {
        approved,
        confidence: signal.composite_signal,
        duration_ms: 0,
        action: signal.recommendation.clone(),
        reasoning,
        chain_of_thought: vec![
            format!(
                "budget_head {} score {:.3}",
                signal.budget_head.label, signal.budget_head.score
            ),
            format!(
                "vendor_trust_head {} score {:.3}",
                signal.vendor_trust_head.label, signal.vendor_trust_head.score
            ),
            format!(
                "risk_head {} score {:.3}",
                signal.risk_head.label, signal.risk_head.score
            ),
            format!(
                "weighted composite {:.3} (budget 40%, vendor 30%, risk 30%)",
                signal.composite_signal
            ),
        ],
        seal: Some(seal_signal(signal)),
    }
}

fn format_score(score: f64) -> String {
    format!("{:.3}", score)
}

fn seal_signal(signal: &TensorSignal) -> String {
    let canonical = format!(
        "budget={:.3}|vendor={:.3}|risk={:.3}|composite={:.3}|rec={}",
        signal.budget_head.score,
        signal.vendor_trust_head.score,
        signal.risk_head.score,
        signal.composite_signal,
        signal.recommendation
    );
    let digest = Sha256::digest(canonical.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_bands() {
        let core = TensorCore::new();
        let payload = serde_json::json!({});
        let s = core.run(500.0, &payload);
        assert!(s.budget_head.score > 0.9);
        assert_eq!(s.budget_head.label, "GREEN");
    }

    #[test]
    fn test_high_risk_escalate() {
        let core = TensorCore::new();
        let payload = serde_json::json!({ "risk_score": 0.95 });
        let s = core.run(75_000.0, &payload);
        assert_eq!(s.recommendation, "ESCALATE");
    }

    #[test]
    fn test_trusted_vendor_composite() {
        let core = TensorCore::new();
        let payload =
            serde_json::json!({ "vendor_id": "v_001", "po_id": "po_002", "risk_score": 0.1 });
        let s = core.run(800.0, &payload);
        assert!(s.composite_signal >= 0.75);
        assert_eq!(s.recommendation, "PROCEED");
    }

    #[test]
    fn seal_is_deterministic() {
        let agent = TensorAgent::new();
        let payload =
            serde_json::json!({ "vendor_id": "v_001", "po_id": "po_002", "risk_score": 0.1 });
        let (_, a) = agent.decide(800.0, &payload);
        let (_, b) = agent.decide(800.0, &payload);
        assert_eq!(a.seal, b.seal);
        assert_eq!(a.action, "PROCEED");
        assert!(a.approved);
        assert_eq!(a.seal.as_ref().map(|s| s.len()), Some(64));
    }
}
