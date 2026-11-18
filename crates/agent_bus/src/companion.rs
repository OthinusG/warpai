//! Private project communication and Agent IO, independent of desktop/UI and enrollment.
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use remote_protocol::{
    managed::PROTOCOL_MAJOR,
    proto::*,
    protocol::{
        read_message_with_limit, write_message_with_limit, ProtocolError, MAX_MANAGED_MESSAGE_SIZE,
    },
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use uuid::Uuid;
#[cfg(test)]
#[path = "companion_files_tests.rs"]
mod file_tests;
#[path = "companion_files.rs"]
pub(crate) mod files;
#[path = "companion_identity.rs"]
pub(crate) mod identity;
#[path = "companion_service.rs"]
mod service;
#[path = "companion_catalog.rs"]
pub(crate) mod catalog;
#[path = "companion_tasks.rs"]
mod tasks;
#[path = "companion_terminals.rs"]
mod terminals;
#[path = "companion_guests.rs"]
pub(crate) mod guests;
#[cfg(unix)]
#[path = "companion_guardian.rs"]
pub mod guardian;
#[cfg(unix)]
#[path = "companion_guardian_catalog.rs"]
pub(crate) mod guardian_catalog;
pub(crate) use tasks::{decode_result as decode_task_result, MAX_RESULT as MAX_TASK_RESULT};
pub use tasks::Command as TaskCommand;

#[cfg(unix)]
pub(crate) type CatalogHandle =
    std::sync::Arc<std::sync::Mutex<guardian_catalog::Catalog>>;
#[cfg(not(unix))]
pub(crate) type CatalogHandle = ();

/// Admitted transport route for an account-global guardian connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Ssh,
    Wsl,
}

#[cfg(unix)]
fn valid_uuid(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|id| !id.is_nil())
}

#[cfg(unix)]
fn catalog_filter_key(filter: &AccountCatalogFilter) -> String {
    use sha2::{Digest, Sha256};

    let mut digest = Sha256::new();
    digest.update((filter.scope_id.len() as u64).to_be_bytes());
    digest.update(filter.scope_id.as_bytes());
    digest.update(filter.project_id.as_bytes());
    format!("{:x}", digest.finalize())
}

#[cfg(unix)]
fn catalog_filter_matches(filter: &AccountCatalogFilter, row: &guardian_catalog::Record) -> bool {
    (filter.scope_id.is_empty() || filter.scope_id == row.scope_id)
        && (filter.project_id.is_empty() || filter.project_id == row.project_id)
}

/// Sanitized catalog projection; never argv, environment, paths or screen bytes.
#[cfg(unix)]
fn catalog_row(record: &guardian_catalog::Record) -> AccountCatalogRow {
    AccountCatalogRow {
        session_id: record.session_id.clone(),
        run_id: record.run_id.clone(),
        project_id: record.project_id.clone(),
        kind: record.kind.clone(),
        policy: record.policy.clone(),
        label: record.label.clone(),
        state: match record.state {
            guardian_catalog::State::Live => "live",
            guardian_catalog::State::Ended => "ended",
            guardian_catalog::State::Unknown => "unknown",
        }
        .into(),
        created_seq: record.created_seq,
        mru_seq: record.mru_seq,
        observed_ms: record.observed_ms,
        exit_status: record.exit_status,
        attachment_generation: record.attachment_generation,
    }
}

struct Project {
    root: PathBuf,
    handle: File,
}

/// One fenced attachment to account-owned project and terminal state.
pub struct Companion {
    identity: identity::Identity,
    fence: ManagedFence,
    initialized: bool,
    project: Option<Project>,
    terminals: std::sync::Arc<terminals::Terminals>,
    tasks: std::sync::Arc<tasks::Projects>,
    files: files::Files,
    peer_pid: Option<u32>,
    route: Option<Route>,
    catalog: Option<CatalogHandle>,
    account_guardian: Option<Uuid>,
    account_operator: bool,
}

impl Companion {
    pub fn new(data_directory: &Path) -> std::io::Result<Self> {
        let identity = identity::Identity::open(data_directory)?;
        Ok(Self {
            fence: ManagedFence {
                service_id: identity.service_id.clone(),
                service_boot_id: Uuid::new_v4().to_string(),
                connection_id: Uuid::new_v4().to_string(),
                project_id: String::new(),
            },
            initialized: false,
            project: None,
            terminals: std::sync::Arc::new(terminals::Terminals::default()),
            tasks: std::sync::Arc::new(tasks::Projects::new(data_directory)),
            files: files::Files::new(data_directory),
            peer_pid: None,
            route: None,
            catalog: None,
            account_guardian: None,
            account_operator: false,
            identity,
        })
    }

