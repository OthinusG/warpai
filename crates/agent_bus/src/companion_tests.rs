use super::*;

fn file_attachment(root: &Path, state: &Path) -> (Companion, ManagedFence) {
    let mut companion = Companion::new(state).unwrap();
    let managed_response::Result::Initialized(init) = result(companion.handle(request(
        managed_request::Operation::Initialize(ManagedInitialize {
            protocol_major: PROTOCOL_MAJOR,
        }),
    ))) else {
        panic!("Initialize")
    };
    let managed_response::Result::ProjectOpened(opened) = result(companion.handle(request(
        managed_request::Operation::ProjectOpen(ProjectOpen {
            fence: init.fence,
            root: root.to_str().unwrap().into(),
        }),
    ))) else {
        panic!("Project open")
    };
    (companion, opened.fence.unwrap())
}

#[test]
fn project_files_stage_content_and_refuse_changed_original_or_wrong_upload() {
    use sha2::{Digest, Sha256};
    let root = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("notes.md"), b"original").unwrap();
    let (mut companion, fence) = file_attachment(root.path(), &state.path().join("private"));
    let query = |action, path: &str| ProjectFilesRequest {
        fence: Some(fence.clone()),
        query_generation: 4,
        action: action as i32,
        path: path.into(),
        ..Default::default()
    };
    let managed_response::Result::ProjectFiles(read) = result(companion.handle(request(
        managed_request::Operation::ProjectFiles(query(
            ProjectFileAction::ProjectFilePrepareRead,
            "notes.md",
        )),
    ))) else {
        panic!("Read snapshot")
    };
    assert_eq!(std::fs::read(&read.transfer_path).unwrap(), b"original");
    assert_eq!(read.sha256, format!("{:x}", Sha256::digest(b"original")));
    let mut prepare = query(ProjectFileAction::ProjectFilePrepareWrite, "notes.md");
    prepare.expected_hash = read.sha256;
    let managed_response::Result::ProjectFiles(write) =
        result(companion.handle(request(managed_request::Operation::ProjectFiles(prepare))))
    else {
        panic!("Write staging")
    };
    std::fs::write(&write.transfer_path, b"edited").unwrap();
    let mut commit = query(ProjectFileAction::ProjectFileCommitWrite, "notes.md");
    commit.transfer_id = write.transfer_id;
    commit.expected_hash = format!("{:x}", Sha256::digest(b"wrong-upload"));
    assert!(
        matches!(result(companion.handle(request(managed_request::Operation::ProjectFiles(commit.clone())))), managed_response::Result::Error(ManagedError { code }) if code == ManagedErrorCode::ManagedConflict as i32)
    );
    assert_eq!(
        std::fs::read(root.path().join("notes.md")).unwrap(),
        b"original"
    );
    std::fs::write(root.path().join("notes.md"), b"external edit").unwrap();
    commit.expected_hash = format!("{:x}", Sha256::digest(b"edited"));
    assert!(
        matches!(result(companion.handle(request(managed_request::Operation::ProjectFiles(commit.clone())))), managed_response::Result::Error(ManagedError { code }) if code == ManagedErrorCode::ManagedConflict as i32)
    );
    std::fs::write(root.path().join("notes.md"), b"original").unwrap();
    assert!(matches!(
        result(companion.handle(request(managed_request::Operation::ProjectFiles(commit)))),
        managed_response::Result::ProjectFiles(_)
    ));
    assert_eq!(
        std::fs::read(root.path().join("notes.md")).unwrap(),
        b"edited"
    );
    assert!(!Path::new(&write.transfer_path).exists());
    drop(companion);
    assert!(!Path::new(&read.transfer_path).exists());
}

