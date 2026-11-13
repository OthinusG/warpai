//! Aggregate serialized-byte budget for one operator-panel response.
//!
//! Every resumable collection can shed its tail down to one row; a trimmed page
//! keeps its cursor on the last real record (or just after Private), so the next read resumes
//! without skipping or repeating rows. When even that cannot fit, the operation
//! reports `capacity_exceeded` instead of dropping unreachable rows.
use crate::capacity_exceeded;
use anyhow::{ensure, Result};
use serde_json::Value;

pub(crate) fn bound_panel(panel: &mut Value, budget: usize) -> Result<()> {
    if serde_json::to_vec(panel)?.len() <= budget {
        return Ok(());
    }
    // The derived attention page is the largest and already paginated, so it
    // sheds rows before any list the operator navigates with.
    while trim_attention(panel) {
        if serde_json::to_vec(panel)?.len() <= budget {
            return Ok(());
        }
    }
    for (container, cursor, key) in [
        (&["events"][..], &["event_cursor"][..], &["sequence"][..]),
        (&["history", "records"][..], &["history", "cursor"][..], &["sequence"][..]),
        (&["reservations"][..], &["reservation_cursor"][..], &["created_seq"][..]),
        (&["resources"][..], &["context_cursor"][..], &["created_seq"][..]),
        (&["spaces"][..], &["space_cursor"][..], &["name"][..]),
        (&["agents"][..], &["agent_cursor"][..], &["agent", "name"][..]),
        (&["tasks"][..], &["task_cursor"][..], &["created_seq"][..]),
        (&["messages"][..], &["message_cursor"][..], &["sequence"][..]),
    ] {
        while trim_collection(panel, container, cursor, key) {
            if serde_json::to_vec(panel)?.len() <= budget {
                return Ok(());
            }
        }
    }
    ensure!(
        serde_json::to_vec(panel)?.len() <= budget,
        capacity_exceeded("The operator panel exceeds the readable response budget")
    );
    Ok(())
}

fn trim_collection(panel: &mut Value, container: &[&str], cursor: &[&str], key: &[&str]) -> bool {
    let (Some((leaf, parents)), Some((field, ancestors))) = (container.split_last(), cursor.split_last()) else {
        return false;
    };
    let last = {
        let Some(items) = locate(panel, parents)
            .and_then(|parent| parent.get_mut(*leaf))
            .and_then(Value::as_array_mut)
        else {
            return false;
        };
        if items.is_empty() {
            return false;
        }
        // Keep one row so the returned cursor still has a resume point; a page
        // that fits nothing reports capacity_exceeded instead of losing rows.
        if items.len() == 1 {
            return false;
        }
        items.pop();
        items
            .last()
            .map(|item| {
                // Private precedes name-sorted spaces; the empty cursor skips only that synthetic row.
                if container == ["spaces"] && item["private"] == true {
                    return Value::String(String::new());
                }
                key.iter()
                    .try_fold(item, |value, field| value.get(*field))
                    .cloned()
                    .unwrap_or(Value::Null)
            })
            .unwrap_or(Value::Null)
    };
    let Some(object) = locate(panel, ancestors).and_then(Value::as_object_mut) else {
        return false;
    };
    object.insert((*field).into(), last);
    true
}

fn locate<'a>(mut value: &'a mut Value, path: &[&str]) -> Option<&'a mut Value> {
    for field in path {
        value = value.get_mut(*field)?;
    }
    Some(value)
}