    /// True only for an admitted WSL route; legacy builds keep their compile-time behavior.
    fn wsl_route(&self) -> bool {
        cfg!(feature = "wsl_companion")
            && self
                .route
                .map_or(true, |route| route == Route::Wsl)
    }

    fn capabilities(&self) -> Vec<String> {
        let mut capabilities = vec!["project_open".into(), "managed_agent".into()];
        // Only the account guardian admits retained runs and the account catalog.
        if self.account_guardian.is_some() {
            capabilities.push("account_catalog".into());
            // The native acceptance gate remains closed for installed helpers.
            #[cfg(test)]
            capabilities.push("guarded_remote_sessions_v1".into());
        }
        if self.wsl_route() {
            capabilities.push("managed_launch_cwd".into());
        }
        capabilities.extend(
            [
                "project_tasks",
                "context_resources",
                "structured_task_messages",
                "execution_threads",
                "project_memory",
                "project_supervision",
                "handoff_briefs_v1",
                "followup_reconcile",
                "project_mcp",
            ]
            .map(str::to_owned),
        );
        if self.tasks.supports_ssh_setup() {
            capabilities.push("native_mcp_setup".into());
        }
        capabilities.push("project_files".into());
        if self.wsl_route() {
            capabilities.push("project_file_chunks".into());
        }
        capabilities.extend(
            [
                "native_screen_activity",
                "project_git_review",
                "worktree_collaboration",
                "worktree_orchestration",
            ]
            .map(str::to_owned),
        );
        capabilities
    }

    /// Logical route namespace; never a filesystem path.
    fn scope_id(&self) -> Option<&'static str> {
        self.route.map(|route| match route {
            Route::Ssh => "ssh",
            Route::Wsl => "wsl",
        })
    }

    #[cfg(unix)]
    fn record_catalog_launch(
        &self,
        request: &TerminalLaunch,
        run_id: &str,
        state: Option<&TerminalState>,
        reserve: bool,
    ) -> Result<(), ManagedErrorCode> {
        if !request.guarded {
            return Ok(());
        }
        let Some(catalog) = &self.catalog else {
            return Ok(());
        };
        let scope_id = self
            .scope_id()
            .ok_or(ManagedErrorCode::ManagedFeatureUnavailable)?;
        let kind = TerminalSessionKind::try_from(request.session_kind.unwrap_or_default())
            .map_err(|_| ManagedErrorCode::ManagedInvalidInput)?;
        let kind = match kind {
            TerminalSessionKind::Agent => "agent",
            TerminalSessionKind::Shell => "shell",
            TerminalSessionKind::Command => "command",
            TerminalSessionKind::Unspecified => return Err(ManagedErrorCode::ManagedInvalidInput),
        };
        let now = unix_millis();
        let mut catalog = catalog
            .lock()
            .map_err(|_| ManagedErrorCode::ManagedUnavailable)?;
        let existing = catalog
            .find(&request.session_id, run_id)
            .map_err(|_| ManagedErrorCode::ManagedUnavailable)?;
        if state.is_none() && !reserve {
            if existing.is_some_and(|row| row.state == guardian_catalog::State::Unknown) {
                catalog.remove(&request.session_id)
                    .map_err(|_| ManagedErrorCode::ManagedUnavailable)?;
            }
            return Ok(());
        }
        if state.is_some() && existing.as_ref().is_some_and(|row| {
            row.state != guardian_catalog::State::Unknown
        }) {
            return Ok(());
        }
        let seq = catalog
            .allocate_seq()
            .map_err(|_| ManagedErrorCode::ManagedUnavailable)?;
        let record = guardian_catalog::Record {
            session_id: request.session_id.clone(),
            run_id: run_id.into(),
            boot_id: self.fence.service_boot_id.clone(),
            service_id: self.fence.service_id.clone(),
            scope_id: scope_id.into(),
            project_id: self.fence.project_id.clone(),
            policy: "guarded".into(),
            kind: kind.into(),
            label: request.label.clone(),
            created_seq: existing.map_or(seq, |row| row.created_seq),
            mru_seq: state.map_or(0, |_| seq),
            state: state.map_or(guardian_catalog::State::Unknown, |_| guardian_catalog::State::Live),
            observed_ms: now,
            exit_status: None,
            attachment_generation: state.map_or(0, |state| state.attachment_generation),
        };
        catalog
            .upsert(&record)
            .map_err(|_| ManagedErrorCode::ManagedUnavailable)
    }

    #[cfg(not(unix))]
    fn record_catalog_launch(
        &self,
        _request: &TerminalLaunch,
        _run_id: &str,
        _state: Option<&TerminalState>,
        _reserve: bool,
    ) -> Result<(), ManagedErrorCode> {
        Ok(())
    }

    #[cfg(unix)]
    fn record_catalog_control(&self, action: TerminalAction, state: &TerminalState) {
        if state.guarded != Some(true) || action != TerminalAction::TerminalAttach {
            return;
        }
        if let Some(catalog) = &self.catalog {
            guardian::sync_attachment(&self.terminals, catalog, &terminals::PendingAttachment {
                session_id: state.session_id.clone(),
                run_id: state.run_id.clone(),
                boot_id: self.fence.service_boot_id.clone(),
                service_id: self.fence.service_id.clone(),
                generation: state.attachment_generation,
            });
        }
    }

    #[cfg(not(unix))]
    fn record_catalog_control(&self, _action: TerminalAction, _state: &TerminalState) {}

}

