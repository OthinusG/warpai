use warp_core::features::FeatureFlag;
use warpui::{App, EntityId, ModelHandle};

use crate::ai::active_agent_views_model::ActiveAgentViewsModel;
use crate::ai::agent::conversation::AIConversationId;
use crate::ai::artifacts::Artifact;
use crate::ai::blocklist::BlocklistAIHistoryEvent;
use crate::terminal::cli_agent_sessions::CLIAgentSessionsModel;
use crate::BlocklistAIHistoryModel;

use super::AgentNotificationsModel;

#[test]
fn native_cli_attention_uses_existing_mailbox_and_idle_is_not_completion() {
    use crate::terminal::{CLIAgent, cli_agent_sessions::CLIAgentSessionsModelEvent};
    use crate::ai::agent_management::notifications::{NotificationCategory, NotificationFilter};
    use warp_agent_bus::readiness::Activity;
    use warpui::SingletonEntity;
    use settings::Setting as _;
    App::test((), |mut app| async move {
        let _guard = FeatureFlag::HOANotifications.override_enabled(true);
        crate::test_util::terminal::initialize_app_for_terminal_view(&mut app);
        crate::settings::AISettings::handle(&app).update(&mut app, |settings, ctx| {
            settings.show_agent_notifications.set_value(true, ctx).unwrap();
        });
        let notifications = AgentNotificationsModel::handle(&app);
        let terminal = EntityId::new();
        for agent in enum_iterator::all::<CLIAgent>() {
            notifications.update(&mut app, |model, ctx| {
                for activity in [Activity::WaitingApproval, Activity::WaitingApproval] {
                    model.handle_cli_agent_session_event(&CLIAgentSessionsModelEvent::NativeActivityObserved {
                        terminal_view_id:terminal, agent, activity,
                    }, ctx);
                }
                assert_eq!(model.notifications.filtered_count(NotificationFilter::All), 1);
                assert_eq!(model.notifications.items_filtered(NotificationFilter::All).next().unwrap().category, NotificationCategory::Request);
                model.handle_cli_agent_session_event(&CLIAgentSessionsModelEvent::NativeActivityObserved {
                    terminal_view_id:terminal, agent, activity:Activity::Idle,
                }, ctx);
                assert_eq!(model.notifications.filtered_count(NotificationFilter::All), 0);
            });
        }
    });
}

fn setup_app(
    app: &mut App,
) -> (
    ModelHandle<BlocklistAIHistoryModel>,
    ModelHandle<AgentNotificationsModel>,
) {
    let history = app.add_singleton_model(|_| BlocklistAIHistoryModel::new(vec![], &[]));
    app.add_singleton_model(|_| CLIAgentSessionsModel::new());
    app.add_singleton_model(|_| ActiveAgentViewsModel::new());
    let notifications = app.add_singleton_model(AgentNotificationsModel::new);
    (history, notifications)
}

fn make_pr_artifact(url: &str, branch: &str) -> Artifact {
    Artifact::PullRequest {
        url: url.to_string(),
        branch: branch.to_string(),
        repo: None,
        number: None,
    }
}

fn make_plan_artifact(doc_uid: &str, title: &str) -> Artifact {
    Artifact::Plan {
        document_uid: doc_uid.to_string(),
        notebook_uid: None,
        title: Some(title.to_string()),
    }
}

#[test]
fn artifact_event_accumulates_into_pending() {
    App::test((), |mut app| async move {
        let _guard = FeatureFlag::HOANotifications.override_enabled(true);
        let (history, notifications) = setup_app(&mut app);

        let conversation_id = AIConversationId::new();
        let terminal_view_id = EntityId::new();

        history.update(&mut app, |_: &mut BlocklistAIHistoryModel, ctx| {
            ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationArtifacts {
                terminal_view_id,
                conversation_id,
                artifact: make_pr_artifact("https://github.com/org/repo/pull/42", "feature-branch"),
            });
        });

        notifications.read(&app, |model, _| {
            let pending = model.pending_artifacts.get(&conversation_id).unwrap();
            assert_eq!(pending.len(), 1);
            assert!(matches!(&pending[0], Artifact::PullRequest { branch, .. } if branch == "feature-branch"));
        });
    });
}

