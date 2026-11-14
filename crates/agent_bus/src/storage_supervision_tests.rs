use super::*;

fn actor(store: &Store, name: &str) -> Agent {
    store
        .register(&format!("terminal-{name}"), "claude", "/project", name)
        .unwrap()
}

fn execute(store: &Store, actor: &Agent, mut value: Value) -> Value {
    value["request_id"] = json!(Uuid::new_v4().to_string());
    let run = if actor.program == OPERATOR_PROGRAM {
        OPERATOR_EPOCH
    } else {
        "run"
    };
    store
        .execute(actor, run, &serde_json::from_value(value).unwrap())
        .unwrap()
}

fn overtaken_followup(
    store: &Store,
    sender: &Agent,
    worker: &Agent,
    expects_result: bool,
) -> (String, String) {
    let task = execute(
        store,
        sender,
        json!({"op": "task_assign", "to": worker.id,
        "description": "work", "acceptance": "proof"}),
    );
    let id = task["id"].as_str().unwrap().to_owned();
    let message = execute(
        store,
        sender,
        json!({"op": "agent_send", "to": worker.id,
        "body": "Old instruction", "task_id": id, "expects_result": expects_result}),
    );
    execute(
        store,
        sender,
        json!({"op": "task_reassign", "task_id": id,
        "assignee": worker.id, "reason": "Advance the revision"}),
    );
    (id, message["id"].as_str().unwrap().to_owned())
}

#[test]
fn reconciled_requests_retire_but_real_results_survive_reopen_until_handled() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("supervision.sqlite");
    let store = Store::open(path.to_str().unwrap()).unwrap();
    let issuer = actor(&store, "issuer");
    let worker = actor(&store, "worker");
    let (first, self_request) = overtaken_followup(&store, &issuer, &worker, true);
    let version = store.operator_task("/project", &first).unwrap().version;
    let receipt = execute(
        &store,
        &issuer,
        json!({"op": "task_followup_reconcile",
        "task_id": first, "message_id": self_request, "expected_version": version,
        "note": "Issuer explicitly ends its own request"}),
    );
    assert!(receipt["result_message_id"].is_null());
    let page = store.supervision_index(&issuer, None, None).unwrap();
    assert_eq!(page["counts"]["requests"], 0);
    assert_eq!(page["counts"]["results"], 0);

    let (second, other_request) = overtaken_followup(&store, &issuer, &worker, true);
    let version = store.operator_task("/project", &second).unwrap().version;
    let receipt = execute(
        &store,
        &Store::operator("/project"),
        json!({
        "op": "task_followup_reconcile", "task_id": second,
        "message_id": other_request, "expected_version": version,
        "note": "Operator explicitly ends the request"}),
    );
    let result_id = receipt["result_message_id"].as_str().unwrap().to_owned();
    drop(store);
    let store = Store::open(path.to_str().unwrap()).unwrap();
    let page = store.supervision_index(&issuer, None, None).unwrap();
    assert_eq!(page["counts"]["requests"], 0);
    assert_eq!(page["counts"]["results"], 1);
    let result = page["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["message_id"] == result_id)
        .unwrap();
    assert_eq!(result["actions"], json!(["process_result"]));
    store
        .execute(
            &issuer,
            "run",
            &Operation::AgentAck {
                message_id: result_id,
            },
        )
        .unwrap();
    assert_eq!(
        store.supervision_index(&issuer, None, None).unwrap()["counts"]["results"],
        0
    );
}

