//! Native SSH and WSL fixture that exercises the gated private wire.
use remote_protocol::proto::*;
use remote_protocol::protocol::{
    read_message_with_limit, write_message_with_limit, MAX_MANAGED_MESSAGE_SIZE,
};
use tokio::{
    process::Command,
    time::{sleep, timeout, Duration, Instant},
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use uuid::Uuid;

struct Bridge {
    child: tokio::process::Child,
    input: tokio_util::compat::Compat<tokio::process::ChildStdin>,
    output: tokio_util::compat::Compat<tokio::process::ChildStdout>,
}

impl Bridge {
    fn connect(mut command: Command) -> Self {
        use std::process::Stdio;
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .expect("Controlled transport process");
        let input = child.stdin.take().unwrap().compat_write();
        let output = child.stdout.take().unwrap().compat();
        Self {
            child,
            input,
            output,
        }
    }

    async fn call(&mut self, operation: managed_request::Operation) -> managed_response::Result {
        let request_id = Uuid::new_v4().to_string();
        let request = ClientMessage {
            request_id: request_id.clone(),
            message: Some(client_message::Message::Managed(ManagedRequest {
                operation: Some(operation),
            })),
        };
        let reply = timeout(Duration::from_secs(20), async {
            write_message_with_limit(&mut self.input, &request, MAX_MANAGED_MESSAGE_SIZE).await?;
            read_message_with_limit::<ServerMessage>(&mut self.output, MAX_MANAGED_MESSAGE_SIZE)
                .await
        })
        .await
        .expect("SSH request deadline")
        .expect("Controlled guardian reply");
        assert_eq!(reply.request_id, request_id);
        let Some(server_message::Message::Managed(reply)) = reply.message else {
            panic!("Expected managed reply");
        };
        reply.result.unwrap()
    }

    async fn open(&mut self, root: &str) -> (ManagedFence, AccountFence) {
        let managed_response::Result::Initialized(init) = self
            .call(managed_request::Operation::Initialize(ManagedInitialize {
                protocol_major: remote_protocol::managed::PROTOCOL_MAJOR,
            }))
            .await
        else {
            panic!("Guardian handshake required");
        };
        let account = init.account_fence.expect("Guardian account admission");
        assert!(!init
            .capabilities
            .iter()
            .any(|cap| cap == "guarded_remote_sessions_v1"));
        let managed_response::Result::ProjectOpened(opened) = self
            .call(managed_request::Operation::ProjectOpen(ProjectOpen {
                fence: init.fence,
                root: root.into(),
            }))
            .await
        else {
            panic!("Fixture project must open");
        };
        (opened.fence.unwrap(), account)
    }

    async fn terminal(&mut self, request: TerminalControl) -> TerminalState {
        match self
            .call(managed_request::Operation::TerminalControl(request))
            .await
        {
            managed_response::Result::TerminalState(state) => state,
            other => panic!("Expected terminal state, got {other:?}"),
        }
    }
}

#[cfg(windows)]
pub async fn verify_same_guardian(root: &str, first: Command, second: Command) {
    let mut first = Bridge::connect(first);
    let (first_fence, first_owner) = first.open(root).await;
    eprintln!("Guardian fixture: first route admitted; opening second route");
    let mut second = Bridge::connect(second);
    let (second_fence, second_owner) = second.open(root).await;
    assert_eq!(first_owner.guardian_id, second_owner.guardian_id);
    assert_eq!(first_owner.boot_id, second_owner.boot_id);
    assert_ne!(first_fence.service_id, second_fence.service_id);
    assert_ne!(first_fence.project_id, second_fence.project_id);
}

pub async fn verify_guarded_reconnect(
    root: &str,
    bridge_gap: Duration,
    command: impl Fn() -> Command,
    tty_resume: Option<&dyn Fn(&str) -> Command>,
) {
    let mut first = Bridge::connect(command());
    let (fence, _) = first.open(root).await;
    let session_id = Uuid::new_v4().to_string();
    let managed_response::Result::TerminalState(launched) = first.call(
        managed_request::Operation::TerminalLaunch(TerminalLaunch {
            fence: Some(fence.clone()),
            session_id: session_id.clone(),
            executable: "/bin/sh".into(),
            arguments: vec!["-c".into(), "printf 'GUARDIAN_PID=%s\\n' \"$$\"; while IFS= read -r l; do printf 'GUARDIAN_ECHO=%s\\n' \"$l\"; done".into()],
            columns: 80,
            rows: 24,
            guarded: true,
            session_kind: Some(TerminalSessionKind::Command as i32),
            ..Default::default()
        })
    ).await else {
        panic!("Guarded native launch required");
    };
    let request =
        |fence: &ManagedFence, state: &TerminalState, action: TerminalAction| TerminalControl {
            fence: Some(fence.clone()),
            session_id: session_id.clone(),
            run_id: state.run_id.clone(),
            action: action as i32,
            attachment_id: state.attachment_id.clone(),
            attachment_generation: state.attachment_generation,
            ..Default::default()
        };
    let deadline = Instant::now() + Duration::from_secs(10);
    let pid = loop {
        let state = first
            .terminal(request(&fence, &launched, TerminalAction::TerminalRead))
            .await;
        let output = String::from_utf8_lossy(&state.output);
        if let Some(pid) = output.lines().find_map(|line| {
            line.strip_prefix("GUARDIAN_PID=")?
                .trim()
                .parse::<u32>()
                .ok()
        }) {
            break pid;
        }
        assert!(Instant::now() < deadline);
        sleep(Duration::from_millis(20)).await;
    };
    first.child.kill().await.unwrap();
    drop(first);
    sleep(bridge_gap).await;
    if let Some(resume) = tty_resume {
        verify_cli_tty(resume(&session_id), pid).await;
    }

    let mut second = Bridge::connect(command());
    let (rejoined, _) = second.open(root).await;
    assert_eq!(rejoined.service_id, fence.service_id);
    assert_eq!(rejoined.service_boot_id, fence.service_boot_id);
    assert_eq!(rejoined.project_id, fence.project_id);
    assert_ne!(rejoined.connection_id, fence.connection_id);
    let mut attach = request(&rejoined, &launched, TerminalAction::TerminalAttach);
    attach.attachment_id.clear();
    attach.attachment_generation = 0;
    let deadline = Instant::now() + Duration::from_secs(10);
    let attached = loop {
        match second
            .call(managed_request::Operation::TerminalControl(attach.clone()))
            .await
        {
            managed_response::Result::TerminalState(state) => break state,
            managed_response::Result::Error(error)
                if error.code == ManagedErrorCode::ManagedConflict as i32
                    && Instant::now() < deadline =>
            {
                sleep(Duration::from_millis(20)).await
            }
            other => panic!("The original run must accept a new writer: {other:?}"),
        }
    };
    assert_eq!(attached.run_id, launched.run_id);
    assert!(attached.attachment_generation > launched.attachment_generation);
    let mut resize = request(&rejoined, &attached, TerminalAction::TerminalResize);
    resize.columns = 100;
    resize.rows = 30;
    resize.resize_sequence = 1;
    assert_eq!(
        second
            .terminal(resize.clone())
            .await
            .accepted_resize_sequence,
        1
    );
    assert_eq!(
        second
            .terminal(resize.clone())
            .await
            .accepted_resize_sequence,
        1
    );
    resize.columns = 101;
    let conflict = second
        .call(managed_request::Operation::TerminalControl(resize))
        .await;
    assert!(matches!(conflict, managed_response::Result::Error(error)
        if error.code == ManagedErrorCode::ManagedConflict as i32));

    let stale = second
        .call(managed_request::Operation::TerminalControl(request(
            &rejoined,
            &launched,
            TerminalAction::TerminalRead,
        )))
        .await;
    assert!(matches!(stale, managed_response::Result::Error(error)
        if error.code == ManagedErrorCode::ManagedStaleAttachment as i32));

    let mut input = request(&rejoined, &attached, TerminalAction::TerminalInput);
    input.input_sequence = 1;
    input.input = b"after-reconnect\r".to_vec();
    second.terminal(input).await;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let state = second
            .terminal(request(&rejoined, &attached, TerminalAction::TerminalRead))
            .await;
        let output = String::from_utf8_lossy(&state.output);
        if output.contains("GUARDIAN_ECHO=after-reconnect") {
            assert!(output.contains(&format!("GUARDIAN_PID={pid}")));
            break;
        }
        assert!(Instant::now() < deadline);
        sleep(Duration::from_millis(20)).await;
    }
    let stopped = second
        .terminal(request(&rejoined, &attached, TerminalAction::TerminalStop))
        .await;
    assert!(stopped.stop_requested);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let state = second
            .terminal(request(&rejoined, &attached, TerminalAction::TerminalRead))
            .await;
        if state.output_closed && state.processes_active == Some(false) {
            assert!(state.exit_code.is_some());
            break;
        }
        assert!(Instant::now() < deadline);
        sleep(Duration::from_millis(20)).await;
    }
}

