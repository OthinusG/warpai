//! Resolve the confirmed SSH terminal for existing project tools.
use warp_agent_bus::{ssh_files::SshConnection, ssh_remote::SshProfile};
use warpui::{AppContext, SingletonEntity, WindowId};

pub(crate) fn session_connection(
    session: &crate::terminal::model::session::Session,
) -> Option<SshConnection> {
    #[cfg(windows)]
    if session.ssh_arguments().is_none() && !session.is_legacy_ssh_session() {
        if let Some(distribution) = session.wsl_distro_name() {
            return Some(SshConnection::Wsl {
                distribution: distribution.into(), user: session.user().into(),
            });
        }
    }
    Some(match session.ssh_control_socket() {
        // The authenticated master is sufficient even when the launch command was not retained.
        Some(socket) => SshConnection::Multiplexed {
            socket: socket.to_owned(),
            wsl: session.wsl_distro_name().map(str::to_owned),
        },
        None if session.wsl_distro_name().is_some() => return None,
        None => SshConnection::Native {
            arguments: session.ssh_arguments()?.to_vec(),
            session: format!("{:?}", session.id()),
        },
    })
}

#[cfg(test)]
mod ssh_tests {
    use super::*;
    use crate::terminal::model::session::{
        command_executor::NoOpCommandExecutor, Session, SessionInfo,
    };

    #[test]
    fn terminal_ssh_session_master_does_not_require_original_ssh_arguments() {
        let socket = std::path::PathBuf::from("/tmp/warpai-confirmed-ssh");
        let session = Session::new(
            SessionInfo::new_for_test().with_ssh_socket_path(socket.clone()),
            std::sync::Arc::new(NoOpCommandExecutor::new()),
        );
        assert!(session.ssh_arguments().is_none());
        assert_eq!(
            session_connection(&session),
            Some(SshConnection::Multiplexed { socket, wsl: None })
        );
    }

    #[test]
    fn terminal_ssh_session_without_master_or_transport_remains_unavailable() {
        let session = Session::new(
            SessionInfo::new_for_test(),
            std::sync::Arc::new(NoOpCommandExecutor::new()),
        );
        assert_eq!(session_connection(&session), None);
    }
}

fn companion_home<'a>(
    session: &'a crate::terminal::model::session::Session,
    connection: &SshConnection,
    app: &'a AppContext,
) -> Option<&'a str> {
    let home = session.home_dir();
    #[cfg(all(windows, feature = "local_tty", not(feature = "remote_tty")))]
    let home = if let SshConnection::Wsl { distribution, user } = &connection {
        // Installation and workspace tools must use the same confirmed account home.
        crate::agent_communication::AgentCommunication::as_ref(app).wsl.targets.iter()
            .find(|target| target.distribution == *distribution && target.user == *user)
            .map(|target| target.home.as_str())
    } else { home };
    let _ = (connection, app);
    home
}

pub(crate) fn selected_ssh(
    app: &AppContext,
    window: WindowId,
) -> Option<(SshProfile, SshConnection)> {
    let active = crate::workspace::ActiveSession::as_ref(app);
    if active.remote_pending(window) {
        return None;
    }
    let session = active.session(window)?;
    let connection = session_connection(&session)?;
    #[cfg(windows)]
    if let SshConnection::Wsl { distribution, user } = &connection {
        #[cfg(all(feature = "local_tty", not(feature = "remote_tty")))]
        {
        let settings = &crate::agent_communication::AgentCommunication::as_ref(app).wsl;
        if !settings.preferences.enabled || !settings.targets.iter().any(|target|
            target.distribution == *distribution && target.user == *user) { return None; }
        }
        if !crate::terminal::wsl::WslInfo::as_ref(app).distributions()
            .any(|entry| entry.name == *distribution && entry.supports_communication()) { return None; }
    }
    let home = companion_home(&session, &connection, app)?;
    let host_info = session.host_info();
    let (companion_path, remote_shell) = warp_agent_bus::installation::companion_path(
        home,
        if connection.is_wsl() { "Linux" }
        else { host_info.os_category.as_deref()? },
    )
    .ok()?;
    #[cfg(windows)]
    let companion_path = if connection.is_wsl() {
        warp_agent_bus::installation::wsl_companion_path(home).ok()?
    } else { companion_path };
    let profile = SshProfile {
        target: if connection.is_wsl() { "wsl".into() } else { session.hostname().into() },
        config_file: None,
        remote_root: active.current_directory(window)?.into(),
        companion_path,
        remote_shell,
    };
    profile.validate().ok()?;
    Some((profile, connection))
}

