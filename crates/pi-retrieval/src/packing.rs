use pi_governance_core::RankedRecord;

pub fn estimate_tokens(input: &str) -> usize {
    ((input.chars().count() + 3) / 4).max(1) + 24
}

pub fn pack_ranked(mut ranked: Vec<RankedRecord>, budget: usize) -> (Vec<RankedRecord>, usize, Vec<String>) {
    let mut packed = Vec::new();
    let mut used = 0usize;
    let mut warnings = Vec::new();

    for mut item in ranked.drain(..) {
        let estimate = estimate_tokens(&item.record.claim);
        if !packed.is_empty() && used + estimate > budget {
            item.explanation.push("excluded by budget".to_string());
            warnings.push(format!("budget reached; skipped record {} and remaining matches", item.record.id));
            break;
        }
        used += estimate;
        item.explanation.push("included within budget".to_string());
        packed.push(item);
    }

    (packed, used, warnings)
}
