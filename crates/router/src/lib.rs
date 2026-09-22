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

pub mod agents;
pub mod router;
pub mod tokenizer;

pub use agents::{AgentDecision, AgentRequest, SovereignAgent, TensorAgent, TensorCore, TensorSignal};
pub use router::{AgentDomain, DeterministicRouter, Intent};
pub use tokenizer::{tokenize, TokenStream};