#[test]
fn project_file_tree_and_mutations_are_root_scoped() {
    let root = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let (mut companion, fence) = file_attachment(root.path(), &state.path().join("private"));
    let query = |action, path: &str| ProjectFilesRequest {
        fence: Some(fence.clone()),
        action: action as i32,
        path: path.into(),
        ..Default::default()
    };
    for path in ["../outside", "/outside", "a\ncommand"] {
        assert!(matches!(
            result(
                companion.handle(request(managed_request::Operation::ProjectFiles(query(
                    ProjectFileAction::ProjectFileCreate,
                    path
                ))))
            ),
            managed_response::Result::Error(_)
        ));
    }
    assert!(matches!(
        result(
            companion.handle(request(managed_request::Operation::ProjectFiles(query(
                ProjectFileAction::ProjectDirectoryCreate,
                "data 多语言"
            ))))
        ),
        managed_response::Result::ProjectFiles(_)
    ));
    assert!(matches!(
        result(
            companion.handle(request(managed_request::Operation::ProjectFiles(query(
                ProjectFileAction::ProjectFileCreate,
                "data 多语言/a.txt"
            ))))
        ),
        managed_response::Result::ProjectFiles(_)
    ));
    let managed_response::Result::ProjectFiles(list) = result(companion.handle(request(
        managed_request::Operation::ProjectFiles(query(ProjectFileAction::ProjectFileList, "")),
    ))) else {
        panic!("Tree")
    };
    assert_eq!(list.snapshot.unwrap().entries[0].subtree_metadata.len(), 1);
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();
        assert!(matches!(
            result(
                companion.handle(request(managed_request::Operation::ProjectFiles(query(
                    ProjectFileAction::ProjectFileCreate,
                    "escape/a.txt"
                ))))
            ),
            managed_response::Result::Error(_)
        ));
        assert!(!outside.path().join("a.txt").exists());
    }
    assert!(matches!(
        result(
            companion.handle(request(managed_request::Operation::ProjectFiles(query(
                ProjectFileAction::ProjectFileDelete,
                "data 多语言"
            ))))
        ),
        managed_response::Result::ProjectFiles(_)
    ));
    assert!(!root.path().join("data 多语言").exists());
}

fn request(operation: managed_request::Operation) -> ClientMessage {
    ClientMessage {
        request_id: Uuid::new_v4().to_string(),
        message: Some(client_message::Message::Managed(ManagedRequest {
            operation: Some(operation),
        })),
    }
}

fn result(reply: ServerMessage) -> managed_response::Result {
    let Some(server_message::Message::Managed(reply)) = reply.message else {
        panic!("Expected managed reply")
    };
    reply.result.unwrap()
}

#[test]
fn companion_opens_native_project_and_fences_every_attachment() {
    let root = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut companion = Companion::new(&state.path().join("companion-state")).unwrap();
    let init = || {
        request(managed_request::Operation::Initialize(ManagedInitialize {
            protocol_major: PROTOCOL_MAJOR,
        }))
    };
    let managed_response::Result::Initialized(initialized) = result(companion.handle(init()))
    else {
        panic!("Expected handshake")
    };
    assert_eq!(
        initialized.capabilities,
        [
            "project_open",
            "managed_agent",
            #[cfg(feature = "wsl_companion")]
            "managed_launch_cwd",
            "project_tasks",
            "context_resources",
            "structured_task_messages",
            "execution_threads",
            "project_memory", "project_supervision", "handoff_briefs_v1", "followup_reconcile",
            "project_mcp",
            #[cfg(not(feature = "wsl_companion"))]
            "native_mcp_setup",
            "project_files",
            #[cfg(feature = "wsl_companion")]
            "project_file_chunks",
            "native_screen_activity",
            "project_git_review",
            "worktree_collaboration",
            "worktree_orchestration",
        ]
    );
    let mut fence = initialized.fence.unwrap();
    assert!(matches!(
        result(companion.handle(init())),
        managed_response::Result::Error(_)
    ));
    let managed_response::Result::ProjectOpened(opened) = result(companion.handle(request(
        managed_request::Operation::ProjectOpen(ProjectOpen {
            fence: Some(fence.clone()),
            root: root.path().to_str().unwrap().into(),
        }),
    ))) else {
        panic!("Expected project")
    };
    assert_eq!(
        Path::new(&opened.canonical_root),
        root.path().canonicalize().unwrap()
    );
    fence = opened.fence.unwrap();
    let legacy = ClientMessage {
        request_id: Uuid::new_v4().to_string(),
        message: Some(client_message::Message::RunCommand(RunCommandRequest {
            command: "must-never-execute".into(),
            ..Default::default()
        })),
    };
    assert!(
        matches!(result(companion.handle(legacy)), managed_response::Result::Error(ManagedError { code })
        if code == i32::from(ManagedErrorCode::ManagedFeatureUnavailable))
    );
    let malformed = ClientMessage {
        request_id: "\u{1b}[untrusted".into(),
        message: None,
    };
    assert!(companion.handle(malformed).request_id.is_empty());
    #[cfg(unix)]
    {
        let renamed = root.path().with_extension("replaced");
        std::fs::rename(root.path(), &renamed).unwrap();
        std::fs::create_dir(root.path()).unwrap();
        assert!(
            matches!(result(companion.handle(request(managed_request::Operation::ProjectTasks(ProjectTasksRequest { fence: Some(fence), query_generation: 1, command_json: serde_json::to_vec(&TaskCommand::Panel(crate::transport::PanelQuery::default())).unwrap() })))), managed_response::Result::Error(ManagedError { code })
            if code == i32::from(ManagedErrorCode::ManagedStaleAttachment))
        );
        std::fs::remove_dir(renamed).unwrap();
    }
}