fn unix_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}

impl Companion {
    /// Only the managed oneof reaches this handler; legacy commands never execute.
    pub fn handle(&mut self, request: ClientMessage) -> ServerMessage {
        let request_id = Uuid::parse_str(&request.request_id).ok().filter(|id| !id.is_nil());
        let result = match (request_id, request.message) {
            (Some(id), Some(client_message::Message::Managed(request))) => {
                self.dispatch(request, id)
            }
            (None, _) => Err(ManagedErrorCode::ManagedInvalidInput),
            _ => Err(ManagedErrorCode::ManagedFeatureUnavailable),
        };
        ServerMessage {
            // Never echo unvalidated peer strings, including oversized request identifiers.
            request_id: if request_id.is_some() {
                request.request_id
            } else {
                String::new()
            },
            message: Some(server_message::Message::Managed(Box::new(
                ManagedResponse {
                    result: Some(result.unwrap_or_else(|code| {
                        managed_response::Result::Error(ManagedError { code: code.into() })
                    })),
                },
            ))),
        }
    }

    fn dispatch(
        &mut self,
        request: ManagedRequest,
        request_id: Uuid,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        match request.operation {
            Some(managed_request::Operation::Initialize(request)) => {
                if self.initialized {
                    return Err(ManagedErrorCode::ManagedInvalidInput);
                }
                if request.protocol_major != PROTOCOL_MAJOR {
                    return Err(ManagedErrorCode::ManagedIncompatibleVersion);
                }
                self.initialized = true;
                #[cfg(unix)]
                let current_guarded_session = if self.account_operator {
                    self.peer_pid.and_then(|pid| self.terminals.current_guarded_session(pid))
                } else {
                    None
                };
                #[cfg(not(unix))]
                let current_guarded_session = None;
                Ok(managed_response::Result::Initialized(ManagedInitialized {
                    protocol_major: PROTOCOL_MAJOR,
                    fence: Some(self.fence.clone()),
                    os: std::env::consts::OS.into(),
                    architecture: std::env::consts::ARCH.into(),
                    capabilities: self.capabilities(),
                    account_id: self.identity.account_id.clone(),
                    // Legacy per-scope services never advertise account custody.
                    account_fence: self.account_guardian.map(|guardian| AccountFence {
                        guardian_id: guardian.to_string(),
                        boot_id: self.fence.service_boot_id.clone(),
                        connection_id: self.fence.connection_id.clone(),
                    }),
                    current_guarded_session,
                }))
            }
            Some(managed_request::Operation::ProjectOpen(request)) => {
                self.check_fence(request.fence.as_ref(), false)?;
                if request.root.is_empty()
                    || request.root.len() > 4096
                    || request.root.chars().any(char::is_control)
                    || !Path::new(&request.root).is_absolute()
                {
                    return Err(ManagedErrorCode::ManagedInvalidInput);
                }
                self.open_project(Path::new(&request.root), None)
            }
            Some(managed_request::Operation::GuardedProjectOpen(request)) => {
                self.open_guarded_project(request)
            }
            Some(managed_request::Operation::TerminalLaunch(request)) => {
                self.check_project(request.fence.as_ref())?;
                if request.guarded && self.account_guardian.is_none() {
                    return Err(ManagedErrorCode::ManagedFeatureUnavailable);
                }
                let recorded = request.clone();
                let state = self.terminals.launch(
                    request,
                    &self.fence,
                    &self.project.as_ref().unwrap().root,
                    &self.tasks,
                    self.wsl_route(),
                    self.catalog.is_some(),
                    |run_id, reserve| self.record_catalog_launch(&recorded, run_id, None, reserve),
                )?;
                #[cfg(unix)]
                if recorded.guarded && self.catalog.is_some()
                    && self.record_catalog_launch(&recorded, &state.run_id, Some(&state), false).is_ok()
                {
                    self.terminals.mark_attachment_reported(
                        &state.session_id, &state.run_id, state.attachment_generation,
                    );
                }
                Ok(managed_response::Result::TerminalState(state))
            }
            Some(managed_request::Operation::TerminalControl(request)) => {
                self.check_project(request.fence.as_ref())?;
                let action = TerminalAction::try_from(request.action).ok();
                let state = self.terminals.control(request, &self.fence, self.account_guardian.is_some())?;
                if let Some(action) = action {
                    self.record_catalog_control(action, &state);
                }
                Ok(managed_response::Result::TerminalState(state))
            }
            Some(managed_request::Operation::ProjectTasks(request)) => {
                self.check_project(request.fence.as_ref())?;
                self.tasks
                    .execute(request, &self.fence, &self.project.as_ref().unwrap().root)
                    .map(managed_response::Result::ProjectTasks)
            }
            Some(managed_request::Operation::ProjectFiles(request)) => {
                self.check_project(request.fence.as_ref())?;
                if self.route == Some(Route::Ssh)
                    && matches!(
                        ProjectFileAction::try_from(request.action),
                        Ok(ProjectFileAction::ProjectFileReadChunk)
                            | Ok(ProjectFileAction::ProjectFileWriteChunk)
                    )
                {
                    return Err(ManagedErrorCode::ManagedFeatureUnavailable);
                }
                self.files
                    .execute(request, self.project.as_ref().unwrap())
                    .map(|result| managed_response::Result::ProjectFiles(Box::new(result)))
            }
            Some(managed_request::Operation::GuestMcpBind(request)) => {
                self.check_project(request.fence.as_ref())?;
                if self.route == Some(Route::Ssh) && self.account_guardian.is_some() {
                    return Err(ManagedErrorCode::ManagedFeatureUnavailable);
                }
                {
                    self.tasks.guests.bind(&self.tasks, &self.fence, &self.project.as_ref().unwrap().root,
                        self.peer_pid, request.agent_pid, &request.program)
                        .map(managed_response::Result::GuestMcpBound)
                }
            }
            Some(managed_request::Operation::AccountCatalogList(request)) => {
                self.dispatch_catalog_list(request)
            }
            Some(managed_request::Operation::AccountCatalogInspect(request)) => {
                self.dispatch_catalog_inspect(request)
            }
            Some(managed_request::Operation::AccountCatalogWatch(request)) => {
                self.dispatch_catalog_watch(request)
            }
            Some(managed_request::Operation::AccountStop(request)) => {
                self.dispatch_account_stop(request_id, request)
            }
            None => Err(ManagedErrorCode::ManagedInvalidInput),
        }
    }