async fn verify_cli_tty(mut command: Command, pid: u32) {
    use std::process::Stdio;
    use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader};

    async fn expect_line(output: &mut (impl AsyncBufRead + Unpin), expected: &str) {
        timeout(Duration::from_secs(10), async {
            let mut line = String::new();
            loop {
                line.clear();
                assert_ne!(output.read_line(&mut line).await.unwrap(), 0, "Missing {expected}");
                if let Some(reason) = line.trim_end().strip_prefix("GUARDIAN_CLI_TTY_ERROR=") {
                    panic!("Native CLI TTY fixture failed: {reason}");
                }
                if line.trim_end() == expected { break; }
            }
        }).await.unwrap_or_else(|_| panic!("TTY deadline waiting for {expected}"));
    }

    let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped())
        .kill_on_drop(true).spawn().expect("Owned SSH TTY");
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    expect_line(&mut output, &format!("GUARDIAN_PID={pid}")).await;
    input.write_all(b"through-cli\r").await.unwrap();
    expect_line(&mut output, "GUARDIAN_ECHO=through-cli").await;
    input.write_all(b"\x02").await.unwrap();
    sleep(Duration::from_millis(100)).await;
    input.write_all(b"q").await.unwrap();
    expect_line(&mut output, "GUARDIAN_CLI_TTY_RESTORED").await;
    assert!(timeout(Duration::from_secs(10), child.wait()).await.unwrap().unwrap().success());
}

