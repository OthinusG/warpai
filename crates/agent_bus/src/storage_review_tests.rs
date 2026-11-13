use super::*;

fn operation(mut value: Value) -> Operation {
    value["request_id"] = json!(Uuid::new_v4().to_string());
    serde_json::from_value(value).unwrap()
}

fn cancelled_read_followup(store: &Store) -> (Agent, String, String) {
    let worker = store
        .register("terminal-worker", "claude", "/project", "worker")
        .unwrap();
    // A stored reviewer keeps the default reassignment lookup inside the agents
    // table; the synthetic operator identity is not a registered agent.
    store
        .register("terminal-reviewer", "claude", "/project", "reviewer")
        .unwrap();
    let operator = Store::operator("/project");
    let execute = |value| {
        store
            .execute(&operator, OPERATOR_EPOCH, &operation(value))
            .unwrap()
    };
    let task = execute(json!({"op": "task_assign", "to": "worker", "reviewer": "reviewer",
        "description": "work", "acceptance": "evidence"}));
    let id = task["id"].as_str().unwrap().to_owned();
    let message = execute(json!({"op": "agent_send", "to": "worker",
        "body": "Revision one instruction", "task_id": id}));
    let message_id = message["id"].as_str().unwrap().to_owned();
    store
        .execute(
            &worker,
            "worker-run",
            &Operation::AgentAck {
                message_id: message_id.clone(),
            },
        )
        .unwrap();
    let next = execute(json!({"op": "task_reassign", "task_id": id,
        "assignee": "worker", "reason": "Advance the revision"}));
    assert_eq!(next["revision"], 2);
    execute(json!({"op": "task_cancel", "task_id": id, "reason": "Stop unstarted work"}));
    for message in store.pending(&worker).unwrap() {
        store
            .execute(
                &worker,
                "worker-run",
                &Operation::AgentAck {
                    message_id: message.id,
                },
            )
            .unwrap();
    }
    store
        .execute_controller(
            "/project",
            &ControllerOperation::TaskArchive {
                task_id: id.clone(),
                request_id: Uuid::new_v4().to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        store
            .operator_attention_page("/project", None, None, true, None, None)
            .unwrap()["counts"]["reconcile"],
        1
    );
    (worker, id, message_id)
}

#[test]
fn read_overtaken_followup_blocks_task_delete_until_reconciled() {
    let store = Store::open(":memory:").unwrap();
    let (worker, id, message_id) = cancelled_read_followup(&store);
    let version = store.operator_task("/project", &id).unwrap().version;
    let delete = || ControllerOperation::TaskDelete {
        task_id: id.clone(),
        expected_version: version,
        request_id: Uuid::new_v4().to_string(),
    };
    let error = store.execute_controller("/project", &delete()).unwrap_err();
    assert_eq!(
        error.downcast_ref::<crate::DomainError>().unwrap().code,
        "invalid_state"
    );
    assert_eq!(store.purge_preview("/project").unwrap()["tasks"], 0);
    assert_eq!(
        store
            .visible_message(&worker, &message_id)
            .unwrap()
            .acknowledged,
        true
    );
    store
        .execute(
            &Store::operator("/project"),
            OPERATOR_EPOCH,
            &Operation::TaskFollowupReconcile {
                task_id: id.clone(),
                message_id: message_id.clone(),
                expected_version: version,
                note: "Explicitly end the old instruction".into(),
                request_id: Uuid::new_v4().to_string(),
            },
        )
        .unwrap();
    assert_eq!(store.purge_preview("/project").unwrap()["tasks"], 1);
    assert_eq!(
        store.execute_controller("/project", &delete()).unwrap()["deleted_task"],
        id
    );
    assert!(store.visible_message(&worker, &message_id).is_err());
}

#[test]
fn read_overtaken_followup_survives_history_purge_until_final_result() {
    let store = Store::open(":memory:").unwrap();
    let (worker, id, message_id) = cancelled_read_followup(&store);
    let purge = || ControllerOperation::HistoryPurge {
        expected_sequence: None,
        archived_tasks: true,
        acknowledged_messages: true,
        request_id: Uuid::new_v4().to_string(),
    };
    assert_eq!(store.purge_preview("/project").unwrap()["tasks"], 0);
    assert_eq!(
        store.execute_controller("/project", &purge()).unwrap()["purged_tasks"],
        json!([])
    );
    assert!(store.operator_task("/project", &id).is_ok());
    assert_eq!(
        store
            .visible_message(&worker, &message_id)
            .unwrap()
            .acknowledged,
        true
    );
    let result = store
        .execute(
            &worker,
            "worker-run",
            &operation(json!({
                "op": "agent_reply", "reply_to": message_id, "body": "Stopped without executing",
                "outcome": "failed",
            })),
        )
        .unwrap();
    store
        .execute(
            &Store::operator("/project"),
            OPERATOR_EPOCH,
            &Operation::AgentAck {
                message_id: result["id"].as_str().unwrap().into(),
            },
        )
        .unwrap();
    assert_eq!(store.purge_preview("/project").unwrap()["tasks"], 1);
    assert_eq!(
        store.execute_controller("/project", &purge()).unwrap()["purged_tasks"],
        json!([id])
    );
    assert!(store.visible_message(&worker, &message_id).is_err());
}

#[test]
fn trimmed_space_pages_resume_after_private_without_skipping_real_spaces() {
    let store = Store::open(":memory:").unwrap();
    for name in ["Alpha", "Beta", "Zeta"] {
        store
            .execute_controller(
                "/project",
                &ControllerOperation::SpaceCreate {
                    name: name.into(),
                    request_id: Uuid::new_v4().to_string(),
                },
            )
            .unwrap();
    }
    let page = |cursor| {
        store
            .execute_controller(
                "/project",
                &ControllerOperation::SpaceList {
                    cursor,
                    limit: Some(50),
                },
            )
            .unwrap()
    };
    let first = page(None);
    let mut panel = json!({"spaces": first["spaces"], "space_cursor": first["cursor"],
        "task": {"result": ""}});
    // Leave room for one actual row under the production aggregate budget.
    let single_row_bytes = first["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let cursor = if row["private"] == true {
                json!("")
            } else {
                row["name"].clone()
            };
            serde_json::to_vec(&json!({"spaces": [row], "space_cursor": cursor,
            "task": {"result": ""}}))
            .unwrap()
            .len()
        })
        .max()
        .unwrap();
    panel["task"]["result"] = json!("x".repeat(crate::MAX_PANEL_RESPONSE - single_row_bytes - 8));
    let mut names = Vec::new();
    let mut private_rows = 0;
    let mut seen_cursors = HashSet::new();
    loop {
        crate::panel_budget::bound_panel(&mut panel, crate::MAX_PANEL_RESPONSE).unwrap();
        assert!(serde_json::to_vec(&panel).unwrap().len() <= crate::MAX_PANEL_RESPONSE);
        let rows = panel["spaces"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        for row in rows {
            if row["private"] == true {
                private_rows += 1;
                assert_eq!(panel["space_cursor"], "");
            } else {
                names.push(row["name"].as_str().unwrap().to_owned());
            }
        }
        let Some(cursor) = panel["space_cursor"].as_str().map(str::to_owned) else {
            break;
        };
        assert!(
            seen_cursors.insert(cursor.clone()),
            "space cursor must advance"
        );
        let next = page(Some(cursor));
        panel["spaces"] = next["spaces"].clone();
        panel["space_cursor"] = next["cursor"].clone();
    }
    assert_eq!(private_rows, 1);
    assert_eq!(names, ["Alpha", "Beta", "Zeta"]);
}