    #[cfg(unix)]
    fn check_account_fence(&self, fence: Option<&AccountFence>) -> Result<(), ManagedErrorCode> {
        let Some(guardian) = self.account_guardian else {
            return Err(ManagedErrorCode::ManagedFeatureUnavailable);
        };
        if !self.initialized {
            return Err(ManagedErrorCode::ManagedStaleAttachment);
        }
        let Some(fence) = fence else {
            return Err(ManagedErrorCode::ManagedStaleAttachment);
        };
        if fence.guardian_id != guardian.to_string()
            || fence.boot_id != self.fence.service_boot_id
            || fence.connection_id != self.fence.connection_id
        {
            return Err(ManagedErrorCode::ManagedStaleAttachment);
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn check_account_fence(&self, _fence: Option<&AccountFence>) -> Result<(), ManagedErrorCode> {
        Err(ManagedErrorCode::ManagedFeatureUnavailable)
    }

    #[cfg(unix)]
    fn dispatch_catalog_list(
        &mut self,
        request: AccountCatalogList,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        self.check_account_fence(request.fence.as_ref())?;
        let limit = match request.limit {
            0 => 16,
            limit @ 1..=32 => usize::try_from(limit).unwrap_or(16),
            _ => return Err(ManagedErrorCode::ManagedInvalidInput),
        };
        let mut catalog = self
            .catalog
            .as_ref()
            .ok_or(ManagedErrorCode::ManagedFeatureUnavailable)?
            .lock()
            .map_err(|_| ManagedErrorCode::ManagedUnavailable)?;
        let revision = catalog.revision();
        let filter = request.filter.unwrap_or_default();
        let filter_key = catalog_filter_key(&filter);
        let mut rows = catalog.list().map_err(|_| ManagedErrorCode::ManagedUnavailable)?;
        rows.retain(|row| catalog_filter_matches(&filter, row));
        let total = rows.len();
        let after = if request.cursor.is_empty() {
            0
        } else {
            let Some((boot, rest)) = request.cursor.split_once(':') else {
                return Err(ManagedErrorCode::ManagedConflict);
            };
            let Some((page_revision, rest)) = rest.split_once(':') else {
                return Err(ManagedErrorCode::ManagedConflict);
            };
            let Some((cursor_filter, session)) = rest.split_once(':') else {
                return Err(ManagedErrorCode::ManagedConflict);
            };
            if boot != self.fence.service_boot_id
                || cursor_filter != filter_key
                || page_revision.parse::<u64>().ok() != Some(revision)
            {
                return Err(ManagedErrorCode::ManagedConflict);
            }
            rows.iter()
                .position(|row| row.session_id == session)
                .map(|index| index + 1)
                .ok_or(ManagedErrorCode::ManagedConflict)?
        };
        let page = rows.iter().skip(after).take(limit).collect::<Vec<_>>();
        let next = (after + page.len() < total).then(|| {
            format!(
                "{}:{}:{}:{}",
                self.fence.service_boot_id,
                revision,
                filter_key,
                page.last().expect("non-empty page").session_id
            )
        });
        Ok(managed_response::Result::AccountCatalogPage(
            AccountCatalogPage {
                rows: page.into_iter().map(catalog_row).collect(),
                total: u32::try_from(total).unwrap_or(u32::MAX),
                revision,
                next,
            },
        ))
    }

    #[cfg(not(unix))]
    fn dispatch_catalog_list(
        &mut self,
        _request: AccountCatalogList,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        Err(ManagedErrorCode::ManagedFeatureUnavailable)
    }

    #[cfg(unix)]
    fn dispatch_catalog_inspect(
        &mut self,
        request: AccountCatalogInspect,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        self.check_account_fence(request.fence.as_ref())?;
        if !valid_uuid(&request.session_id) || !valid_uuid(&request.run_id) {
            return Err(ManagedErrorCode::ManagedInvalidInput);
        }
        let mut catalog = self
            .catalog
            .as_ref()
            .ok_or(ManagedErrorCode::ManagedFeatureUnavailable)?
            .lock()
            .map_err(|_| ManagedErrorCode::ManagedUnavailable)?;
        let record = catalog
            .find(&request.session_id, &request.run_id)
            .map_err(|_| ManagedErrorCode::ManagedUnavailable)?
            .ok_or(ManagedErrorCode::ManagedNotFound)?;
        Ok(managed_response::Result::AccountCatalogRow(catalog_row(
            &record,
        )))
    }

    #[cfg(not(unix))]
    fn dispatch_catalog_inspect(
        &mut self,
        _request: AccountCatalogInspect,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        Err(ManagedErrorCode::ManagedFeatureUnavailable)
    }

    #[cfg(unix)]
    fn dispatch_catalog_watch(
        &mut self,
        request: AccountCatalogWatch,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        self.check_account_fence(request.fence.as_ref())?;
        let limit = match request.limit {
            0 => 16,
            limit @ 1..=32 => usize::try_from(limit).unwrap_or(16),
            _ => return Err(ManagedErrorCode::ManagedInvalidInput),
        };
        let catalog = self
            .catalog
            .as_ref()
            .ok_or(ManagedErrorCode::ManagedFeatureUnavailable)?
            .lock()
            .map_err(|_| ManagedErrorCode::ManagedUnavailable)?;
        let filter = request.filter.unwrap_or_default();
        let filter_key = catalog_filter_key(&filter);
        // An empty cursor establishes the baseline; it never replays history.
        let after = if request.cursor.is_empty() {
            catalog.revision()
        } else {
            let Some((boot, rest)) = request.cursor.split_once(':') else {
                return Err(ManagedErrorCode::ManagedConflict);
            };
            let Some((revision, cursor_filter)) = rest.split_once(':') else {
                return Err(ManagedErrorCode::ManagedConflict);
            };
            if boot != self.fence.service_boot_id || cursor_filter != filter_key {
                return Err(ManagedErrorCode::ManagedConflict);
            }
            revision
                .parse::<u64>()
                .map_err(|_| ManagedErrorCode::ManagedConflict)?
        };
        let batch = catalog
            .watch(after, limit, |row| catalog_filter_matches(&filter, row))
            .map_err(|_| ManagedErrorCode::ManagedConflict)?;
        let events = batch
            .events
            .into_iter()
            .map(|(change, row)| AccountCatalogEvent {
                change: match change {
                    guardian_catalog::Change::Upserted => AccountCatalogChange::Upserted as i32,
                    guardian_catalog::Change::Removed => AccountCatalogChange::Removed as i32,
                },
                row: Some(catalog_row(&row)),
            })
            .collect();
        Ok(managed_response::Result::AccountCatalogEvents(
            AccountCatalogEvents {
                events,
                next: Some(format!("{}:{}:{}", self.fence.service_boot_id, batch.next, filter_key)),
                gap: batch.gap,
            },
        ))
    }

    #[cfg(not(unix))]
    fn dispatch_catalog_watch(
        &mut self,
        _request: AccountCatalogWatch,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        Err(ManagedErrorCode::ManagedFeatureUnavailable)
    }

    /// Account-owned stop batch; receipts report requests, never task completion.
    #[cfg(unix)]
    fn dispatch_account_stop(
        &mut self,
        request_id: Uuid,
        request: AccountStop,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        self.check_account_fence(request.fence.as_ref())?;
        if !self.account_operator {
            return Err(ManagedErrorCode::ManagedPermissionDenied);
        }
        if !request.project_id.is_empty() && !valid_uuid(&request.project_id) {
            return Err(ManagedErrorCode::ManagedInvalidInput);
        }
        let project = (!request.project_id.is_empty()).then_some(request.project_id.as_str());
        let target = match request.target.as_ref() {
            Some(target) => {
                if !valid_uuid(&target.session_id) || !valid_uuid(&target.run_id) {
                    return Err(ManagedErrorCode::ManagedInvalidInput);
                }
                Some((target.session_id.as_str(), target.run_id.as_str()))
            }
            None => None,
        };
        let runs = self.terminals.stop_guarded(request_id, project, target)?;
        let mut partial = false;
        let receipts = runs
            .into_iter()
            .map(|run| {
                let outcome = if !run.stop_requested {
                    partial = true;
                    AccountStopOutcome::Unknown
                } else if run.exit_code.is_some() {
                    AccountStopOutcome::Observed
                } else {
                    partial = true;
                    AccountStopOutcome::Requested
                };
                AccountStopReceipt {
                    session_id: run.session_id,
                    run_id: run.run_id,
                    outcome: outcome as i32,
                    exit_status: run.exit_code,
                }
            })
            .collect();
        Ok(managed_response::Result::AccountStopReceipts(
            AccountStopReceipts {
                receipts,
                status: if partial {
                    AccountStopStatus::Partial as i32
                } else {
                    AccountStopStatus::Completed as i32
                },
            },
        ))
    }

    #[cfg(not(unix))]
    fn dispatch_account_stop(
        &mut self,
        _request_id: Uuid,
        _request: AccountStop,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        Err(ManagedErrorCode::ManagedFeatureUnavailable)
    }

    fn open_project(
        &mut self,
        root: &Path,
        expected_id: Option<&str>,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        let root = root.canonicalize().map_err(path_error)?;
        let canonical_root = root.to_str().filter(|_| root.is_dir())
            .ok_or(ManagedErrorCode::ManagedInvalidInput)?.to_owned();
        let handle = open_root(&root).map_err(path_error)?;
        let (id, root_identity) = self.identity.project(&handle).map_err(path_error)?;
        if expected_id.is_some_and(|expected| expected != id) {
            return Err(ManagedErrorCode::ManagedStaleAttachment);
        }
        self.terminals.disconnect(&self.fence.connection_id);
        self.files.clear();
        self.project = Some(Project { root, handle });
        self.fence.project_id = id;
        Ok(managed_response::Result::ProjectOpened(ProjectOpened {
            fence: Some(self.fence.clone()),
            canonical_root,
            root_identity,
        }))
    }

    #[cfg(unix)]
    fn open_guarded_project(
        &mut self,
        request: GuardedProjectOpen,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        self.check_fence(request.fence.as_ref(), false)?;
        if self.account_guardian.is_none() || !self.account_operator || self.project.is_some() {
            return Err(ManagedErrorCode::ManagedPermissionDenied);
        }
        if !valid_uuid(&request.session_id) || !valid_uuid(&request.run_id) {
            return Err(ManagedErrorCode::ManagedInvalidInput);
        }
        let (root, project_id) = self.terminals.guarded_project(
            &self.fence, &request.session_id, &request.run_id,
        )?;
        self.open_project(&root, Some(&project_id))
    }

    #[cfg(not(unix))]
    fn open_guarded_project(
        &mut self,
        _request: GuardedProjectOpen,
    ) -> Result<managed_response::Result, ManagedErrorCode> {
        Err(ManagedErrorCode::ManagedFeatureUnavailable)
    }

    fn check_project(&self, fence: Option<&ManagedFence>) -> Result<(), ManagedErrorCode> {
        self.check_fence(fence, true)?;
        if !same_root(self.project.as_ref().unwrap()).map_err(path_error)? {
            return Err(ManagedErrorCode::ManagedStaleAttachment);
        }
        Ok(())
    }

    fn check_fence(
        &self,
        fence: Option<&ManagedFence>,
        project: bool,
    ) -> Result<(), ManagedErrorCode> {
        let Some(fence) = fence else {
            return Err(ManagedErrorCode::ManagedStaleAttachment);
        };
        if !self.initialized
            || fence.service_id != self.fence.service_id
            || fence.service_boot_id != self.fence.service_boot_id
            || fence.connection_id != self.fence.connection_id
            || if project {
                self.project.is_none() || fence.project_id != self.fence.project_id
            } else {
                !fence.project_id.is_empty()
            }
        {
            return Err(ManagedErrorCode::ManagedStaleAttachment);
        }
        Ok(())
    }
}

fn path_error(error: std::io::Error) -> ManagedErrorCode {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        ManagedErrorCode::ManagedPermissionDenied
    } else if error.kind() == std::io::ErrorKind::NotFound {
        ManagedErrorCode::ManagedNotFound
    } else {
        ManagedErrorCode::ManagedUnavailable
    }
}

#[cfg(unix)]
pub(crate) fn open_root(root: &Path) -> std::io::Result<File> {
    File::open(root)
}

#[cfg(windows)]
pub(crate) fn open_root(root: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(0x02000000)
        .open(root)
}

#[cfg(unix)]
fn same_root(project: &Project) -> std::io::Result<bool> {
    root_matches(&project.handle, &project.root)
}

#[cfg(unix)]
pub(crate) fn root_matches(handle: &File, root: &Path) -> std::io::Result<bool> {
    use std::os::unix::fs::MetadataExt;
    let owned = handle.metadata()?;
    let current = root.metadata()?;
    Ok(current.is_dir() && owned.dev() == current.dev() && owned.ino() == current.ino())
}

#[cfg(windows)]
fn same_root(project: &Project) -> std::io::Result<bool> {
    root_matches(&project.handle, &project.root)
}

#[cfg(windows)]
pub(crate) fn root_matches(handle: &File, root: &Path) -> std::io::Result<bool> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::{
        Foundation::HANDLE,
        Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION},
    };
    fn identity(file: &File) -> std::io::Result<(u32, u32, u32)> {
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        // Both handles are owned directory handles valid throughout this native call.
        unsafe { GetFileInformationByHandle(HANDLE(file.as_raw_handle()), &mut info) }
            .map_err(std::io::Error::other)?;
        Ok((
            info.dwVolumeSerialNumber,
            info.nFileIndexHigh,
            info.nFileIndexLow,
        ))
    }
    Ok(identity(handle)? == identity(&open_root(root)?)?)
}

