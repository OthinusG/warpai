//! Lazy fixed-version reads reuse the panel's native controls and connection fences.
use super::*;
use warp_agent_bus::{
    context::{ContextContent, ContextRef, ContextResource},
    handoff::{HandoffPart, HandoffSection, HandoffSnapshot},
    Operation,
};

pub(super) struct HandoffView {
    pub task: String,
    pub reference: ContextRef,
    pub generation: u64,
    pub result: Option<Result<HandoffRead, &'static str>>,
    pub evidence: Option<(
        String,
        Option<Result<warp_agent_bus::handoff::HandoffEvidence, &'static str>>,
    )>,
}

pub(super) struct HandoffRead {
    snapshot: HandoffSnapshot,
    status: serde_json::Value,
    result: serde_json::Value,
    source_task: serde_json::Value,
    source_workspace: Option<String>,
}

pub(super) struct HandoffHistory {
    task: String,
    references: Vec<ContextRef>,
    next_cursor: Option<u64>,
    loading: bool,
    error: Option<&'static str>,
}

impl HandoffHistory {
    #[cfg(debug_assertions)]
    pub(super) fn ready_with_versions(&self) -> bool {
        !self.loading && self.error.is_none() && !self.references.is_empty()
    }
}

impl CollaborationPanel {
    pub(super) fn load_handoff_history(
        &mut self,
        cursor: Option<u64>,
        ctx: &mut ViewContext<Self>,
    ) {
        if !self.connected
            || self.current_context(ctx) != self.context
            || self
                .handoff_history
                .as_ref()
                .is_some_and(|history| history.loading)
        {
            return;
        }
        let Some(task) = self.query.selected_task.clone() else {
            return;
        };
        let Some(snapshot) = self
            .snapshot
            .as_ref()
            .filter(|snapshot| snapshot.handoff_briefs_v1)
        else {
            return;
        };
        let project = snapshot.project.clone();
        let remote = self.remote.is_some();
        let client = self.remote_client.clone();
        let broker = super::super::BROKER.get().cloned();
        let generation = self.generation;
        let context = self.context.clone();
        self.handoff_history = Some(HandoffHistory {
            task: task.clone(),
            references: vec![],
            next_cursor: None,
            loading: true,
            error: None,
        });
        let read_task = task.clone();
        ctx.notify();
        ctx.spawn(
            async move {
                let operation = Operation::TaskHistory {
                    task_id: read_task,
                    handoff_only: true,
                    cursor,
                    limit: Some(50),
                };
                let value = if remote {
                    let mut guard = client.lock().await;
                    let client = guard
                        .as_mut()
                        .ok_or_else(|| anyhow::anyhow!("Remote connection unavailable"))?;
                    read_remote_handoff_operation(client, &project, operation, generation).await?
                } else {
                    broker
                        .ok_or_else(|| anyhow::anyhow!("Broker unavailable"))?
                        .operator(&project, &operation)?
                };
                let events: Vec<Event> = serde_json::from_value(value["events"].clone())?;
                let references = events
                    .into_iter()
                    .map(|event| serde_json::from_value(event.payload["reference"].clone()))
                    .collect::<Result<Vec<ContextRef>, _>>()?;
                Ok::<_, anyhow::Error>((references, value["cursor"].as_u64()))
            },
            move |panel, result, ctx| {
                if panel.generation != generation
                    || panel.context != context
                    || panel.current_context(ctx) != context
                    || panel.query.selected_task.as_ref() != Some(&task)
                    || !panel.communication_enabled(ctx)
                {
                    return;
                }
                if let Some(history) = panel
                    .handoff_history
                    .as_mut()
                    .filter(|history| history.task == task)
                {
                    history.loading = false;
                    match result {
                        Ok((references, cursor)) => {
                            history.references = references;
                            history.next_cursor = cursor;
                        }
                        Err(error) => history.error = Some(handoff_error(&error)),
                    }
                }
                ctx.notify();
            },
        );
    }

