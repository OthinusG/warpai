//! Account-owned SSH MCP setup; only explicit human settings actions configure vendors.
use crate::{setup, wsl_setup::Action};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::{io::Read, path::{Path, PathBuf}};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub action: Action,
    pub commands: Vec<(String, String)>,
    pub search_paths: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub account_id: String,
    pub preferences: setup::Preferences,
    pub available: Vec<setup::Available>,
    pub status: String,
}

struct SetupLock(std::fs::File);
impl Drop for SetupLock {
    fn drop(&mut self) {
        // Vendor probes can inherit descriptors; release the logical setup scope explicitly.
        let _ = self.0.unlock();
    }
}

pub(crate) fn configure(request: Request) -> Result<Response> {
    validate(&request)?;
    let directory = crate::companion::data_directory()?;
    configure_in(request, &directory, &std::env::current_exe()?)
}

fn validate(request: &Request) -> Result<()> {
    ensure!(request.commands.len() <= 128 && request.commands.iter().all(|(program, command)| {
        [program, command].into_iter().all(|value| !value.is_empty() && !value.starts_with('-') && value.len() <= 64
            && value.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b)))
    }), "Invalid Agent command");
    ensure!(request.search_paths.len() <= 128 && request.search_paths.iter().all(|path|
        path.len() <= 4096 && !path.chars().any(char::is_control) && Path::new(path).is_absolute()),
        "Invalid remote search path");
    if let Action::Select(command) = &request.action {
        ensure!(request.commands.iter().any(|(_, name)| name == command), "Unknown Agent selection");
    }
    Ok(())
}

fn configure_in(request: Request, directory: &Path, bridge: &Path) -> Result<Response> {
    validate(&request)?;
    crate::companion::identity::private_directory(directory)?;
    let lock = crate::companion::identity::private_file(&directory.join("mcp-settings.lock"))?;
    lock.try_lock().map_err(|_| anyhow::anyhow!("Another SSH setup operation is active"))?;
    let _lock = SetupLock(lock);
    let path = directory.join("mcp-settings.json");
    let mut bytes = Vec::new();
    crate::companion::identity::private_file(&path)?.take(512 * 1024 + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 512 * 1024, "SSH preferences too large");
    let mut preferences: setup::Preferences = if bytes.is_empty() {
        Default::default()
    } else { serde_json::from_slice(&bytes)? };
    let search_paths = request.search_paths.into_iter().map(PathBuf::from).collect();
    let mut available = setup::discover(request.commands, bridge, search_paths);
    let failed = crate::wsl_setup::apply_native_action(&request.action, &mut preferences,
        &mut available, &path, "SSH", setup::configure_native_mcp)?;
    let response = Response {
        account_id: crate::companion::identity::account_id()?,
        preferences, available,
        status: if failed { "Some MCP settings could not be updated; other integrations were preserved." }
            else if matches!(request.action, Action::Rescan) { "Agent scan complete for this SSH account." }
            else { "SSH MCP settings saved." }.into(),
    };
    ensure!(serde_json::to_vec(&response)?.len() <= 512 * 1024, "SSH setup response too large");
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn setup_rejects_unbounded_or_non_native_inputs_before_discovery() {
        let request = |command: &str, path: &str| Request {
            action: Action::Rescan, commands: vec![("codex".into(), command.into())],
            search_paths: vec![path.into()],
        };
        for (command, path) in [("--help", "relative"), ("codex;other", "/tmp"), ("codex", "~/.bin"), ("codex", "/tmp\n")] {
            assert!(validate(&request(command, path)).is_err());
        }
        assert!(validate(&Request { action: Action::Rescan, commands: vec![], search_paths: vec![] }).is_ok());
    }

    #[test]
    fn account_preferences_are_private_scoped_and_setup_lock_is_released() {
        let directory = tempfile::tempdir().unwrap();
        let bridge = std::env::current_exe().unwrap();
        let account = directory.path().join("account");
        let request = |action| Request { action, commands: vec![], search_paths: vec![] };
        let first = configure_in(request(Action::Enable(true)), &account, &bridge).unwrap();
        assert!(first.preferences.enabled);
        assert!(first.available.is_empty());
        let second = configure_in(request(Action::Enable(false)), &account, &bridge).unwrap();
        assert!(!second.preferences.enabled);
        assert_eq!(first.account_id, second.account_id);
        let other = tempfile::tempdir().unwrap();
        assert!(!configure_in(request(Action::Rescan), &other.path().join("account"), &bridge).unwrap().preferences.enabled);
    }

    #[test]
    fn native_codex_mcp_preserves_unrelated_entries_and_rejects_user_changes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        let original = "# User settings\nmodel = 'fixture'\n[mcp_servers.user_tool]\ncommand = 'user-tool'\n";
        std::fs::write(&path, original).unwrap();
        let installed = setup::Installed { active: true, program: "codex".into(),
            executable: std::env::current_exe().unwrap(), adapter: setup::Adapter::Codex(path.clone()),
            bridge: directory.path().join("warpai-companion"), search_paths: vec![], launch_options: Default::default() };
        setup::configure_native_mcp(&installed, true, false).unwrap();
        assert!(setup::configure_native_mcp(&installed, true, false).is_err());
        setup::configure_native_mcp(&installed, true, true).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let value: toml::Value = toml::from_str(&text).unwrap();
        assert!(value["mcp_servers"][setup::SERVER].get("env").is_none());
        assert_eq!(value["mcp_servers"][setup::SERVER]["args"].as_array().unwrap(),
            &[toml::Value::String("mcp".into()), toml::Value::String("codex".into())]);
        setup::configure_native_mcp(&installed, false, true).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# User settings"));
        assert!(text.contains("user-tool"));
        assert!(text.contains("fixture"));
    }
}
