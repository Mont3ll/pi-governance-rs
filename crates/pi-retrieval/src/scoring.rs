use pi_governance_core::{
    RankedRecord, Record, RecordClass, RecordStatus, ScopeLevel, ScoreBreakdown,
};
use std::cmp::Ordering;
use std::collections::HashSet;

pub fn tokenize(input: &str) -> Vec<String> {
    let mut terms: Vec<String> = input
        .split(|c: char| !c.is_alphanumeric())
        .filter_map(|term| {
            let term = term.trim().to_lowercase();
            if term.len() >= 3 {
                Some(term)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    terms.sort();
    terms
}

pub fn eligible(
    record: &Record,
    project: Option<&str>,
    classes: &[RecordClass],
    include_global: bool,
    include_contested: bool,
    min_confidence: Option<f32>,
) -> bool {
    match record.status {
        RecordStatus::Active => {}
        RecordStatus::Contested if include_contested => {}
        RecordStatus::Contested | RecordStatus::Superseded | RecordStatus::Tombstoned => {
            return false
        }
    }
    if let Some(min) = min_confidence {
        if record.confidence < min {
            return false;
        }
    }
    if !classes.is_empty() && !classes.contains(&record.class) {
        return false;
    }
    match project {
        Some(project_key) => match record.scope.level {
            ScopeLevel::Project => record.scope.key.as_deref() == Some(project_key),
            ScopeLevel::Global => include_global,
            ScopeLevel::Domain | ScopeLevel::Session => false,
        },
        None => true,
    }
}

fn class_priority(class: &RecordClass) -> f32 {
    match class {
        RecordClass::IdentityRule => 1.0,
        RecordClass::Requirement => 0.9,
        RecordClass::Correction => 0.85,
        RecordClass::Workflow => 0.8,
        RecordClass::ProjectState => 0.7,
        RecordClass::Preference => 0.6,
        RecordClass::Observation => 0.45,
        RecordClass::EvidenceNote => 0.4,
    }
}

pub fn rank_record(record: &Record, query_terms: &[String], project: Option<&str>) -> RankedRecord {
    let claim = record.claim.to_lowercase();
    let tags_lower: Vec<String> = record.tags.iter().map(|t| t.to_lowercase()).collect();
    let scope_key = record.scope.key.clone().unwrap_or_default().to_lowercase();
    let class_text = record.class.as_str().to_string();
    let haystack = format!("{claim} {} {class_text} {scope_key}", tags_lower.join(" "));

    let mut matched_terms = Vec::new();
    let mut tag_hits = 0usize;
    for term in query_terms {
        if haystack.contains(term) {
            matched_terms.push(term.clone());
        }
        if tags_lower
            .iter()
            .any(|tag| tag == term || tag.contains(term))
        {
            tag_hits += 1;
        }
    }
    matched_terms.sort();
    matched_terms.dedup();

    let query_match = if query_terms.is_empty() {
        0.0
    } else {
        matched_terms.len() as f32 / query_terms.len() as f32
    };
    let project_scope = match project {
        Some(project_key)
            if record.scope.level == ScopeLevel::Project
                && record.scope.key.as_deref() == Some(project_key) =>
        {
            1.0
        }
        Some(_) if record.scope.level == ScopeLevel::Global => 0.65,
        None if record.scope.level == ScopeLevel::Global => 0.8,
        None => 0.7,
        _ => 0.0,
    };
    let tag_match = if query_terms.is_empty() {
        0.0
    } else {
        tag_hits as f32 / query_terms.len() as f32
    }
    .min(1.0);
    let confidence = record.confidence.clamp(0.0, 1.0);
    let evidence = (record.evidence.len() as f32 / 3.0).min(1.0);
    let recency = 1.0;
    let status_penalty = if record.status == RecordStatus::Contested {
        0.2
    } else {
        0.0
    };
    let breakdown = ScoreBreakdown {
        query_match,
        project_scope,
        class_priority: class_priority(&record.class),
        tag_match,
        confidence,
        evidence,
        recency,
        status_penalty,
    };
    let score = breakdown.query_match * 0.35
        + breakdown.project_scope * 0.20
        + breakdown.class_priority * 0.15
        + breakdown.tag_match * 0.10
        + breakdown.confidence * 0.10
        + breakdown.evidence * 0.05
        + breakdown.recency * 0.05
        - breakdown.status_penalty;
    let mut explanation = vec![
        format!("query_match={:.3}", breakdown.query_match),
        format!("project_scope={:.3}", breakdown.project_scope),
        format!("class_priority={:.3}", breakdown.class_priority),
        format!("tag_match={:.3}", breakdown.tag_match),
        format!("confidence={:.3}", breakdown.confidence),
        format!("evidence={:.3}", breakdown.evidence),
        format!("recency={:.3}", breakdown.recency),
    ];
    if status_penalty > 0.0 {
        explanation.push(format!("status_penalty={:.3}", status_penalty));
    }
    let mut matched_fields = Vec::new();
    if matched_terms.iter().any(|term| claim.contains(term)) {
        matched_fields.push("claim".to_string());
    }
    if tag_hits > 0 {
        matched_fields.push("tags".to_string());
    }
    if matched_terms.iter().any(|term| scope_key.contains(term)) {
        matched_fields.push("project".to_string());
    }
    if matched_terms.iter().any(|term| class_text.contains(term)) {
        matched_fields.push("class".to_string());
    }
    RankedRecord {
        record: record.clone(),
        score,
        deterministic_score: score,
        lexical_score: score,
        hybrid_score: score,
        matched_fields,
        breakdown,
        matched_terms,
        explanation,
    }
}

pub fn rank_record_with_mode(
    record: &Record,
    query: &str,
    query_terms: &[String],
    project: Option<&str>,
    mode: &str,
) -> RankedRecord {
    let mut ranked = rank_record(record, query_terms, project);
    let claim = record.claim.to_lowercase();
    let tags = record
        .tags
        .iter()
        .map(|t| t.to_lowercase())
        .collect::<Vec<_>>();
    let project_text = record.scope.key.clone().unwrap_or_default().to_lowercase();
    let class_text = record.class.as_str().to_lowercase();
    let evidence_text = record
        .evidence
        .iter()
        .map(|e| e.uri.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let query_l = query.to_lowercase();
    let mut raw = 0.0f32;
    let mut matched_fields = ranked.matched_fields.clone();
    let mut matched_terms = ranked.matched_terms.clone();
    for term in query_terms {
        if claim.contains(term) {
            raw += 4.0;
            matched_fields.push("claim".to_string());
            matched_terms.push(term.clone());
        }
        if tags.iter().any(|t| t.contains(term)) {
            raw += 3.0;
            matched_fields.push("tags".to_string());
            matched_terms.push(term.clone());
        }
        if project_text.contains(term) {
            raw += 2.0;
            matched_fields.push("project".to_string());
            matched_terms.push(term.clone());
        }
        if class_text.contains(term) {
            raw += 1.5;
            matched_fields.push("class".to_string());
            matched_terms.push(term.clone());
        }
        if evidence_text.contains(term) {
            raw += 1.0;
            matched_fields.push("evidence".to_string());
            matched_terms.push(term.clone());
        }
    }
    if !query_l.is_empty() && claim.contains(&query_l) {
        raw += 4.0;
        matched_fields.push("claim".to_string());
    }
    raw *= record.confidence.clamp(0.1, 1.0);
    raw += (record.evidence.len() as f32 * 0.05).min(0.2);
    let max_raw = (query_terms.len().max(1) as f32) * 11.5 + 4.2;
    let lexical = (raw / max_raw).clamp(0.0, 1.0);
    let deterministic = ranked.score.clamp(0.0, 1.0);
    let hybrid = (deterministic * 0.55 + lexical * 0.45).clamp(0.0, 1.0);
    matched_fields.sort();
    matched_fields.dedup();
    matched_terms.sort();
    matched_terms.dedup();
    ranked.deterministic_score = deterministic;
    ranked.lexical_score = lexical;
    ranked.hybrid_score = hybrid;
    ranked.matched_fields = matched_fields;
    ranked.matched_terms = matched_terms;
    ranked.score = match mode {
        "lexical" => lexical,
        "hybrid" => hybrid,
        _ => deterministic,
    };
    ranked.explanation.push(format!("retriever={mode}"));
    ranked
        .explanation
        .push(format!("deterministic_score={deterministic:.3}"));
    ranked
        .explanation
        .push(format!("lexical_score={lexical:.3}"));
    ranked.explanation.push(format!("hybrid_score={hybrid:.3}"));
    ranked
}

pub fn sort_ranked(records: &mut [RankedRecord]) {
    records.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| b.record.updated_at.cmp(&a.record.updated_at))
            .then_with(|| a.record.id.cmp(&b.record.id))
    });
}
