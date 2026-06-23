use crate::{PolicyProfile, CURRENT_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiConfig {
    pub schema_version: u32,
    #[serde(default)]
    pub default_policy: PolicyProfile,
    #[serde(default)]
    pub namespaces: BTreeMap<String, NamespacePolicyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespacePolicyConfig {
    pub policy: PolicyProfile,
}

impl Default for PiConfig {
    fn default() -> Self {
        Self { schema_version: CURRENT_SCHEMA_VERSION, default_policy: PolicyProfile::Standard, namespaces: BTreeMap::new() }
    }
}

impl PiConfig {
    pub fn effective_policy(&self, namespace: &str) -> PolicyProfile {
        self.namespaces.get(namespace).map(|cfg| cfg.policy).unwrap_or(self.default_policy)
    }

    pub fn set_policy(&mut self, namespace: impl Into<String>, policy: PolicyProfile) {
        self.namespaces.insert(namespace.into(), NamespacePolicyConfig { policy });
    }
}
