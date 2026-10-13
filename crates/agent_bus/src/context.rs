//! Immutable project context and typed peer/task bodies; no filesystem or network reads.
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextRef {
    pub id: String,
    pub revision: u32,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RepositoryEntry {
    pub path: String,
    #[serde(default)]
    pub symbols: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContextContent {
    RepositoryIndex { entries: Vec<RepositoryEntry> },
    Analysis { summary: String, findings: Vec<String> },
    CodeSnippet { path: String, language: String, start_line: u32, code: String },
    DesignDecision { decision: String, rationale: String, alternatives: Vec<String> },
}

impl ContextContent {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::RepositoryIndex { .. } => "repository_index",
            Self::Analysis { .. } => "analysis",
            Self::CodeSnippet { .. } => "code_snippet",
            Self::DesignDecision { .. } => "design_decision",
        }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(serde_json::to_vec(self)?.len() <= 32 * 1024,
            crate::capacity_exceeded("Context content exceeds the managed transport limit; publish scoped resources"));
        let path = |value: &str| -> Result<()> {
            ensure!(!value.is_empty() && value.len() <= 4096 && !value.contains(['\\', ':'])
                && !value.starts_with('/') && !value.chars().any(char::is_control)
                && value.split('/').all(|part| !matches!(part, "" | "." | "..")),
                crate::invalid_input("Context paths must be workspace-relative"));
            Ok(())
        };
        match self {
            Self::RepositoryIndex { entries } => {
                ensure!(!entries.is_empty() && entries.len() <= 1024, crate::invalid_input("Invalid context index size"));
                for entry in entries { path(&entry.path)?; }
            }
            Self::Analysis { summary, findings } => {
                ensure!(!summary.trim().is_empty() && findings.len() <= 1024, crate::invalid_input("Invalid context analysis"));
            }
            Self::CodeSnippet { path: source, language, start_line, code } => {
                path(source)?;
                ensure!(!language.is_empty() && *start_line > 0 && !code.is_empty(), crate::invalid_input("Invalid context snippet"));
                ensure!(!source.split('/').any(|part| part == ".ssh" || part == ".env" || part.starts_with(".env.")
                    || part.ends_with(".pem") || part.ends_with(".key")),
                    crate::scope_denied("Credential files cannot be context snippets"));
            }
            Self::DesignDecision { decision, rationale, alternatives } => {
                ensure!(!decision.trim().is_empty() && !rationale.trim().is_empty() && alternatives.len() <= 128,
                    crate::invalid_input("Invalid context decision"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextSummary {
    pub reference: ContextRef,
    pub project: String,
    pub title: String,
    pub kind: String,
    pub author: String,
    pub source_revision: Option<String>,
    #[serde(default)]
    pub source_workspace: Option<String>,
    pub created_seq: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextResource {
    #[serde(flatten)]
    pub summary: ContextSummary,
    pub content: ContextContent,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Instruction {
    pub objective: String,
    #[serde(default)]
    pub context_refs: Vec<ContextRef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum StructuredBody {
    Instruction { schema_version: u32, instruction: Instruction },
    Task { schema_version: u32, event: String, task_id: String, revision: u32,
        objective: String, acceptance: String, context_refs: Vec<ContextRef>, details: TaskDetails },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskDetails {
    pub instruction: String,
    pub state: String,
    pub result: Option<String>,
    pub evidence_ids: Vec<String>,
}

const BODY_PREFIX: &str = "\u{1e}warpai-body-v1:";

impl ContextRef {
    pub fn uri(&self) -> String { format!("warpai-context://{}/{}/{}", self.id, self.revision, self.sha256) }
    pub fn from_uri(uri: &str) -> Result<Self> {
        let parts = uri.strip_prefix("warpai-context://").ok_or_else(|| crate::invalid_input("Invalid context URI"))?.split('/').collect::<Vec<_>>();
        ensure!(parts.len() == 3 && uuid::Uuid::parse_str(parts[0]).is_ok(), crate::invalid_input("Invalid context URI"));
        let revision = parts[1].parse::<u32>().map_err(|_| crate::invalid_input("Invalid context revision"))?;
        ensure!(revision > 0 && parts[2].len() == 64 && parts[2].bytes().all(|byte| byte.is_ascii_hexdigit()), crate::invalid_input("Invalid context URI"));
        Ok(Self { id: parts[0].into(), revision, sha256: parts[2].into() })
    }
}

impl StructuredBody {
    pub(crate) fn encode(&self) -> Result<String> {
        ensure!(matches!(self, Self::Instruction { schema_version: 1, .. } | Self::Task { schema_version: 1, .. }), crate::invalid_input("Unsupported message body schema"));
        Ok(format!("{BODY_PREFIX}{}", serde_json::to_string(self)?)) }
    pub fn decode(value: &str) -> Option<Self> { serde_json::from_str(value.strip_prefix(BODY_PREFIX)?).ok() }
    pub fn display(&self) -> String {
        match self {
            Self::Instruction { instruction, .. } => instruction.objective.clone(),
            Self::Task { objective, acceptance, details, .. } => format!("{}\n{}\n{}", objective, acceptance, details.instruction),
        }
    }
}

pub fn display_wire_body(value: &serde_json::Value) -> String {
    if let Some(text) = value.as_str() {
        return StructuredBody::decode(text).map_or_else(|| text.to_owned(), |body| body.display());
    }
    serde_json::from_value::<StructuredBody>(value.clone()).map(|body| body.display()).unwrap_or_default()
}

/// Keep the SQLite text column and legacy readers while putting an object on the MCP wire.
pub(crate) mod message_body {
    use super::*;
    pub fn serialize<S: serde::Serializer>(value: &str, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match StructuredBody::decode(value) {
            Some(body) => body.serialize(serializer),
            None => value.serialize(serializer),
        }
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<String, D::Error> {
        match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::String(value) => Ok(value),
            value => serde_json::from_value::<StructuredBody>(value)
                .map_err(serde::de::Error::custom)?.encode().map_err(serde::de::Error::custom),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn context_uris_and_content_reject_untrusted_paths_and_oversize_payloads() {
        for uri in ["file:///etc/passwd", "https://example.com", "warpai-context://../1/hash"] { assert!(ContextRef::from_uri(uri).is_err()); }
        let reference = ContextRef { id:uuid::Uuid::new_v4().to_string(), revision:1, sha256:"a".repeat(64) };
        assert_eq!(ContextRef::from_uri(&reference.uri()).unwrap(), reference);
        for path in ["../secret", "/etc/passwd", "a/../../b", ".env.local", "a/key.pem"] {
            assert!(ContextContent::CodeSnippet { path:path.into(), language:"rust".into(), start_line:1, code:"fixture".into() }.validate().is_err());
        }
        assert!(ContextContent::Analysis { summary:"x".repeat(33 * 1024), findings:vec![] }.validate().is_err());
        assert!(StructuredBody::Instruction { schema_version:2, instruction:Instruction { objective:"future".into(), context_refs:vec![] } }.encode().is_err());
        assert!(StructuredBody::decode(r#"{"type":"instruction","schema_version":1}"#).is_none());
    }
}