    pub(super) fn load_handoff_evidence(&mut self, id: &str, ctx: &mut ViewContext<Self>) {
        if !self.connected || self.current_context(ctx) != self.context {
            return;
        }
        let Some(view) = self.handoff.as_mut() else {
            return;
        };
        if self.query.selected_task.as_ref() != Some(&view.task)
            || !view.result.as_ref().is_some_and(|result| {
                result
                    .as_ref()
                    .is_ok_and(|read| read.snapshot.evidence_ids.iter().any(|value| value == id))
            })
        {
            return;
        }
        let task = view.task.clone();
        let reference = view.reference.clone();
        let read_reference = reference.clone();
        let id = id.to_owned();
        let read_id = id.clone();
        view.evidence = Some((id.clone(), None));
        view.generation = self.generation;
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let project = snapshot.project.clone();
        let client = self.remote_client.clone();
        let remote = self.remote.is_some();
        let broker = super::super::BROKER.get().cloned();
        let generation = self.generation;
        let context = self.context.clone();
        ctx.notify();
        ctx.spawn(
            async move {
                let part = HandoffPart::Evidence { id: read_id };
                let value = if remote {
                    let mut guard = client.lock().await;
                    let client = guard
                        .as_mut()
                        .ok_or_else(|| anyhow::anyhow!("Remote connection unavailable"))?;
                    read_remote_handoff(client, &project, &read_reference, Some(part), generation)
                        .await?
                } else {
                    broker
                        .ok_or_else(|| anyhow::anyhow!("Broker unavailable"))?
                        .operator(
                            &project,
                            &Operation::ContextGet {
                                reference: read_reference,
                                handoff_part: Some(part),
                            },
                        )?
                };
                Ok::<_, anyhow::Error>(serde_json::from_value::<
                    warp_agent_bus::handoff::HandoffEvidence,
                >(value["source"].clone())?)
            },
            move |panel, result, ctx| {
                if panel.generation != generation
                    || panel.context != context
                    || panel.current_context(ctx) != context
                    || panel.query.selected_task.as_ref() != Some(&task)
                    || !panel.communication_enabled(ctx)
                {
                    return;
                }
                if let Some(view) = panel
                    .handoff
                    .as_mut()
                    .filter(|view| view.reference == reference && view.task == task)
                {
                    if let Some((selected, value)) = &mut view.evidence {
                        if *selected == id {
                            *value = Some(result.map_err(|error| handoff_error(&error)));
                        }
                    }
                }
                ctx.notify();
            },
        );
    }

    fn handoff_references(&self) -> Vec<ContextRef> {
        let Some(snapshot) = &self.snapshot else {
            return vec![];
        };
        if !snapshot.handoff_briefs_v1 {
            return vec![];
        }
        let Some(task) = snapshot
            .task
            .as_ref()
            .filter(|task| self.query.selected_task.as_ref() == Some(&task.id))
        else {
            return vec![];
        };
        let mut references: Vec<_> = task.handoff_ref.iter().cloned().collect();
        for input in &task.handoff_inputs {
            if !references.contains(&input.reference) {
                references.push(input.reference.clone());
            }
        }
        if let Some(history) = self
            .handoff_history
            .as_ref()
            .filter(|history| history.task == task.id)
        {
            for reference in &history.references {
                if !references.contains(reference) {
                    references.push(reference.clone());
                }
            }
        }
        for event in &self.events {
            if event.resource.as_deref() == Some(&task.id) && event.kind == "handoff_created" {
                if let Ok(reference) = serde_json::from_value(event.payload["reference"].clone()) {
                    if !references.contains(&reference) {
                        references.push(reference);
                    }
                }
            }
        }
        references
    }

