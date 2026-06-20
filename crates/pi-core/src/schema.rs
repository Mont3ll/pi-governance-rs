use serde::{Deserialize, Serialize};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub fn current_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaFileAudit {
    pub file_name: String,
    pub entries: usize,
    pub missing_schema_version: usize,
    pub mismatched_schema_version: usize,
    pub invalid_json_lines: usize,
}

impl SchemaFileAudit {
    pub fn clean(file_name: impl Into<String>) -> Self {
        Self {
            file_name: file_name.into(),
            entries: 0,
            missing_schema_version: 0,
            mismatched_schema_version: 0,
            invalid_json_lines: 0,
        }
    }
}
