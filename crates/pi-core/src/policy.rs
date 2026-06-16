use crate::types::*;

fn normalize_claim(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn validate_record(record: &Record, existing: &[Record]) -> GovernanceDecision {
    let claim = record.claim.trim();

    if claim.len() < 8 {
        return GovernanceDecision::reject("claim is too short to be durable memory");
    }

    if !(0.0..=1.0).contains(&record.confidence) {
        return GovernanceDecision::reject("confidence must be between 0.0 and 1.0");
    }

    let mut decision = GovernanceDecision::allow();

    if record.class.requires_evidence() && record.evidence.is_empty() {
        decision.escalate_to_manual("durable record class requires at least one evidence reference");
    }

    if record.class.is_high_sensitivity() {
        decision.escalate_to_manual("identity-level rules require manual review or explicit force");
    }

    let normalized = normalize_claim(&record.claim);

    let duplicate = existing.iter().any(|other| {
        other.status == RecordStatus::Active
            && normalize_claim(&other.claim) == normalized
            && other.class == record.class
    });

    if duplicate {
        decision.escalate_to_manual("possible duplicate active record with same class and normalized claim");
    }

    if record.confidence < 0.35 {
        decision.add_warning("low confidence record; consider keeping it observational");
    }

    decision
}

pub fn validate_patch(patch: &Patch, existing: &[Record]) -> GovernanceDecision {
    match patch.operation {
        PatchOperation::ProposeRecord => {
            let Some(record) = patch.proposed_record.as_ref() else {
                return GovernanceDecision::reject("propose_record patch requires proposed_record");
            };

            validate_record(record, existing)
        }

        PatchOperation::SupersedeRecord => {
            let Some(target_id) = patch.target_id.as_ref() else {
                return GovernanceDecision::reject("supersede_record patch requires target_id");
            };

            let target_exists = existing
                .iter()
                .any(|record| record.id == *target_id && record.status == RecordStatus::Active);

            if !target_exists {
                return GovernanceDecision::reject("supersede target does not exist or is not active");
            }

            let Some(record) = patch.proposed_record.as_ref() else {
                return GovernanceDecision::reject(
                    "supersede_record patch requires replacement proposed_record",
                );
            };

            let mut decision = validate_record(record, existing);
            decision.escalate_to_manual("supersession requires review");
            decision
        }

        PatchOperation::TombstoneRecord => {
            let Some(target_id) = patch.target_id.as_ref() else {
                return GovernanceDecision::reject("tombstone_record patch requires target_id");
            };

            let target_exists = existing
                .iter()
                .any(|record| record.id == *target_id && record.status == RecordStatus::Active);

            if !target_exists {
                return GovernanceDecision::reject("tombstone target does not exist or is not active");
            }

            if patch.reason.trim().len() < 8 {
                return GovernanceDecision::reject("tombstone requires a meaningful reason");
            }

            GovernanceDecision::manual("deletion/tombstone requires review")
        }

        PatchOperation::ReinforceRecord => {
            let Some(target_id) = patch.target_id.as_ref() else {
                return GovernanceDecision::reject("reinforce_record patch requires target_id");
            };

            let target_exists = existing
                .iter()
                .any(|record| record.id == *target_id && record.status == RecordStatus::Active);

            if !target_exists {
                return GovernanceDecision::reject("reinforcement target does not exist or is not active");
            }

            if patch.evidence.is_empty() {
                return GovernanceDecision::reject(
                    "reinforcement requires at least one evidence reference",
                );
            }

            GovernanceDecision::allow()
        }
    }
}
