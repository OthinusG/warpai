pub mod event;
pub mod listener;
#[cfg(not(target_family = "wasm"))]
pub(crate) mod plugin_manager;

use std::collections::{HashMap, HashSet};

use warpui::{Entity, EntityId, ModelContext, ModelHandle, SingletonEntity};

use crate::ai::blocklist::InputConfig;

use self::listener::CLIAgentSessionListener;
use super::CLIAgent;
use event::{CLIAgentEvent, CLIAgentEventType};

/// Status of a tracked CLI agent session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CLIAgentSessionStatus {
    Starting,
    Idle,
    InProgress,
    Success,
    Blocked { message: Option<String> },
}

impl CLIAgentSessionStatus {
    pub fn to_conversation_status(&self) -> Option<crate::ai::agent::conversation::ConversationStatus> {
        use crate::ai::agent::conversation::ConversationStatus;
        match self {
            Self::Starting | Self::Idle => None,
            Self::InProgress => Some(ConversationStatus::InProgress),
            Self::Success => Some(ConversationStatus::Success),
            Self::Blocked { message } => Some(ConversationStatus::Blocked {
                blocked_action: message.clone().unwrap_or_default(),
            }),
        }
    }
}

/// Rich context accumulated from CLI agent session events.
#[derive(Debug, Clone, Default)]
pub struct CLIAgentSessionContext {
    pub cwd: Option<String>,
    pub project: Option<String>,
    pub session_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input_preview: Option<String>,
    pub summary: Option<String>,
    pub query: Option<String>,
    pub response: Option<String>,
}

/// State of the rich input editor for composing a prompt to send to a CLI agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CLIAgentInputState {
    /// The rich input editor is not open.
    Closed,
    /// The rich input editor is open.
    Open {
        /// How this session was opened (for telemetry).
        entrypoint: CLIAgentInputEntrypoint,
        /// The input config that was active before opening rich input.
        previous_input_config: InputConfig,
        /// Whether the previous lock state was established while the input buffer was empty.
        previous_was_lock_set_with_empty_buffer: bool,
    },
}

/// Why the CLI agent rich input was closed (for telemetry).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CLIAgentRichInputCloseReason {
    /// User explicitly closed (Escape, Ctrl-G, footer button).
    Manual,
    /// Auto-closed due to agent status change (e.g. Blocked).
    AutoToggle,
    /// Auto-dismissed after submitting a prompt.
    Submit,
    /// Closed for another reason (chip removed, session ended, shared session sync).
    Other,
}

/// How a [`CLIAgentInputState`] was opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CLIAgentInputEntrypoint {
    /// User pressed Ctrl-G while a CLI agent was active.
    CtrlG,
    /// User clicked the rich input button in the CLI agent footer.
    FooterButton,
    /// Automatically opened when the CLI agent resumed work (left a blocked state)
    /// and the auto-show setting is enabled.
    AutoShow,
    /// Rich input was opened to mirror a shared-session participant's state.
    SharedSessionSync,
}

impl CLIAgentSessionContext {
    pub(crate) fn display_title(&self) -> Option<String> {
        self.latest_user_prompt().or_else(|| self.title_like_text())
    }

    pub(crate) fn latest_user_prompt(&self) -> Option<String> {
        self.query
            .as_deref()
            .map(str::trim)
            .filter(|query| !query.is_empty())
            .map(str::to_owned)
    }

    /// Returns summary text suitable as a fallback title when no user prompt is available.
    pub(crate) fn title_like_text(&self) -> Option<String> {
        self.summary
            .as_deref()
            .map(str::trim)
            .filter(|summary| !summary.is_empty())
            .map(str::to_owned)
    }
}

/// A tracked CLI agent session.
#[derive(Debug, Clone)]
pub struct CLIAgentSession {
    pub agent: CLIAgent,
    pub status: CLIAgentSessionStatus,
    pub session_context: CLIAgentSessionContext,
    /// Rich input editor state.
    pub input_state: CLIAgentInputState,
    /// Whether status-driven auto-toggle is enabled for this session.
    pub should_auto_toggle_input: bool,
    /// Plugin-backed event listener, if the CLI agent plugin is installed.
    /// `None` for sessions created by command detection alone.
    /// Dropping this handle cleans up the listener's PTY event subscription.
    pub listener: Option<ModelHandle<CLIAgentSessionListener>>,
    /// The plugin version reported by the `SessionStart` event.
    /// `None` if the plugin predates version reporting or hasn't connected yet.
    pub plugin_version: Option<String>,
    /// `None` when the session is local.
    /// `Some("user@hostname")` when running over SSH (warpified or legacy).
    /// Used as a key for per-host plugin install failure tracking.
    pub remote_host: Option<String>,
    /// Draft text saved from the rich input composer when it was closed.
    /// Restored into the editor when the composer is reopened.
    pub draft_text: Option<String>,
    /// When the session was detected via a custom toolbar command pattern,
    /// the first word of the command (the binary/alias the user typed).
    /// Used to customize plugin instructions and force manual install mode.
    pub custom_command_prefix: Option<String>,
}

