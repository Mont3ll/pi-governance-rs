use crate::schema::current_schema_version;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

pub const DEFAULT_NAMESPACE: &str = "default";

pub fn default_namespace() -> String {
    DEFAULT_NAMESPACE.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NamespaceId(pub String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyProfile {
    Permissive,
    Standard,
    Strict,
}

impl Default for PolicyProfile {
    fn default() -> Self { Self::Standard }
}

impl PolicyProfile {
    pub fn as_str(&self) -> &'static str {
        match self { Self::Permissive => "permissive", Self::Standard => "standard", Self::Strict => "strict" }
    }
}

impl fmt::Display for PolicyProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.as_str()) }
}

impl FromStr for PolicyProfile {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_lowercase().replace('_', "-").as_str() {
            "permissive" => Ok(Self::Permissive),
            "standard" => Ok(Self::Standard),
            "strict" => Ok(Self::Strict),
            _ => Err(format!("unknown policy profile: {input}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLayer {
    L1Identity,
    L2Playbook,
    L3Session,
}

impl MemoryLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::L1Identity => "l1_identity",
            Self::L2Playbook => "l2_playbook",
            Self::L3Session => "l3_session",
        }
    }
}

impl Default for MemoryLayer { fn default() -> Self { Self::L2Playbook } }

impl fmt::Display for MemoryLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.as_str()) }
}

