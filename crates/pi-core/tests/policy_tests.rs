use pi_core::{
    validate_patch, validate_record, DecisionStatus, EvidenceKind, EvidenceRef, Patch, Record, RecordClass, Scope,
};

#[test]
fn rejects_claims_that_are_too_short() {
    let record = Record::new(
        RecordClass::Preference,
        "short",
        0.70,
        Scope::global(),
        Vec::new(),
        vec![EvidenceRef::new(EvidenceKind::Conversation, "conversation:test")],
    );

    let decision = validate_record(&record, &[]);

    assert_eq!(decision.status, DecisionStatus::Reject);
}

#[test]
fn identity_level_records_require_manual_review() {
    let record = Record::new(
        RecordClass::IdentityRule,
        "User requires identity-level preferences to be reviewed before storage.",
        0.90,
        Scope::global(),
        vec!["identity".to_string()],
        vec![EvidenceRef::new(EvidenceKind::Conversation, "conversation:test")],
    );

    let decision = validate_record(&record, &[]);

    assert_eq!(decision.status, DecisionStatus::ManualReview);
    assert!(decision
        .reasons
        .iter()
        .any(|reason| reason.contains("identity-level")));
}

#[test]
fn tombstone_patches_require_manual_review() {
    let record = Record::new(
        RecordClass::Requirement,
        "Records should be tombstoned rather than physically deleted from the audit trail.",
        0.80,
        Scope::global(),
        vec!["audit".to_string()],
        vec![EvidenceRef::new(EvidenceKind::Conversation, "conversation:test")],
    );

    let patch = Patch::tombstone_record(
        record.id.clone(),
        vec![EvidenceRef::new(EvidenceKind::HumanReview, "review:test")],
        "human requested removal while retaining audit history",
    );

    let decision = validate_patch(&patch, &[record]);

    assert_eq!(decision.status, DecisionStatus::ManualReview);
    assert!(decision
        .reasons
        .iter()
        .any(|reason| reason.contains("tombstone requires review")));
}
