use pi_core::{
    ContextBlock, ContextBundle, Record, RecordClass, RecordStatus, RetrievalBudget,
};
use std::cmp::Ordering;
use std::collections::HashSet;

fn tokenize(input: &str) -> HashSet<String> {
    input
        .split(|c: char| !c.is_alphanumeric())
        .filter_map(|term| {
            let term = term.trim().to_lowercase();
            if term.len() >= 3 {
                Some(term)
            } else {
                None
            }
        })
        .collect()
}

fn estimate_tokens(input: &str) -> usize {
    let word_count = input.split_whitespace().count();
    ((word_count as f32) * 1.35).ceil() as usize + 24
}

fn block_type_for(class: &RecordClass) -> &'static str {
    match class {
        RecordClass::IdentityRule => "hard_rule",
        RecordClass::Preference => "preference",
        RecordClass::ProjectState => "project_state",
        RecordClass::Requirement => "requirement",
        RecordClass::Correction => "correction",
        RecordClass::Workflow => "workflow",
        RecordClass::Observation => "observation",
        RecordClass::EvidenceNote => "evidence",
    }
}

fn score_record(record: &Record, query_terms: &HashSet<String>) -> f32 {
    let haystack = format!(
        "{} {} {} {:?}",
        record.claim,
        record.tags.join(" "),
        record.class,
        record.scope.key.clone().unwrap_or_default()
    )
    .to_lowercase();

    let mut score = 0.0;

    for term in query_terms {
        if haystack.contains(term) {
            score += 1.0;
        }

        if record.tags.iter().any(|tag| tag.to_lowercase() == *term) {
            score += 1.5;
        }
    }

    score += record.confidence * 0.25;

    match record.class {
        RecordClass::IdentityRule => score += 0.3,
        RecordClass::Requirement => score += 0.2,
        RecordClass::Correction => score += 0.2,
        _ => {}
    }

    score
}

pub fn retrieve(
    records: &[Record],
    query: impl Into<String>,
    project: Option<String>,
    budget: RetrievalBudget,
) -> ContextBundle {
    let query = query.into();
    let query_terms = tokenize(&query);

    let mut scored: Vec<(f32, &Record)> = records
        .iter()
        .filter(|record| record.status == RecordStatus::Active)
        .filter(|record| record.scope.matches_project_filter(project.as_deref()))
        .map(|record| (score_record(record, &query_terms), record))
        .filter(|(score, _)| *score > 0.15)
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));

    let mut blocks = Vec::new();
    let mut used_estimated_tokens = 0usize;
    let mut warnings = Vec::new();

    for (_, record) in scored {
        let token_estimate = estimate_tokens(&record.claim);

        if used_estimated_tokens + token_estimate > budget.max_tokens {
            warnings.push(format!(
                "budget reached; skipped record {} and remaining matches",
                record.id
            ));
            break;
        }

        used_estimated_tokens += token_estimate;

        blocks.push(ContextBlock {
            record_id: record.id.clone(),
            block_type: block_type_for(&record.class).to_string(),
            content: record.claim.clone(),
            confidence: record.confidence,
            source_count: record.evidence.len(),
        });
    }

    ContextBundle {
        query,
        project,
        budget,
        used_estimated_tokens,
        blocks,
        warnings,
    }
}

pub fn render_markdown(bundle: &ContextBundle) -> String {
    let mut output = String::new();

    output.push_str("# PI Context Bundle\n\n");
    output.push_str(&format!("Query: `{}`\n\n", bundle.query));

    if let Some(project) = &bundle.project {
        output.push_str(&format!("Project: `{project}`\n\n"));
    }

    output.push_str(&format!(
        "Budget: {} tokens requested, approximately {} used\n\n",
        bundle.budget.max_tokens, bundle.used_estimated_tokens
    ));

    for block in &bundle.blocks {
        output.push_str(&format!("## {} — {}\n\n", block.block_type, block.record_id));
        output.push_str(&format!("{}\n\n", block.content));
        output.push_str(&format!(
            "- confidence: {:.2}\n- sources: {}\n\n",
            block.confidence, block.source_count
        ));
    }

    if !bundle.warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &bundle.warnings {
            output.push_str(&format!("- {warning}\n"));
        }
    }

    output
}