impl FromStr for MemoryLayer {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_lowercase().replace('_', "-").as_str() {
            "l1" | "l1-identity" | "identity" => Ok(Self::L1Identity),
            "l2" | "l2-playbook" | "playbook" => Ok(Self::L2Playbook),
            "l3" | "l3-session" | "session" => Ok(Self::L3Session),
            _ => Err(format!("unknown memory layer: {input}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind { Fact, Event, Instruction, Task }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RuleType { AvoidPattern, PreferPattern, Convention, Architecture, Workflow, Preference, Testing, Correction, Tool }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TrustClass { DirectUserInstruction, UserCorrection, AgentInference, RepositoryText, GeneratedContent, ThirdPartyDocumentation, CodebaseAnalysis, HumanReview, Unknown }

impl Default for TrustClass { fn default() -> Self { Self::Unknown } }
impl fmt::Display for TrustClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::DirectUserInstruction => "direct_user_instruction",
            Self::UserCorrection => "user_correction",
            Self::AgentInference => "agent_inference",
            Self::RepositoryText => "repository_text",
            Self::GeneratedContent => "generated_content",
            Self::ThirdPartyDocumentation => "third_party_documentation",
            Self::CodebaseAnalysis => "codebase_analysis",
            Self::HumanReview => "human_review",
            Self::Unknown => "unknown",
        };
        write!(f, "{value}")
    }
}
impl FromStr for TrustClass {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_lowercase().replace('-', "_").as_str() {
            "direct_user_instruction" => Ok(Self::DirectUserInstruction),
            "user_correction" => Ok(Self::UserCorrection),
            "agent_inference" => Ok(Self::AgentInference),
            "repository_text" => Ok(Self::RepositoryText),
            "generated_content" => Ok(Self::GeneratedContent),
            "third_party_documentation" => Ok(Self::ThirdPartyDocumentation),
            "codebase_analysis" => Ok(Self::CodebaseAnalysis),
            "human_review" => Ok(Self::HumanReview),
            "unknown" => Ok(Self::Unknown),
            _ => Err(format!("unknown trust class: {input}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Durability { Temporary, Task, Project, LongTerm, Unknown }

impl Default for Durability { fn default() -> Self { Self::Unknown } }
impl FromStr for Durability {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_lowercase().replace('-', "_").as_str() {
            "temporary" => Ok(Self::Temporary),
            "task" => Ok(Self::Task),
            "project" => Ok(Self::Project),
            "long_term" => Ok(Self::LongTerm),
            "unknown" => Ok(Self::Unknown),
            _ => Err(format!("unknown durability: {input}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind { ManualCli, ManualMcp, SessionText, TranscriptFile, Stdin, AgentObservation, CodebaseAnalysis, ImportedBundle, Unknown }

impl Default for SourceKind { fn default() -> Self { Self::Unknown } }
impl FromStr for SourceKind {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_lowercase().replace('-', "_").as_str() {
            "manual_cli" => Ok(Self::ManualCli),
            "manual_mcp" => Ok(Self::ManualMcp),
            "session_text" => Ok(Self::SessionText),
            "transcript_file" => Ok(Self::TranscriptFile),
            "stdin" => Ok(Self::Stdin),
            "agent_observation" => Ok(Self::AgentObservation),
            "codebase_analysis" => Ok(Self::CodebaseAnalysis),
            "imported_bundle" => Ok(Self::ImportedBundle),
            "unknown" => Ok(Self::Unknown),
            _ => Err(format!("unknown source kind: {input}")),
        }
    }
}

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

    pub fn inferred_layer(&self) -> MemoryLayer {
        match self {
            RecordClass::IdentityRule => MemoryLayer::L1Identity,
            RecordClass::Observation | RecordClass::EvidenceNote => MemoryLayer::L3Session,
            _ => MemoryLayer::L2Playbook,
        }
    }

    pub fn inferred_memory_kind(&self) -> MemoryKind {
        match self {
            RecordClass::Workflow | RecordClass::Preference | RecordClass::Correction | RecordClass::Requirement => MemoryKind::Instruction,
            RecordClass::Observation | RecordClass::EvidenceNote => MemoryKind::Event,
            _ => MemoryKind::Fact,
        }
    }

    pub fn inferred_rule_type(&self) -> Option<RuleType> {
        match self {
            RecordClass::Workflow => Some(RuleType::Workflow),
            RecordClass::Preference => Some(RuleType::Preference),
            RecordClass::Correction => Some(RuleType::Correction),
            RecordClass::Requirement => Some(RuleType::Convention),
            _ => None,
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
    Contested,
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
    #[serde(default)]
    pub trust_class: TrustClass,
    #[serde(default)]
    pub durability: Durability,
    #[serde(default)]
    pub source_kind: SourceKind,
}

impl EvidenceRef {
    pub fn new(kind: EvidenceKind, uri: impl Into<String>) -> Self {
        Self {
            schema_version: current_schema_version(),
            kind,
            uri: uri.into(),
            note: None,
            trust_class: TrustClass::Unknown,
            durability: Durability::Unknown,
            source_kind: SourceKind::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub id: String,
    pub class: RecordClass,
    pub claim: String,
    pub evidence: Vec<EvidenceRef>,
    pub confidence: f32,
    pub status: RecordStatus,
    #[serde(default)]
    pub layer: MemoryLayer,
    #[serde(default)]
    pub memory_kind: Option<MemoryKind>,
    #[serde(default)]
    pub rule_type: Option<RuleType>,
    #[serde(default)]
    pub trust_class: TrustClass,
    #[serde(default)]
    pub durability: Durability,
    #[serde(default)]
    pub source_kind: SourceKind,
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
        let layer = class.inferred_layer();
        let memory_kind = class.inferred_memory_kind();
        let rule_type = class.inferred_rule_type();

        Self {
            schema_version: current_schema_version(),
            namespace: default_namespace(),
            id: format!("rec_{}", Uuid::new_v4()),
            class,
            claim: claim.into(),
            evidence,
            confidence,
            status: RecordStatus::Active,
            layer,
            memory_kind: Some(memory_kind),
            rule_type,
            trust_class: TrustClass::Unknown,
            durability: Durability::Unknown,
            source_kind: SourceKind::Unknown,
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
    ContestRecord,
    ResolveContest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContestResolution {
    Uphold,
    Tombstone,
    Supersede,
}

impl fmt::Display for ContestResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            ContestResolution::Uphold => "uphold",
            ContestResolution::Tombstone => "tombstone",
            ContestResolution::Supersede => "supersede",
        };

        write!(f, "{value}")
    }
}

impl FromStr for ContestResolution {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let key = input
            .trim()
            .to_lowercase()
            .replace('_', "-")
            .replace(' ', "-");

        match key.as_str() {
            "uphold" | "keep" | "restore" => Ok(Self::Uphold),
            "tombstone" | "delete" | "remove" => Ok(Self::Tombstone),
            "supersede" | "replace" => Ok(Self::Supersede),
            _ => Err(format!("unknown contest resolution: {input}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatchStatus {
    Proposed,
    Applied,
    Rejected,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub id: String,
    pub operation: PatchOperation,
    pub status: PatchStatus,
    pub target_id: Option<String>,
    pub proposed_record: Option<Record>,
    pub contest_resolution: Option<ContestResolution>,
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
            namespace: record.namespace.clone(),
            id: format!("patch_{}", Uuid::new_v4()),
            operation: PatchOperation::ProposeRecord,
            status: PatchStatus::Proposed,
            target_id: None,
            evidence: record.evidence.clone(),
            proposed_record: Some(record),
            contest_resolution: None,
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
            namespace: replacement.namespace.clone(),
            id: format!("patch_{}", Uuid::new_v4()),
            operation: PatchOperation::SupersedeRecord,
            status: PatchStatus::Proposed,
            target_id: Some(target_id.into()),
            evidence: replacement.evidence.clone(),
            proposed_record: Some(replacement),
            contest_resolution: None,
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
            namespace: default_namespace(),
            id: format!("patch_{}", Uuid::new_v4()),
            operation: PatchOperation::TombstoneRecord,
            status: PatchStatus::Proposed,
            target_id: Some(target_id.into()),
            evidence,
            proposed_record: None,
            contest_resolution: None,
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
            namespace: default_namespace(),
            id: format!("patch_{}", Uuid::new_v4()),
            operation: PatchOperation::ReinforceRecord,
            status: PatchStatus::Proposed,
            target_id: Some(target_id.into()),
            evidence,
            proposed_record: None,
            contest_resolution: None,
            reason: reason.into(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn contest_record(
        target_id: impl Into<String>,
        evidence: Vec<EvidenceRef>,
        reason: impl Into<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            schema_version: current_schema_version(),
            namespace: default_namespace(),
            id: format!("patch_{}", Uuid::new_v4()),
            operation: PatchOperation::ContestRecord,
            status: PatchStatus::Proposed,
            target_id: Some(target_id.into()),
            evidence,
            proposed_record: None,
            contest_resolution: None,
            reason: reason.into(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn resolve_contest(
        target_id: impl Into<String>,
        resolution: ContestResolution,
        replacement: Option<Record>,
        evidence: Vec<EvidenceRef>,
        reason: impl Into<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            schema_version: current_schema_version(),
            namespace: default_namespace(),
            id: format!("patch_{}", Uuid::new_v4()),
            operation: PatchOperation::ResolveContest,
            status: PatchStatus::Proposed,
            target_id: Some(target_id.into()),
            evidence,
            proposed_record: replacement,
            contest_resolution: Some(resolution),
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

    pub fn deferred_copy(&self) -> Self {
        let mut patch = self.clone();
        patch.status = PatchStatus::Deferred;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalFormat {
    Markdown,
    Json,
}

pub fn default_retriever() -> String { "deterministic".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalOptions {
    pub query: String,
    #[serde(default = "default_retriever")]
    pub retriever: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub project: Option<String>,
    pub budget: usize,
    pub format: RetrievalFormat,
    pub explain: bool,
    pub classes: Vec<RecordClass>,
    pub include_global: bool,
    pub include_contested: bool,
    pub min_confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub query_match: f32,
    pub project_scope: f32,
    pub class_priority: f32,
    pub tag_match: f32,
    pub confidence: f32,
    pub evidence: f32,
    pub recency: f32,
    pub status_penalty: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedRecord {
    pub record: Record,
    pub score: f32,
    #[serde(default)]
    pub deterministic_score: f32,
    #[serde(default)]
    pub lexical_score: f32,
    #[serde(default)]
    pub hybrid_score: f32,
    #[serde(default)]
    pub matched_fields: Vec<String>,
    #[serde(rename = "score_breakdown")]
    pub breakdown: ScoreBreakdown,
    pub matched_terms: Vec<String>,
    pub explanation: Vec<String>,
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
    #[serde(default = "default_retriever")]
    pub retriever: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub project: Option<String>,
    pub budget: RetrievalBudget,
    pub used_estimated_tokens: usize,
    #[serde(default)]
    pub omitted_count: usize,
    pub explain: bool,
    pub blocks: Vec<ContextBlock>,
    pub records: Vec<RankedRecord>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub empty_reason: Option<String>,
    #[serde(default)]
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecallEventClient { Cli, Mcp }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecallEventOperation { Retrieve, BuildContext, RecallXray }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallEvent {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub client: RecallEventClient,
    pub operation: RecallEventOperation,
    pub query_hash: String,
    pub selected_record_ids: Vec<String>,
    pub budget_requested: usize,
    pub budget_used: usize,
}

impl RecallEvent {
    pub fn new(namespace: impl Into<String>, client: RecallEventClient, operation: RecallEventOperation, query_hash: impl Into<String>, selected_record_ids: Vec<String>, budget_requested: usize, budget_used: usize) -> Self {
        Self { schema_version: current_schema_version(), namespace: namespace.into(), id: format!("recall_{}", Uuid::new_v4()), timestamp: Utc::now(), client, operation, query_hash: query_hash.into(), selected_record_ids, budget_requested, budget_used }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreEvent {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_namespace")]
    pub namespace: String,
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
            namespace: default_namespace(),
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
            namespace: default_namespace(),
            id: format!("evt_{}", Uuid::new_v4()),
            severity: "warning".to_string(),
            message: message.into(),
            object_id,
            created_at: Utc::now(),
        }
    }
}
