//! Deterministic intent router — zero-LLM keyword-based classification
//! with decision caching, trace ID generation, and WORM-sealed outputs.

use crate::agents::{AgentDecision, TensorAgent, TensorSignal};
use crate::tokenizer::{tokenize, TokenStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Intent classification types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentDomain {
    Finance,
    Crm,
    Procurement,
    Bifrost,
    Treasury,
    Risk,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    pub domain: AgentDomain,
    pub sub_intent: String,
    pub confidence: f64,
}

// ---------------------------------------------------------------------------
// DeterministicRouter
// ---------------------------------------------------------------------------

pub struct DeterministicRouter {
    tensor: TensorAgent,
    decision_cache: Arc<RwLock<HashMap<String, AgentDecision>>>,
}

impl DeterministicRouter {
    pub fn new() -> Self {
        Self {
            tensor: TensorAgent::new(),
            decision_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Score a transaction through the tensor engine and return the signal.
    pub fn tensor_score(&self, amount: f64, payload: &Value) -> TensorSignal {
        self.tensor.evaluate(amount, payload)
    }

    // -- Trace ID -----------------------------------------------------------

    pub fn generate_trace_id() -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut hasher = Sha256::new();
        hasher.update(ts.to_le_bytes());
        let digest = hasher.finalize();
        let hex: String = digest[..4]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        format!("tr-{ts}-{hex}")
    }

    // -- Seal ---------------------------------------------------------------

    pub fn compute_seal(data: &str) -> String {
        let digest = Sha256::digest(data.as_bytes());
        digest.iter().map(|b| format!("{b:02x}")).collect()
    }

    // -- Intent classification ----------------------------------------------

    pub fn classify_intent(domain: &str, message: &str) -> Intent {
        let stream = tokenize(message);
        let domain_enum = Self::parse_domain(domain);
        let sub = Self::match_sub_intent(domain_enum, &stream);
        Intent {
            domain: domain_enum,
            sub_intent: sub.0,
            confidence: sub.1,
        }
    }

    fn parse_domain(raw: &str) -> AgentDomain {
        match raw.to_lowercase().as_str() {
            "finance" | "fin" => AgentDomain::Finance,
            "crm" | "customer" => AgentDomain::Crm,
            "procurement" | "proc" => AgentDomain::Procurement,
            "bifrost" | "events" => AgentDomain::Bifrost,
            "treasury" | "treas" => AgentDomain::Treasury,
            "risk" | "audit" => AgentDomain::Risk,
            _ => AgentDomain::Finance,
        }
    }

    fn match_sub_intent(domain: AgentDomain, stream: &TokenStream) -> (String, f64) {
        match domain {
            AgentDomain::Finance => Self::finance_intent(stream),
            AgentDomain::Crm => Self::crm_intent(stream),
            AgentDomain::Procurement => Self::procurement_intent(stream),
            AgentDomain::Bifrost => Self::bifrost_intent(stream),
            AgentDomain::Treasury => Self::treasury_intent(stream),
            AgentDomain::Risk => Self::risk_intent(stream),
        }
    }

    fn finance_intent(s: &TokenStream) -> (String, f64) {
        if s.has_phrase("cash flow") || s.has("cashflow") {
            ("CashFlow".into(), 0.95)
        } else if s.has_phrase("gl entry") || s.has("ledger") || s.has("journal") {
            ("GlEntry".into(), 0.90)
        } else if s.has_phrase("revenue recognition") || s.has("revrec") {
            ("RevenueRecognition".into(), 0.92)
        } else if s.has_phrase("triple entry") || s.has("triple-entry") {
            ("TripleEntry".into(), 0.93)
        } else if s.has_phrase("ledger integrity") || s.has("reconcil") {
            ("LedgerIntegrity".into(), 0.88)
        } else {
            ("General".into(), 0.50)
        }
    }

    fn crm_intent(s: &TokenStream) -> (String, f64) {
        if s.has("pipeline") || s.has("funnel") {
            ("Pipeline".into(), 0.92)
        } else if s.has_phrase("deal status") || s.has("deal") || s.has("opportunity") {
            ("DealStatus".into(), 0.90)
        } else if s.has_phrase("revenue forecast") || s.has("forecast") {
            ("RevenueForecast".into(), 0.88)
        } else if s.has_phrase("contact score") || s.has("lead") || s.has("contact") {
            ("ContactScore".into(), 0.85)
        } else {
            ("General".into(), 0.50)
        }
    }

    fn procurement_intent(s: &TokenStream) -> (String, f64) {
        if s.has_phrase("vendor trust") || s.has("vendor") || s.has("supplier") {
            ("VendorTrust".into(), 0.93)
        } else if s.has_phrase("spend analysis") || s.has("spend") {
            ("SpendAnalysis".into(), 0.90)
        } else if s.has_phrase("po status") || s.has("po") || s.has("purchase order") {
            ("PoStatus".into(), 0.92)
        } else if s.has_phrase("supply chain") || s.has("supply") {
            ("SupplyChain".into(), 0.88)
        } else {
            ("General".into(), 0.50)
        }
    }

    fn bifrost_intent(s: &TokenStream) -> (String, f64) {
        if s.has_phrase("event pipeline") || s.has("event") {
            ("EventPipeline".into(), 0.92)
        } else if s.has_phrase("trace id") || s.has("traceid") || s.has("trace") {
            ("TraceId".into(), 0.90)
        } else if s.has_phrase("schema validation") || s.has("schema") {
            ("SchemaValidation".into(), 0.88)
        } else if s.has("routing") || s.has("route") {
            ("Routing".into(), 0.85)
        } else {
            ("General".into(), 0.50)
        }
    }

    fn treasury_intent(s: &TokenStream) -> (String, f64) {
        if s.has("reserves") || s.has("reserve") {
            ("Reserves".into(), 0.92)
        } else if s.has_phrase("freeze status") || s.has("frozen") || s.has("freeze") {
            ("FreezeStatus".into(), 0.90)
        } else if s.has_phrase("sovereign balance") || s.has("balance") {
            ("SovereignBalance".into(), 0.88)
        } else {
            ("General".into(), 0.50)
        }
    }

    fn risk_intent(s: &TokenStream) -> (String, f64) {
        if s.has_phrase("threat score") || s.has("threat") {
            ("ThreatScore".into(), 0.92)
        } else if s.has_phrase("audit trail") || s.has("audit") {
            ("AuditTrail".into(), 0.90)
        } else if s.has_phrase("compliance flag") || s.has("compliance") {
            ("ComplianceFlag".into(), 0.88)
        } else if s.has_phrase("anomaly pattern") || s.has("anomaly") {
            ("AnomalyPattern".into(), 0.85)
        } else {
            ("General".into(), 0.50)
        }
    }

    // -- Natural language response templates --------------------------------

    pub fn route_message(domain: &str, message: &str) -> String {
        let intent = Self::classify_intent(domain, message);
        match intent.domain {
            AgentDomain::Finance => Self::finance_response(&intent.sub_intent),
            AgentDomain::Crm => Self::crm_response(&intent.sub_intent),
            AgentDomain::Procurement => Self::procurement_response(&intent.sub_intent),
            AgentDomain::Bifrost => Self::bifrost_response(&intent.sub_intent),
            AgentDomain::Treasury => Self::treasury_response(&intent.sub_intent),
            AgentDomain::Risk => Self::risk_response(&intent.sub_intent),
        }
    }

    fn finance_response(sub: &str) -> String {
        match sub {
            "CashFlow" => "Cash flow analysis runs against the triple-entry GL. Each debit and credit posts a paired WORM-sealed entry.".into(),
            "GlEntry" => "GL entries are validated against the triple-entry ledger. Each posting carries a SHA-256 seal for tamper evidence.".into(),
            "RevenueRecognition" => "Revenue recognition follows ASC 606 guidelines. Performance obligations are tracked per WORM-sealed contract.".into(),
            "TripleEntry" => "Triple-entry accounting posts three entries per transaction: debit, credit, and a sovereign witness. All entries are sealed.".into(),
            "LedgerIntegrity" => "Ledger integrity checks compare running hashes across all three entry sets. Mismatches trigger automatic escalation.".into(),
            _ => "Finance domain: specify cash flow, GL entries, revenue recognition, or ledger integrity.".into(),
        }
    }

    fn crm_response(sub: &str) -> String {
        match sub {
            "Pipeline" => "Pipeline view aggregates all open opportunities by stage. Velocity and conversion rates are computed deterministically.".into(),
            "DealStatus" => "Deal status shows current stage, last activity, and next action. All transitions are audit-logged.".into(),
            "RevenueForecast" => "Revenue forecast projects pipeline weighted by deal probability. Historical close rates calibrate the model.".into(),
            "ContactScore" => "Contact scoring weighs engagement frequency, recency, and deal history. Scores update on every interaction.".into(),
            _ => "CRM domain: specify pipeline, deal status, revenue forecast, or contact scoring.".into(),
        }
    }

    fn procurement_response(sub: &str) -> String {
        match sub {
            "VendorTrust" => "Vendor trust score combines PO history, delivery reliability, and risk signals. Known vendors with POs score highest.".into(),
            "SpendAnalysis" => "Spend analysis breaks down expenditures by vendor, category, and period. Trends are flagged deterministically.".into(),
            "PoStatus" => "PO status tracks creation, approval, fulfillment, and payment stages. Each transition is WORM-sealed.".into(),
            "SupplyChain" => "Supply chain view shows vendor dependencies, lead times, and risk concentrations across your supplier base.".into(),
            _ => "Procurement domain: specify vendor trust, spend analysis, PO status, or supply chain.".into(),
        }
    }

    fn bifrost_response(sub: &str) -> String {
        match sub {
            "EventPipeline" => "Event pipeline processes incoming events through schema validation, routing, and envelope sealing.".into(),
            "TraceId" => "Trace IDs follow the format tr-{timestamp}-{hex4}. Each ID is globally unique and deterministic for the same nanosecond.".into(),
            "SchemaValidation" => "Schema validation enforces structural contracts on all inbound events before routing.".into(),
            "Routing" => "Routing matches event types to agent domains using keyword-based deterministic classification.".into(),
            _ => "Bifrost domain: specify event pipeline, trace ID, schema validation, or routing.".into(),
        }
    }

    fn treasury_response(sub: &str) -> String {
        match sub {
            "Reserves" => "Treasury reserves track sovereign balances across all custody accounts. Reserve ratios are computed in real-time.".into(),
            "FreezeStatus" => "Freeze status shows any locked balances with reason codes and release timestamps. Frozen funds cannot route.".into(),
            "SovereignBalance" => "Sovereign balance aggregates all held funds minus liabilities. Balance seals are posted to the WORM ledger.".into(),
            _ => "Treasury domain: specify reserves, freeze status, or sovereign balance.".into(),
        }
    }

    fn risk_response(sub: &str) -> String {
        match sub {
            "ThreatScore" => "Threat score combines vendor risk, transaction anomalies, and historical patterns into a single risk signal.".into(),
            "AuditTrail" => "Audit trail shows every decision, seal, and trace ID in chronological order. Entries are immutable once written.".into(),
            "ComplianceFlag" => "Compliance flags are raised when transactions exceed thresholds or match known risky patterns.".into(),
            "AnomalyPattern" => "Anomaly detection compares current behavior against historical baselines. Deviations above threshold are flagged.".into(),
            _ => "Risk domain: specify threat score, audit trail, compliance flag, or anomaly pattern.".into(),
        }
    }

    // -- Process event (tensor scoring + sealed decision) --------------------

    pub async fn process_event(
        &self,
        domain: &str,
        event_type: &str,
        amount: f64,
        payload: &Value,
    ) -> AgentDecision {
        let trace_id = Self::generate_trace_id();
        let signal = self.tensor.evaluate(amount, payload);
        let mut decision = self.build_decision(domain, event_type, &signal, &trace_id);
        decision.duration_ms = 0;

        let cache_key = format!("{domain}:{event_type}:{trace_id}");
        self.decision_cache
            .write()
            .unwrap()
            .insert(cache_key, decision.clone());

        decision
    }

    fn build_decision(
        &self,
        domain: &str,
        event_type: &str,
        signal: &TensorSignal,
        trace_id: &str,
    ) -> AgentDecision {
        let approved = signal.recommendation == "PROCEED";
        let reasoning = format!(
            "[{domain}:{event_type}] composite={:.3} budget={} vendor={} risk={} → {}",
            signal.composite_signal,
            signal.budget_head.label,
            signal.vendor_trust_head.label,
            signal.risk_head.label,
            signal.recommendation,
        );

        let seal_input = format!(
            "{domain}:{event_type}:{:.3}:{trace_id}",
            signal.composite_signal,
        );
        let seal = Self::compute_seal(&seal_input);

        AgentDecision {
            approved,
            confidence: signal.composite_signal,
            duration_ms: 0,
            action: signal.recommendation.clone(),
            reasoning,
            chain_of_thought: vec![
                format!("trace_id={trace_id}"),
                format!("domain={domain} event_type={event_type}"),
                format!("budget_head {} {:.3}", signal.budget_head.label, signal.budget_head.score),
                format!("vendor_trust_head {} {:.3}", signal.vendor_trust_head.label, signal.vendor_trust_head.score),
                format!("risk_head {} {:.3}", signal.risk_head.label, signal.risk_head.score),
                format!("composite {:.3} → {}", signal.composite_signal, signal.recommendation),
            ],
            seal: Some(seal),
        }
    }

    // -- Cache access -------------------------------------------------------

    pub fn get_cached_decision(&self, key: &str) -> Option<AgentDecision> {
        self.decision_cache.read().unwrap().get(key).cloned()
    }

    pub fn cache_len(&self) -> usize {
        self.decision_cache.read().unwrap().len()
    }
}

impl Default for DeterministicRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_parsing() {
        let intent = DeterministicRouter::classify_intent("finance", "show me cash flow analysis");
        assert_eq!(intent.domain, AgentDomain::Finance);
        assert_eq!(intent.sub_intent, "CashFlow");
        assert!(intent.confidence >= 0.9);
    }

    #[test]
    fn test_procurement_intent() {
        let intent = DeterministicRouter::classify_intent("procurement", "vendor trust score");
        assert_eq!(intent.domain, AgentDomain::Procurement);
        assert_eq!(intent.sub_intent, "VendorTrust");
    }

    #[test]
    fn test_route_message_returns_template() {
        let resp = DeterministicRouter::route_message("finance", "show me cash flow");
        assert!(resp.contains("triple-entry"));
    }

    #[test]
    fn test_trace_id_format() {
        let id = DeterministicRouter::generate_trace_id();
        assert!(id.starts_with("tr-"));
        // "tr-" (3) + nanos (19-20 digits) + "-" (1) + hex4 (8 chars)
        assert!(id.len() >= 31 && id.len() <= 32);
        let parts: Vec<&str> = id.strip_prefix("tr-").unwrap().split('-').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].len(), 8);
    }

    #[test]
    fn test_seal_is_deterministic() {
        let a = DeterministicRouter::compute_seal("hello");
        let b = DeterministicRouter::compute_seal("hello");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[tokio::test]
    async fn test_process_event_proceed() {
        let router = DeterministicRouter::new();
        let payload = serde_json::json!({
            "vendor_id": "v_001",
            "po_id": "po_002",
            "risk_score": 0.1
        });
        let decision = router.process_event("procurement", "po.created", 800.0, &payload).await;
        assert!(decision.approved);
        assert_eq!(decision.action, "PROCEED");
        assert!(decision.seal.is_some());
    }

    #[tokio::test]
    async fn test_process_event_escalate() {
        let router = DeterministicRouter::new();
        let payload = serde_json::json!({ "risk_score": 0.95 });
        let decision = router.process_event("procurement", "po.created", 75_000.0, &payload).await;
        assert!(!decision.approved);
        assert_eq!(decision.action, "ESCALATE");
    }

    #[tokio::test]
    async fn test_decision_caching() {
        let router = DeterministicRouter::new();
        let payload = serde_json::json!({ "vendor_id": "v_001", "po_id": "po_002", "risk_score": 0.1 });
        let d = router.process_event("finance", "gl.entry", 500.0, &payload).await;
        assert_eq!(router.cache_len(), 1);
        let cached = router.get_cached_decision(&format!("finance:gl.entry:{}", d.chain_of_thought[0].replace("trace_id=", "")));
        assert!(cached.is_some());
    }
}