/// Serial bounded reads provide backpressure and at most one response in flight.
pub async fn serve_stdio() -> Result<(), ProtocolError> {
    service::proxy_stdio().await
}

pub async fn serve_account_service() -> Result<(), ProtocolError> {
    service::serve().await
}

#[cfg(unix)]
pub async fn serve_guardian() -> Result<(), ProtocolError> {
    guardian::serve().await
}

#[cfg(unix)]
fn guardian_route(name: &str) -> Option<Route> {
    match name {
        "ssh" => Some(Route::Ssh),
        "wsl" if cfg!(all(target_os = "linux", feature = "wsl_companion")) => Some(Route::Wsl),
        _ => None,
    }
}

/// Installed helper only: bridge verified stdio to the account guardian.
/// The GUI invokes this flag on the verified installed helper.
#[cfg(unix)]
pub async fn serve_guardian_stdio(route: &str) -> Result<(), &'static str> {
    let route = guardian_route(route).ok_or("unknown route")?;
    let directory = guardian::scope_directory(route)
        .await
        .map_err(|_| "route scope")?;
    let scope = identity::Identity::open(&directory)
        .map_err(|_| "scope identity")?
        .service_id;
    let (stream, guardian, boot) = guardian::connect(route)
        .await
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::InvalidData => "guardian IPC or guest image verification",
            std::io::ErrorKind::Unsupported => "guardian ABI",
            std::io::ErrorKind::PermissionDenied => "guardian IPC or guest image permissions",
            std::io::ErrorKind::TimedOut => "guardian startup deadline",
            std::io::ErrorKind::WouldBlock => "guardian connection capacity",
            _ => "guardian startup or IPC",
        })?;
    service::forward_stdio(stream, Some(scope), Some((guardian, boot)))
        .await
        .map_err(|error| match error {
            ProtocolError::Io(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                "guardian route identity"
            }
            _ => "managed forwarding",
        })
}