#[cfg(unix)]
pub async fn verify_detached_agent_delivery(root: &str, executable: &str, command: impl Fn() -> Command) {
    use warp_agent_bus::Operation;

    fn control(fence: &ManagedFence, run: &TerminalState, action: TerminalAction) -> TerminalControl {
        TerminalControl {
            fence: Some(fence.clone()),
            session_id: run.session_id.clone(),
            run_id: run.run_id.clone(),
            action: action as i32,
            attachment_id: run.attachment_id.clone(),
            attachment_generation: run.attachment_generation,
            ..Default::default()
        }
    }

    async fn output(bridge: &mut Bridge, fence: &ManagedFence, run: &TerminalState, marker: &str) -> String {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let state = bridge.terminal(control(fence, run, TerminalAction::TerminalRead)).await;
            let output = String::from_utf8_lossy(&state.output);
            if output.contains(marker) { return output.into_owned(); }
            assert!(Instant::now() < deadline, "Native Agent never emitted {marker}");
            sleep(Duration::from_millis(20)).await;
        }
    }

    async fn launch(bridge: &mut Bridge, fence: &ManagedFence, executable: &str) -> TerminalState {
        let result = bridge.call(managed_request::Operation::TerminalLaunch(TerminalLaunch {
            fence: Some(fence.clone()),
            session_id: Uuid::new_v4().to_string(),
            executable: executable.into(),
            arguments: ["--exact", "managed_agent_child", "--ignored", "--nocapture"].map(str::to_owned).to_vec(),
            columns: 1000,
            rows: 24,
            guarded: true,
            agent_program: Some("fixture".into()),
            session_kind: Some(TerminalSessionKind::Agent as i32),
            ..Default::default()
        })).await;
        let managed_response::Result::TerminalState(run) = result else {
            panic!("Guarded Agent launch failed: {result:?}");
        };
        output(bridge, fence, &run, &format!("MCP_NATIVE_RUN={}", run.run_id)).await;
        run
    }

    async fn operation(bridge: &mut Bridge, fence: &ManagedFence, run: &mut TerminalState, sequence: u64, operation: Operation) -> serde_json::Value {
        *run = bridge.terminal(TerminalControl {
            input: format!("{}\r", serde_json::to_string(&operation).unwrap()).into_bytes(),
            input_sequence: run.accepted_input_sequence + 1,
            ..control(fence, run, TerminalAction::TerminalInput)
        }).await;
        let marker = format!("FIXTURE_RESULT_{sequence}=");
        let output = output(bridge, fence, run, &marker).await;
        serde_json::from_str(output.split_once(&marker).unwrap().1.lines().next().unwrap()).unwrap()
    }

    async fn attach(bridge: &mut Bridge, fence: &ManagedFence, run: &TerminalState) -> TerminalState {
        let attached = bridge.terminal(TerminalControl {
            attachment_id: String::new(),
            attachment_generation: 0,
            ..control(fence, run, TerminalAction::TerminalAttach)
        }).await;
        assert_eq!(attached.run_id, run.run_id);
        attached
    }

    let root = tempfile::tempdir_in(root).unwrap();
    let path = root.path().to_str().unwrap();
    let mut sender = Bridge::connect(command());
    let (sender_fence, account) = sender.open(path).await;
    let mut sender_run = launch(&mut sender, &sender_fence, executable).await;
    let mut receiver = Bridge::connect(command());
    let (receiver_fence, _) = receiver.open(path).await;
    let mut receiver_run = launch(&mut receiver, &receiver_fence, executable).await;
    let ready = operation(&mut receiver, &receiver_fence, &mut receiver_run, 1, Operation::AgentReady).await;
    assert_eq!(ready["ready"], true);

    receiver.child.kill().await.unwrap();
    drop(receiver);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let result = sender.call(managed_request::Operation::AccountCatalogInspect(AccountCatalogInspect {
            fence: Some(account.clone()),
            session_id: receiver_run.session_id.clone(),
            run_id: receiver_run.run_id.clone(),
        })).await;
        let managed_response::Result::AccountCatalogRow(row) = result else {
            panic!("Guarded Agent catalog inspect failed: {result:?}");
        };
        assert_eq!(row.state, "live");
        if row.writer_attached == Some(false) { break; }
        assert!(Instant::now() < deadline, "Receiver bridge did not release its writer");
        sleep(Duration::from_millis(20)).await;
    }

    let sent = operation(&mut sender, &sender_fence, &mut sender_run, 1, Operation::AgentSend {
        to: format!("fixture-{}", receiver_run.session_id),
        body: "Remote fixture message".into(),
        content: None,
        expects_result: false,
        subject: None,
        thread_id: None,
        reply_to: None,
        task_id: None,
        request_id: Uuid::new_v4().to_string(),
    }).await;
    let id = Uuid::parse_str(sent["id"].as_str().unwrap()).unwrap();
    let receipt = root.path().join(format!("agent-wake-{}-{id}", receiver_run.session_id));
    let deadline = Instant::now() + Duration::from_secs(10);
    while std::fs::read_to_string(&receipt).as_deref().ok() != Some(receiver_run.run_id.as_str()) {
        assert!(Instant::now() < deadline, "The original Agent did not consume the wake while detached");
        sleep(Duration::from_millis(20)).await;
    }

    let mut receiver = Bridge::connect(command());
    let (rejoined, _) = receiver.open(path).await;
    assert_eq!(rejoined.service_boot_id, receiver_fence.service_boot_id);
    assert_ne!(rejoined.connection_id, receiver_fence.connection_id);
    let mut attached = attach(&mut receiver, &rejoined, &receiver_run).await;
    let retained = output(&mut receiver, &rejoined, &attached, "PEER_NOTIFICATION_RECEIVED").await;
    assert_eq!(retained.matches("PEER_NOTIFICATION_RECEIVED").count(), 1);
    let inbox = operation(&mut receiver, &rejoined, &mut attached, 2, Operation::AgentInbox { cursor: None, limit: None }).await;
    assert!(inbox["messages"].as_array().unwrap().iter().any(|message| message["id"] == id.to_string()));

    operation(&mut receiver, &rejoined, &mut attached, 3, Operation::AgentAck { message_id: id.to_string() }).await;
    let ready = operation(&mut receiver, &rejoined, &mut attached, 4, Operation::AgentReady).await;
    assert_eq!(ready["ready"], true);
    receiver.terminal(control(&rejoined, &attached, TerminalAction::TerminalDetach)).await;
    receiver.child.kill().await.unwrap();
    drop(receiver);

    let assigned = operation(&mut sender, &sender_fence, &mut sender_run, 2, Operation::TaskAssign {
        to: format!("fixture-{}", receiver_run.session_id),
        description: "Owned detached task fixture".into(),
        acceptance: "Commit a task receipt before any bridge is reopened".into(),
        context_refs: vec![], reviewer: None, dependencies: vec![], start_deadline: None,
        execution_timeout_seconds: None, review_timeout_seconds: None, request_id: Uuid::new_v4().to_string(),
    }).await;
    let task = Uuid::parse_str(assigned["id"].as_str().unwrap()).unwrap();
    assert_eq!(assigned["state"], "queued");
    sender.terminal(control(&sender_fence, &sender_run, TerminalAction::TerminalDetach)).await;
    sender.child.kill().await.unwrap();
    drop(sender);
    std::fs::write(root.path().join("owned-bridges-closed"), b"").unwrap();
    let receipt = root.path().join(format!("agent-task-{}-{task}", receiver_run.session_id));
    let deadline = Instant::now() + Duration::from_secs(10);
    let receipt: serde_json::Value = loop {
        if let Some(receipt) = std::fs::read(&receipt).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()) {
            break receipt;
        }
        assert!(Instant::now() < deadline, "The original Agent did not submit the tracked task with both bridges closed");
        sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(receipt["run"], receiver_run.run_id);
    let attempt = Uuid::parse_str(receipt["attempt"].as_str().unwrap()).unwrap();

    let mut sender = Bridge::connect(command());
    let (sender_fence, _) = sender.open(path).await;
    let mut sender_run = attach(&mut sender, &sender_fence, &sender_run).await;
    let submitted = operation(&mut sender, &sender_fence, &mut sender_run, 3, Operation::TaskGet { task_id: task.to_string() }).await;
    assert_eq!(submitted["state"], "submitted");
    assert_eq!(submitted["attempt_id"], attempt.to_string());
    let mut receiver = Bridge::connect(command());
    let (rejoined, _) = receiver.open(path).await;
    let attached = attach(&mut receiver, &rejoined, &attached).await;
    for (bridge, fence, run) in [(&mut sender, &sender_fence, &sender_run), (&mut receiver, &rejoined, &attached)] {
        assert!(bridge.terminal(control(fence, run, TerminalAction::TerminalStop)).await.stop_requested);
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let state = bridge.terminal(control(fence, run, TerminalAction::TerminalRead)).await;
            if state.output_closed && state.processes_active == Some(false) {
                assert!(state.exit_code.is_some());
                break;
            }
            assert!(Instant::now() < deadline, "Guarded Agent must exit and drain before the next account test");
            sleep(Duration::from_millis(20)).await;
        }
    }
}

