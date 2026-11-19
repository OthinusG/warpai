use super::*;
use crate::editor::EditorView;
use warp_agent_bus::companion_ctl::GuardedProcess;
use warpui::elements::ChildView;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ProcessAction {
    Open,
    Back,
    Resume(String),
    Rename(String),
    SaveName,
    Stop(String),
    ConfirmStop,
    Logs(String),
}

#[derive(Default)]
pub(super) struct Processes {
    target: Option<(SshProfile, SshConnection)>,
    generation: u64,
    rows: Option<Vec<GuardedProcess>>,
    open: bool,
    pending: bool,
    last_poll: Option<Instant>,
    rename: Option<(String, ViewHandle<EditorView>)>,
    stop: Option<String>,
    logs: Option<String>,
    message: Option<&'static str>,
    scroll: ClippedScrollStateHandle,
    buttons: std::cell::RefCell<HashMap<ProcessAction, MouseStateHandle>>,
}

impl Processes {
    pub(super) fn reset(&mut self) {
        *self = Self {
            generation: self.generation.wrapping_add(1),
            ..Self::default()
        };
    }

    pub(super) fn is_open(&self) -> bool {
        self.open
    }

    fn button(
        &self,
        appearance: &Appearance,
        label: &str,
        action: ProcessAction,
        enabled: bool,
    ) -> Box<dyn Element> {
        let style = if matches!(action, ProcessAction::ConfirmStop) {
            ButtonVariant::Secondary
        } else {
            ButtonVariant::Text
        };
        let state = self
            .buttons
            .borrow_mut()
            .entry(action.clone())
            .or_default()
            .clone();
        let button = appearance
            .ui_builder()
            .button(style, state)
            .with_text_label(warpui::localization::text(label).into());
        let button = if enabled { button } else { button.disabled() };
        button
            .build()
            .on_click(move |ctx, _, _| {
                ctx.dispatch_typed_action(Action::GuardedProcess(action.clone()))
            })
            .finish()
    }

    pub(super) fn entry(&self, appearance: &Appearance) -> Option<Box<dyn Element>> {
        self.rows.as_ref()?;
        Some(self.button(appearance, "Warpai Processes", ProcessAction::Open, true))
    }

    pub(super) fn render(&self, appearance: &Appearance) -> Box<dyn Element> {
        let mut body = Flex::column().with_spacing(GAP_ROW);
        if let Some(message) = self.message {
            body.add_child(note(appearance, warpui::localization::text(message)));
        }
        if let Some(logs) = &self.logs {
            body.add_child(detail(appearance, logs.clone()));
        }
        if let Some((_, editor)) = &self.rename {
            body.add_child(ChildView::new(editor).finish());
            body.add_child(self.button(appearance, "Save", ProcessAction::SaveName, !self.pending));
        }
        if self.stop.is_some() {
            body.add_child(self.button(
                appearance,
                "Stop task",
                ProcessAction::ConfirmStop,
                !self.pending,
            ));
        }
        if let Some(rows) = &self.rows {
            if rows.is_empty() {
                body.add_child(note(
                    appearance,
                    warpui::localization::text("No guarded processes"),
                ));
            }
            for row in rows {
                let live = row.state == "live";
                let available = live || row.state == "ended";
                let mut buttons = Wrap::row()
                    .with_spacing(GAP_ROW)
                    .with_run_spacing(GAP_TIGHT);
                for (caption, action, enabled) in [
                    (
                        "Resume",
                        ProcessAction::Resume(row.id.clone()),
                        live && row.attached != Some(true),
                    ),
                    ("Rename", ProcessAction::Rename(row.id.clone()), available),
                    ("Logs", ProcessAction::Logs(row.id.clone()), available),
                    ("Stop", ProcessAction::Stop(row.id.clone()), live),
                ] {
                    buttons.add_child(self.button(
                        appearance,
                        caption,
                        action,
                        enabled && !self.pending,
                    ));
                }
                let mut item = Flex::column().with_spacing(GAP_TIGHT);
                item.add_child(heading(
                    appearance,
                    row.name.clone().unwrap_or_else(|| row.id.clone()),
                ));
                item.add_child(note(
                    appearance,
                    row.program
                        .clone()
                        .unwrap_or_else(|| warpui::localization::text("Unknown process").into()),
                ));
                item.add_child(note(
                    appearance,
                    warpui::localization::text(&row.state).to_owned(),
                ));
                item.add_child(buttons.finish());
                body.add_child(section(
                    appearance,
                    item.finish(),
                    Padding::default().with_bottom(GAP_ROW),
                ));
            }
        }
        let theme = appearance.theme();
        let scroll = ClippedScrollable::vertical(
            self.scroll.clone(),
            body.finish(),
            ScrollbarWidth::Auto,
            theme.nonactive_ui_detail().into(),
            theme.active_ui_detail().into(),
            Fill::None,
        )
        .with_overlayed_scrollbar()
        .finish();
        Container::new(
            Flex::column()
                .with_spacing(GAP_ROW)
                .with_child(self.button(appearance, "Back", ProcessAction::Back, true))
                .with_child(heading(
                    appearance,
                    warpui::localization::text("Warpai Processes"),
                ))
                .with_child(Expanded::new(1., scroll).finish())
                .finish(),
        )
        .with_padding(Padding::uniform(GAP_SECTION))
        .finish()
    }
}