impl CLIAgentSession {
    pub fn is_remote(&self) -> bool {
        self.remote_host.is_some()
    }

    /// Applies an event to this session, updating context and status.
    /// Returns the new status if it changed, or `None` if the event was irrelevant.
    fn apply_event(&mut self, event: &CLIAgentEvent) -> Option<CLIAgentSessionStatus> {
        self.session_context.cwd = event.cwd.clone().or(self.session_context.cwd.take());
        self.session_context.project = event
            .project
            .clone()
            .or(self.session_context.project.take());
        self.session_context.session_id = event
            .session_id
            .clone()
            .or(self.session_context.session_id.take());

        let new_status = match &event.event {
            CLIAgentEventType::PromptSubmit => {
                self.session_context.query = event.payload.query.clone();
                self.session_context.response = None;
                CLIAgentSessionStatus::InProgress
            }
            CLIAgentEventType::ToolComplete => {
                if !matches!(self.status, CLIAgentSessionStatus::Blocked { .. }) {
                    return None;
                }
                CLIAgentSessionStatus::InProgress
            }
            CLIAgentEventType::Stop => {
                self.session_context.query = event.payload.query.clone();
                self.session_context.response = event.payload.response.clone();
                CLIAgentSessionStatus::Success
            }
            CLIAgentEventType::PermissionRequest => {
                self.session_context.summary = event.payload.summary.clone();
                self.session_context.tool_name = event.payload.tool_name.clone();
                self.session_context.tool_input_preview = event.payload.tool_input_preview.clone();
                CLIAgentSessionStatus::Blocked {
                    message: event.payload.summary.clone(),
                }
            }
            CLIAgentEventType::QuestionAsked => CLIAgentSessionStatus::Blocked {
                message: event
                    .payload
                    .summary
                    .clone()
                    .or_else(|| Some("Waiting for your answer".to_owned())),
            },
            CLIAgentEventType::PermissionReplied => {
                if !matches!(self.status, CLIAgentSessionStatus::Blocked { .. }) {
                    return None;
                }
                CLIAgentSessionStatus::InProgress
            }
            CLIAgentEventType::IdlePrompt => {
                if matches!(self.status, CLIAgentSessionStatus::Success | CLIAgentSessionStatus::Idle | CLIAgentSessionStatus::Blocked { .. }) {
                    return None;
                }
                CLIAgentSessionStatus::Idle
            },
            CLIAgentEventType::SessionStart => {
                self.plugin_version = event.payload.plugin_version.clone();
                return None;
            }
            CLIAgentEventType::Unknown(_) => return None,
        };

        self.status = new_status.clone();
        Some(new_status)
    }
}

/// Events emitted by `CLIAgentSessionsModel` for subscribers (e.g., `AgentNotificationsModel`).
#[allow(dead_code)] // `agent` fields on Started/InputSessionChanged/Ended are used for logging and future subscribers.
#[derive(Debug, Clone)]
pub enum CLIAgentSessionsModelEvent {
    Started {
        terminal_view_id: EntityId,
        agent: CLIAgent,
    },
    /// Broker-verified projection; never feeds plugin/cloud completion or MCP back into itself.
    NativeActivityObserved {
        terminal_view_id: EntityId,
        agent: CLIAgent,
        activity: warp_agent_bus::readiness::Activity,
    },
    StatusChanged {
        terminal_view_id: EntityId,
        agent: CLIAgent,
        status: CLIAgentSessionStatus,
        session_context: Box<CLIAgentSessionContext>,
    },
    InputSessionChanged {
        terminal_view_id: EntityId,
        agent: CLIAgent,
        /// The input state BEFORE this change. When transitioning from
        /// `Open` → `Closed`, contains the saved input config to restore.
        previous_input_state: CLIAgentInputState,
        /// The input state AFTER this change.
        new_input_state: CLIAgentInputState,
    },
    Ended {
        terminal_view_id: EntityId,
        agent: CLIAgent,
    },
    /// The agent session has been updated. Subscribers may use this as a trigger for best-effort
    /// saving of state derived from the agent's session.
    SessionUpdated {
        terminal_view_id: EntityId,
        agent: CLIAgent,
    },
}

