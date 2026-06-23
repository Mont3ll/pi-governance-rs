use pi_core::{
    ContextBlock, ContextBundle, RankedRecord, Record, RecordClass, RetrievalBudget,
    RetrievalFormat, RetrievalOptions,
};

use crate::packing::pack_ranked;
use crate::scoring::{eligible, rank_record, sort_ranked, tokenize};

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

pub fn retrieve(
    records: &[Record],
    query: impl Into<String>,
    project: Option<String>,
    budget: RetrievalBudget,
) -> ContextBundle {
    retrieve_with_options(records, RetrievalOptions {
        query: query.into(),
        project,
        budget: budget.max_tokens,
        format: RetrievalFormat::Markdown,
        explain: false,
        classes: Vec::new(),
        include_global: true,
        include_contested: false,
        min_confidence: None,
    })
}

pub fn retrieve_with_options(records: &[Record], options: RetrievalOptions) -> ContextBundle {
    let query_terms = tokenize(&options.query);
    let mut ranked: Vec<RankedRecord> = records
        .iter()
        .filter(|record| eligible(
            record,
            options.project.as_deref(),
            &options.classes,
            options.include_global,
            options.include_contested,
            options.min_confidence,
        ))
        .map(|record| rank_record(record, &query_terms, options.project.as_deref()))
        .filter(|ranked| query_terms.is_empty() || !ranked.matched_terms.is_empty() || ranked.score > 0.30)
        .collect();

    sort_ranked(&mut ranked);
    let (packed, used_estimated_tokens, warnings) = pack_ranked(ranked, options.budget);

    let blocks = packed.iter().map(|ranked| ContextBlock {
        record_id: ranked.record.id.clone(),
        block_type: block_type_for(&ranked.record.class).to_string(),
        content: ranked.record.claim.clone(),
        confidence: ranked.record.confidence,
        source_count: ranked.record.evidence.len(),
    }).collect();

    ContextBundle {
        query: options.query,
        project: options.project,
        budget: RetrievalBudget { max_tokens: options.budget },
        used_estimated_tokens,
        explain: options.explain,
        blocks,
        records: packed,
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

    for (idx, block) in bundle.blocks.iter().enumerate() {
        output.push_str(&format!("## {} — {}\n\n", block.block_type, block.record_id));
        output.push_str(&format!("{}\n\n", block.content));
        output.push_str(&format!(
            "- confidence: {:.2}\n- sources: {}\n",
            block.confidence, block.source_count
        ));
        if bundle.explain {
            if let Some(ranked) = bundle.records.get(idx) {
                output.push_str(&format!("- score: {:.3}\n", ranked.score));
                if !ranked.matched_terms.is_empty() {
                    output.push_str(&format!("- matched terms: {}\n", ranked.matched_terms.join(", ")));
                }
                if !ranked.explanation.is_empty() {
                    output.push_str("- explanation:\n");
                    for item in &ranked.explanation {
                        output.push_str(&format!("  - {item}\n"));
                    }
                }
            }
        }
        output.push('\n');
    }

    if !bundle.warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &bundle.warnings {
            output.push_str(&format!("- {warning}\n"));
        }
    }

    output
}