impl CollaborationPanel {
    pub(super) fn refresh_processes(&mut self, ctx: &mut ViewContext<Self>) {
        let target = self.selected_ssh(ctx);
        if self.processes.target != target {
            self.processes.reset();
            self.processes.target = target;
        }
        let Some((profile, connection)) = self.processes.target.clone() else {
            return;
        };
        if self.processes.pending
            || self
                .processes
                .last_poll
                .is_some_and(|last| last.elapsed() < Duration::from_secs(5))
        {
            return;
        }
        let generation = self.processes.generation;
        self.processes.last_poll = Some(Instant::now());
        self.processes.pending = true;
        ctx.spawn(
            async move {
                let (code, bytes) = connection
                    .guardian_cli(&profile, &["list", "--json"])
                    .await
                    .ok()?;
                if code != 0 {
                    return None;
                }
                let mut rows = serde_json::from_slice::<Vec<GuardedProcess>>(&bytes).ok()?;
                if rows.len() > 64 {
                    return None;
                }
                rows.retain(|row| uuid::Uuid::parse_str(&row.id).is_ok());
                Some(rows)
            },
            move |panel, rows, ctx| {
                if !panel.process_target_current(generation, ctx) {
                    return;
                }
                panel.processes.pending = false;
                if let Some(rows) = &rows {
                    panel
                        .processes
                        .buttons
                        .borrow_mut()
                        .retain(|action, _| match action {
                            ProcessAction::Resume(id)
                            | ProcessAction::Rename(id)
                            | ProcessAction::Stop(id)
                            | ProcessAction::Logs(id) => rows.iter().any(|row| &row.id == id),
                            _ => true,
                        });
                } else if panel.processes.open {
                    panel.processes.message = Some("Guarded processes unavailable");
                }
                panel.processes.rows = rows;
                ctx.notify();
            },
        );
    }

    fn process_target_current(&self, generation: u64, ctx: &ViewContext<Self>) -> bool {
        self.visible
            && self.processes.generation == generation
            && self.processes.target == self.selected_ssh(ctx)
    }

