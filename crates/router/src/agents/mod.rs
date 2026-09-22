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

//! Sovereign agent surface: shared decision types plus the tensor and loc agents.

pub mod tensor;

pub use tensor::{HeadSignal, TensorAgent, TensorCore, TensorSignal};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Structured verdict produced by any SnapKitty sovereign agent.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AgentDecision {
    pub approved: bool,
    pub confidence: f64,
    pub duration_ms: u64,
    pub action: String,
    pub reasoning: String,
    pub chain_of_thought: Vec<String>,
    pub seal: Option<String>,
}

impl AgentDecision {
    pub fn reply(action: impl Into<String>, reasoning: impl Into<String>, confidence: f64) -> Self {
        Self {
            approved: true,
            confidence,
            duration_ms: 0,
            action: action.into(),
            reasoning: reasoning.into(),
            chain_of_thought: Vec::new(),
            seal: None,
        }
    }
}

/// Canonical request envelope consumed by [`SovereignAgent::process`].
#[derive(Debug, Clone)]
pub struct AgentRequest {
    pub agent: String,
    pub message: String,
    pub amount: Option<f64>,
    pub payload: Value,
}

impl AgentRequest {
    pub fn chat(agent: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            agent: agent.into(),
            message: message.into(),
            amount: None,
            payload: Value::Object(serde_json::Map::new()),
        }
    }

    pub fn score(amount: f64, payload: Value) -> Self {
        Self {
            agent: "tensor".to_string(),
            message: String::new(),
            amount: Some(amount),
            payload,
        }
    }
}

pub trait SovereignAgent: Send + Sync {
    fn name(&self) -> &'static str;
    fn process(&self, req: &AgentRequest) -> AgentDecision;
}