#[test]
fn active_coordinator_recovers_inherited_tasks_without_granting_worker_scope() {
    let store = Store::open(":memory:").unwrap();
    let previous = actor(&store, "previous");
    let successor = actor(&store, "successor");
    let worker = actor(&store, "worker");
    let bystander = actor(&store, "bystander");
    store
        .set_worktree_role(&previous, "run", "coordinator")
        .unwrap();
    let task = execute(
        &store,
        &previous,
        json!({"op": "task_assign", "to": worker.id,
        "description": "Inherited work", "acceptance": "proof"}),
    );
    assert_eq!(
        store.supervision_index(&successor, None, None).unwrap()["total"],
        0
    );
    store
        .set_worktree_role(&successor, "run", "coordinator")
        .unwrap();
    let page = store.supervision_index(&successor, None, Some(1)).unwrap();
    assert_eq!(page["total"], 1);
    assert_eq!(page["rows"][0]["task_id"], task["id"]);
    assert_eq!(page["rows"][0]["role"], "supervisor");
    assert_eq!(
        store.supervision_index(&bystander, None, None).unwrap()["total"],
        0
    );
    let unrelated = store
        .register("other-terminal", "claude", "/other", "other")
        .unwrap();
    assert_eq!(
        store.supervision_index(&unrelated, None, None).unwrap()["total"],
        0
    );
    store.worktree_offline(&successor.id, "run").unwrap();
    assert_eq!(
        store.supervision_index(&successor, None, None).unwrap()["total"],
        0
    );
    assert_eq!(
        store.supervision_index(&previous, None, None).unwrap()["total"],
        1
    );
}

#[test]
fn archived_tasks_remain_in_recovery_only_until_their_old_followup_is_ended() {
    let store = Store::open(":memory:").unwrap();
    let issuer = actor(&store, "issuer");
    let worker = actor(&store, "worker");
    let (id, message_id) = overtaken_followup(&store, &issuer, &worker, false);
    store
        .execute(
            &worker,
            "run",
            &Operation::AgentAck {
                message_id: message_id.clone(),
            },
        )
        .unwrap();
    execute(
        &store,
        &issuer,
        json!({"op": "task_cancel", "task_id": id, "reason": "Stop unstarted work"}),
    );
    store
        .execute_controller(
            "/project",
            &ControllerOperation::TaskArchive {
                task_id: id.clone(),
                request_id: Uuid::new_v4().to_string(),
            },
        )
        .unwrap();
    let page = store.supervision_index(&issuer, None, None).unwrap();
    assert_eq!(page["total"], 1);
    assert_eq!(page["rows"][0]["task_id"], id);
    assert_eq!(page["rows"][0]["actions"], json!(["reconcile"]));
    let version = store.operator_task("/project", &id).unwrap().version;
    execute(
        &store,
        &issuer,
        json!({"op": "task_followup_reconcile", "task_id": id,
        "message_id": message_id, "expected_version": version, "note": "Explicitly ended"}),
    );
    assert_eq!(
        store.supervision_index(&issuer, None, None).unwrap()["total"],
        0
    );
}

#[test]
fn standalone_questions_are_visible_only_to_the_recipient_until_answered() {
    let store = Store::open(":memory:").unwrap();
    let issuer = actor(&store, "issuer");
    let worker = actor(&store, "worker");
    let bystander = actor(&store, "bystander");
    let request = execute(
        &store,
        &issuer,
        json!({"op": "agent_send", "to": worker.id,
        "body": "Inspect the contract", "expects_result": true}),
    );
    let question = execute(
        &store,
        &worker,
        json!({"op": "agent_ask",
        "reply_to": request["id"], "question": "Which contract version?"}),
    );
    let events = store
        .count("SELECT COUNT(*) AS count FROM events", &[])
        .unwrap();
    let page = store.supervision_index(&issuer, None, Some(1)).unwrap();
    assert_eq!(page["counts"]["questions"], 1);
    assert_eq!(page["counts"]["requests"], 1);
    let next = store
        .supervision_index(&issuer, page["cursor"].as_u64(), Some(1))
        .unwrap();
    assert_eq!(next["rows"][0]["message_id"], question["id"]);
    assert_eq!(next["rows"][0]["actions"], json!(["answer"]));
    assert_eq!(next["rows"][0]["body"], "Which contract version?");
    assert_eq!(next["rows"][0]["role"], "recipient");
    assert!(next["cursor"].is_null());
    assert_eq!(
        store.supervision_index(&bystander, None, None).unwrap()["total"],
        0
    );
    assert_eq!(
        store
            .count("SELECT COUNT(*) AS count FROM events", &[])
            .unwrap(),
        events
    );
    execute(
        &store,
        &issuer,
        json!({"op": "agent_answer", "question_id": question["id"],
        "answer": "Use version two"}),
    );
    let answered = store.supervision_index(&issuer, None, None).unwrap();
    assert_eq!(answered["counts"]["questions"], 0);
    assert_eq!(answered["counts"]["requests"], 1);
    assert_eq!(answered["total"], 1);
}