impl CLIAgentSessionsModelEvent {
    pub fn terminal_view_id(&self) -> EntityId {
        match self {
            CLIAgentSessionsModelEvent::Started {
                terminal_view_id, ..
            }
            | CLIAgentSessionsModelEvent::NativeActivityObserved {
                terminal_view_id, ..
            }
            | CLIAgentSessionsModelEvent::StatusChanged {
                terminal_view_id, ..
            }
            | CLIAgentSessionsModelEvent::InputSessionChanged {
                terminal_view_id, ..
            }
            | CLIAgentSessionsModelEvent::Ended {
                terminal_view_id, ..
            }
            | CLIAgentSessionsModelEvent::SessionUpdated {
                terminal_view_id, ..
            } => *terminal_view_id,
        }
    }
}

/// Singleton model that tracks pane-scoped CLI agent state and plugin-enriched session context.
pub struct CLIAgentSessionsModel {
    sessions: HashMap<EntityId, CLIAgentSession>,
    observed_status: HashMap<EntityId, (warp_agent_bus::readiness::Activity, CLIAgentSessionStatus)>,
    pending_peer_work: HashSet<EntityId>,
    /// Tracks (agent, remote_host) pairs where an auto plugin operation (install or update) has failed.
    /// Shared across all views so failure in one tab is reflected everywhere.
    plugin_auto_failures: HashSet<(CLIAgent, Option<String>)>,
}

impl Entity for CLIAgentSessionsModel {
    type Event = CLIAgentSessionsModelEvent;
}

impl SingletonEntity for CLIAgentSessionsModel {}

