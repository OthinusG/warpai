//! Bounded Agent explanations and Broker-owned immutable task provenance.
use crate::{context::ContextRef, project_memory::NativeThreadBinding};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "status",
    content = "items",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum HandoffSection<T> {
    Provided(Vec<T>),
    NotProvided,
    NotApplicable,
}

impl<T> Default for HandoffSection<T> {
    fn default() -> Self {
        Self::NotProvided
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HandoffDecision {
    pub decision: String,
    pub rationale: String,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HandoffIssue {
    pub issue: String,
    #[serde(default)]
    pub impact: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub required_decision: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HandoffSubmission {
    #[serde(default)]
    pub decisions: HandoffSection<HandoffDecision>,
    #[serde(default)]
    pub unresolved_issues: HandoffSection<HandoffIssue>,
    #[serde(default)]
    pub next_steps: HandoffSection<String>,
}

impl HandoffSubmission {
    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(
            serde_json::to_vec(self)?.len() <= 8 * 1024,
            crate::capacity_exceeded(
                "Handoff explanations exceed 8 KiB; shorten them and reference evidence"
            )
        );
        let valid = |value: &str| crate::text(value).is_ok();
        if let HandoffSection::Provided(items) = &self.decisions {
            ensure!(
                items.len() <= 32
                    && items.iter().all(|item| valid(&item.decision)
                        && valid(&item.rationale)
                        && item.evidence_ids.len() <= 32),
                crate::invalid_input("Invalid handoff decisions")
            );
        }
        if let HandoffSection::Provided(items) = &self.unresolved_issues {
            ensure!(
                items.len() <= 32
                    && items.iter().all(|item| valid(&item.issue)
                        && [&item.impact, &item.state, &item.required_decision]
                            .into_iter()
                            .all(|value| value.as_ref().is_none_or(|value| valid(value)))),
                crate::invalid_input("Invalid handoff issues")
            );
        }
        if let HandoffSection::Provided(items) = &self.next_steps {
            ensure!(
                items.len() <= 32 && items.iter().all(|item| valid(item)),
                crate::invalid_input("Invalid handoff next steps")
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HandoffSnapshot {
    pub schema_version: u32,
    pub task_id: String,
    pub task_revision: u32,
    pub execution_thread_id: String,
    pub attempt_id: String,
    pub producer: String,
    pub binding: Option<NativeThreadBinding>,
    pub request_id: String,
    pub reason: String,
    pub source_event_id: String,
    pub source_event_sequence: u64,
    pub result_preview: String,
    pub preview_truncated: bool,
    pub evidence_ids: Vec<String>,
    pub input_context_refs: Vec<ContextRef>,
    pub agent_report: HandoffSubmission,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sections_preserve_missing_empty_and_not_applicable_and_bound_utf8() {
        let missing: HandoffSubmission = serde_json::from_str("{}").unwrap();
        assert!(matches!(missing.next_steps, HandoffSection::NotProvided));
        for section in [
            HandoffSection::<String>::Provided(vec![]),
            HandoffSection::NotApplicable,
        ] {
            let value = serde_json::to_value(section).unwrap();
            assert_ne!(value["status"], "not_provided");
        }
        let oversized = HandoffSubmission {
            next_steps: HandoffSection::Provided(vec!["中".repeat(3000)]),
            ..Default::default()
        };
        assert!(oversized.validate().is_err());
        assert!(HandoffSubmission {
            next_steps: HandoffSection::Provided(vec!["\u{1b}[2J".into()]),
            ..Default::default()
        }
        .validate()
        .is_err());
        assert!(serde_json::from_str::<HandoffSubmission>(r#"{"producer":"spoof"}"#).is_err());
    }

    #[test]
    fn legacy_requests_omit_new_fields_and_strict_input_stays_strict() {
        let submit: crate::Operation = serde_json::from_str(r#"{"op":"task_submit","task_id":"fixture","revision":1,"result":"Done","evidence":"Reported","request_id":"fixture"}"#).unwrap();
        assert!(serde_json::to_value(submit)
            .unwrap()
            .get("handoff")
            .is_none());
        let get: crate::Operation = serde_json::from_str(
            r#"{"op":"context_get","reference":{"id":"fixture","revision":1,"sha256":"fixture"}}"#,
        )
        .unwrap();
        assert!(serde_json::to_value(get)
            .unwrap()
            .get("handoff_part")
            .is_none());
        assert!(serde_json::from_str::<HandoffSubmission>(
            r#"{"next_steps":{"status":"not_provided","items":["hidden"]}}"#
        )
        .is_err());
    }
}

/// Read only the bounded source records explicitly covered by an authorized brief.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HandoffPart {
    Status,
    Task,
    Result,
    Evidence { id: String },
}

/// Evidence provenance and current verification are separate from the immutable brief.
#[derive(Clone, Serialize, Deserialize)]
pub struct HandoffEvidence {
    pub evidence: crate::Evidence,
    pub reported_by: Option<String>,
    pub task_revision: Option<u32>,
    pub attempt_available: bool,
    pub binding: Option<NativeThreadBinding>,
    pub same_attempt: bool,
    pub verification_is_current: bool,
    pub test_outcome_source: String,
    pub review_is_separate: bool,
}
