use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use pi_core::{Durability, EvidenceKind, EvidenceRef, MemoryKind, MemoryLayer, RecordClass, RecordStatus, RetrievalFormat, RetrievalOptions, RuleType, Scope, SourceKind, StoreEvent, TrustClass};
use pi_retrieval::retrieve_with_options;
use pi_store::JsonlStore;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryWorthDecision { Reject, DailyOnly, Candidate, Inquiry }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryWorthReport {
    pub decision: MemoryWorthDecision,
    pub confidence: f32,
    pub suggested_layer: MemoryLayer,
    pub suggested_class: RecordClass,
    pub suggested_memory_kind: MemoryKind,
    pub suggested_rule_type: Option<RuleType>,
    pub suggested_tags: Vec<String>,
    pub durability: Durability,
    pub trust_class: TrustClass,
    pub source_kind: SourceKind,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck { pub name: String, pub status: String, pub message: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verified: bool,
    pub decision: String,
    pub requires_review: bool,
    pub checks: Vec<VerificationCheck>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureCandidate {
    pub claim: String,
    pub decision: MemoryWorthDecision,
    pub patch_id: Option<String>,
    pub suggested_layer: MemoryLayer,
    pub trust_class: TrustClass,
    pub durability: Durability,
    pub memory_kind: MemoryKind,
    pub rule_type: Option<RuleType>,
    pub verification: VerificationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureReport {
    pub input_summary: String,
    pub candidates: Vec<CaptureCandidate>,
    pub daily_only: Vec<String>,
    pub inquiries: Vec<String>,
    pub rejected: Vec<String>,
    pub applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDecision { pub text: String, pub project: Option<String>, pub namespace: String, pub created_at: DateTime<Utc> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallIncluded {
    pub record_id: String,
    pub layer: MemoryLayer,
    pub score: f32,
    pub matched_terms: Vec<String>,
    pub matched_fields: Vec<String>,
    pub trust_class: TrustClass,
    pub memory_kind: Option<MemoryKind>,
    pub evidence_state: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallExcluded { pub record_id: String, pub reason: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallBudget { pub requested: usize, pub used: usize, pub omitted_count: usize }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallXrayReport {
    pub query: String,
    pub namespace: String,
    pub project: Option<String>,
    pub included: Vec<RecallIncluded>,
    pub excluded: Vec<RecallExcluded>,
    pub budget: RecallBudget,
    pub warnings: Vec<String>,
}

fn lower(input: &str) -> String { input.to_lowercase() }

pub fn is_secret_like(text: &str) -> bool {
    let l = lower(text);
    ["api_key", "apikey", "secret=", "password", "private key", "begin rsa", "begin openssh", "token="].iter().any(|n| l.contains(n))
}

pub fn score_memory_worth(observation: &str, trust_class: Option<TrustClass>, source_kind: Option<SourceKind>) -> MemoryWorthReport {
    let l = lower(observation);
    let mut warnings = Vec::new();
    let mut reasons = Vec::new();
    let mut tags = Vec::new();
    let inferred_trust = trust_class.unwrap_or_else(|| if l.contains("don't") || l.contains("do not") || l.contains("not ") { TrustClass::UserCorrection } else { TrustClass::DirectUserInstruction });
    let inferred_source = source_kind.unwrap_or(SourceKind::ManualCli);

    if is_secret_like(observation) {
        warnings.push("secret-like content rejected".to_string());
        return MemoryWorthReport { decision: MemoryWorthDecision::Reject, confidence: 0.99, suggested_layer: MemoryLayer::L2Playbook, suggested_class: RecordClass::Observation, suggested_memory_kind: MemoryKind::Fact, suggested_rule_type: None, suggested_tags: tags, durability: Durability::Unknown, trust_class: inferred_trust, source_kind: inferred_source, reasons: vec!["secret-like content".to_string()], warnings };
    }
    if ["thanks", "ok", "hello", "nice"].contains(&l.trim()) || l.trim().len() < 8 {
        return MemoryWorthReport { decision: MemoryWorthDecision::Reject, confidence: 0.9, suggested_layer: MemoryLayer::L3Session, suggested_class: RecordClass::Observation, suggested_memory_kind: MemoryKind::Event, suggested_rule_type: None, suggested_tags: tags, durability: Durability::Temporary, trust_class: inferred_trust, source_kind: inferred_source, reasons: vec!["trivial or non-durable text".to_string()], warnings };
    }

    let correction = l.contains("don't") || l.contains("do not") || l.starts_with("not ") || l.contains(" instead") || l.contains("not ") && l.contains(",");
    let durable = ["always", "from now on", "going forward", "in future", "prefer", "never", "remember that", "the convention is", "we decided", "#decision", "use "].iter().any(|p| l.contains(p));
    let temporary = l.contains("today") || l.contains("temporary") || l.contains("for this task") || l.contains("right now");
    if l.contains("release") { tags.push("release".to_string()); }
    if l.contains("test") { tags.push("testing".to_string()); }
    if l.contains("retrieval") { tags.push("retrieval".to_string()); }
    if l.contains("note") || l.contains("sed") { tags.push("tooling".to_string()); }

    if temporary {
        reasons.push("temporary task-like content".to_string());
        return MemoryWorthReport { decision: MemoryWorthDecision::DailyOnly, confidence: 0.7, suggested_layer: MemoryLayer::L3Session, suggested_class: RecordClass::Observation, suggested_memory_kind: MemoryKind::Event, suggested_rule_type: None, suggested_tags: tags, durability: Durability::Task, trust_class: inferred_trust, source_kind: inferred_source, reasons, warnings };
    }

    if durable || correction {
        reasons.push(if correction { "durable correction" } else { "durable workflow instruction" }.to_string());
        let rule = if correction { Some(RuleType::Correction) } else if l.contains("test") || l.contains("release") { Some(RuleType::Workflow) } else if l.contains("prefer") { Some(RuleType::Preference) } else { Some(RuleType::Convention) };
        return MemoryWorthReport { decision: MemoryWorthDecision::Candidate, confidence: 0.82, suggested_layer: MemoryLayer::L2Playbook, suggested_class: if correction { RecordClass::Correction } else { RecordClass::Workflow }, suggested_memory_kind: MemoryKind::Instruction, suggested_rule_type: rule, suggested_tags: tags, durability: Durability::Project, trust_class: inferred_trust, source_kind: inferred_source, reasons, warnings };
    }

    MemoryWorthReport { decision: MemoryWorthDecision::Inquiry, confidence: 0.45, suggested_layer: MemoryLayer::L2Playbook, suggested_class: RecordClass::Observation, suggested_memory_kind: MemoryKind::Fact, suggested_rule_type: None, suggested_tags: tags, durability: Durability::Unknown, trust_class: inferred_trust, source_kind: inferred_source, reasons: vec!["ambiguous durable intent".to_string()], warnings }
}

pub fn claim_from_capture(text: &str) -> String {
    let trimmed = text.trim().trim_matches('"');
    let l = lower(trimmed);
    if l.contains("don't use") && l.contains("use ") && l.contains(" instead") {
        if let Some((_, after_dont)) = trimmed.split_once("don't use") {
            if let Some((bad, rest)) = after_dont.split_once(",") {
                let good = rest.replace("use", "").replace("instead", "").trim().to_string();
                return format!("Use {} instead of {}.", good, bad.trim());
            }
        }
    }
    trimmed.trim_start_matches("#decision").trim_start_matches("decision:").trim().trim_end_matches('.').to_string() + "."
}

pub fn verify_candidate(claim: &str, layer: MemoryLayer, trust_class: TrustClass, durability: Durability) -> VerificationResult {
    let mut checks = Vec::new();
    let mut warnings = Vec::new();
    let mut requires_review = false;
    let mut rejected = false;
    if claim.trim().is_empty() { checks.push(VerificationCheck { name: "non_empty_claim".to_string(), status: "fail".to_string(), message: Some("claim is empty".to_string()) }); rejected = true; }
    else { checks.push(VerificationCheck { name: "non_empty_claim".to_string(), status: "pass".to_string(), message: None }); }
    if is_secret_like(claim) { checks.push(VerificationCheck { name: "not_secret_like".to_string(), status: "fail".to_string(), message: Some("secret-like content".to_string()) }); rejected = true; }
    else { checks.push(VerificationCheck { name: "not_secret_like".to_string(), status: "pass".to_string(), message: None }); }
    if layer == MemoryLayer::L1Identity { requires_review = true; checks.push(VerificationCheck { name: "l1_manual_review".to_string(), status: "warn".to_string(), message: Some("L1 requires manual review".to_string()) }); }
    match trust_class {
        TrustClass::RepositoryText | TrustClass::GeneratedContent | TrustClass::ThirdPartyDocumentation | TrustClass::CodebaseAnalysis | TrustClass::AgentInference | TrustClass::Unknown => { requires_review = true; warnings.push("low-trust source requires manual review".to_string()); checks.push(VerificationCheck { name: "low_trust_source".to_string(), status: "warn".to_string(), message: Some("source cannot auto-apply".to_string()) }); }
        _ => checks.push(VerificationCheck { name: "trust_class_present".to_string(), status: "pass".to_string(), message: None }),
    }
    if matches!(durability, Durability::Temporary | Durability::Task) && layer != MemoryLayer::L3Session { requires_review = true; warnings.push("temporary/task durability should stay L3".to_string()); }
    VerificationResult { verified: !rejected && !requires_review, decision: if rejected { "reject" } else if requires_review { "manual_review" } else { "allow" }.to_string(), requires_review, checks, warnings }
}

pub fn session_event(namespace: &str, project: Option<&str>, text: &str, source_kind: SourceKind) -> StoreEvent {
    let payload = json!({"kind":"session_entry","project":project,"text":text,"source_kind":source_kind,"decisions":extract_decisions(text)});
    let mut event = StoreEvent::info(payload.to_string(), None);
    event.namespace = namespace.to_string();
    event
}

pub fn extract_decisions(text: &str) -> Vec<String> {
    let mut decisions = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        let lower = lower(l);
        if lower.starts_with("#decision") { decisions.push(l.trim_start_matches("#decision").trim().to_string()); }
        else if lower.starts_with("decision:") { decisions.push(l.split_once(':').map(|(_, r)| r.trim()).unwrap_or(l).to_string()); }
        else if let Some((_, rest)) = lower.split_once("we decided") { decisions.push(rest.trim().to_string()); }
    }
    if decisions.is_empty() && lower(text).contains("#decision") {
        if let Some((_, rest)) = text.split_once("#decision") { decisions.push(rest.trim().to_string()); }
    }
    decisions
}

pub fn search_session_events(store: &JsonlStore, namespace: &str, query: &str, project: Option<&str>, after: Option<DateTime<Utc>>) -> Result<Vec<StoreEvent>> {
    let q = lower(query);
    Ok(store.load_events()?.into_iter().filter(|e| {
        e.namespace == namespace && e.message.contains("session_entry") && lower(&e.message).contains(&q) && after.map(|a| e.created_at >= a).unwrap_or(true) && project.map(|p| e.message.contains(&format!("\"project\":\"{p}\""))).unwrap_or(true)
    }).collect())
}

pub fn session_decisions(store: &JsonlStore, namespace: &str, project: Option<&str>, days: Option<i64>) -> Result<Vec<SessionDecision>> {
    let after = days.map(|d| Utc::now() - Duration::days(d));
    let mut out = Vec::new();
    for e in store.load_events()? {
        if e.namespace != namespace || !e.message.contains("session_entry") || after.map(|a| e.created_at < a).unwrap_or(false) { continue; }
        let v: serde_json::Value = serde_json::from_str(&e.message).unwrap_or_default();
        let event_project = v.get("project").and_then(|p| p.as_str()).map(str::to_string);
        if project.is_some() && event_project.as_deref() != project { continue; }
        if let Some(arr) = v.get("decisions").and_then(|d| d.as_array()) {
            for d in arr { if let Some(text) = d.as_str() { out.push(SessionDecision { text: text.to_string(), project: event_project.clone(), namespace: e.namespace.clone(), created_at: e.created_at }); } }
        }
    }
    Ok(out)
}

pub fn build_context(store: &JsonlStore, namespace: &str, query: &str, project: Option<String>, budget: usize, include_l3: bool, include_contested: bool) -> Result<(String, serde_json::Value)> {
    let opts = RetrievalOptions { query: query.to_string(), retriever: "hybrid".to_string(), namespace: namespace.to_string(), project: project.clone(), budget, format: RetrievalFormat::Json, explain: true, classes: Vec::new(), include_global: true, include_contested, min_confidence: None };
    let bundle = retrieve_with_options(&store.load_records()?, opts);
    let mut l1 = Vec::new(); let mut l2 = Vec::new(); let mut l3 = Vec::new(); let mut contested = Vec::new();
    for ranked in &bundle.records {
        let r = &ranked.record;
        if r.status == RecordStatus::Contested { contested.push(r.claim.clone()); if !include_contested { continue; } }
        match r.layer { MemoryLayer::L1Identity => l1.push(r.claim.clone()), MemoryLayer::L2Playbook => l2.push(r.claim.clone()), MemoryLayer::L3Session => if include_l3 { l3.push(r.claim.clone()) }, }
    }
    let mut md = String::from("## Governed Memory Context\n\n### L1 — Standing Rules\n");
    md.push_str(&format_items(&l1));
    md.push_str("\n### L2 — Relevant Playbooks\n"); md.push_str(&format_items(&l2));
    md.push_str("\n### L3 — Recent Session Context\n");
    let l3_text = if include_l3 { format_items(&l3) } else { "- excluded by default\n".to_string() };
    md.push_str(&l3_text);
    md.push_str("\n### Contested / Review Before Relying\n"); md.push_str(&format_items(&contested));
    md.push_str("\n### Open Questions\n- none detected\n\n### Retrieval Notes\n"); md.push_str(&format!("- retriever: hybrid\n- namespace: {namespace}\n- used estimated tokens: {}\n", bundle.used_estimated_tokens));
    let json = json!({"query":query,"namespace":namespace,"project":project,"l1":l1,"l2":l2,"l3":l3,"contested":contested,"retrieval_notes":{"retriever":"hybrid","used_estimated_tokens":bundle.used_estimated_tokens,"warnings":bundle.warnings}});
    Ok((md, json))
}

fn format_items(items: &[String]) -> String { if items.is_empty() { "- none\n".to_string() } else { items.iter().map(|i| format!("- {i}\n")).collect() } }

pub fn recall_xray(store: &JsonlStore, namespace: &str, query: &str, project: Option<String>, budget: usize, include_l3: bool, include_contested: bool) -> Result<RecallXrayReport> {
    let records = store.load_records()?;
    let opts = RetrievalOptions { query: query.to_string(), retriever: "hybrid".to_string(), namespace: namespace.to_string(), project: project.clone(), budget, format: RetrievalFormat::Json, explain: true, classes: Vec::new(), include_global: true, include_contested, min_confidence: None };
    let bundle = retrieve_with_options(&records, opts);
    let included_ids: std::collections::HashSet<String> = bundle.records.iter().map(|r| r.record.id.clone()).collect();
    let included = bundle.records.iter().map(|r| RecallIncluded { record_id: r.record.id.clone(), layer: r.record.layer, score: r.score, matched_terms: r.matched_terms.clone(), matched_fields: r.matched_fields.clone(), trust_class: r.record.trust_class, memory_kind: r.record.memory_kind, evidence_state: if r.record.evidence.is_empty() { "missing" } else { "present" }.to_string(), reason: "matched query and filters".to_string() }).collect::<Vec<_>>();
    let excluded = records.into_iter().filter(|r| r.namespace == namespace && !included_ids.contains(&r.id)).map(|r| {
        let reason = if r.status == RecordStatus::Tombstoned { "status tombstoned" } else if r.status == RecordStatus::Superseded { "status superseded" } else if r.status == RecordStatus::Contested && !include_contested { "contested excluded" } else if r.layer == MemoryLayer::L3Session && !include_l3 { "L3 excluded by default" } else if project.as_ref().is_some_and(|p| !r.scope.matches_project_filter(Some(p))) { "project mismatch" } else { "no lexical match or budget omitted" };
        RecallExcluded { record_id: r.id, reason: reason.to_string() }
    }).collect::<Vec<_>>();
    Ok(RecallXrayReport { query: query.to_string(), namespace: namespace.to_string(), project, included, excluded, budget: RecallBudget { requested: budget, used: bundle.used_estimated_tokens, omitted_count: bundle.warnings.len() }, warnings: bundle.warnings })
}

pub fn evidence_for_capture(source_kind: SourceKind, trust_class: TrustClass, durability: Durability) -> EvidenceRef {
    let mut ev = EvidenceRef::new(EvidenceKind::Conversation, format!("capture:{}", Utc::now().timestamp_millis()));
    ev.source_kind = source_kind;
    ev.trust_class = trust_class;
    ev.durability = durability;
    ev
}

pub fn parse_days_after(days: Option<i64>) -> Option<DateTime<Utc>> { days.map(|d| Utc::now() - Duration::days(d)) }

pub fn read_text_input(text: Option<String>, file: Option<std::path::PathBuf>, stdin: bool) -> Result<String> {
    if let Some(t) = text { return Ok(t); }
    if let Some(path) = file { return std::fs::read_to_string(&path).with_context(|| format!("failed to read {path:?}")); }
    if stdin {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }
    anyhow::bail!("provide --text, --file, or --stdin")
}

pub fn scope_for_project(project: Option<String>) -> Scope { project.map(Scope::project).unwrap_or_else(Scope::global) }