#[cfg(unix)]
pub async fn verify_cli_stop_survives_caller(root: &str, helper: &str, command: Command) {
    let mut bridge = Bridge::connect(command);
    let (fence, _) = bridge.open(root).await;
    let mut runs = Vec::new();
    let mut session = *Uuid::new_v4().as_bytes();
    for (order, script) in [
        "IFS= read -r trigger; exec \"$1\" ctl stop",
        "while IFS= read -r line; do :; done",
    ].into_iter().enumerate() {
        // The registry dispatches in ID order; terminate the caller first.
        session[15] = order as u8;
        let result = bridge.call(managed_request::Operation::TerminalLaunch(TerminalLaunch {
            fence: Some(fence.clone()),
            session_id: Uuid::from_bytes(session).to_string(),
            executable: "/bin/sh".into(),
            arguments: vec!["-c".into(), script.into(), "warpai-fixture".into(), helper.into()],
            columns: 80,
            rows: 24,
            guarded: true,
            session_kind: Some(TerminalSessionKind::Command as i32),
            ..Default::default()
        })).await;
        let managed_response::Result::TerminalState(state) = result else {
            panic!("Stop fixture launch failed: {result:?}");
        };
        assert_eq!(state.processes_active, Some(true));
        runs.push(state);
    }
    let request = |state: &TerminalState, action: TerminalAction| TerminalControl {
        fence: Some(fence.clone()),
        session_id: state.session_id.clone(),
        run_id: state.run_id.clone(),
        action: action as i32,
        attachment_id: state.attachment_id.clone(),
        attachment_generation: state.attachment_generation,
        ..Default::default()
    };
    bridge.terminal(TerminalControl {
        input: b"stop\r".to_vec(),
        input_sequence: 1,
        ..request(&runs[0], TerminalAction::TerminalInput)
    }).await;
    let deadline = Instant::now() + Duration::from_secs(10);
    for run in runs {
        loop {
            let state = bridge.terminal(request(&run, TerminalAction::TerminalRead)).await;
            if state.output_closed && state.processes_active == Some(false) {
                assert!(state.exit_code.is_some());
                break;
            }
            assert!(Instant::now() < deadline, "CLI stop must finish both captured runs");
            sleep(Duration::from_millis(20)).await;
        }
    }
}
