//! Context versions reuse the account Store's transactions, budgets and project authority.
use super::*;
use crate::context::{ContextContent, ContextRef, ContextResource, ContextSummary};
use sha2::{Digest, Sha256};

impl Store {
    fn context_meta<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let row = diesel::sql_query("SELECT value FROM meta WHERE key=?")
            .bind::<Text, _>(key).get_result::<ValueRow>(&mut *self.connection.borrow_mut()).optional()?;
        row.map(|row| serde_json::from_str(&row.value).map_err(anyhow::Error::from)).transpose()
    }

    fn save_context_meta(&self, key: &str, value: &impl serde::Serialize) -> Result<()> {
        diesel::sql_query("INSERT INTO meta(key,value) VALUES (?,?) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
            .bind::<Text, _>(key).bind::<Text, _>(serde_json::to_string(value)?)
            .execute(&mut *self.connection.borrow_mut())?;
        Ok(())
    }

    pub(super) fn context_get(&self, actor: &Agent, reference: &ContextRef) -> Result<ContextResource> {
        self.authorize(actor)?;
        ensure!(Uuid::parse_str(&reference.id).is_ok() && reference.revision > 0
            && reference.sha256.len() == 64 && reference.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()),
            invalid_input("Invalid context reference"));
        let resource: ContextResource = self.context_meta(&format!("context/version/{}/{}", reference.id, reference.revision))?
            .ok_or_else(|| crate::domain("not_found", "Context revision is unavailable", false, None))?;
        ensure!(resource.summary.project == actor.project, scope_denied("Context belongs to another project"));
        ensure!(resource.summary.reference == *reference, invalid_state("Context reference digest changed"));
        let digest = format!("{:x}", Sha256::digest(serde_json::to_vec(&resource.content)?));
        ensure!(digest == reference.sha256, invalid_state("Context snapshot failed integrity verification"));
        Ok(resource)
    }

    pub(super) fn context_list(&self, actor: &Agent, cursor: Option<u64>, limit: Option<u32>) -> Result<Value> {
        self.authorize(actor)?;
        let limit = limit.unwrap_or(PAGE_DEFAULT).clamp(1, PAGE_MAX);
        let rows = diesel::sql_query("SELECT value FROM meta WHERE key LIKE 'context/head/%' AND json_extract(value,'$.project')=? AND json_extract(value,'$.created_seq')>? ORDER BY json_extract(value,'$.created_seq') LIMIT ?")
            .bind::<Text, _>(&actor.project).bind::<BigInt, _>(cursor.unwrap_or(0).min(i64::MAX as u64) as i64)
            .bind::<BigInt, _>(limit as i64 + 1).load::<ValueRow>(&mut *self.connection.borrow_mut())?;
        let mut summaries = rows.into_iter().map(|row| serde_json::from_str::<ContextSummary>(&row.value)).collect::<std::result::Result<Vec<_>, _>>()?;
        let cursor = (summaries.len() > limit as usize).then(|| summaries[limit as usize - 1].created_seq);
        summaries.truncate(limit as usize);
        Ok(json!({"resources":summaries,"cursor":cursor}))
    }

    pub(super) fn publish_context(&self, actor: &Agent, resource_id: Option<&str>, expected_revision: Option<u32>,
        title: &str, content: &ContextContent, source_revision: Option<&str>) -> Result<Value> {
        self.budget_available()?;
        content.validate()?;
        ensure!(!title.trim().is_empty() && title.len() <= crate::MAX_SUBJECT && !title.chars().any(char::is_control),
            invalid_input("Invalid context title"));
        ensure!(source_revision.is_none_or(|value| !value.is_empty() && value.len() <= crate::MAX_SUBJECT
            && !value.chars().any(char::is_control)), invalid_input("Invalid context source revision"));
        let (id, revision) = if let Some(id) = resource_id {
            ensure!(Uuid::parse_str(id).is_ok(), invalid_input("Invalid context resource identity"));
            let head: ContextSummary = self.context_meta(&format!("context/head/{id}"))?
                .ok_or_else(|| crate::domain("not_found", "Context resource is unavailable", false, None))?;
            ensure!(head.project == actor.project, scope_denied("Context belongs to another project"));
            ensure!(expected_revision == Some(head.reference.revision),
                crate::version_conflict("Context revision changed", head.reference.revision as u64));
            ensure!(head.kind == content.kind(), invalid_input("Context resource kind cannot change"));
            (id.to_owned(), head.reference.revision.checked_add(1).ok_or_else(|| invalid_state("Context revision overflow"))?)
        } else {
            ensure!(expected_revision.is_none(), invalid_input("New context cannot supply an expected revision"));
            (Uuid::new_v4().to_string(), 1)
        };
        let summary = ContextSummary { reference:ContextRef { id:id.clone(), revision,
                sha256:format!("{:x}", Sha256::digest(serde_json::to_vec(content)?)) },
            project:actor.project.clone(), title:title.into(), kind:content.kind().into(), author:actor.id.clone(),
            source_revision:source_revision.map(str::to_owned), created_seq:self.next_sequence(&actor.project)? };
        let resource = ContextResource { summary:summary.clone(), content:content.clone() };
        let key = format!("context/version/{id}/{revision}");
        ensure!(self.context_meta::<ContextResource>(&key)?.is_none(), invalid_state("Context revision already exists"));
        self.save_context_meta(&key, &resource)?;
        self.save_context_meta(&format!("context/head/{id}"), &summary)?;
        self.record(&actor.project, "context_published", &actor.id, None, None,
            json!({"reference":summary.reference,"kind":summary.kind,"title":summary.title,"source_revision":summary.source_revision}))?;
        Ok(json!(summary))
    }

    pub(super) fn validate_context_refs(&self, actor: &Agent, references: &[ContextRef]) -> Result<()> {
        ensure!(references.len() <= 16, invalid_input("Too many context references"));
        let mut seen = std::collections::HashSet::new();
        for reference in references {
            ensure!(seen.insert((&reference.id, reference.revision)), invalid_input("Duplicate context reference"));
            self.context_get(actor, reference)?;
        }
        Ok(())
    }

    pub(super) fn task_context_refs(&self, task: &str) -> Result<Vec<ContextRef>> {
        Ok(self.context_meta(&format!("context/task/{task}"))?.unwrap_or_default())
    }

    pub(super) fn save_task_context_refs(&self, actor: &Agent, task: &str, references: &[ContextRef]) -> Result<()> {
        self.validate_context_refs(actor, references)?;
        self.save_context_meta(&format!("context/task/{task}"), &references)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn publication(request_id: String) -> Operation {
        Operation::ContextPublish { resource_id: None, expected_revision: None,
            title: "Repository analysis".into(), content: ContextContent::Analysis {
                summary: "Shared findings".into(), findings: vec!["Use the project contract".into()] },
            source_revision: Some("fixture-revision".into()), request_id }
    }
    #[test]
    fn context_versions_survive_restart_and_reject_foreign_or_changed_references() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("bus.sqlite");
        let store = Store::open(database.to_str().unwrap()).unwrap();
        let actor = Store::operator("/project");
        let operation = publication(Uuid::new_v4().to_string());
        let first = store.execute(&actor, OPERATOR_EPOCH, &operation).unwrap();
        assert_eq!(store.execute(&actor, OPERATOR_EPOCH, &operation).unwrap(), first);
        let reference: ContextRef = serde_json::from_value(first["reference"].clone()).unwrap();
        let update = Operation::ContextPublish { resource_id: Some(reference.id.clone()), expected_revision: Some(1),
            title: "Updated analysis".into(), content: ContextContent::Analysis { summary: "New findings".into(), findings: vec![] },
            source_revision: None, request_id: Uuid::new_v4().to_string() };
        let second = store.execute(&actor, OPERATOR_EPOCH, &update).unwrap();
        assert_eq!(second["reference"]["revision"], 2);
        let mut stale = update.clone();
        if let Operation::ContextPublish { request_id, .. } = &mut stale { *request_id = Uuid::new_v4().to_string(); }
        assert!(store.execute(&actor, OPERATOR_EPOCH, &stale).is_err());
        assert!(store.context_get(&Store::operator("/other"), &reference).is_err());
        let mut corrupt = reference.clone(); corrupt.sha256 = "0".repeat(64);
        assert!(store.context_get(&actor, &corrupt).is_err());
        assert!(store.validate_context_refs(&actor, &[reference.clone(), reference.clone()]).is_err());
        drop(store);
        let store = Store::open(database.to_str().unwrap()).unwrap();
        let resource = store.context_get(&actor, &reference).unwrap();
        assert!(matches!(resource.content, ContextContent::Analysis { summary, .. } if summary == "Shared findings"));
        let page = store.context_list(&actor, None, Some(1)).unwrap();
        assert_eq!(page["resources"].as_array().unwrap().len(), 1);
        assert_eq!(page["resources"][0]["reference"]["revision"], 2);
        assert!(page["resources"][0].get("content").is_none());
        assert!(page["cursor"].is_null());
    }
    #[test]
    fn task_and_peer_messages_pin_context_without_copying_resource_content() {
        let store = Store::open(":memory:").unwrap();
        let lead = store.register("lead", "codex", "/project", "lead").unwrap();
        let worker = store.register("worker", "custom", "/project", "worker").unwrap();
        let published = store.execute(&lead, "lead-run", &publication(Uuid::new_v4().to_string())).unwrap();
        let reference: ContextRef = serde_json::from_value(published["reference"].clone()).unwrap();
        let assign = Operation::TaskAssign { to: worker.id.clone(), description: "Inspect the contract".into(),
            acceptance: "Report evidence".into(), context_refs: vec![reference.clone()], reviewer: None,
            dependencies: vec![], start_deadline: None, execution_timeout_seconds: None, review_timeout_seconds: None,
            request_id: Uuid::new_v4().to_string() };
        let task = store.execute(&lead, "lead-run", &assign).unwrap();
        assert_eq!(task["context_refs"][0], json!(reference));
        let inbox = store.execute(&worker, "worker-run", &Operation::AgentInbox { cursor: None, limit: None }).unwrap();
        let body = &inbox["messages"][0]["body"];
        assert_eq!(body["type"], "task");
        assert_eq!(body["task_id"], task["id"]);
        assert_eq!(body["objective"], "Inspect the contract");
        assert_eq!(body["context_refs"][0], json!(reference));
        assert!(!body.to_string().contains("Shared findings"));
        let send = Operation::AgentSend { expects_result: true, to:worker.id.clone(), body:String::new(),
            content:Some(crate::context::Instruction { objective:"Review the snapshot".into(), context_refs:vec![reference.clone()] }),
            subject:None, thread_id:None, reply_to:None, task_id:None, request_id:Uuid::new_v4().to_string() };
        let sent = store.execute(&lead, "lead-run", &send).unwrap();
        assert_eq!(sent["body"]["type"], "instruction");
        assert_eq!(sent["body"]["instruction"]["context_refs"][0], json!(reference));
        let mut invalid = send.clone();
        if let Operation::AgentSend { content:Some(content), request_id, .. } = &mut invalid {
            content.context_refs[0].sha256 = "0".repeat(64); *request_id = Uuid::new_v4().to_string();
        }
        assert!(store.execute(&lead, "lead-run", &invalid).is_err());
        let inbox = store.execute(&worker, "worker-run", &Operation::AgentInbox { cursor:None, limit:None }).unwrap();
        assert_eq!(inbox["messages"].as_array().unwrap().len(), 2);
    }
}