fn trim_attention(panel: &mut Value) -> bool {
    let Some(attention) = panel.get_mut("attention").and_then(Value::as_object_mut) else {
        return false;
    };
    let Some(tasks) = attention.get_mut("tasks").and_then(Value::as_array_mut) else {
        return false;
    };
    if tasks.len() <= 1 {
        return false;
    }
    tasks.pop();
    let last = tasks
        .last()
        .and_then(|row| row.get("created_seq"))
        .cloned()
        .unwrap_or(Value::Null);
    attention.insert("cursor".into(), last);
    attention.insert("truncated".into(), Value::Bool(true));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn panel() -> Value {
        json!({
            "agents": [{"agent": {"name": "a"}}, {"agent": {"name": "b"}}],
            "agent_cursor": "b",
            "tasks": [{"id": "t1", "created_seq": 1}, {"id": "t2", "created_seq": 2}],
            "task_cursor": 2,
            "events": [{"sequence": 10}, {"sequence": 11}],
            "event_cursor": 11,
            "messages": [], "message_cursor": null,
            "history": null,
            "reservations": [], "reservation_cursor": null,
            "resources": [], "context_cursor": null,
            "spaces": [], "space_cursor": null,
            "attention": {"tasks": [], "total": 0, "truncated": false,
                "counts": {"answer": 0, "review": 0, "permission": 0, "recovery": 0,
                    "unreachable": 0, "dependency": 0, "overdue": 0, "reconcile": 0},
                "cursor": null},
        })
    }

    #[test]
    fn multibyte_attention_is_trimmed_without_skipping_tasks() {
        let body: String = "中".repeat(512) + "🎯";
        let tasks: Vec<Value> = (1..=50)
            .map(|sequence| json!({
                "task_id": format!("task-{sequence}"), "description": "工作",
                "state": "running", "revision": 2, "version": 3,
                "created_seq": sequence, "actions": ["reconcile"],
                "followup_count": 8,
                "followups": (0..8).map(|index| json!({
                    "message_id": format!("m-{sequence}-{index}"), "revision": 1,
                    "kind": "message", "sender": "issuer", "recipient": "worker",
                    "thread_id": Value::Null, "body": body, "truncated": false,
                })).collect::<Vec<_>>(),
            }))
            .collect();
        let mut value = panel();
        value["attention"] = json!({"tasks": tasks, "total": 50, "truncated": false,
            "counts": {"answer": 0, "review": 0, "permission": 0, "recovery": 0,
                "unreachable": 0, "dependency": 0, "overdue": 0, "reconcile": 50},
            "cursor": null});
        let budget = 64 * 1024;
        assert!(serde_json::to_vec(&value).unwrap().len() > budget);
        bound_panel(&mut value, budget).unwrap();
        assert!(serde_json::to_vec(&value).unwrap().len() <= budget);
        let attention = value["attention"].as_object().unwrap();
        assert_eq!(attention["truncated"], true);
        let rows = attention["tasks"].as_array().unwrap();
        assert!(!rows.is_empty());
        assert!(rows.len() < 50);
        // Whole rows are dropped, so every surviving row still carries its previews.
        assert!(rows.iter().all(|row| row["followups"].as_array().unwrap().len() == 8));
        // The cursor must point at the last returned row so paging resumes there.
        let cursor = attention["cursor"].as_u64().unwrap();
        assert_eq!(cursor, rows.last().unwrap()["created_seq"].as_u64().unwrap());
    }

    #[test]
    fn every_collection_keeps_a_resumable_cursor_when_trimmed() {
        let mut value = panel();
        value["events"] = serde_json::Value::Array(
            (0..40).map(|index| json!({"sequence": index + 1, "body": "x".repeat(64)})).collect(),
        );
        value["history"] = json!({"records": (0..40)
            .map(|index| json!({"type": "event", "sequence": index + 1, "data": "x".repeat(64)}))
            .collect::<Vec<_>>(), "cursor": null, "preview": {}, "capacity": {}});
        let budget = 2048;
        assert!(serde_json::to_vec(&value).unwrap().len() > budget);
        bound_panel(&mut value, budget).unwrap();
        assert!(serde_json::to_vec(&value).unwrap().len() <= budget);
        for (container, cursor, key) in [
            (&["agents"][..], &["agent_cursor"][..], &["agent", "name"][..]),
            (&["tasks"][..], &["task_cursor"][..], &["created_seq"][..]),
            (&["events"][..], &["event_cursor"][..], &["sequence"][..]),
            (&["history", "records"][..], &["history", "cursor"][..], &["sequence"][..]),
        ] {
            let Some(current) = locate(&mut value, &cursor[..cursor.len() - 1])
                .and_then(|parent| parent.get(cursor[cursor.len() - 1]))
                .cloned()
            else {
                continue;
            };
            let last = {
                let Some(items) = locate(&mut value, container).and_then(|items| items.as_array()) else {
                    continue;
                };
                items.last().map(|item| {
                    key.iter()
                        .try_fold(item, |value, field| value.get(*field))
                        .cloned()
                        .unwrap_or(Value::Null)
                })
            };
            match last {
                Some(last) => assert_eq!(current, last, "{container:?} cursor must resume at its last row"),
                None => assert!(current.is_null(), "{container:?} emptied with a stale cursor"),
            }
        }
    }

    #[test]
    fn an_unfittable_panel_reports_capacity_instead_of_losing_rows() {
        let mut value = panel();
        let error = bound_panel(&mut value, 300).unwrap_err();
        assert_eq!(
            error.downcast_ref::<crate::DomainError>().map(|error| error.code.as_str()),
            Some("capacity_exceeded")
        );
        // The caller keeps the previous snapshot; the failed response is not partial.
        assert_eq!(value["attention"]["tasks"].as_array().unwrap().len(), 0);
    }
}
