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