#[test]
fn ssh_route_hides_wsl_capabilities_and_rejects_guest_binding() {
    let root = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut companion = Companion::new(&state.path().join("remote-data")).unwrap();
    companion.route = Some(Route::Ssh);
    let managed_response::Result::Initialized(initialized) = result(companion.handle(request(
        managed_request::Operation::Initialize(ManagedInitialize {
            protocol_major: PROTOCOL_MAJOR,
        }),
    ))) else {
        panic!("Expected handshake")
    };
    assert!(!initialized
        .capabilities
        .iter()
        .any(|cap| cap == "managed_launch_cwd" || cap == "project_file_chunks"));
    let managed_response::Result::ProjectOpened(opened) = result(companion.handle(request(
        managed_request::Operation::ProjectOpen(ProjectOpen {
            fence: initialized.fence,
            root: root.path().to_str().unwrap().into(),
        }),
    ))) else {
        panic!("Expected project")
    };
    let fence = opened.fence.unwrap();
    let bind = |fence| {
        request(managed_request::Operation::GuestMcpBind(GuestMcpBind {
            fence: Some(fence),
            agent_pid: std::process::id(),
            program: "codex".into(),
        }))
    };
    assert!(matches!(
        result(companion.handle(bind(fence.clone()))),
        managed_response::Result::Error(ManagedError { code })
            if code == i32::from(ManagedErrorCode::ManagedFeatureUnavailable)
    ));
}

#[test]
fn project_tasks_require_current_native_project_before_any_store_access() {
    let root = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let directory = state.path().join("remote-data");
    let mut companion = Companion::new(&directory).unwrap();
    let managed_response::Result::Initialized(initialized) = result(companion.handle(request(
        managed_request::Operation::Initialize(ManagedInitialize {
            protocol_major: PROTOCOL_MAJOR,
        }),
    ))) else {
        panic!("Handshake required")
    };
    let managed_response::Result::ProjectOpened(opened) = result(companion.handle(request(
        managed_request::Operation::ProjectOpen(ProjectOpen {
            fence: initialized.fence,
            root: root.path().to_str().unwrap().into(),
        }),
    ))) else {
        panic!("Project required")
    };
    let fence = opened.fence.unwrap();
    let mut stale = fence.clone();
    stale.connection_id = Uuid::new_v4().to_string();
    let command_json =
        serde_json::to_vec(&TaskCommand::Panel(crate::transport::PanelQuery::default())).unwrap();
    let mut query = ProjectTasksRequest {
        fence: Some(stale),
        query_generation: 9,
        command_json,
    };
    assert!(
        matches!(result(companion.handle(request(managed_request::Operation::ProjectTasks(query.clone())))), managed_response::Result::Error(ManagedError { code }) if code == ManagedErrorCode::ManagedStaleAttachment as i32)
    );
    assert!(!directory.join("projects").exists());
    query.fence = Some(fence);
    assert!(matches!(
        result(
            companion.handle(request(managed_request::Operation::ProjectTasks(
                query.clone()
            )))
        ),
        managed_response::Result::ProjectTasks(_)
    ));
    let selected = root.path().join("selected");
    std::fs::create_dir(&selected).unwrap();
    let managed_response::Result::ProjectOpened(next) = result(companion.handle(request(
        managed_request::Operation::ProjectOpen(ProjectOpen {
            fence: Some(ManagedFence {
                project_id: String::new(),
                ..companion.fence.clone()
            }),
            root: selected.to_str().unwrap().into(),
        }),
    ))) else {
        panic!("New project required")
    };
    std::fs::rename(&selected, root.path().join("moved")).unwrap();
    std::fs::create_dir(&selected).unwrap();
    query.fence = next.fence;
    assert!(
        matches!(result(companion.handle(request(managed_request::Operation::ProjectTasks(query)))), managed_response::Result::Error(ManagedError { code }) if code == ManagedErrorCode::ManagedStaleAttachment as i32)
    );
}