pub async fn serve_guest_mcp(program: String) -> anyhow::Result<()> {
    guests::serve_mcp(program).await
}

pub(crate) fn data_directory() -> std::io::Result<PathBuf> {
    let directory = dirs::data_local_dir()
        .ok_or_else(|| std::io::Error::other("Companion data directory unavailable"))?
        .join("warpai")
        .join(if cfg!(feature = "wsl_companion") { "wsl-remote" } else { "remote" });
    #[cfg(all(feature = "wsl_companion", target_os = "linux"))]
    let directory = {
        let distribution = std::env::var("WSL_DISTRO_NAME")
            .map_err(|_| std::io::Error::other("WSL distribution identity unavailable"))?;
        directory.join(guests::distribution_namespace(&distribution)?)
    };
    Ok(directory)
}

#[cfg(all(feature = "wsl_companion", target_os = "linux"))]
pub(crate) fn guest_preferences_path() -> std::io::Result<PathBuf> {
    Ok(data_directory()?.join("mcp-settings.json"))
}

async fn serve_channel<S>(
    stream: S,
    directory: &Path,
    boot: &str,
    terminals: std::sync::Arc<terminals::Terminals>,
    tasks: std::sync::Arc<tasks::Projects>,
    peer_pid: Option<u32>,
    route: Option<Route>,
    catalog: Option<CatalogHandle>,
    account_guardian: Option<Uuid>,
    account_operator: bool,
) -> Result<(), ProtocolError>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (reader, writer) = tokio::io::split(stream);
    let mut reader = reader.compat();
    let mut writer = writer.compat_write();
    let mut companion = Companion::new(directory)?;
    companion.fence.service_boot_id = boot.into();
    companion.terminals = terminals;
    companion.tasks = tasks;
    companion.peer_pid = peer_pid;
    companion.route = route;
    companion.catalog = catalog;
    companion.account_guardian = account_guardian;
    companion.account_operator = account_operator;
    loop {
        let request =
            match read_message_with_limit::<ClientMessage>(&mut reader, MAX_MANAGED_MESSAGE_SIZE)
                .await
            {
                Ok(request) => request,
                Err(ProtocolError::UnexpectedEof) => return Ok(()),
                Err(error) => return Err(error),
            };
        // Native spawn/lease operations cannot block the async executor or Agent IO.
        let (owner, reply) = tokio::task::spawn_blocking(move || {
            let reply = companion.handle(request);
            (companion, reply)
        })
        .await
        .map_err(std::io::Error::other)?;
        companion = owner;
        write_message_with_limit(&mut writer, &reply, MAX_MANAGED_MESSAGE_SIZE).await?;
    }
}

impl Drop for Companion {
    fn drop(&mut self) {
        self.terminals.disconnect(&self.fence.connection_id);
    }
}

#[cfg(test)]
#[path = "companion_tests.rs"]
mod tests;