    pub(super) fn process_action(&mut self, action: &ProcessAction, ctx: &mut ViewContext<Self>) {
        let generation = self.processes.generation;
        if !self.process_target_current(generation, ctx) {
            return;
        }
        let Some((profile, connection)) = self.processes.target.clone() else {
            return;
        };
        match action {
            ProcessAction::Open => {
                self.processes.open = self.processes.rows.is_some();
                ctx.notify();
                return;
            }
            ProcessAction::Back => {
                self.processes.open = false;
                self.processes.rename = None;
                self.processes.stop = None;
                self.processes.logs = None;
                self.processes.message = None;
                ctx.notify();
                return;
            }
            _ if !self.processes.open || self.processes.pending => return,
            _ => (),
        }
        let (operation, id, name) = match action {
            ProcessAction::Resume(id) => {
                if let Ok(id) = uuid::Uuid::parse_str(id) {
                    if let Ok(arguments) = connection.guardian_resume_argv(&profile, id) {
                        if self.resume_in_new_pane(arguments, ctx) {
                            return;
                        }
                    }
                }
                self.processes.message = Some("Could not open guarded process");
                ctx.notify();
                return;
            }
            ProcessAction::Rename(id) => {
                let Some(row) = self
                    .processes
                    .rows
                    .as_ref()
                    .and_then(|rows| rows.iter().find(|row| &row.id == id))
                else {
                    return;
                };
                let name = row.name.clone().unwrap_or_default();
                let editor = ctx.add_typed_action_view(|ctx| {
                    let mut editor = EditorView::single_line(Default::default(), ctx);
                    editor.set_placeholder_text(warpui::localization::text("New task name"), ctx);
                    editor.set_buffer_text(&name, ctx);
                    editor
                });
                ctx.focus(&editor);
                self.processes.rename = Some((id.clone(), editor));
                self.processes.stop = None;
                ctx.notify();
                return;
            }
            ProcessAction::Stop(id) => {
                self.processes.stop = Some(id.clone());
                self.processes.rename = None;
                ctx.notify();
                return;
            }
            ProcessAction::ConfirmStop => {
                let Some(id) = self.processes.stop.take() else {
                    return;
                };
                ("stop", id, None)
            }
            ProcessAction::Logs(id) => ("logs", id.clone(), None),
            ProcessAction::SaveName => {
                let Some((id, editor)) = self.processes.rename.as_ref() else {
                    return;
                };
                let name = editor.as_ref(ctx).buffer_text(ctx);
                if name.trim().is_empty() || name.len() > 128 || name.chars().any(char::is_control)
                {
                    self.processes.message = Some("Invalid task name");
                    ctx.notify();
                    return;
                }
                ("rename", id.clone(), Some(name))
            }
            _ => return,
        };
        if uuid::Uuid::parse_str(&id).is_err() {
            return;
        }
        self.processes.pending = true;
        self.processes.message = None;
        self.processes.logs = None;
        ctx.spawn(
            async move {
                let mut arguments = vec![operation, "--id", id.as_str()];
                arguments.extend(name.as_deref());
                let (code, bytes) = connection.guardian_cli(&profile, &arguments).await.ok()?;
                let output = String::from_utf8_lossy(&bytes).into_owned();
                (code == 0
                    || (operation == "stop"
                        && output
                            .lines()
                            .any(|line| line.starts_with(&id) && line.ends_with(" requested"))))
                .then_some(output)
            },
            move |panel, output, ctx| {
                if !panel.process_target_current(generation, ctx) {
                    return;
                }
                panel.processes.pending = false;
                if !panel.processes.open {
                    return;
                }
                if let Some(output) = output {
                    if operation == "logs" {
                        panel.processes.logs = Some(output);
                    } else {
                        panel.processes.rename = None;
                        panel.processes.last_poll = None;
                        panel.refresh_processes(ctx);
                    }
                } else {
                    panel.processes.message = Some("Guarded process command failed");
                }
                ctx.notify();
            },
        );
        ctx.notify();
    }

    fn resume_in_new_pane(&self, arguments: Vec<String>, ctx: &mut ViewContext<Self>) -> bool {
        use crate::{
            pane_group::Direction, root_view::RootView, terminal::available_shells::AvailableShell,
        };
        let Some(root) = ctx.root_view::<RootView>(self.window) else {
            return false;
        };
        let Some(workspace) = root.as_ref(ctx).workspace_view().cloned() else {
            return false;
        };
        let group = workspace.as_ref(ctx).active_tab_pane_group().clone();
        group.update(ctx, |group, ctx| {
            let pane = group.add_terminal_pane_ignoring_default_session_mode(
                Direction::Right,
                Some(AvailableShell::default()),
                ctx,
            );
            let Some(terminal) = group.terminal_view_from_pane_id(pane, ctx) else {
                return false;
            };
            terminal.update(ctx, |view, ctx| view.queue_guardian_resume(arguments, ctx));
            true
        })
    }
}