pub(crate) fn selection_key(profile: &SshProfile, connection: &SshConnection, app: &AppContext) -> String {
    let key = format!(
        "{}:{}:{}:{}",
        connection.scope_key(),
        profile.target,
        profile.remote_root,
        profile.companion_path
    );
    #[cfg(all(windows, feature = "local_tty", not(feature = "remote_tty")))]
    if connection.is_wsl() {
        return format!("{key}:{}", crate::agent_communication::AgentCommunication::as_ref(app).wsl.generation);
    }
    let _ = app;
    key
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use crate::terminal::model::session::{command_executor::NoOpCommandExecutor, Session, SessionInfo};

    #[cfg(all(feature = "local_tty", not(feature = "remote_tty")))]
    #[test]
    fn verified_wsl_setup_invalidates_cached_selection_without_changing_authority() {
        warpui::App::test((), |mut app| async move {
            crate::test_util::terminal::initialize_app_for_terminal_view(&mut app);
            let profile = SshProfile { target:"wsl".into(), config_file:None,
                remote_root:"/home/guest/project".into(), companion_path:"/home/guest/.config/warpai/wsl/bin/warpai-wsl-companion".into(),
                remote_shell:warp_agent_bus::ssh_remote::RemoteShell::Posix };
            let wsl = SshConnection::Wsl { distribution:"Ubuntu".into(), user:"guest".into() };
            let ssh = SshConnection::Native { arguments:vec!["owned".into()], session:"owned".into() };
            let session = Session::new(SessionInfo::new_for_test().with_user("guest".into()),
                std::sync::Arc::new(NoOpCommandExecutor::new()));
            crate::agent_communication::AgentCommunication::handle(&app).update(&mut app, |model, _| {
                model.wsl.targets = vec![crate::agent_communication::wsl_settings::Target {
                    distribution: "Ubuntu".into(), user: "guest".into(), home: "/home/confirmed".into(),
                }];
            });
            app.read(|ctx| {
                assert_eq!(companion_home(&session, &wsl, ctx), Some("/home/confirmed"));
                assert_eq!(companion_home(&session, &ssh, ctx), session.home_dir());
                let other = SshConnection::Wsl { distribution:"Ubuntu".into(), user:"other".into() };
                assert_eq!(companion_home(&session, &other, ctx), None);
            });
            let before = app.read(|ctx| selection_key(&profile, &wsl, ctx));
            let mut moved_profile = profile.clone();
            moved_profile.companion_path = "/home/confirmed/.config/warpai/wsl/bin/warpai-wsl-companion".into();
            assert_ne!(app.read(|ctx| selection_key(&moved_profile, &wsl, ctx)), before);
            let ssh_before = app.read(|ctx| selection_key(&profile, &ssh, ctx));
            crate::agent_communication::AgentCommunication::handle(&app).update(&mut app, |model, _| {
                model.wsl.generation += 1;
            });
            assert_ne!(app.read(|ctx| selection_key(&profile, &wsl, ctx)), before);
            assert_eq!(app.read(|ctx| selection_key(&profile, &ssh, ctx)), ssh_before);
        });
    }

    #[test]
    fn wsl_selection_keeps_guest_account_and_nested_ssh_distinct() {
        let mut info = SessionInfo::new_for_test().with_user("guest".into());
        info.wsl_name = Some("Ubuntu Test".into());
        let session = Session::new(info.clone(), std::sync::Arc::new(NoOpCommandExecutor::new()));
        assert_eq!(session_connection(&session), Some(SshConnection::Wsl {
            distribution: "Ubuntu Test".into(), user: "guest".into(),
        }));
        let nested = Session::new(info.with_ssh_socket_path("/tmp/ssh-owned".into()),
            std::sync::Arc::new(NoOpCommandExecutor::new()));
        assert_eq!(session_connection(&nested), Some(SshConnection::Multiplexed {
            socket: "/tmp/ssh-owned".into(), wsl: Some("Ubuntu Test".into()),
        }), "A confirmed nested SSH master must retain its WSL transport");
    }
}