    pub(super) fn load_handoff(&mut self, reference: &ContextRef, ctx: &mut ViewContext<Self>) {
        if !self.connected
            || self.current_context(ctx) != self.context
            || !self.handoff_references().contains(reference)
        {
            return;
        }
        let Some(task) = self.query.selected_task.clone() else {
            return;
        };
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let project = snapshot.project.clone();
        let client = self.remote_client.clone();
        let remote = self.remote.is_some();
        let broker = super::super::BROKER.get().cloned();
        let generation = self.generation;
        let context = self.context.clone();
        let reference = reference.clone();
        self.handoff = Some(HandoffView {
            task: task.clone(),
            reference: reference.clone(),
            generation,
            result: None,
            evidence: None,
        });
        ctx.notify();
        let read_reference = reference.clone();
        ctx.spawn(
            async move {
                let (resource, status, result, source_task) = if remote {
                    let mut guard = client.lock().await;
                    let client = guard
                        .as_mut()
                        .ok_or_else(|| anyhow::anyhow!("Remote connection unavailable"))?;
                    let resource =
                        read_remote_handoff(client, &project, &read_reference, None, generation)
                            .await?;
                    let status = read_remote_handoff(
                        client,
                        &project,
                        &read_reference,
                        Some(HandoffPart::Status),
                        generation,
                    )
                    .await?;
                    let result = read_remote_handoff(
                        client,
                        &project,
                        &read_reference,
                        Some(HandoffPart::Result),
                        generation,
                    )
                    .await?;
                    let source_task = read_remote_handoff(
                        client,
                        &project,
                        &read_reference,
                        Some(HandoffPart::Task),
                        generation,
                    )
                    .await?;
                    (resource, status, result, source_task)
                } else {
                    let broker = broker.ok_or_else(|| anyhow::anyhow!("Broker unavailable"))?;
                    let resource = broker.operator(
                        &project,
                        &Operation::ContextGet {
                            reference: read_reference.clone(),
                            handoff_part: None,
                        },
                    )?;
                    let status = broker.operator(
                        &project,
                        &Operation::ContextGet {
                            reference: read_reference.clone(),
                            handoff_part: Some(HandoffPart::Status),
                        },
                    )?;
                    let result = broker.operator(
                        &project,
                        &Operation::ContextGet {
                            reference: read_reference.clone(),
                            handoff_part: Some(HandoffPart::Result),
                        },
                    )?;
                    let source_task = broker.operator(
                        &project,
                        &Operation::ContextGet {
                            reference: read_reference.clone(),
                            handoff_part: Some(HandoffPart::Task),
                        },
                    )?;
                    (resource, status, result, source_task)
                };
                let resource: ContextResource = serde_json::from_value(resource)?;
                let source_workspace = resource.summary.source_workspace;
                let ContextContent::Handoff { snapshot } = resource.content else {
                    anyhow::bail!("Expected a handoff resource");
                };
                Ok::<_, anyhow::Error>(HandoffRead {
                    snapshot: *snapshot,
                    status: status["source"].clone(),
                    result: result["source"].clone(),
                    source_task: source_task["source"].clone(),
                    source_workspace,
                })
            },
            move |panel, result, ctx| {
                if panel.generation != generation
                    || panel.context != context
                    || panel.current_context(ctx) != context
                    || panel.query.selected_task.as_ref() != Some(&task)
                    || !panel.communication_enabled(ctx)
                {
                    return;
                }
                let Some(view) = panel
                    .handoff
                    .as_mut()
                    .filter(|view| view.task == task && view.reference == reference)
                else {
                    return;
                };
                view.result = Some(result.map_err(|error| handoff_error(&error)));
                ctx.notify();
            },
        );
    }

