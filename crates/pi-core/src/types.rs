use crate::schema::current_schema_version;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RecordClass {
    IdentityRule,
    Preference,
    ProjectState,
    Requirement,
    Correction,
    Workflow,
    Observation,
    EvidenceNote,
}

impl RecordClass {
    pub fn is_high_sensitivity(&self) -> bool {
        matches!(self, RecordClass::IdentityRule)
    }

    pub fn requires_evidence(&self) -> bool {
        !matches!(self, RecordClass::Observation | RecordClass::EvidenceNote)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RecordClass::IdentityRule => "identity_rule",
            RecordClass::Preference => "preference",
            RecordClass::ProjectState => "project_state",
            RecordClass::Requirement => "requirement",
            RecordClass::Correction => "correction",
            RecordClass::Workflow => "workflow",
            RecordClass::Observation => "observation",
            RecordClass::EvidenceNote => "evidence_note",
        }
    }
}

impl fmt::Display for RecordClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RecordClass {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let key = input
            .trim()
            .to_lowercase()
            .replace('_', "-")
            .replace(' ', "-");

        match key.as_str() {
            "identity" | "identity-rule" | "hard-rule" => Ok(Self::IdentityRule),
            "preference" | "pref" => Ok(Self::Preference),
            "project-state" | "state" => Ok(Self::ProjectState),
            "requirement" | "req" => Ok(Self::Requirement),
            "correction" => Ok(Self::Correction),
            "workflow" | "playbook" => Ok(Self::Workflow),
            "observation" | "note" => Ok(Self::Observation),
            "evidence-note" | "evidence" => Ok(Self::EvidenceNote),
            _ => Err(format!("unknown record class: {input}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecordStatus {
    Active,
    Superseded,
    Tombstoned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScopeLevel {
    Global,
    Project,
    Session,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Scope {
    pub level: ScopeLevel,
    pub key: Option<String>,
}

impl Scope {
    pub fn global() -> Self {
        Self {
            level: ScopeLevel::Global,
            key: None,
        }
    }

    pub fn project(project: impl Into<String>) -> Self {
        Self {
            level: ScopeLevel::Project,
            key: Some(project.into()),
        }
    }

    pub fn session(session: impl Into<String>) -> Self {
        Self {
            level: ScopeLevel::Session,
            key: Some(session.into()),
        }
    }

    pub fn matches_project_filter(&self, project: Option<&str>) -> bool {
        match project {
            None => true,
            Some(project_key) => match self.level {
                ScopeLevel::Global => true,
                ScopeLevel::Project => self.key.as_deref() == Some(project_key),
                ScopeLevel::Session => false,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Conversation,
    File,
    Url,
    Test,
    Commit,
    UserCorrection,
    HumanReview,
}

impl FromStr for EvidenceKind {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let key = input
            .trim()
            .to_lowercase()
            .replace('_', "-")
            .replace(' ', "-");

        match key.as_str() {
            "conversation" | "chat" => Ok(Self::Conversation),
            "file" => Ok(Self::File),
            "url" | "link" => Ok(Self::Url),
            "test" => Ok(Self::Test),
            "commit" => Ok(Self::Commit),
            "user-correction" | "correction" => Ok(Self::UserCorrection),
            "human-review" | "review" => Ok(Self::HumanReview),
            _ => Err(format!("unknown evidence kind: {input}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    pub kind: EvidenceKind,
    pub uri: String,
    pub note: Option<String>,
}

impl EvidenceRef {
    pub fn new(kind: EvidenceKind, uri: impl Into<String>) -> Self {
        Self {
            schema_version: current_schema_version(),
            kind,
            uri: uri.into(),
            note: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub class: RecordClass,
    pub claim: String,
    pub evidence: Vec<EvidenceRef>,
    pub confidence: f32,
    pub status: RecordStatus,
    pub scope: Scope,
    pub tags: Vec<String>,
    pub supersedes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Record {
    pub fn new(
        class: RecordClass,
        claim: impl Into<String>,
        confidence: f32,
        scope: Scope,
        tags: Vec<String>,
        evidence: Vec<EvidenceRef>,
    ) -> Self {
        let now = Utc::now();

        Self {
            schema_version: current_schema_version(),
            id: format!("rec_{}", Uuid::new_v4()),
            class,
            claim: claim.into(),
            evidence,
            confidence,
            status: RecordStatus::Active,
            scope,
            tags,
            supersedes: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatchOperation {
    ProposeRecord,
    SupersedeRecord,
    TombstoneRecord,
    ReinforceRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatchStatus {
    Proposed,
    Applied,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub operation: PatchOperation,
    pub status: PatchStatus,
    pub target_id: Option<String>,
    pub proposed_record: Option<Record>,
    pub evidence: Vec<EvidenceRef>,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Patch {
    pub fn propose_record(record: Record, reason: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            schema_version: current_schema_version(),
            id: format!("patch_{}", Uuid::new_v4()),
            operation: PatchOperation::ProposeRecord,
            status: PatchStatus::Proposed,
            target_id: None,
            evidence: record.evidence.clone(),
            proposed_record: Some(record),
            reason: reason.into(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn supersede_record(
        target_id: impl Into<String>,
        replacement: Record,
        reason: impl Into<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            schema_version: current_schema_version(),
            id: format!("patch_{}", Uuid::new_v4()),
            operation: PatchOperation::SupersedeRecord,
            status: PatchStatus::Proposed,
            target_id: Some(target_id.into()),
            evidence: replacement.evidence.clone(),
            proposed_record: Some(replacement),
            reason: reason.into(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn tombstone_record(
        target_id: impl Into<String>,
        evidence: Vec<EvidenceRef>,
        reason: impl Into<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            schema_version: current_schema_version(),
            id: format!("patch_{}", Uuid::new_v4()),
            operation: PatchOperation::TombstoneRecord,
            status: PatchStatus::Proposed,
            target_id: Some(target_id.into()),
            evidence,
            proposed_record: None,
            reason: reason.into(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn reinforce_record(
        target_id: impl Into<String>,
        evidence: Vec<EvidenceRef>,
        reason: impl Into<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            schema_version: current_schema_version(),
            id: format!("patch_{}", Uuid::new_v4()),
            operation: PatchOperation::ReinforceRecord,
            status: PatchStatus::Proposed,
            target_id: Some(target_id.into()),
            evidence,
            proposed_record: None,
            reason: reason.into(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn applied_copy(&self) -> Self {
        let mut patch = self.clone();
        patch.status = PatchStatus::Applied;
        patch.updated_at = Utc::now();
        patch
    }

    pub fn rejected_copy(&self) -> Self {
        let mut patch = self.clone();
        patch.status = PatchStatus::Rejected;
        patch.updated_at = Utc::now();
        patch
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Allow,
    ManualReview,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceDecision {
    pub status: DecisionStatus,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}

impl GovernanceDecision {
    pub fn allow() -> Self {
        Self {
            status: DecisionStatus::Allow,
            reasons: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn manual(reason: impl Into<String>) -> Self {
        Self {
            status: DecisionStatus::ManualReview,
            reasons: vec![reason.into()],
            warnings: Vec::new(),
        }
    }

    pub fn reject(reason: impl Into<String>) -> Self {
        Self {
            status: DecisionStatus::Reject,
            reasons: vec![reason.into()],
            warnings: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    pub fn escalate_to_manual(&mut self, reason: impl Into<String>) {
        if self.status == DecisionStatus::Allow {
            self.status = DecisionStatus::ManualReview;
        }
        self.reasons.push(reason.into());
    }

    pub fn can_apply(&self, force: bool) -> bool {
        match self.status {
            DecisionStatus::Allow => true,
            DecisionStatus::ManualReview => force,
            DecisionStatus::Reject => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalBudget {
    pub max_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBlock {
    pub record_id: String,
    pub block_type: String,
    pub content: String,
    pub confidence: f32,
    pub source_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBundle {
    pub query: String,
    pub project: Option<String>,
    pub budget: RetrievalBudget,
    pub used_estimated_tokens: usize,
    pub blocks: Vec<ContextBlock>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreEvent {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub severity: String,
    pub message: String,
    pub object_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl StoreEvent {
    pub fn info(message: impl Into<String>, object_id: Option<String>) -> Self {
        Self {
            schema_version: current_schema_version(),
            id: format!("evt_{}", Uuid::new_v4()),
            severity: "info".to_string(),
            message: message.into(),
            object_id,
            created_at: Utc::now(),
        }
    }

    pub fn warning(message: impl Into<String>, object_id: Option<String>) -> Self {
        Self {
            schema_version: current_schema_version(),
            id: format!("evt_{}", Uuid::new_v4()),
            severity: "warning".to_string(),
            message: message.into(),
            object_id,
            created_at: Utc::now(),
        }
    }
}
