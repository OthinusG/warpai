use super::*;
use crate::editor::EditorView;
use crate::terminal::TerminalView;
use warp_agent_bus::companion_ctl::GuardedProcess;
use warpui::elements::ChildView;
use warpui::WeakViewHandle;

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

fn process_can_resume(row: &GuardedProcess) -> bool {
    row.state == "live" && row.attached != Some(true)
}

#[derive(Default)]
pub(super) struct Processes {
    target: Option<(SshProfile, SshConnection)>,
    generation: u64,
    rows: Option<Vec<GuardedProcess>>,
    resuming: HashMap<String, WeakViewHandle<TerminalView>>,
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
            resuming: std::mem::take(&mut self.resuming),
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
                        process_can_resume(row) && !self.resuming.contains_key(&row.id),
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
        if self.prune_resuming_processes(ctx) {
            ctx.notify();
        }
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

    fn prune_resuming_processes(&mut self, app: &AppContext) -> bool {
        let count = self.processes.resuming.len();
        self.processes.resuming.retain(|_, terminal| {
            terminal.upgrade(app).is_some_and(|terminal| {
                let view = terminal.as_ref(app);
                view.has_pending_command_or_awaiting_completion(app)
                    || view.model.lock().has_guarded_task_attachment()
            })
        });
        self.processes.resuming.len() != count
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
                self.prune_resuming_processes(ctx);
                let can_resume = self.processes.rows.as_ref().is_some_and(|rows| {
                    rows.iter()
                        .any(|row| row.id == *id && process_can_resume(row))
                });
                if !can_resume || self.processes.resuming.contains_key(id) {
                    return;
                }
                let terminal = uuid::Uuid::parse_str(id)
                    .ok()
                    .and_then(|task| connection.guardian_resume_argv(&profile, task).ok())
                    .and_then(|arguments| self.resume_in_new_pane(arguments, ctx));
                if let Some(terminal) = terminal {
                    self.processes.resuming.insert(id.clone(), terminal);
                } else {
                    self.processes.message = Some("Could not open guarded process");
                }
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

    fn resume_in_new_pane(
        &self,
        arguments: Vec<String>,
        ctx: &mut ViewContext<Self>,
    ) -> Option<WeakViewHandle<TerminalView>> {
        use crate::{
            pane_group::Direction, root_view::RootView, terminal::available_shells::AvailableShell,
        };
        let root = ctx.root_view::<RootView>(self.window)?;
        let workspace = root.as_ref(ctx).workspace_view().cloned()?;
        let group = workspace.as_ref(ctx).active_tab_pane_group().clone();
        group.update(ctx, |group, ctx| {
            let pane = group.add_terminal_pane_ignoring_default_session_mode(
                Direction::Right,
                Some(AvailableShell::default()),
                ctx,
            );
            let terminal = group.terminal_view_from_pane_id(pane, ctx)?;
            terminal.update(ctx, |view, ctx| view.queue_guardian_resume(arguments, ctx));
            Some(terminal.downgrade())
        })
    }
}

#[cfg(debug_assertions)]
pub(super) fn guarded_resume_steps(wsl: bool) -> Vec<warpui::integration::TestStep> {
    use crate::{
        root_view::RootView,
        workspace::{
            view::left_panel::{LeftPanelAction, LeftPanelView},
            ActiveSession, WorkspaceAction,
        },
    };
    use warpui::integration::TestStep;

    let scope = if wsl { "WSL" } else { "SSH" };
    let label = format!("ui-guardian-{}", uuid::Uuid::new_v4().simple());
    let original = Arc::new(std::sync::Mutex::new(None));
    let expected = Arc::new(std::sync::Mutex::new(None::<(String, String)>));

    let seed = {
        let label = label.clone();
        let original = original.clone();
        let expected = expected.clone();
        TestStep::new(&format!("Start a real guarded {scope} process"))
            .set_timeout(Duration::from_secs(75))
            .with_action(move |app, window, _| {
                let source = app.read(|ctx| {
                    let terminal = ActiveSession::as_ref(ctx).terminal_view_id(window).unwrap();
                    let (profile, connection) = crate::remote_server::selected_session::selected_ssh(ctx, window).unwrap();
                    assert_eq!(connection.is_wsl(), wsl);
                    (terminal, profile, connection)
                });
                let root = app.root_view::<RootView>(window).unwrap();
                let workspace = root.read(app, |view, _| view.workspace_view().unwrap().clone());
                let group = workspace.read(app, |view, _| view.active_tab_pane_group().clone());
                let (pane, count, left_panel_open) = group.read(app, |group, ctx| (
                    group.terminal_pane_ids().find(|&pane| {
                        group.terminal_view_from_pane_id(pane, ctx).is_some_and(|view| view.id() == source.0)
                    }).unwrap(),
                    group.visible_pane_ids().len(),
                    group.left_panel_open,
                ));
                *original.lock().unwrap() = Some((pane, count, source.0, left_panel_open));

                let left = app.views_of_type::<LeftPanelView>(window).unwrap()[0].clone();
                left.update(app, |left, ctx| left.handle_action_with_force_open(&LeftPanelAction::Collaboration, false, ctx));
                if !left_panel_open {
                    workspace.update(app, |workspace, ctx| {
                        workspace.handle_action(&WorkspaceAction::ToggleLeftPanel, ctx);
                    });
                }
                let panel = app.views_of_type::<CollaborationPanel>(window).unwrap()[0].clone();
                let (profile, connection) = (source.1, source.2);
                let label = label.clone();
                let expected = expected.clone();
                panel.update(app, |panel, ctx| {
                    panel.remote = None;
                    panel.refresh_processes(ctx);
                    ctx.spawn(async move {
                        let powershell;
                        let mut command = if profile.remote_shell == warp_agent_bus::ssh_remote::RemoteShell::PowerShell {
                            powershell = format!(r"{}\System32\WindowsPowerShell\v1.0\powershell.exe",
                                std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into()));
                            vec!["run", &powershell, "-NoProfile", "-Command",
                                r#"Write-Output "WARPAI_GUI_PID=$PID"; while (($line = [Console]::ReadLine()) -ne $null) { if ($line -ceq 'warpai-fixture-finish') { Write-Output 'WARPAI_GUI_FINISHED'; break } }"#]
                        } else {
                            vec!["run", "/bin/sh", "-c", "printf 'WARPAI_GUI_PID=%s\n' \"$$\"; while IFS= read -r line; do if [ \"$line\" = warpai-fixture-finish ]; then printf '%s\n' WARPAI_GUI_FINISHED; break; fi; done"]
                        };
                        command.extend(["--name", &label, "--background"]);
                        let (code, reply) = connection.guardian_cli(&profile, &command).await.unwrap();
                        assert_eq!(code, 0, "Owned guardian launch failed");
                        let reply = String::from_utf8(reply).unwrap();
                        let id = reply.trim().strip_prefix("Started task ").unwrap().to_owned();
                        let deadline = Instant::now() + Duration::from_secs(15);
                        loop {
                            let (code, output) = connection.guardian_cli(&profile, &["logs", "--id", &id]).await.unwrap();
                            assert_eq!(code, 0);
                            if let Some(marker) = String::from_utf8_lossy(&output).lines()
                                .find(|line| line.starts_with("WARPAI_GUI_PID="))
                            {
                                break (id, marker.to_owned());
                            }
                            assert!(Instant::now() < deadline, "No PID from the original task");
                            Timer::after(Duration::from_millis(100)).await;
                        }
                    }, move |panel, launched, ctx| {
                        *expected.lock().unwrap() = Some(launched);
                        panel.refresh_processes(ctx);
                    });
                });
            })
    };

    let open = || {
        let expected = expected.clone();
        let label = label.clone();
        TestStep::new(&format!("Open Warpai Processes for {scope}"))
            .set_timeout(Duration::from_secs(75))
            .add_named_assertion(
                "confirmed guardian exposes the named process",
                move |app, window| {
                    let panel = app.views_of_type::<CollaborationPanel>(window).unwrap()[0].clone();
                    let expected = expected.lock().unwrap();
                    warpui::async_assert!(expected.as_ref().is_some_and(|(id, _)| panel.read(
                        app,
                        |panel, ctx| {
                            panel.visible
                                && !panel.processes.pending
                                && panel.processes.entry(Appearance::as_ref(ctx)).is_some()
                                && panel.processes.rows.as_ref().is_some_and(|rows| {
                                    rows.iter().any(|row| {
                                        row.id == *id
                                            && row.name.as_deref() == Some(label.as_str())
                                            && row.program.is_some()
                                            && row.state == "live"
                                            && row.attached == Some(false)
                                    })
                                })
                        }
                    )))
                },
            )
    };

    let resume = || {
        let expected = expected.clone();
        let original = original.clone();
        TestStep::new(&format!("Resume the {scope} task from the process panel"))
            .set_timeout(Duration::from_secs(75))
            .with_action({
                let expected = expected.clone();
                move |app, window, _| {
                    let panel = app.views_of_type::<CollaborationPanel>(window).unwrap()[0].clone();
                    let id = expected.lock().unwrap().as_ref().unwrap().0.clone();
                    panel.update(app, |panel, ctx| {
                        panel.handle_action(&Action::GuardedProcess(ProcessAction::Open), ctx);
                        assert!(panel.processes.is_open(), "Process panel must open");
                        panel.handle_action(&Action::GuardedProcess(ProcessAction::Back), ctx);
                        assert!(
                            !panel.processes.is_open(),
                            "Back must return to collaboration"
                        );
                        panel.handle_action(&Action::GuardedProcess(ProcessAction::Open), ctx);
                        let action = Action::GuardedProcess(ProcessAction::Resume(id.clone()));
                        panel.handle_action(&action, ctx);
                        assert!(panel.processes.resuming.contains_key(&id));
                        panel.handle_action(&action, ctx);
                    });
                }
            })
            .add_named_assertion(
                "the same PID is attached only in a new split",
                move |app, window| {
                    let original = original.lock().unwrap();
                    let (pane, count, terminal, _) = original.as_ref().unwrap();
                    let expected = expected.lock().unwrap();
                    let marker = &expected.as_ref().unwrap().1;
                    let root = app.root_view::<RootView>(window).unwrap();
                    let workspace =
                        root.read(app, |view, _| view.workspace_view().unwrap().clone());
                    let group = workspace.read(app, |view, _| view.active_tab_pane_group().clone());
                    let correct = group.read(app, |group, ctx| {
                        let focused = group.focused_pane_id(ctx);
                        group.visible_pane_ids().len() == count + 1
                            && focused != *pane
                            && group
                                .terminal_view_from_pane_id(*pane, ctx)
                                .is_some_and(|view| {
                                    view.id() == *terminal
                                        && !view
                                            .as_ref(ctx)
                                            .model
                                            .lock()
                                            .has_guarded_task_attachment()
                                })
                            && group
                                .terminal_view_from_pane_id(focused, ctx)
                                .is_some_and(|view| {
                                    let model = view.as_ref(ctx).model.lock();
                                    model.has_guarded_task_attachment()
                                        && model
                                            .block_list()
                                            .active_block()
                                            .output_to_string()
                                            .contains(marker)
                                })
                    });
                    warpui::async_assert!(correct)
                },
            )
    };

    let release = |finish| {
        TestStep::new(if finish {
            "Finish the owned task through the resumed pane"
        } else {
            "Detach the guarded process pane"
        })
        .set_timeout(Duration::from_secs(30))
        .with_action(move |app, window, _| {
            let view = app
                .read(|ctx| ActiveSession::as_ref(ctx).terminal_view_id(window))
                .unwrap();
            let terminal = app
                .views_of_type::<TerminalView>(window)
                .unwrap()
                .into_iter()
                .find(|terminal| terminal.id() == view)
                .unwrap();
            terminal.update(app, |terminal, ctx| {
                if finish {
                    terminal.write_to_pty(&b"warpai-fixture-finish\r"[..], ctx);
                } else {
                    terminal.handle_action(
                        &crate::terminal::view::TerminalAction::DetachGuardedTask,
                        ctx,
                    );
                }
            });
        })
        .add_named_assertion(
            "guarded process view returns to its shell",
            move |app, window| {
                let view = app.read(|ctx| ActiveSession::as_ref(ctx).terminal_view_id(window));
                let detached = app
                    .views_of_type::<TerminalView>(window)
                    .unwrap()
                    .into_iter()
                    .find(|terminal| Some(terminal.id()) == view)
                    .is_some_and(|terminal| {
                        terminal.read(app, |terminal, ctx| {
                            let model = terminal.model.lock();
                            !terminal.has_pending_command_or_awaiting_completion(ctx)
                                && !model.has_guarded_task_attachment()
                                && (!finish
                                    || model.block_list().blocks().iter().any(|block| {
                                        block.is_done()
                                            && block.exit_code().value() == 0
                                            && block
                                                .output_to_string()
                                                .contains("WARPAI_GUI_FINISHED")
                                    }))
                        })
                    });
                warpui::async_assert!(detached)
            },
        )
    };
    let restore = {
        let original = original.clone();
        TestStep::new(&format!("Return to the {scope} source pane")).with_action(
            move |app, window, _| {
                let original = original.lock().unwrap();
                let (pane, _, _, _) = original.as_ref().unwrap();
                let root = app.root_view::<RootView>(window).unwrap();
                let workspace = root.read(app, |view, _| view.workspace_view().unwrap().clone());
                let group = workspace.read(app, |view, _| view.active_tab_pane_group().clone());
                group.update(app, |group, ctx| {
                    let detached = group.focused_pane_id(ctx);
                    assert_ne!(detached, *pane);
                    group.close_pane_and_focus(detached, *pane, ctx);
                });
            },
        )
    };
    let mut steps = vec![
        seed,
        open(),
        resume(),
        release(false),
        restore,
        open(),
        resume(),
        release(true),
    ];

    let cleanup = TestStep::new(&format!(
        "Close the {scope} split and restore the source pane"
    ))
    .set_timeout(Duration::from_secs(30))
    .with_action({
        let original = original.clone();
        move |app, window, _| {
            let (pane, _, _, left_panel_open) = *original.lock().unwrap().as_ref().unwrap();
            let root = app.root_view::<RootView>(window).unwrap();
            let workspace = root.read(app, |view, _| view.workspace_view().unwrap().clone());
            workspace.update(app, |workspace, ctx| {
                if !left_panel_open && workspace.is_left_panel_open(ctx) {
                    workspace.handle_action(&WorkspaceAction::ToggleLeftPanel, ctx);
                }
            });
            let group = workspace.read(app, |view, _| view.active_tab_pane_group().clone());
            group.update(app, |group, ctx| {
                let focused = group.focused_pane_id(ctx);
                assert_ne!(focused, pane);
                group.close_pane_and_focus(focused, pane, ctx);
            });
        }
    })
    .add_named_assertion(
        "source terminal focus and visible panes are restored",
        move |app, window| {
            let source = original.lock().unwrap();
            let (pane, count, terminal, _) = source.as_ref().unwrap();
            let root = app.root_view::<RootView>(window).unwrap();
            let workspace = root.read(app, |view, _| view.workspace_view().unwrap().clone());
            let group = workspace.read(app, |view, _| view.active_tab_pane_group().clone());
            warpui::async_assert!(group.read(app, |group, ctx| {
                group.focused_pane_id(ctx) == *pane
                    && group.visible_pane_ids().len() == *count
                    && group
                        .terminal_view_from_pane_id(*pane, ctx)
                        .is_some_and(|view| view.id() == *terminal)
            }))
        },
    );

    steps.push(cleanup);
    steps
}