impl CLIAgentSessionsModel {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            observed_status: HashMap::new(),
            pending_peer_work: HashSet::new(),
            plugin_auto_failures: HashSet::new(),
        }
    }

    pub fn session(&self, terminal_view_id: EntityId) -> Option<&CLIAgentSession> {
        self.sessions.get(&terminal_view_id)
    }

    pub fn has_pending_peer_work(&self, terminal_view_id: EntityId) -> bool {
        self.pending_peer_work.contains(&terminal_view_id)
    }

    /// A UI projection of unacknowledged MCP work; focusing a tab never consumes it.
    pub(crate) fn observe_pending_peer_work(&mut self, terminal_view_id: EntityId, pending: bool, ctx: &mut ModelContext<Self>) {
        let pending = pending && self.sessions.contains_key(&terminal_view_id);
        let changed = if pending { self.pending_peer_work.insert(terminal_view_id) }
            else { self.pending_peer_work.remove(&terminal_view_id) };
        if changed {
            if let Some(session) = self.session(terminal_view_id) {
                ctx.emit(CLIAgentSessionsModelEvent::SessionUpdated { terminal_view_id, agent:session.agent });
            }
            ctx.notify();
        }
    }

    /// Verified grid status is available even when the Agent has no rich plugin.
    pub fn status_for_display(&self, terminal_view_id: EntityId) -> Option<crate::ai::agent::conversation::ConversationStatus> {
        let session = self.session(terminal_view_id)?;
        self.observed_status.get(&terminal_view_id).map(|(_, status)| status)
            .or_else(|| (session.listener.is_some() && listener::agent_supports_rich_status(&session.agent)).then_some(&session.status))
            .and_then(CLIAgentSessionStatus::to_conversation_status)
    }

    /// Broker projections update presentation without claiming a native completion event.
    pub(crate) fn observe_activity(
        &mut self, terminal_view_id: EntityId,
        activity: warp_agent_bus::readiness::Activity, native_screen: bool, ctx: &mut ModelContext<Self>,
    ) {
        use warp_agent_bus::readiness::Activity;
        let Some(session) = self.sessions.get_mut(&terminal_view_id) else { return; };
        if !native_screen && matches!(session.status, CLIAgentSessionStatus::Blocked { .. }) { return; }
        let status = match activity {
            Activity::Starting => CLIAgentSessionStatus::Starting,
            Activity::Idle if session.status == CLIAgentSessionStatus::Success && listener::agent_supports_rich_status(&session.agent) => CLIAgentSessionStatus::Success,
            Activity::Idle => CLIAgentSessionStatus::Idle,
            Activity::Working => CLIAgentSessionStatus::InProgress,
            Activity::WaitingApproval | Activity::WaitingInput | Activity::Error => CLIAgentSessionStatus::Blocked { message: None },
            _ => return,
        };
        let tracked = native_screen || self.observed_status.contains_key(&terminal_view_id);
        let previous = tracked.then(|| self.observed_status.insert(terminal_view_id, (activity, status.clone()))).flatten();
        let observed_changed = tracked && previous.as_ref() != Some(&(activity, status.clone()));
        if session.status == status && !observed_changed { return; }
        session.status = status;
        if observed_changed {
            ctx.emit(CLIAgentSessionsModelEvent::NativeActivityObserved {
                terminal_view_id, agent:session.agent, activity,
            });
        }
        ctx.emit(CLIAgentSessionsModelEvent::SessionUpdated { terminal_view_id, agent: session.agent });
        ctx.notify();
    }

    /// Returns `true` if any tracked session has an agent actively working.
    pub fn any_in_progress(&self) -> bool {
        self.sessions.iter().any(|(id, session)| self.observed_status.get(id)
            .map(|(activity, _)| *activity == warp_agent_bus::readiness::Activity::Working)
            .unwrap_or_else(|| matches!(session.status, CLIAgentSessionStatus::InProgress)))
    }

    /// Returns `true` if the rich input editor is currently open for this terminal.
    pub fn is_input_open(&self, terminal_view_id: EntityId) -> bool {
        self.sessions
            .get(&terminal_view_id)
            .is_some_and(|s| matches!(s.input_state, CLIAgentInputState::Open { .. }))
    }

    /// Registers a plugin-backed listener on the session for this terminal.
    ///
    /// If a session for the same agent already exists (e.g. created earlier by
    /// command detection), it is upgraded with the listener and plugin context.
    /// Otherwise a new session is created.
    ///
    /// The optional `cwd` / `project` / `session_id` fields supply initial
    /// context when available (e.g. from a `SessionStart` event). Passing
    /// `None` for all three is fine — happens when the plugin is installed
    /// mid-session and there is no start event to extract context from.
    #[allow(clippy::too_many_arguments)]
    pub fn register_listener(
        &mut self,
        terminal_view_id: EntityId,
        agent: CLIAgent,
        cwd: Option<String>,
        project: Option<String>,
        session_id: Option<String>,
        plugin_version: Option<String>,
        remote_host: Option<String>,
        should_auto_toggle_input: bool,
        listener: ModelHandle<CLIAgentSessionListener>,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(session) = self
            .sessions
            .get_mut(&terminal_view_id)
            .filter(|s| s.agent == agent)
        {
            // Upgrade existing session with plugin context.
            session.listener = Some(listener);
            session.plugin_version = plugin_version;
            session.remote_host = remote_host;
            session.should_auto_toggle_input = should_auto_toggle_input;
            session.session_context.cwd = cwd.or(session.session_context.cwd.take());
            session.session_context.project = project.or(session.session_context.project.take());
            session.session_context.session_id =
                session_id.or(session.session_context.session_id.take());
            return;
        }

        self.set_session(
            terminal_view_id,
            CLIAgentSession {
                agent,
                status: CLIAgentSessionStatus::Starting,
                session_context: CLIAgentSessionContext {
                    cwd,
                    project,
                    session_id,
                    ..Default::default()
                },
                input_state: CLIAgentInputState::Closed,
                should_auto_toggle_input,
                listener: Some(listener),
                plugin_version,
                remote_host,
                draft_text: None,
                custom_command_prefix: None,
            },
            ctx,
        );
    }

    pub fn remove_session(&mut self, terminal_view_id: EntityId, ctx: &mut ModelContext<Self>) {
        self.observed_status.remove(&terminal_view_id);
        self.pending_peer_work.remove(&terminal_view_id);
        if let Some(session) = self.sessions.remove(&terminal_view_id) {
            ctx.emit(CLIAgentSessionsModelEvent::Ended {
                terminal_view_id,
                agent: session.agent,
            });
        }
    }

    /// Updates the session's status and context from a parsed CLI agent event.
    pub fn update_from_event(
        &mut self,
        terminal_view_id: EntityId,
        event: &CLIAgentEvent,
        ctx: &mut ModelContext<Self>,
    ) {
        let Some(session) = self.sessions.get_mut(&terminal_view_id) else {
            return;
        };

        let event_type = &event.event;
        if let Some(new_status) = session.apply_event(event) {
            let agent = session.agent;
            ctx.emit(CLIAgentSessionsModelEvent::StatusChanged {
                terminal_view_id,
                agent,
                status: new_status,
                session_context: Box::new(session.session_context.clone()),
            });
        }

        if matches!(
            event_type,
            CLIAgentEventType::SessionStart
                | CLIAgentEventType::PromptSubmit
                | CLIAgentEventType::ToolComplete
        ) {
            ctx.emit(CLIAgentSessionsModelEvent::SessionUpdated {
                terminal_view_id,
                agent: session.agent,
            });
        }
    }

    pub fn open_input(
        &mut self,
        terminal_view_id: EntityId,
        entrypoint: CLIAgentInputEntrypoint,
        previous_input_config: InputConfig,
        previous_was_lock_set_with_empty_buffer: bool,
        should_auto_toggle_input: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let Some(session) = self.sessions.get_mut(&terminal_view_id) else {
            return;
        };

        let previous_input_state = session.input_state;
        session.input_state = CLIAgentInputState::Open {
            entrypoint,
            previous_input_config,
            previous_was_lock_set_with_empty_buffer,
        };
        session.should_auto_toggle_input = should_auto_toggle_input;

        ctx.emit(CLIAgentSessionsModelEvent::InputSessionChanged {
            terminal_view_id,
            agent: session.agent,
            previous_input_state,
            new_input_state: session.input_state,
        });
    }

    pub fn close_input(
        &mut self,
        terminal_view_id: EntityId,
        should_auto_toggle_input: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let Some(session) = self.sessions.get_mut(&terminal_view_id) else {
            return;
        };
        if session.input_state == CLIAgentInputState::Closed {
            return;
        }

        let previous_input_state = session.input_state;
        session.input_state = CLIAgentInputState::Closed;
        session.should_auto_toggle_input = should_auto_toggle_input;
        ctx.emit(CLIAgentSessionsModelEvent::InputSessionChanged {
            terminal_view_id,
            agent: session.agent,
            previous_input_state,
            new_input_state: CLIAgentInputState::Closed,
        });
    }

    pub fn set_session(
        &mut self,
        terminal_view_id: EntityId,
        session: CLIAgentSession,
        ctx: &mut ModelContext<Self>,
    ) {
        let agent = session.agent;
        self.observed_status.remove(&terminal_view_id);
        self.pending_peer_work.remove(&terminal_view_id);
        // Close any open rich input before replacing, so subscribers can
        // restore input config before the session ends.
        self.close_input(terminal_view_id, false, ctx);
        if let Some(old) = self.sessions.insert(terminal_view_id, session) {
            ctx.emit(CLIAgentSessionsModelEvent::Ended {
                terminal_view_id,
                agent: old.agent,
            });
        }

        ctx.emit(CLIAgentSessionsModelEvent::Started {
            terminal_view_id,
            agent,
        });
    }

    /// Records that an auto plugin operation (install or update) failed for the given agent/host.
    /// `remote_host` is `None` for local sessions, `Some("user@hostname")` for remote.
    #[cfg(not(target_family = "wasm"))]
    pub fn record_plugin_auto_failure(&mut self, agent: CLIAgent, remote_host: Option<String>) {
        self.plugin_auto_failures.insert((agent, remote_host));
    }

    /// Saves draft text from the rich input composer for the given terminal.
    /// Stores `None` for empty or whitespace-only text.
    pub fn set_draft(&mut self, terminal_view_id: EntityId, text: String) {
        if let Some(session) = self.sessions.get_mut(&terminal_view_id) {
            session.draft_text = if text.trim().is_empty() {
                None
            } else {
                Some(text)
            };
        }
    }

    /// Clears any saved draft text for the given terminal.
    pub fn clear_draft(&mut self, terminal_view_id: EntityId) {
        if let Some(session) = self.sessions.get_mut(&terminal_view_id) {
            session.draft_text = None;
        }
    }

    /// Returns and clears the draft text for the given terminal, if any.
    pub fn take_draft(&mut self, terminal_view_id: EntityId) -> Option<String> {
        self.sessions
            .get_mut(&terminal_view_id)
            .and_then(|s| s.draft_text.take())
    }

    /// Whether an auto plugin operation has previously failed for this agent on this host.
    pub fn has_plugin_auto_failed(&self, agent: CLIAgent, remote_host: &Option<String>) -> bool {
        self.plugin_auto_failures
            .contains(&(agent, remote_host.clone()))
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