    pub(super) fn handoff_sections(&self, task: &Task) -> Vec<Section> {
        let mut sections = vec![Section {
            title: "Task handoff".into(),
            rows: vec![warpui::localization::text(
                "Handoffs are Agent context, never execution authorization.",
            )
            .into()],
        }];
        if self.query.selected_task.as_ref() != Some(&task.id) {
            sections[0]
                .rows
                .push(warpui::localization::text("Loading handoff…").into());
            return sections;
        }
        if !self
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.handoff_briefs_v1)
        {
            sections[0]
                .rows
                .push(warpui::localization::text("Handoff unavailable: update Companion.").into());
            return sections;
        }
        if task
            .handoff_inputs
            .iter()
            .any(|input| input.requires_revalidation())
        {
            sections[0].rows.push(warpui::localization::text("An input handoff is stale or unavailable. Revalidate its source before continuing.").into());
        }
        if task.handoff_ref.is_none() {
            sections[0]
                .rows
                .push(warpui::localization::text("No current handoff snapshot.").into());
        }
        if let Some(history) = self
            .handoff_history
            .as_ref()
            .filter(|history| history.task == task.id)
        {
            let status = if history.loading {
                Some("Loading handoff…")
            } else if let Some(error) = history.error {
                Some(error)
            } else if history.references.is_empty() {
                Some("No handoff versions on this page.")
            } else {
                None
            };
            if let Some(status) = status {
                sections[0]
                    .rows
                    .push(warpui::localization::text(status).into());
            }
        }
        let Some(view) = self.handoff.as_ref().filter(|view| view.task == task.id) else {
            return sections;
        };
        let read = match &view.result {
            None => {
                sections[0].rows.push(
                    warpui::localization::text(if view.generation == self.generation {
                        "Loading handoff…"
                    } else {
                        "Could not load handoff. Retry the read."
                    })
                    .into(),
                );
                return sections;
            }
            Some(Err(error)) => {
                sections[0]
                    .rows
                    .push(warpui::localization::text(error).into());
                return sections;
            }
            Some(Ok(read)) => read,
        };
        let snapshot = &read.snapshot;
        let source_is_selected = snapshot.task_id == task.id;
        let pinned = task
            .handoff_inputs
            .iter()
            .find(|input| input.reference == view.reference);
        let source_state = if source_is_selected {
            task.state.as_str()
        } else if let Some(input) = pinned {
            input
                .status
                .as_ref()
                .map_or("unavailable", |status| status.state.as_str())
        } else {
            read.status["state"].as_str().unwrap_or("unknown")
        };
        let is_current = if source_is_selected {
            task.handoff_ref.as_ref() == Some(&view.reference)
        } else if let Some(input) = pinned {
            !input.requires_revalidation()
        } else {
            read.status["result_is_current"] == true
        };
        sections[0].rows.extend([
            warpui::localization::format_text(
                "Task {task} · revision {revision} · attempt {attempt} · resource v{version}",
                &[
                    ("task", &snapshot.task_id),
                    ("revision", &snapshot.task_revision.to_string()),
                    ("attempt", &snapshot.attempt_id),
                    ("version", &view.reference.revision.to_string()),
                ],
            ),
            warpui::localization::format_text(
                "Producer: {producer} · snapshot: {reason} · source state at read: {state}",
                &[
                    (
                        "producer",
                        &self
                            .snapshot
                            .as_ref()
                            .map(|panel| panel.participant_label(&snapshot.producer))
                            .unwrap_or_default(),
                    ),
                    ("reason", warpui::localization::text(&snapshot.reason)),
                    ("state", warpui::localization::text(source_state)),
                ],
            ),
            warpui::localization::text("Agent-reported test outcome; review is separate.").into(),
        ]);
        if !is_current {
            sections[0].rows.push(
                warpui::localization::text(
                    "Historical handoff; refresh the source task to check its current review.",
                )
                .into(),
            );
        }
        let uncertain = if source_is_selected {
            task.attempts
                .iter()
                .any(|attempt| attempt.finished_at.is_none() && attempt.certainty != "active")
        } else {
            read.status["execution_uncertain"] == true
        };
        if uncertain {
            sections[0].rows.push(
                warpui::localization::text(
                    "Execution remains uncertain; a handoff does not confirm a stop.",
                )
                .into(),
            );
        }
        sections[0].rows.push(
            snapshot
                .binding
                .as_ref()
                .map(|binding| binding.workspace.clone())
                .or_else(|| read.source_workspace.clone())
                .unwrap_or_else(|| {
                    warpui::localization::text("Source workspace not provided.").into()
                }),
        );
        if !source_is_selected {
            sections.push(Section {
                title: "Source task".into(),
                rows: ["objective", "acceptance"]
                    .into_iter()
                    .filter_map(|key| read.source_task[key].as_str().map(str::to_owned))
                    .collect(),
            });
        }
        sections.push(Section {
            title: "Result".into(),
            rows: ["result", "evidence", "reason", "note"]
                .into_iter()
                .filter_map(|key| read.result[key].as_str().map(str::to_owned))
                .collect(),
        });
        sections.push(Section {
            title: "Decisions".into(),
            rows: section_rows(&snapshot.agent_report.decisions, |item| {
                format!("{}\n{}", item.decision, item.rationale)
            }),
        });
        sections.push(Section {
            title: "Unresolved issues".into(),
            rows: section_rows(&snapshot.agent_report.unresolved_issues, |item| {
                let missing = warpui::localization::text("Not provided");
                [
                    item.issue.clone(),
                    warpui::localization::format_text(
                        "Impact: {value}",
                        &[("value", item.impact.as_deref().unwrap_or(missing))],
                    ),
                    warpui::localization::format_text(
                        "Issue state: {value}",
                        &[("value", item.state.as_deref().unwrap_or(missing))],
                    ),
                    warpui::localization::format_text(
                        "Decision needed: {value}",
                        &[(
                            "value",
                            item.required_decision.as_deref().unwrap_or(missing),
                        )],
                    ),
                ]
                .join("\n")
            }),
        });
        sections.push(Section {
            title: "Next steps".into(),
            rows: section_rows(&snapshot.agent_report.next_steps, Clone::clone),
        });
        sections.push(Section {
            title: "Evidence".into(),
            rows: if snapshot.evidence_ids.is_empty() {
                vec![warpui::localization::text("No registered evidence in this snapshot.").into()]
            } else {
                snapshot.evidence_ids.clone()
            },
        });
        if let Some((id, result)) = &view.evidence {
            let mut rows = vec![id.clone()];
            match result {
                None => rows.push(
                    warpui::localization::text(if view.generation == self.generation {
                        "Loading handoff…"
                    } else {
                        "Could not load handoff. Retry the read."
                    })
                    .into(),
                ),
                Some(Err(error)) => rows.push(warpui::localization::text(error).into()),
                Some(Ok(source)) => {
                    let evidence = &source.evidence;
                    let reporter = source
                        .reported_by
                        .as_deref()
                        .map(|id| {
                            self.snapshot
                                .as_ref()
                                .map(|snapshot| snapshot.participant_label(id))
                                .unwrap_or_else(|| id.into())
                        })
                        .unwrap_or_else(|| warpui::localization::text("Not provided").into());
                    rows.push(warpui::localization::format_text(
                        "Evidence reported by {agent}",
                        &[("agent", &reporter)],
                    ));
                    if !source.same_attempt {
                        rows.push(
                            warpui::localization::text(
                                "Evidence belongs to a different execution attempt.",
                            )
                            .into(),
                        );
                    }
                    if let Some(binding) = &source.binding {
                        rows.push(binding.workspace.clone());
                    }

                    rows.push(
                        warpui::localization::text(
                            "Agent-reported test outcome; review is separate.",
                        )
                        .into(),
                    );
                    for (label, value) in [
                        ("Path", evidence.path.as_ref()),
                        ("Command", evidence.command.as_ref()),
                        ("Outcome", evidence.outcome.as_ref()),
                        ("Commit", evidence.commit.as_ref()),
                        ("Summary", evidence.summary.as_ref()),
                        ("Attempt", evidence.attempt_id.as_ref()),
                    ] {
                        if let Some(value) = value {
                            rows.push(format!("{}: {value}", warpui::localization::text(label)));
                        }
                    }
                    if let Some(revision) = source.task_revision {
                        rows.push(warpui::localization::format_text(
                            "Evidence task revision: {revision}",
                            &[("revision", &revision.to_string())],
                        ));
                    }
                    if let Some(code) = evidence.exit_code {
                        rows.push(warpui::localization::format_text(
                            "Exit code: {value}",
                            &[("value", &code.to_string())],
                        ));
                    }
                    if evidence.kind == "test" {
                        rows.push(
                            warpui::localization::text(
                                "No structured environment details were provided.",
                            )
                            .into(),
                        );
                    }
                    if evidence.verified {
                        rows.push(
                            warpui::localization::text(
                                "A content verification was recorded; tests remain Agent-reported.",
                            )
                            .into(),
                        );
                    }
                }
            }
            sections.push(Section {
                title: "Handoff evidence".into(),
                rows,
            });
        }
        sections
    }

    pub(super) fn render_handoff_controls(&self, app: &AppContext) -> Box<dyn Element> {
        let builder = Appearance::as_ref(app).ui_builder();
        let mut column = Flex::column().with_spacing(GAP_TIGHT);
        if !self
            .snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.handoff_briefs_v1)
        {
            return column.finish();
        }
        for (key, label, cursor) in [
            ("history:first", "Handoff history", None),
            (
                "history:next",
                "Next handoff page",
                self.handoff_history
                    .as_ref()
                    .and_then(|history| history.next_cursor),
            ),
        ] {
            if key == "history:next" && cursor.is_none() {
                continue;
            }
            let state = self
                .handoff_buttons
                .borrow_mut()
                .entry(key.into())
                .or_default()
                .clone();
            column.add_child(
                builder
                    .button(ButtonVariant::Secondary, state)
                    .with_text_label(warpui::localization::text(label).into())
                    .build()
                    .on_click(move |ctx, _, _| {
                        ctx.dispatch_typed_action(Action::LoadHandoffHistory(cursor))
                    })
                    .finish(),
            );
        }
        for reference in self.handoff_references() {
            let state = self
                .handoff_buttons
                .borrow_mut()
                .entry(reference.uri())
                .or_default()
                .clone();
            let label = warpui::localization::format_text(
                "Handoff {id} · v{version}",
                &[
                    ("id", &reference.id.chars().take(8).collect::<String>()),
                    ("version", &reference.revision.to_string()),
                ],
            );
            column.add_child(
                builder
                    .button(ButtonVariant::Secondary, state)
                    .with_text_label(label.into())
                    .build()
                    .on_click(move |ctx, _, _| {
                        ctx.dispatch_typed_action(Action::LoadHandoff(reference.clone()))
                    })
                    .finish(),
            );
        }
        if let Some(view) = self
            .handoff
            .as_ref()
            .filter(|view| self.query.selected_task.as_ref() == Some(&view.task))
        {
            if let Some(Ok(read)) = &view.result {
                for id in &read.snapshot.evidence_ids {
                    let state = self
                        .handoff_buttons
                        .borrow_mut()
                        .entry(format!("{}:{id}", view.reference.uri()))
                        .or_default()
                        .clone();
                    let label = warpui::localization::format_text(
                        "Read evidence {id}",
                        &[("id", &id.chars().take(8).collect::<String>())],
                    );
                    let id = id.clone();
                    column.add_child(
                        builder
                            .button(ButtonVariant::Secondary, state)
                            .with_text_label(label.into())
                            .build()
                            .on_click(move |ctx, _, _| {
                                ctx.dispatch_typed_action(Action::LoadHandoffEvidence(id.clone()))
                            })
                            .finish(),
                    );
                }
            }
        }
        column.finish()
    }
}

