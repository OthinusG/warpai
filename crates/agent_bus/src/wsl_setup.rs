//! Guest-owned MCP installation through existing native vendor adapters.
use crate::setup::{self, Available, Preferences};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRequest {
    pub shell_pid: u32,
    pub action: NativeAction,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum NativeAction {
    Observe {
        draft: bool,
        blocked: bool,
        input_epoch: u64,
        submitted: bool,
        cancelled: bool,
        native_ready: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        screen_run: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        screen_activity: Option<crate::readiness::Activity>,
    },
    Claim {
        wake: crate::transport::Wake,
    },
    Validate {
        wake: crate::transport::Wake,
    },
    Finish {
        wake: crate::transport::Wake,
        submitted: bool,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeState {
    pub run: String,
    pub program: String,
    pub wake: Option<crate::transport::Wake>,
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<crate::readiness::Activity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_work: Option<bool>,
}

#[derive(Serialize, Deserialize)]
#[serde(
    tag = "action",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Action {
    Enable(bool),
    Select(String),
    Rescan,
    RemoveAll,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub action: Action,
    pub commands: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub account: String,
    pub preferences: Preferences,
    pub available: Vec<Available>,
    pub status: String,
}

/// Resolve Windows-backed paths even with custom automount roots and symlinks.
#[cfg(all(feature = "wsl_companion", target_os = "linux"))]
pub(crate) fn native_executable(path: &std::path::Path) -> bool {
    let Ok(path) = path.canonicalize() else { return false; };
    // Non-WSL Linux fixtures have no imported Windows drives.
    if !std::path::Path::new("/usr/bin/wslpath").is_file() { return true; }
    let Ok(output) = std::process::Command::new("/usr/bin/wslpath")
        .arg("-w").arg(path).output() else { return false; };
    output.status.success() && output.stdout.len() <= 16384
        && native_windows_translation(&output.stdout)
}

#[cfg(any(test, all(feature = "wsl_companion", target_os = "linux")))]
fn native_windows_translation(bytes: &[u8]) -> bool {
    let Ok(value) = std::str::from_utf8(bytes) else { return false; };
    let value = value.trim_end_matches(['\r', '\n']);
    value.starts_with(r"\\wsl.localhost\") || value.starts_with(r"\\wsl$\")
}

#[cfg(all(feature = "wsl_companion", target_os = "linux"))]
pub fn uninstall() -> Result<()> {
    uninstall_inner(false)
}

#[cfg(all(feature = "wsl_companion", target_os = "linux"))]
pub fn recover_uninstall() -> Result<()> {
    println!("warpai-install:cleanup");
    run_setup(true)?;
    uninstall_inner(true)
}

#[cfg(all(feature = "wsl_companion", target_os = "linux"))]
fn uninstall_inner(recovery: bool) -> Result<()> {
    use sha2::Digest;
    let executable = std::env::current_exe()?;
    let expected = std::path::PathBuf::from(crate::installation::wsl_companion_path(
        dirs::home_dir()
            .and_then(|home| home.to_str().map(str::to_owned))
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Linux home unavailable"))?,
    )?);
    ensure!(
        recovery || executable == expected,
        "Unexpected WSL installation"
    );
    let directory = expected
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Missing installation"))?;
    for path in directory.ancestors().take(4) {
        ensure!(
            !path
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.file_type().is_symlink()),
            "Installation directory cannot be a link"
        );
    }
    ensure!(
        !expected
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink()),
        "Companion cannot be a link"
    );
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_secs(if recovery { 65 } else { 0 });
    if recovery {
        println!("warpai-install:stop");
    }
    loop {
        let mut active = false;
        for (count, entry) in std::fs::read_dir("/proc")?.enumerate() {
            ensure!(count < 16384, "Process limit exceeded");
            let entry = entry?;
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<u32>().ok())
            else {
                continue;
            };
            if pid == std::process::id() {
                continue;
            }
            if let Ok(path) = std::fs::read_link(entry.path().join("exe")) {
                active |= path
                    .to_str()
                    .map(|path| path.trim_end_matches(" (deleted)"))
                    == expected.to_str();
            }
        }
        if !active {
            break;
        }
        ensure!(
            std::time::Instant::now() < deadline,
            "WSL Companion is active"
        );
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if recovery && error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    for (count, entry) in entries.enumerate() {
        ensure!(count < 256, "Installation entry limit exceeded");
        let entry = entry?;
        let name = entry.file_name();
        let Some(hash) = name
            .to_str()
            .and_then(|name| name.strip_prefix("companion-runtime-"))
        else {
            continue;
        };
        if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            continue;
        }
        ensure!(entry.file_type()?.is_dir(), "Runtime must be a directory");
        if !recovery {
            let sums_path = entry.path().join("SHA256SUMS");
            ensure!(
                !sums_path.symlink_metadata()?.file_type().is_symlink(),
                "Runtime metadata cannot be a link"
            );
            let mut sums = Vec::new();
            std::fs::File::open(sums_path)?
                .take(512 * 1024 + 1)
                .read_to_end(&mut sums)?;
            ensure!(
                sums.len() <= 512 * 1024 && format!("{:x}", sha2::Sha256::digest(&sums)) == hash,
                "Unrecognized runtime directory"
            );
        }
        std::fs::remove_dir_all(entry.path())?;
    }
    let manifest = directory.join("companion-manifest.json");
    match manifest.symlink_metadata() {
        Ok(metadata) => {
            ensure!(
                metadata.is_file() && !metadata.file_type().is_symlink(),
                "Invalid manifest"
            );
            std::fs::remove_file(manifest)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => return Err(error.into()),
    }
    match std::fs::remove_file(expected) {
        Ok(()) => (),
        Err(error) if recovery && error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

#[cfg(all(feature = "wsl_companion", target_os = "linux"))]
pub fn run() -> Result<()> {
    run_setup(false)
}

#[cfg(all(feature = "wsl_companion", target_os = "linux"))]
fn run_setup(removing: bool) -> Result<()> {
    let mut bytes = Vec::new();
    std::io::stdin().take(65537).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 65536, "Setup request too large");
    let request: Request = serde_json::from_slice(&bytes)?;
    ensure!(!removing || matches!(request.action, Action::RemoveAll), "Recovery only removes WSL integration");
    ensure!(
        request.commands.len() <= 128
            && request.commands.iter().all(|(program, command)| {
                [program, command].into_iter().all(|value| {
                    !value.is_empty()
                        && value.len() <= 64
                        && value
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
                })
            }),
        "Invalid Agent command"
    );
    let path = crate::companion::guest_preferences_path()?;
    let _lock = lock_preferences(&path)?;
    let mut preferences: Preferences = match std::fs::File::open(&path) {
        Ok(file) => {
            let mut bytes = Vec::new();
            file.take(512 * 1024 + 1).read_to_end(&mut bytes)?;
            ensure!(bytes.len() <= 512 * 1024, "Guest preferences too large");
            serde_json::from_slice(&bytes)?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Default::default(),
        Err(error) => return Err(error.into()),
    };
    let bridge = std::env::current_exe()?;
    let mut available = setup::discover(
        request.commands,
        &bridge,
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect(),
    );
    let failed = apply_action(&request.action, &mut preferences, &mut available, &path)?;
    ensure!(!removing || !failed, "Guest MCP removal did not complete; settings were preserved");
    for row in &mut available {
        if row.program == "codex"
            && !preferences
                .selected
                .get(&row.command)
                .is_some_and(|entry| entry.active)
        {
            row.status = "Available for guest MCP setup".into();
        }
    }
    let account = std::process::Command::new("id").arg("-un").output()?;
    ensure!(
        account.status.success() && account.stdout.len() <= 256,
        "Linux account unavailable"
    );
    let response = Response {
        account: String::from_utf8(account.stdout)?.trim().into(),
        preferences,
        available,
        status: if matches!(request.action, Action::Rescan) {
            "Agent scan complete for this WSL account."
        } else if failed {
            "Some MCP settings could not be updated; other integrations were preserved."
        } else {
            "WSL MCP settings saved."
        }
        .into(),
    };
    let bytes = serde_json::to_vec(&response)?;
    ensure!(bytes.len() <= 512 * 1024, "Setup response too large");
    if !removing { std::io::stdout().write_all(&bytes)?; }
    Ok(())
}

#[cfg(all(feature = "wsl_companion", target_os = "linux"))]
fn lock_preferences(path: &std::path::Path) -> Result<std::fs::File> {
    use std::os::{fd::AsRawFd, unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt}};
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid guest preferences path"))?;
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(parent)?;
    let metadata = std::fs::symlink_metadata(parent)?;
    ensure!(metadata.is_dir() && metadata.uid() == unsafe { libc::geteuid() },
        "Guest setup directory must be owned and cannot be a link");
    // Older setup scans created this service namespace with the default umask.
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(parent.join("mcp-settings.lock"))?;
    // Atomic saves replace the preferences inode, so lock a separate stable file.
    ensure!(
        unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
        "Another WSL setup operation is active; rescan after it finishes"
    );
    Ok(file)
}

#[cfg(all(feature = "wsl_companion", target_os = "linux"))]
fn apply_action(
    action: &Action,
    preferences: &mut Preferences,
    available: &mut [Available],
    path: &std::path::Path,
) -> Result<bool> {
    if matches!(action, Action::Rescan) {
        return Ok(false);
    }
    let command = match action {
        Action::Select(command) => Some(command.as_str()),
        _ => None,
    };
    let select_enable = command.is_some_and(|name| {
        !preferences
            .selected
            .get(name)
            .is_some_and(|entry| entry.active)
    });
    if matches!(action, Action::Select(_)) {
        preferences.enabled = true;
    }
    if let Action::Enable(enabled) = action {
        preferences.enabled = *enabled;
    }
    if matches!(action, Action::RemoveAll) {
        preferences.enabled = false;
        for row in available.iter() {
            if let Some(installed) = &row.installed {
                preferences
                    .selected
                    .entry(row.command.clone())
                    .or_insert_with(|| installed.clone());
            }
        }
        // Revoke participation even when removing a vendor configuration fails.
        for installed in preferences.selected.values_mut() {
            installed.active = false;
        }
        setup::save_preferences(path, preferences)?;
    }
    let mut failed = false;
    for (name, installed) in &mut preferences.selected {
        if !preferences.enabled || command == Some(name.as_str()) {
            match setup::configure_guest(installed, false, true) {
                Ok(()) => installed.active = false,
                Err(_) => {
                    installed.active = false;
                    failed = true;
                }
            }
        }
    }
    if preferences.enabled {
        for row in available.iter_mut() {
            let Some(installed) = &row.installed else {
                continue;
            };
            let selected = preferences.selected.get(&row.command);
            let enable = if command == Some(row.command.as_str()) {
                select_enable
            } else {
                selected.is_some_and(|entry| entry.active)
            };
            if !enable {
                continue;
            }
            let mut entry = installed.clone();
            match setup::configure_guest(&entry, true, selected.is_some()) {
                Ok(()) => {
                    entry.active = true;
                    row.status = "WSL MCP enabled".into();
                }
                Err(_) => {
                    entry.active = false;
                    row.status = "WSL MCP setup failed; existing configuration preserved".into();
                    failed = true;
                }
            }
            preferences.selected.insert(row.command.clone(), entry);
        }
    }
    setup::save_preferences(path, preferences)?;
    Ok(failed)
}

#[cfg(all(test, feature = "wsl_companion", target_os = "linux"))]
mod tests {
    use super::*;
    use crate::setup::{Adapter, Installed, SERVER};

    #[test]
    fn windows_path_translation_excludes_host_installs() {
        for native in [r"\\wsl.localhost\Ubuntu\home\guest\.local\bin\dsh", r"\\wsl$\Ubuntu\usr\bin\codex"] {
            assert!(native_windows_translation(native.as_bytes()));
        }
        for host in [r"C:\Users\guest\AppData\Roaming\npm\codex", r"D:\agents\codex", "", "/home/guest/codex"] {
            assert!(!native_windows_translation(host.as_bytes()));
        }
    }

    #[test]
    fn native_screen_fields_are_optional_and_absent_for_legacy_peers() {
        let legacy = serde_json::json!({"shell_pid":42,"action":{"action":"observe","draft":false,
            "blocked":false,"input_epoch":0,"submitted":false,"cancelled":false,"native_ready":null}});
        let request: NativeRequest = serde_json::from_value(legacy.clone()).unwrap();
        assert_eq!(serde_json::to_value(request).unwrap(), legacy);
        let legacy = serde_json::json!({"run":"run","program":"codex","wake":null,"valid":false});
        let state: NativeState = serde_json::from_value(legacy.clone()).unwrap();
        assert_eq!(serde_json::to_value(state).unwrap(), legacy);
    }

    #[test]
    fn setup_namespace_can_start_service_and_hardens_previous_scan_directories() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        for legacy in [false, true] {
            let parent = directory.path().join(if legacy { "legacy" } else { "fresh" });
            if legacy {
                std::fs::create_dir(&parent).unwrap();
                std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
            let _lock = lock_preferences(&parent.join("mcp-settings.json")).unwrap();
            assert_eq!(parent.metadata().unwrap().permissions().mode() & 0o777, 0o700);
            assert!(crate::companion::Companion::new(&parent).is_ok());
        }
        let linked = directory.path().join("linked");
        std::os::unix::fs::symlink(directory.path().join("legacy"), &linked).unwrap();
        assert!(lock_preferences(&linked.join("mcp-settings.json")).is_err());
    }

    #[test]
    fn account_lock_rejects_concurrent_setup_and_symlinks() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("mcp-settings.json");
        let lock = lock_preferences(&path).unwrap();
        assert!(lock_preferences(&path).is_err());
        let other = directory.path().join("other/mcp-settings.json");
        assert!(lock_preferences(&other).is_ok());
        drop(lock);
        assert!(lock_preferences(&path).is_ok());
        std::fs::remove_file(directory.path().join("mcp-settings.lock")).unwrap();
        std::os::unix::fs::symlink(&other, directory.path().join("mcp-settings.lock")).unwrap();
        assert!(lock_preferences(&path).is_err());
    }

    #[test]
    fn maintenance_preserves_other_accounts_and_rescan_is_read_only() {
        let directory = tempfile::tempdir().unwrap();
        let mut accounts = Vec::new();
        for account in [
            "selected",
            "other-user",
            "other-distro",
            "windows-local",
            "ssh",
        ] {
            let home = directory.path().join(account);
            std::fs::create_dir(&home).unwrap();
            let config = home.join("config.toml");
            std::fs::write(
                &config,
                "[mcp_servers.user_tool]\ncommand = \"user-tool\"\n",
            )
            .unwrap();
            let installed = Installed {
                active: true,
                program: "codex".into(),
                executable: "/bin/sh".into(),
                adapter: Adapter::Codex(config.clone()),
                bridge: home.join("warpai-wsl-companion"),
                search_paths: vec![],
                launch_options: Default::default(),
            };
            setup::configure_guest(&installed, true, false).unwrap();
            accounts.push((config.clone(), installed, std::fs::read(&config).unwrap()));
        }
        let path = directory.path().join("selected/mcp-settings.json");
        // Discovery also cleans up owned entries absent from the saved selection.
        let mut preferences = Preferences {
            enabled: true,
            ..Default::default()
        };
        setup::save_preferences(&path, &preferences).unwrap();
        let saved = std::fs::read(&path).unwrap();
        let mut available = vec![Available {
            command: "codex".into(),
            program: "codex".into(),
            installed: Some(accounts[0].1.clone()),
            status: "Available".into(),
        }];
        assert!(!apply_action(&Action::Rescan, &mut preferences, &mut available, &path).unwrap());
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        assert_eq!(std::fs::read(&accounts[0].0).unwrap(), accounts[0].2);
        assert!(preferences.enabled);
        assert!(
            !apply_action(&Action::RemoveAll, &mut preferences, &mut available, &path).unwrap()
        );
        assert!(!preferences.enabled);
        assert!(!preferences.selected["codex"].active);
        let config: toml::Value =
            toml::from_str(&std::fs::read_to_string(&accounts[0].0).unwrap()).unwrap();
        assert!(config["mcp_servers"].get(SERVER).is_none());
        assert_eq!(
            config["mcp_servers"]["user_tool"]["command"].as_str(),
            Some("user-tool")
        );
        for (config, _, original) in &accounts[1..] {
            assert_eq!(std::fs::read(config).unwrap(), *original);
        }
        assert!(!apply_action(
            &Action::Select("codex".into()),
            &mut preferences,
            &mut available,
            &path
        )
        .unwrap());
        assert!(preferences.enabled && preferences.selected["codex"].active);
    }
}