#[cfg(unix)]
#[test]
fn guarded_run_survives_connection_loss_behind_one_writer_attachment() {
    use std::time::{Duration, Instant};
    let root = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let (mut companion, fence) = file_attachment(root.path(), &state.path().join("private"));
    companion.account_guardian = Some(Uuid::parse_str(&fence.service_id).unwrap());
    let mut next = fence.clone();
    next.connection_id = Uuid::new_v4().to_string();
    let label_only = TerminalLaunch {
        fence: Some(fence.clone()),
        session_id: Uuid::new_v4().to_string(),
        executable: "/bin/sh".into(),
        arguments: Vec::new(),
        columns: 80,
        rows: 24,
        agent_program: None,
        working_directory: None,
        guarded: false,
        label: Some("legacy-label".into()),
        session_kind: None,
    };
    assert!(matches!(
        result(companion.handle(request(managed_request::Operation::TerminalLaunch(
            label_only
        )))),
        managed_response::Result::Error(ManagedError { code })
            if code == ManagedErrorCode::ManagedInvalidInput as i32
    ));
    let launch = TerminalLaunch {
        fence: Some(fence.clone()),
        session_id: Uuid::new_v4().to_string(),
        executable: "/bin/sh".into(),
        arguments: vec![
            "-c".into(),
            "while IFS= read -r line; do printf 'ECHO=%s\\n' \"$line\"; done".into(),
        ],
        columns: 80,
        rows: 24,
        agent_program: None,
        working_directory: None,
        guarded: true,
        label: Some("guarded-test".into()),
        session_kind: Some(TerminalSessionKind::Shell as i32),
    };
    let managed_response::Result::TerminalState(launched) = result(companion.handle(request(
        managed_request::Operation::TerminalLaunch(launch.clone()),
    ))) else {
        panic!("Guarded launch required")
    };
    let mut changed_kind = launch;
    changed_kind.session_kind = Some(TerminalSessionKind::Command as i32);
    assert!(matches!(
        result(companion.handle(request(managed_request::Operation::TerminalLaunch(
            changed_kind
        )))),
        managed_response::Result::Error(ManagedError { code })
            if code == ManagedErrorCode::ManagedConflict as i32
    ));
    assert_eq!(launched.guarded, Some(true));
    assert_eq!(launched.label.as_deref(), Some("guarded-test"));
    assert!(launched.input_receipt.is_none());
    let generation = launched.attachment_generation;
    let attachment = launched.attachment_id.clone();
    let control = |fence: &ManagedFence, action: TerminalAction| TerminalControl {
        fence: Some(fence.clone()),
        session_id: launched.session_id.clone(),
        run_id: launched.run_id.clone(),
        action: action as i32,
        attachment_id: attachment.clone(),
        attachment_generation: generation,
        ..Default::default()
    };
    let read = |companion: &mut Companion, fence: &ManagedFence, offset: u64| {
        let control = TerminalControl {
            output_offset: offset,
            ..control(fence, TerminalAction::TerminalRead)
        };
        companion.terminals.control(control, fence, true).unwrap()
    };
    let read_until = |companion: &mut Companion, fence: &ManagedFence, needle: &str, mut offset: u64| {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut seen = String::new();
        while Instant::now() < deadline {
            let reply = read(companion, fence, offset);
            seen.push_str(&String::from_utf8_lossy(&reply.output));
            offset = reply.output_offset + reply.output.len() as u64;
            if seen.contains(needle) {
                return offset;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("Missing {needle:?} in {seen:?}")
    };

    companion.terminals.disconnect(&fence.connection_id);
    // The dropped connection cannot operate the retained run, and the run stays alive.
    let mut input = control(&fence, TerminalAction::TerminalInput);
    input.input = b"stale\n".to_vec();
    input.input_sequence = 1;
    assert!(matches!(
        companion.terminals.control(input, &fence, true),
        Err(ManagedErrorCode::ManagedStaleAttachment)
    ));
    // A detached run has no writer, so exact-tuple resume is admitted; takeovers are
    // only fenced against a live writer (covered after detach below).
    let attached = companion
        .terminals
        .control(control(&next, TerminalAction::TerminalAttach), &next, true)
        .unwrap();
    assert_eq!(attached.guarded, Some(true));
    assert_eq!(attached.attachment_generation, generation);
    assert_eq!(attached.attachment_id, attachment);
    let mut first = control(&next, TerminalAction::TerminalInput);
    first.input = b"hello\n".to_vec();
    first.input_sequence = 1;
    let first = companion.terminals.control(first, &next, true).unwrap();
    let first_receipt = first.input_receipt.clone().expect("Accepted input needs a receipt");
    assert_eq!(first_receipt.input_sequence, 1);
    assert_eq!(first_receipt.attachment_generation, generation);
    assert!(Uuid::parse_str(&first_receipt.receipt_id).is_ok());
    let mut duplicate = control(&next, TerminalAction::TerminalInput);
    duplicate.input = b"hello\n".to_vec();
    duplicate.input_sequence = 1;
    let duplicate = companion.terminals.control(duplicate, &next, true).unwrap();
    assert_eq!(
        duplicate.input_receipt.map(|receipt| receipt.receipt_id),
        Some(first_receipt.receipt_id.clone())
    );
    let mut conflicting = control(&next, TerminalAction::TerminalInput);
    conflicting.input = b"other\n".to_vec();
    conflicting.input_sequence = 1;
    assert!(matches!(
        companion.terminals.control(conflicting, &next, true),
        Err(ManagedErrorCode::ManagedInputConflict)
    ));
    let mut skipped = control(&next, TerminalAction::TerminalInput);
    skipped.input = b"skipped\n".to_vec();
    skipped.input_sequence = 3;
    assert!(matches!(
        companion.terminals.control(skipped, &next, true),
        Err(ManagedErrorCode::ManagedInputConflict)
    ));
    let offset = read_until(&mut companion, &next, "ECHO=hello", 0);
    assert!(companion
        .terminals
        .control(control(&next, TerminalAction::TerminalDetach), &next, true)
        .is_ok());
    assert!(matches!(
        companion.terminals.control(
            TerminalControl {
                output_offset: offset,
                ..control(&next, TerminalAction::TerminalRead)
            },
            &next,
            true,
        ),
        Err(ManagedErrorCode::ManagedStaleAttachment)
    ));
    // A fresh client restarts the writer epoch only through a fenced takeover.
    let mut takeover = control(&next, TerminalAction::TerminalAttach);
    takeover.attachment_id = String::new();
    takeover.attachment_generation = generation;
    takeover.takeover = true;
    let adopted = companion.terminals.control(takeover, &next, true).unwrap();
    assert_eq!(adopted.attachment_generation, generation + 1);
    assert_ne!(adopted.attachment_id, attachment);
    let mut stale_takeover = control(&next, TerminalAction::TerminalAttach);
    stale_takeover.attachment_id = String::new();
    stale_takeover.attachment_generation = generation;
    stale_takeover.takeover = true;
    assert!(matches!(
        companion.terminals.control(stale_takeover, &next, true),
        Err(ManagedErrorCode::ManagedStaleAttachment)
    ));
    // The superseded tuple cannot control through the adopted connection.
    let mut stale_input = control(&next, TerminalAction::TerminalInput);
    stale_input.input = b"stale\n".to_vec();
    stale_input.input_sequence = 2;
    assert!(matches!(
        companion.terminals.control(stale_input, &next, true),
        Err(ManagedErrorCode::ManagedStaleAttachment)
    ));
    let mut second = TerminalControl {
        attachment_id: adopted.attachment_id.clone(),
        attachment_generation: adopted.attachment_generation,
        ..control(&next, TerminalAction::TerminalInput)
    };
    second.input = b"again\n".to_vec();
    second.input_sequence = 1;
    let second = companion.terminals.control(second, &next, true).unwrap();
    let second_receipt = second.input_receipt.expect("Adopted input needs a receipt");
    assert_eq!(second_receipt.input_sequence, 1);
    assert_eq!(
        second_receipt.attachment_generation,
        adopted.attachment_generation
    );
    assert_ne!(second_receipt.receipt_id, first_receipt.receipt_id);
    let read_adopted = |companion: &mut Companion, fence: &ManagedFence, offset: u64| {
        let control = TerminalControl {
            output_offset: offset,
            ..TerminalControl {
                fence: Some(fence.clone()),
                session_id: launched.session_id.clone(),
                run_id: launched.run_id.clone(),
                action: TerminalAction::TerminalRead as i32,
                attachment_id: adopted.attachment_id.clone(),
                attachment_generation: adopted.attachment_generation,
                ..Default::default()
            }
        };
        companion.terminals.control(control, fence, true).unwrap()
    };
    let read_until_adopted = |companion: &mut Companion, fence: &ManagedFence, needle: &str, mut offset: u64| {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut seen = String::new();
        while Instant::now() < deadline {
            let reply = read_adopted(companion, fence, offset);
            seen.push_str(&String::from_utf8_lossy(&reply.output));
            offset = reply.output_offset + reply.output.len() as u64;
            if seen.contains(needle) {
                return offset;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("Missing {needle:?} in {seen:?}")
    };
    let offset = read_until_adopted(&mut companion, &next, "ECHO=again", offset);
    // Stop stays writer-bound until an authenticated operator path exists.
    let mut stop = control(&next, TerminalAction::TerminalStop);
    stop.attachment_id = adopted.attachment_id.clone();
    stop.attachment_generation = adopted.attachment_generation;
    assert!(companion.terminals.control(stop, &next, true).is_ok());
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut exited = None;
    let mut polled = offset;
    while Instant::now() < deadline {
        let reply = read_adopted(&mut companion, &next, polled);
        polled = reply.output_offset + reply.output.len() as u64;
        if reply.output_closed && reply.processes_active == Some(false) {
            exited = Some(reply);
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let exited = exited.expect("Guarded run must exit after stop");
    assert!(exited.exit_code.is_some());
    let mut release = control(&next, TerminalAction::TerminalRelease);
    release.attachment_id = adopted.attachment_id.clone();
    release.attachment_generation = adopted.attachment_generation;
    assert!(companion.terminals.control(release, &next, true).is_ok());
}

#[cfg(unix)]
#[test]
fn guarded_session_lifecycle_is_recorded_in_the_catalog() {
    use std::time::{Duration, Instant};
    let root = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let (mut companion, fence) = file_attachment(root.path(), &state.path().join("private"));
    let catalog_path = state.path().join("guardian/catalog.sqlite3");
    std::fs::create_dir_all(catalog_path.parent().unwrap()).unwrap();
    let catalog = std::sync::Arc::new(std::sync::Mutex::new(
        guardian_catalog::Catalog::open(&catalog_path).unwrap(),
    ));
    companion.catalog = Some(catalog.clone());
    companion.account_guardian = Some(Uuid::parse_str(&fence.service_id).unwrap());
    companion.route = Some(Route::Ssh);
    let managed_response::Result::TerminalState(launched) = result(companion.handle(request(
        managed_request::Operation::TerminalLaunch(TerminalLaunch {
            fence: Some(fence.clone()),
            session_id: Uuid::new_v4().to_string(),
            executable: "/bin/sh".into(),
            arguments: vec!["-c".into(), "sleep 0.2".into()],
            columns: 80,
            rows: 24,
            agent_program: None,
            working_directory: None,
            guarded: true,
            label: Some("catalog-test".into()),
            session_kind: Some(TerminalSessionKind::Shell as i32),
        }),
    ))) else {
        panic!("Guarded launch required")
    };
    {
        let rows = catalog.lock().unwrap().list().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].session_id, launched.session_id);
        assert_eq!(rows[0].run_id, launched.run_id);
        assert_eq!(rows[0].kind, "shell");
        assert_eq!(rows[0].policy, "guarded");
        assert_eq!(rows[0].label.as_deref(), Some("catalog-test"));
        assert_eq!(rows[0].state, guardian_catalog::State::Live);
    }
    let account = AccountFence {
        guardian_id: fence.service_id.clone(),
        boot_id: fence.service_boot_id.clone(),
        connection_id: fence.connection_id.clone(),
    };
    let managed_response::Result::AccountCatalogPage(page) = result(companion.handle(request(
        managed_request::Operation::AccountCatalogList(AccountCatalogList {
            fence: Some(account.clone()),
            filter: Some(AccountCatalogFilter {
                scope_id: String::new(),
                project_id: fence.project_id.clone(),
            }),
            limit: 16,
            cursor: String::new(),
        }),
    ))) else {
        panic!("Account catalog page required")
    };
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.rows[0].session_id, launched.session_id);
    assert_eq!(page.rows[0].run_id, launched.run_id);
    assert_eq!(page.rows[0].label.as_deref(), Some("catalog-test"));
    assert_eq!(page.rows[0].state, "live");
    assert_eq!(page.total, 1);
    assert_eq!(page.revision, catalog.lock().unwrap().revision());
    assert!(page.next.is_none());
    assert!(matches!(
        result(companion.handle(request(managed_request::Operation::AccountCatalogInspect(
            AccountCatalogInspect {
                fence: Some(account.clone()),
                session_id: launched.session_id.clone(),
                run_id: launched.run_id.clone(),
            },
        )))),
        managed_response::Result::AccountCatalogRow(_)
    ));
    let mut stale_account = account;
    stale_account.boot_id = Uuid::new_v4().to_string();
    assert!(matches!(
        result(companion.handle(request(managed_request::Operation::AccountCatalogList(
            AccountCatalogList {
                fence: Some(stale_account),
                filter: None,
                limit: 16,
                cursor: String::new(),
            },
        )))),
        managed_response::Result::Error(ManagedError { code })
            if code == ManagedErrorCode::ManagedStaleAttachment as i32
    ));
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        guardian::sync_ended(&companion.terminals, &catalog);
        if catalog.lock().unwrap().list().unwrap()[0].state == guardian_catalog::State::Ended {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let rows = catalog.lock().unwrap().list().unwrap();
    assert_eq!(rows[0].state, guardian_catalog::State::Ended);
    assert_eq!(rows[0].exit_status, Some(0));
}

#[cfg(unix)]
#[test]
fn legacy_stop_requires_the_owning_connection() {
    use std::time::{Duration, Instant};
    let root = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let (mut companion, fence) = file_attachment(root.path(), &state.path().join("private"));
    let managed_response::Result::TerminalState(launched) = result(companion.handle(request(
        managed_request::Operation::TerminalLaunch(TerminalLaunch {
            fence: Some(fence.clone()),
            session_id: Uuid::new_v4().to_string(),
            executable: "/bin/sh".into(),
            arguments: vec![
                "-c".into(),
                "trap 'exit 0' TERM; while :; do sleep 1; done".into(),
            ],
            columns: 80,
            rows: 24,
            agent_program: None,
            working_directory: None,
            guarded: false,
            label: None,
            session_kind: None,
        }),
    ))) else {
        panic!("Legacy launch required")
    };
    assert_eq!(launched.guarded, Some(false));
    assert!(launched.attachment_id.is_empty());
    assert_eq!(launched.attachment_generation, 0);
    assert!(launched.input_receipt.is_none());
    let stop = |fence: &ManagedFence| TerminalControl {
        fence: Some(fence.clone()),
        session_id: launched.session_id.clone(),
        run_id: launched.run_id.clone(),
        action: TerminalAction::TerminalStop as i32,
        ..Default::default()
    };
    let mut other = fence.clone();
    other.connection_id = Uuid::new_v4().to_string();
    assert!(matches!(
        companion.terminals.control(stop(&other), &other, false),
        Err(ManagedErrorCode::ManagedStaleAttachment)
    ));
    assert!(companion.terminals.control(stop(&fence), &fence, false).is_ok());
    let read = |companion: &mut Companion, fence: &ManagedFence, offset: u64| {
        companion
            .terminals
            .control(
                TerminalControl {
                    fence: Some(fence.clone()),
                    session_id: launched.session_id.clone(),
                    run_id: launched.run_id.clone(),
                    action: TerminalAction::TerminalRead as i32,
                    output_offset: offset,
                    ..Default::default()
                },
                fence,
                false,
            )
            .unwrap()
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut offset = 0;
    while Instant::now() < deadline {
        let reply = read(&mut companion, &fence, offset);
        offset = reply.output_offset + reply.output.len() as u64;
        if reply.output_closed && reply.processes_active == Some(false) {
            assert!(companion
                .terminals
                .control(
                    TerminalControl {
                        fence: Some(fence.clone()),
                        session_id: launched.session_id.clone(),
                        run_id: launched.run_id.clone(),
                        action: TerminalAction::TerminalRelease as i32,
                        ..Default::default()
                    },
                    &fence,
                    false,
                )
                .is_ok());
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("Legacy run did not exit after stop");
}