fn section_rows<T>(section: &HandoffSection<T>, display: impl Fn(&T) -> String) -> Vec<String> {
    match section {
        HandoffSection::Provided(items) if !items.is_empty() => items.iter().map(display).collect(),
        HandoffSection::Provided(_) => {
            vec![warpui::localization::text("Provided: no entries").into()]
        }
        HandoffSection::NotProvided => vec![warpui::localization::text("Not provided").into()],
        HandoffSection::NotApplicable => vec![warpui::localization::text("Not applicable").into()],
    }
}

async fn read_remote_handoff(
    client: &mut HostClient,
    project: &str,
    reference: &ContextRef,
    part: Option<HandoffPart>,
    generation: u64,
) -> anyhow::Result<serde_json::Value> {
    read_remote_handoff_operation(
        client,
        project,
        Operation::ContextGet {
            reference: reference.clone(),
            handoff_part: part,
        },
        generation,
    )
    .await
}

async fn read_remote_handoff_operation(
    client: &mut HostClient,
    project: &str,
    operation: Operation,
    generation: u64,
) -> anyhow::Result<serde_json::Value> {
    anyhow::ensure!(
        client
            .capabilities()
            .iter()
            .any(|feature| feature == "handoff_briefs_v1"),
        warp_agent_bus::DomainError {
            code: "feature_unavailable".into(),
            message: "Handoff unavailable: update Companion.".into(),
            retryable: false,
            version: None
        }
    );
    let command = TaskCommand::Scoped {
        scope: project.into(),
        command: Box::new(TaskCommand::Operator(operation)),
    };
    CollaborationPanel::remote_command(client, &command, generation).await
}

fn handoff_error(error: &anyhow::Error) -> &'static str {
    match error
        .downcast_ref::<warp_agent_bus::DomainError>()
        .map(|error| error.code.as_str())
    {
        Some("scope_denied" | "unauthorized") => "Handoff unavailable: access denied.",
        Some("feature_unavailable") => "Handoff unavailable: update Companion.",
        Some("not_found") => "Saved handoff is unavailable.",
        _ => "Could not load handoff. Retry the read.",
    }
}
