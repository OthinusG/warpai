//! Project memory indexes immutable resources; execution threads reuse tracked tasks.
use crate::context::{ContextRef, ContextSummary};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub resource: ContextSummary,
    pub source_task: Option<String>,
    pub source_task_revision: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub version: u64,
    pub project: String,
    pub current: Option<MemorySnapshot>,
    pub proposed: Option<MemorySnapshot>,
    pub updated_seq: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeThreadBinding {
    pub attempt_id: String,
    pub revision: u32,
    pub agent: String,
    pub terminal: String,
    pub program: String,
    pub run: String,
    pub workspace: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionThread {
    pub id: String,
    pub task_id: String,
    pub parent_thread_id: Option<String>,
    pub state: String,
    pub revision: u32,
    pub issuer: String,
    pub assignee: String,
    pub context_refs: Vec<ContextRef>,
    pub bindings: Vec<NativeThreadBinding>,
    pub history_truncated: bool,
}
