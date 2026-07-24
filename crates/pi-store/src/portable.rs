use anyhow::{bail, Context, Result};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use pi_governance_core::{Patch, Record, StoreEvent, CURRENT_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

use crate::backup::{create_store_backup, StoreBackupReport};
use crate::jsonl::JsonlStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreExportOptions {
    pub namespace: Option<String>,
    pub all_namespaces: bool,
    pub project: Option<String>,
    pub redacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreImportOptions {
    pub namespace: String,
    pub preserve_namespaces: bool,
    pub dry_run: bool,
    pub backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionMetadata {
    pub enabled: bool,
    pub fields_checked: Vec<String>,
    pub fields_redacted: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleProducer {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreExportBundle {
    pub schema_version: u32,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub producer: Option<BundleProducer>,
    pub exported_at: DateTime<Utc>,
    pub redacted: bool,
    pub redaction: RedactionMetadata,
    pub namespace: Option<String>,
    pub all_namespaces: bool,
    pub project: Option<String>,
    pub records: Vec<Record>,
    pub patches: Vec<Patch>,
    pub events: Vec<StoreEvent>,
    #[serde(default)]
    pub evidence: Vec<Value>,
    #[serde(default)]
    pub inquiries: Vec<Value>,
    #[serde(default)]
    pub sessions: Vec<Value>,
    #[serde(default)]
    pub reinforcement: Vec<Value>,
    #[serde(default)]
    pub tombstones: Vec<Value>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreImportReport {
    pub schema_version: u32,
    pub dry_run: bool,
    pub backup_requested: bool,
    pub backup: Option<StoreBackupReport>,
    pub records_in_bundle: usize,
    pub patches_in_bundle: usize,
    pub events_in_bundle: usize,
    pub imported_records: usize,
    pub imported_patches: usize,
    pub imported_events: usize,
    pub skipped_records: usize,
    pub skipped_patches: usize,
    pub skipped_events: usize,
    pub changed: bool,
    pub warnings: Vec<String>,
}

impl JsonlStore {
    pub fn export_bundle(&self, options: StoreExportOptions) -> Result<StoreExportBundle> {
        let records = self.load_records()?;
        let patches = self.load_patches()?;
        let events = self.load_events()?;

        let namespace_filter = if options.all_namespaces {
            None
        } else {
            options.namespace.as_deref()
        };
        let mut selected_records: Vec<Record> = records
            .into_iter()
            .filter(|record| {
                namespace_filter
                    .map(|namespace| record.namespace == namespace)
                    .unwrap_or(true)
            })
            .filter(|record| {
                options
                    .project
                    .as_deref()
                    .map(|project| record.scope.matches_project_filter(Some(project)))
                    .unwrap_or(true)
            })
            .collect();

        let selected_record_ids: HashSet<String> = selected_records
            .iter()
            .map(|record| record.id.clone())
            .collect();

        let mut selected_patches: Vec<Patch> = patches
            .into_iter()
            .filter(|patch| {
                namespace_filter
                    .map(|namespace| patch.namespace == namespace)
                    .unwrap_or(true)
            })
            .filter(|patch| {
                patch
                    .target_id
                    .as_ref()
                    .map(|target_id| selected_record_ids.contains(target_id))
                    .unwrap_or(false)
                    || patch
                        .proposed_record
                        .as_ref()
                        .map(|record| {
                            selected_record_ids.contains(&record.id)
                                || options
                                    .project
                                    .as_deref()
                                    .map(|project| {
                                        record.scope.matches_project_filter(Some(project))
                                    })
                                    .unwrap_or(true)
                        })
                        .unwrap_or(false)
                    || options.project.is_none()
            })
            .collect();

        let selected_patch_ids: HashSet<String> = selected_patches
            .iter()
            .map(|patch| patch.id.clone())
            .collect();

        let mut namespace_events: Vec<StoreEvent> = events
            .into_iter()
            .filter(|event| {
                namespace_filter
                    .map(|namespace| event.namespace == namespace)
                    .unwrap_or(true)
            })
            .collect();
        let mut evidence = take_compatibility_events(&mut namespace_events, "evidence");
        let mut inquiries = take_compatibility_events(&mut namespace_events, "inquiry");
        let mut sessions = take_compatibility_events(&mut namespace_events, "session");
        let mut reinforcement = take_compatibility_events(&mut namespace_events, "reinforcement");
        let mut tombstones = take_compatibility_events(&mut namespace_events, "tombstone");
        let mut warnings = Vec::new();
        if let Some(project) = options.project.as_deref() {
            evidence = filter_compatibility_values(
                evidence,
                "evidence",
                project,
                &["record_id", "related_memory_ids"],
                &selected_record_ids,
                &mut warnings,
            );
            inquiries = filter_compatibility_values(
                inquiries,
                "inquiry",
                project,
                &["record_ids", "related_memory_ids", "answer_memory_id"],
                &selected_record_ids,
                &mut warnings,
            );
            sessions = filter_compatibility_values(
                sessions,
                "session",
                project,
                &[],
                &selected_record_ids,
                &mut warnings,
            );
            reinforcement = filter_compatibility_values(
                reinforcement,
                "reinforcement",
                project,
                &["memory_id"],
                &selected_record_ids,
                &mut warnings,
            );
            tombstones = filter_compatibility_values(
                tombstones,
                "tombstone",
                project,
                &["memory_id", "deleted_record_id"],
                &selected_record_ids,
                &mut warnings,
            );
        }
        let mut selected_events: Vec<StoreEvent> = namespace_events
            .into_iter()
            .filter(|event| {
                options.project.is_none()
                    || event
                        .object_id
                        .as_ref()
                        .map(|object_id| {
                            selected_record_ids.contains(object_id)
                                || selected_patch_ids.contains(object_id)
                        })
                        .unwrap_or(false)
            })
            .collect();
        if options.redacted {
            redact_records(&mut selected_records);
            redact_patches(&mut selected_patches);
            redact_events(&mut selected_events);
            redact_portable_values(&mut evidence, &["source_summary", "source_excerpt"]);
            redact_portable_values(&mut inquiries, &["question", "context"]);
            redact_portable_values(&mut tombstones, &["content", "content_hash"]);
            sessions.clear();
        }

        Ok(StoreExportBundle {
            schema_version: CURRENT_SCHEMA_VERSION,
            format: Some("pi-governance".to_string()),
            producer: Some(BundleProducer {
                name: "pi-governance-rs".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
            exported_at: Utc::now(),
            redacted: options.redacted,
            redaction: RedactionMetadata {
                enabled: options.redacted,
                fields_checked: vec![
                    "records.evidence.uri".to_string(),
                    "patches.evidence.uri".to_string(),
                    "events.message".to_string(),
                ],
                fields_redacted: if options.redacted {
                    vec![
                        "records.evidence.uri".to_string(),
                        "patches.evidence.uri".to_string(),
                        "events.message".to_string(),
                    ]
                } else {
                    Vec::new()
                },
                notes: vec![
                    "Redaction is best-effort and does not replace user review.".to_string()
                ],
            },
            namespace: options.namespace,
            all_namespaces: options.all_namespaces,
            project: options.project,
            records: selected_records,
            patches: selected_patches,
            events: selected_events,
            evidence,
            inquiries,
            sessions,
            reinforcement,
            tombstones,
            warnings,
        })
    }

    pub fn export_bundle_to_path(
        &self,
        path: &Path,
        options: StoreExportOptions,
    ) -> Result<StoreExportBundle> {
        let bundle = self.export_bundle(options)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create export parent dir {:?}", parent))?;
        }

        let json = serde_json::to_string_pretty(&bundle)?;
        fs::write(path, json).with_context(|| format!("failed to write export file {:?}", path))?;

        Ok(bundle)
    }

    pub fn read_portable_bundle_from_path(&self, path: &Path) -> Result<StoreExportBundle> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read portable bundle {:?}", path))?;
        let value: Value = serde_json::from_str(&contents)
            .with_context(|| format!("failed to parse portable bundle JSON {:?}", path))?;
        normalize_portable_bundle(value)
            .with_context(|| format!("failed to normalize portable bundle {:?}", path))
    }

    pub fn import_bundle_from_path(
        &self,
        path: &Path,
        options: StoreImportOptions,
    ) -> Result<StoreImportReport> {
        let bundle = self.read_portable_bundle_from_path(path)?;
        self.import_bundle(bundle, options)
    }

    pub fn import_bundle(
        &self,
        bundle: StoreExportBundle,
        options: StoreImportOptions,
    ) -> Result<StoreImportReport> {
        let bundle = materialize_auxiliary_events(bundle)?;
        self.init()?;
        let session = self.write_session()?;

        if bundle.schema_version > CURRENT_SCHEMA_VERSION {
            bail!(
                "unsupported_export_schema: bundle schema_version {} is newer than current schema_version {}",
                bundle.schema_version,
                CURRENT_SCHEMA_VERSION
            );
        }

        let mut warnings = Vec::new();

        if bundle.redacted {
            warnings.push(
                "imported bundle is redacted; evidence URIs and event messages may be placeholders"
                    .to_string(),
            );
        }

        if bundle.schema_version != CURRENT_SCHEMA_VERSION {
            warnings.push(format!(
                "bundle schema_version {} differs from current schema_version {}; run pi migrate after import if needed",
                bundle.schema_version, CURRENT_SCHEMA_VERSION
            ));
        }

        let mut records = session.load_records()?;
        let mut patches = session.load_patches()?;
        let mut events = session.load_events()?;

        let existing_record_ids: HashSet<(String, String)> = records
            .iter()
            .map(|record| (record.namespace.clone(), record.id.clone()))
            .collect();
        let existing_patch_ids: HashSet<String> =
            patches.iter().map(|patch| patch.id.clone()).collect();
        let existing_event_ids: HashSet<String> =
            events.iter().map(|event| event.id.clone()).collect();

        let mut incoming_record_groups: BTreeMap<(String, String), Vec<Record>> = BTreeMap::new();
        for mut record in bundle.records.iter().cloned() {
            if !options.preserve_namespaces {
                record.namespace = options.namespace.clone();
            }
            incoming_record_groups
                .entry((record.namespace.clone(), record.id.clone()))
                .or_default()
                .push(record);
        }
        let mut import_records = Vec::new();
        for ((namespace, id), group) in incoming_record_groups {
            if existing_record_ids.contains(&(namespace.clone(), id.clone())) {
                continue;
            }
            if group.len() == 1 {
                import_records.push(group.into_iter().next().expect("single record group"));
                continue;
            }
            let first = serde_json::to_value(&group[0])?;
            if group
                .iter()
                .skip(1)
                .all(|record| serde_json::to_value(record).ok().as_ref() == Some(&first))
            {
                warnings.push(format!(
                    "collapsed exact duplicate record {namespace}/{id} during import"
                ));
                import_records.push(group.into_iter().next().expect("duplicate record group"));
            } else {
                warnings.push(format!("quarantined divergent duplicate record {namespace}/{id}; no row from this group was imported"));
            }
        }
        let import_patches: Vec<Patch> = bundle
            .patches
            .iter()
            .filter(|patch| !existing_patch_ids.contains(&patch.id))
            .cloned()
            .map(|mut patch| {
                if !options.preserve_namespaces {
                    patch.namespace = options.namespace.clone();
                    if let Some(record) = &mut patch.proposed_record {
                        record.namespace = options.namespace.clone();
                    }
                }
                patch
            })
            .collect();
        let import_events: Vec<StoreEvent> = bundle
            .events
            .iter()
            .filter(|event| !existing_event_ids.contains(&event.id))
            .cloned()
            .map(|mut event| {
                if !options.preserve_namespaces {
                    event.namespace = options.namespace.clone();
                }
                event
            })
            .collect();

        let skipped_records = bundle.records.len().saturating_sub(import_records.len());
        let skipped_patches = bundle.patches.len().saturating_sub(import_patches.len());
        let skipped_events = bundle.events.len().saturating_sub(import_events.len());
        let changed =
            !import_records.is_empty() || !import_patches.is_empty() || !import_events.is_empty();

        let mut backup = None;

        if changed && !options.dry_run {
            if options.backup {
                backup = Some(create_store_backup(
                    &self.root,
                    &[
                        ("records.jsonl", self.records_path.as_path()),
                        ("patches.jsonl", self.patches_path.as_path()),
                        ("events.jsonl", self.events_path.as_path()),
                    ],
                )?);
            }

            records.extend(import_records.clone());
            patches.extend(import_patches.clone());
            events.extend(import_events.clone());

            session.overwrite_records_atomic(&records)?;
            session.overwrite_patches_atomic(&patches)?;
            session.overwrite_events_atomic(&events)?;
        }

        Ok(StoreImportReport {
            schema_version: CURRENT_SCHEMA_VERSION,
            dry_run: options.dry_run,
            backup_requested: options.backup,
            backup,
            records_in_bundle: bundle.records.len(),
            patches_in_bundle: bundle.patches.len(),
            events_in_bundle: bundle.events.len(),
            imported_records: import_records.len(),
            imported_patches: import_patches.len(),
            imported_events: import_events.len(),
            skipped_records,
            skipped_patches,
            skipped_events,
            changed,
            warnings,
        })
    }
}

fn normalize_timestamp(value: Option<&str>, fallback: DateTime<Utc>) -> Result<DateTime<Utc>> {
    match value {
        None => Ok(fallback),
        Some(raw) if raw.len() == 10 => {
            let date = NaiveDate::parse_from_str(raw, "%Y-%m-%d")?;
            Ok(Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).expect("midnight is valid")))
        }
        Some(raw) => Ok(DateTime::parse_from_rfc3339(raw)?.with_timezone(&Utc)),
    }
}

fn set_timestamp(
    object: &mut serde_json::Map<String, Value>,
    key: &str,
    fallback: DateTime<Utc>,
) -> Result<()> {
    let parsed = normalize_timestamp(object.get(key).and_then(Value::as_str), fallback)?;
    object.insert(key.to_string(), Value::String(parsed.to_rfc3339()));
    Ok(())
}

fn normalize_evidence_value(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    object.entry("schema_version").or_insert(json!(1));
    object.entry("kind").or_insert(json!("conversation"));
    object
        .entry("uri")
        .or_insert(json!("imported:portable-evidence"));
    object.entry("note").or_insert(Value::Null);
    for (key, allowed) in [
        (
            "trust_class",
            &[
                "direct_user_instruction",
                "user_correction",
                "agent_inference",
                "repository_text",
                "generated_content",
                "third_party_documentation",
                "codebase_analysis",
                "human_review",
                "unknown",
            ][..],
        ),
        (
            "durability",
            &["temporary", "task", "project", "long_term", "unknown"][..],
        ),
        (
            "source_kind",
            &[
                "manual_cli",
                "manual_mcp",
                "session_text",
                "transcript_file",
                "stdin",
                "agent_observation",
                "codebase_analysis",
                "imported_bundle",
                "unknown",
            ][..],
        ),
    ] {
        let valid = object
            .get(key)
            .and_then(Value::as_str)
            .map(|item| allowed.contains(&item))
            .unwrap_or(false);
        if !valid {
            object.insert(key.to_string(), json!("unknown"));
        }
    }
}

fn normalize_record_value(
    value: &mut Value,
    fallback_namespace: &str,
    fallback_time: DateTime<Utc>,
) -> Result<()> {
    let object = value
        .as_object_mut()
        .context("portable record must be an object")?;
    object.entry("schema_version").or_insert(json!(1));
    object
        .entry("namespace")
        .or_insert(json!(fallback_namespace));
    object.entry("class").or_insert(json!("workflow"));
    object.entry("evidence").or_insert(json!([]));
    object.entry("layer").or_insert(json!("l2_playbook"));
    object.entry("trust_class").or_insert(json!("unknown"));
    object.entry("durability").or_insert(json!("unknown"));
    object
        .entry("source_kind")
        .or_insert(json!("imported_bundle"));
    object
        .entry("scope")
        .or_insert(json!({"level":"global","key":null}));
    object.entry("tags").or_insert(json!([]));
    object.entry("supersedes").or_insert(json!([]));
    if object.get("status").and_then(Value::as_str) == Some("deleted") {
        object.insert("status".into(), json!("tombstoned"));
    }
    if let Some(evidence) = object.get_mut("evidence").and_then(Value::as_array_mut) {
        for item in evidence {
            normalize_evidence_value(item);
        }
    }
    set_timestamp(object, "created_at", fallback_time)?;
    set_timestamp(object, "updated_at", fallback_time)?;
    Ok(())
}

fn normalize_patch_value(
    value: &mut Value,
    fallback_namespace: &str,
    project: Option<&str>,
    fallback_time: DateTime<Utc>,
) -> Result<()> {
    let object = value
        .as_object_mut()
        .context("portable patch must be an object")?;
    let patch_id = object
        .get("id")
        .and_then(Value::as_str)
        .context("portable patch is missing id")?
        .to_string();
    object.entry("schema_version").or_insert(json!(1));
    object
        .entry("namespace")
        .or_insert(json!(fallback_namespace));
    object.entry("operation").or_insert(json!("propose_record"));
    object.entry("target_id").or_insert(Value::Null);
    object.entry("contest_resolution").or_insert(Value::Null);
    object.entry("evidence").or_insert(json!([]));
    object.entry("reason").or_insert(json!(
        "Imported portable patch; review required before application."
    ));
    set_timestamp(object, "created_at", fallback_time)?;
    set_timestamp(object, "updated_at", fallback_time)?;
    let is_propose = object.get("operation").and_then(Value::as_str) == Some("propose_record");
    let has_proposed_record = object
        .get("proposed_record")
        .map(Value::is_object)
        .unwrap_or(false);
    if is_propose && !has_proposed_record {
        let claim = object
            .get("claim")
            .and_then(Value::as_str)
            .unwrap_or("Imported portable candidate")
            .to_string();
        let class = match object.get("rule_type").and_then(Value::as_str) {
            Some("correction" | "avoid_pattern") => "correction",
            Some("preference" | "prefer_pattern") => "preference",
            Some("convention" | "architecture") => "requirement",
            _ => "workflow",
        };
        let mut record = json!({
            "schema_version":1,"namespace":fallback_namespace,"id":format!("imported_{patch_id}"),"class":class,"claim":claim,
            "evidence":[],"confidence":0.7,"status":"active","layer":object.get("layer").cloned().unwrap_or(json!("l2_playbook")),
            "memory_kind":object.get("memory_kind").cloned().unwrap_or(json!("instruction")),"rule_type":object.get("rule_type").cloned().unwrap_or(Value::Null),
            "trust_class":"agent_inference","durability":"project","source_kind":"imported_bundle",
            "scope":project.map(|key| json!({"level":"project","key":key})).unwrap_or(json!({"level":"global","key":null})),
            "tags":object.get("tags").cloned().unwrap_or(json!([])),"supersedes":[],"created_at":fallback_time.to_rfc3339(),"updated_at":fallback_time.to_rfc3339()
        });
        normalize_record_value(&mut record, fallback_namespace, fallback_time)?;
        object.insert("proposed_record".into(), record);
    } else if has_proposed_record {
        if let Some(record) = object.get_mut("proposed_record") {
            normalize_record_value(record, fallback_namespace, fallback_time)?;
        }
    }
    Ok(())
}

fn normalize_portable_bundle(mut value: Value) -> Result<StoreExportBundle> {
    let object = value
        .as_object_mut()
        .context("portable bundle must be an object")?;
    let schema = object
        .get("schema_version")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    if schema > CURRENT_SCHEMA_VERSION as u64 {
        bail!("unsupported_export_schema: bundle schema_version {schema} is newer than current schema_version {CURRENT_SCHEMA_VERSION}");
    }
    let fallback_time = normalize_timestamp(
        object.get("exported_at").and_then(Value::as_str),
        Utc::now(),
    )?;
    object.insert("exported_at".into(), json!(fallback_time.to_rfc3339()));
    object.entry("redacted").or_insert(json!(false));
    object
        .entry("redaction")
        .or_insert(json!({"enabled":false,"fields_checked":[],"fields_redacted":[],"notes":[]}));
    object.entry("all_namespaces").or_insert(json!(false));
    object.entry("project").or_insert(Value::Null);
    object.entry("events").or_insert(json!([]));
    for key in [
        "evidence",
        "inquiries",
        "sessions",
        "reinforcement",
        "tombstones",
    ] {
        object.entry(key).or_insert(json!([]));
    }
    let namespace = object
        .get("namespace")
        .and_then(Value::as_str)
        .unwrap_or("default")
        .to_string();
    let project = object
        .get("project")
        .and_then(Value::as_str)
        .map(str::to_string);
    for record in object
        .entry("records")
        .or_insert(json!([]))
        .as_array_mut()
        .context("records must be an array")?
    {
        normalize_record_value(record, &namespace, fallback_time)?;
    }
    for patch in object
        .entry("patches")
        .or_insert(json!([]))
        .as_array_mut()
        .context("patches must be an array")?
    {
        normalize_patch_value(patch, &namespace, project.as_deref(), fallback_time)?;
    }
    serde_json::from_value(value)
        .context("normalized bundle does not satisfy the pi-governance v1 contract")
}

fn artifact_event(category: &str, value: &Value, namespace: &str) -> Result<StoreEvent> {
    let object = value
        .as_object()
        .context("portable auxiliary artifact must be an object")?;
    let artifact_id = object
        .get("id")
        .and_then(Value::as_str)
        .context("portable auxiliary artifact is missing id")?;
    let created = object
        .get("created_at")
        .or_else(|| object.get("timestamp"))
        .or_else(|| object.get("deleted_at"))
        .and_then(Value::as_str);
    Ok(StoreEvent {
        schema_version: CURRENT_SCHEMA_VERSION,
        namespace: namespace.to_string(),
        id: format!("interop_{category}_{artifact_id}"),
        severity: "info".into(),
        category: format!("interop_{category}"),
        message: serde_json::to_string(value)?,
        object_id: Some(artifact_id.to_string()),
        created_at: normalize_timestamp(created, Utc::now())?,
    })
}

fn materialize_auxiliary_events(mut bundle: StoreExportBundle) -> Result<StoreExportBundle> {
    let namespace = bundle.namespace.as_deref().unwrap_or("default").to_string();
    for (category, values) in [
        ("evidence", &bundle.evidence),
        ("inquiry", &bundle.inquiries),
        ("session", &bundle.sessions),
        ("reinforcement", &bundle.reinforcement),
        ("tombstone", &bundle.tombstones),
    ] {
        for value in values {
            bundle
                .events
                .push(artifact_event(category, value, &namespace)?);
        }
    }
    bundle.evidence.clear();
    bundle.inquiries.clear();
    bundle.sessions.clear();
    bundle.reinforcement.clear();
    bundle.tombstones.clear();
    Ok(bundle)
}

fn take_compatibility_events(events: &mut Vec<StoreEvent>, category: &str) -> Vec<Value> {
    let target = format!("interop_{category}");
    let mut values = Vec::new();
    let mut retained = Vec::new();
    for event in events.drain(..) {
        if event.category == target {
            if let Ok(value) = serde_json::from_str(&event.message) {
                values.push(value);
            }
        } else {
            retained.push(event);
        }
    }
    *events = retained;
    values
}

fn value_references_selected(
    value: &Value,
    fields: &[&str],
    selected_ids: &HashSet<String>,
) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    fields.iter().any(|field| match object.get(*field) {
        Some(Value::String(id)) => selected_ids.contains(id),
        Some(Value::Array(ids)) => ids
            .iter()
            .filter_map(Value::as_str)
            .any(|id| selected_ids.contains(id)),
        _ => false,
    })
}

fn value_matches_project(value: &Value, project: &str) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.get("project").and_then(Value::as_str) == Some(project)
        || (object.get("scope_level").and_then(Value::as_str) == Some("project")
            && object.get("scope_ref").and_then(Value::as_str) == Some(project))
}

fn filter_compatibility_values(
    values: Vec<Value>,
    section: &str,
    project: &str,
    relation_fields: &[&str],
    selected_ids: &HashSet<String>,
    warnings: &mut Vec<String>,
) -> Vec<Value> {
    let before = values.len();
    let retained: Vec<Value> = values
        .into_iter()
        .filter(|value| {
            value_references_selected(value, relation_fields, selected_ids)
                || value_matches_project(value, project)
        })
        .collect();
    let omitted = before.saturating_sub(retained.len());
    if omitted > 0 {
        warnings.push(format!("{section}: omitted {omitted} artifact(s) without a selected record relationship or explicit project scope"));
    }
    retained
}

fn redact_portable_values(values: &mut [Value], fields: &[&str]) {
    for value in values {
        if let Some(object) = value.as_object_mut() {
            for field in fields {
                if object.contains_key(*field) {
                    object.insert((*field).to_string(), json!("redacted"));
                }
            }
        }
    }
}

fn redact_records(records: &mut [Record]) {
    for record in records {
        for evidence in &mut record.evidence {
            evidence.uri = "redacted:evidence".to_string();
            evidence.note = evidence.note.as_ref().map(|_| "redacted".to_string());
        }
    }
}

fn redact_patches(patches: &mut [Patch]) {
    for patch in patches {
        for evidence in &mut patch.evidence {
            evidence.uri = "redacted:evidence".to_string();
            evidence.note = evidence.note.as_ref().map(|_| "redacted".to_string());
        }

        if let Some(record) = patch.proposed_record.as_mut() {
            redact_records(std::slice::from_mut(record));
        }
    }
}

fn redact_events(events: &mut [StoreEvent]) {
    for event in events {
        event.message = "redacted event message".to_string();
    }
}