#[test]
fn multiple_artifacts_accumulated_across_turns() {
    App::test((), |mut app| async move {
        let _guard = FeatureFlag::HOANotifications.override_enabled(true);
        let (history, notifications) = setup_app(&mut app);

        let conversation_id = AIConversationId::new();
        let terminal_view_id = EntityId::new();

        history.update(&mut app, |_: &mut BlocklistAIHistoryModel, ctx| {
            ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationArtifacts {
                terminal_view_id,
                conversation_id,
                artifact: make_plan_artifact("doc-1", "My Plan"),
            });
        });
        history.update(&mut app, |_: &mut BlocklistAIHistoryModel, ctx| {
            ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationArtifacts {
                terminal_view_id,
                conversation_id,
                artifact: make_pr_artifact("https://github.com/org/repo/pull/1", "main"),
            });
        });

        notifications.read(&app, |model, _| {
            let pending = model.pending_artifacts.get(&conversation_id).unwrap();
            assert_eq!(pending.len(), 2);
            assert!(matches!(&pending[0], Artifact::Plan { title: Some(t), .. } if t == "My Plan"));
            assert!(matches!(&pending[1], Artifact::PullRequest { .. }));
        });
    });
}

#[test]
fn flush_drains_pending_artifacts() {
    App::test((), |mut app| async move {
        let _guard = FeatureFlag::HOANotifications.override_enabled(true);
        let (history, notifications) = setup_app(&mut app);

        let conversation_id = AIConversationId::new();
        let terminal_view_id = EntityId::new();

        history.update(&mut app, |_: &mut BlocklistAIHistoryModel, ctx| {
            ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationArtifacts {
                terminal_view_id,
                conversation_id,
                artifact: make_pr_artifact("https://github.com/org/repo/pull/1", "branch-1"),
            });
        });

        notifications.update(&mut app, |model, _| {
            let artifacts = model.flush_pending_artifacts(conversation_id);
            assert_eq!(artifacts.len(), 1);
            assert!(matches!(&artifacts[0], Artifact::PullRequest { branch, .. } if branch == "branch-1"));
        });

        notifications.read(&app, |model, _| {
            assert!(!model.pending_artifacts.contains_key(&conversation_id));
        });
    });
}

#[test]
fn flush_returns_empty_vec_when_no_artifacts() {
    App::test((), |mut app| async move {
        let _guard = FeatureFlag::HOANotifications.override_enabled(true);
        let (_history, notifications) = setup_app(&mut app);

        let conversation_id = AIConversationId::new();

        notifications.update(&mut app, |model, _| {
            let artifacts = model.flush_pending_artifacts(conversation_id);
            assert!(artifacts.is_empty());
        });
    });
}

#[test]
fn deletion_cleans_up_pending_artifacts() {
    App::test((), |mut app| async move {
        let _guard = FeatureFlag::HOANotifications.override_enabled(true);
        let (history, notifications) = setup_app(&mut app);

        let conversation_id = AIConversationId::new();
        let terminal_view_id = EntityId::new();

        history.update(&mut app, |_: &mut BlocklistAIHistoryModel, ctx| {
            ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationArtifacts {
                terminal_view_id,
                conversation_id,
                artifact: make_pr_artifact("https://github.com/org/repo/pull/1", "branch-1"),
            });
        });

        history.update(&mut app, |_: &mut BlocklistAIHistoryModel, ctx| {
            ctx.emit(BlocklistAIHistoryEvent::DeletedConversation {
                terminal_view_id,
                conversation_id,
                conversation_title: None,
            });
        });

        notifications.read(&app, |model, _| {
            assert!(!model.pending_artifacts.contains_key(&conversation_id));
        });
    });
}

#[test]
fn separate_conversations_have_independent_pending_artifacts() {
    App::test((), |mut app| async move {
        let _guard = FeatureFlag::HOANotifications.override_enabled(true);
        let (history, notifications) = setup_app(&mut app);

        let conv_a = AIConversationId::new();
        let conv_b = AIConversationId::new();
        let terminal_view_id = EntityId::new();

        history.update(&mut app, |_: &mut BlocklistAIHistoryModel, ctx| {
            ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationArtifacts {
                terminal_view_id,
                conversation_id: conv_a,
                artifact: make_pr_artifact("https://github.com/org/repo/pull/1", "branch-a"),
            });
        });
        history.update(&mut app, |_: &mut BlocklistAIHistoryModel, ctx| {
            ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationArtifacts {
                terminal_view_id,
                conversation_id: conv_b,
                artifact: make_plan_artifact("doc-b", "Plan B"),
            });
        });

        notifications.update(&mut app, |model, _| {
            let a = model.flush_pending_artifacts(conv_a);
            assert_eq!(a.len(), 1);
            assert!(matches!(&a[0], Artifact::PullRequest { branch, .. } if branch == "branch-a"));

            let b = model.flush_pending_artifacts(conv_b);
            assert_eq!(b.len(), 1);
            assert!(matches!(&b[0], Artifact::Plan { title: Some(t), .. } if t == "Plan B"));
        });
    });
}
