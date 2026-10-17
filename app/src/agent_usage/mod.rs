//! Opt-in, desktop-local provider usage; credentials never enter persisted metadata.
pub(crate) mod providers;
use async_compat::CompatExt as _;
use providers::{Provider, Reading};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Write as _},
    path::PathBuf,
    time::{Duration, Instant},
};
use warpui::{Entity, ModelContext, SingletonEntity};
use warpui_extras::secure_storage::AppContextExt as _;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Account {
    pub(crate) id: String,
    pub(crate) provider: Provider,
    pub(crate) label: String,
    pub(crate) context: String,
    pub(crate) visible: bool,
    pub(crate) cli_login: bool,
    #[serde(default = "detect_5h_default")]
    pub(crate) detect_5h: bool,
}
fn detect_5h_default() -> bool { true }
#[derive(Default)]
pub(crate) struct AgentUsage {
    pub(crate) accounts: Vec<Account>,
    pub(crate) readings: HashMap<String, Result<Reading, &'static str>>,
    pub(crate) checking: bool,
    pub(crate) status: String,
    pub(crate) updated: Option<Instant>,
    generation: u64,
    last_qoder_query: HashMap<String, Instant>,
    configuration_error: bool,
    #[cfg(debug_assertions)]
    capture_preview: bool,
}
fn metadata_path() -> PathBuf {
    warp_core::paths::data_dir().join("agent-usage-accounts.json")
}
fn key(id: &str) -> String {
    format!("warpai-agent-usage-{id}")
}
fn valid_account(account: &Account) -> bool {
    uuid::Uuid::parse_str(&account.id).is_ok()
        && !account.label.trim().is_empty()
        && account.label.len() <= 80
        && account.context.len() <= 200
        && !account.label.chars().any(char::is_control)
        && !account.context.chars().any(char::is_control)
}
impl AgentUsage {
    pub(crate) fn new(ctx: &mut ModelContext<Self>) -> Self {
        let mut model = Self::default();
        match bounded_json(metadata_path()) {
            Ok(value) => match serde_json::from_value::<Vec<Account>>(value) {
                Ok(accounts)
                    if accounts.len() <= 64
                        && accounts.iter().all(valid_account)
                        && accounts
                            .iter()
                            .map(|a| &a.id)
                            .collect::<std::collections::HashSet<_>>()
                            .len()
                            == accounts.len() =>
                {
                    model.accounts = accounts
                }
                _ => {
                    model.status =
                        "Could not load usage accounts; existing configuration was preserved".into()
                }
            },
            Err(_) if metadata_path().exists() => {
                model.status =
                    "Could not load usage accounts; existing configuration was preserved".into()
            }
            Err(_) => (),
        }
        model.configuration_error = !model.status.is_empty();
        Self::schedule(ctx);
        model
    }
    fn schedule(ctx: &mut ModelContext<Self>) {
        ctx.spawn(
            async { warpui::r#async::Timer::after(Duration::from_secs(60)).await },
            |model, _, ctx| {
                if model
                    .updated
                    .is_none_or(|time| time.elapsed() >= Duration::from_secs(300))
                {
                    model.refresh(ctx);
                }
                Self::schedule(ctx);
            },
        );
    }
    fn persist(accounts: &[Account]) -> Result<(), &'static str> {
        let bytes = serde_json::to_vec(accounts).map_err(|_| "Could not save usage accounts")?;
        crate::agent_communication::setup::atomic_write(&metadata_path(), &bytes)
            .map_err(|_| "Could not save usage accounts")
    }
    pub(crate) fn add(
        &mut self,
        provider: Provider,
        label: String,
        context: String,
        token: String,
        detect_5h: bool,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        if provider == Provider::OpenCodeZen && context.is_empty() {
            self.status = "A workspace ID is required for Zen balance".into();
            ctx.notify();
            return false;
        }
        let cli_login = token.is_empty();
        let account = Account {
            id: uuid::Uuid::new_v4().to_string(),
            provider,
            label,
            context,
            visible: true,
            cli_login,
            detect_5h,
        };
        if !self.configuration_error
            && self.accounts.len() < 64
            && valid_account(&account)
            && token.len() <= 16 * 1024
            && !token.chars().any(char::is_control)
            && (!cli_login
                || matches!(
                    provider,
                    Provider::Codex
                        | Provider::Claude
                        | Provider::Gemini
                        | Provider::Kimi
                        | Provider::Grok
                        | Provider::Antigravity
                        | Provider::Qoder
                        | Provider::QoderCN
                ))
        {
            if !cli_login
                && ctx
                    .secure_storage()
                    .write_value(&key(&account.id), &token)
                    .is_err()
            {
                self.status = "Could not store credential in OS credential storage".into();
                ctx.notify();
                return false;
            }
            let mut next = self.accounts.clone();
            next.push(account.clone());
            if let Err(error) = Self::persist(&next) {
                if !cli_login {
                    let _ = ctx.secure_storage().remove_value(&key(&account.id));
                }
                self.status = error.into();
                ctx.notify();
                return false;
            }
            self.accounts = next;
            self.status.clear();
            self.generation += 1;
            self.refresh(ctx);
            ctx.notify();
            true
        } else {
            self.status = "Enter a label and the required credential; account limit is 64".into();
            ctx.notify();
            false
        }
    }
    pub(crate) fn toggle(&mut self, id: &str, ctx: &mut ModelContext<Self>) {
        if self.configuration_error {
            return;
        }
        let mut next = self.accounts.clone();
        if let Some(account) = next.iter_mut().find(|a| a.id == id) {
            account.visible = !account.visible;
        }
        match Self::persist(&next) {
            Ok(()) => {
                self.accounts = next;
                self.generation += 1;
                self.refresh(ctx);
            }
            Err(error) => self.status = error.into(),
        }
        ctx.notify();
    }
    pub(crate) fn toggle_5h(&mut self, id: &str, ctx: &mut ModelContext<Self>) {
        if self.configuration_error { return; }
        let mut next = self.accounts.clone();
        if let Some(account) = next.iter_mut().find(|account| account.id == id) {
            account.detect_5h = !account.detect_5h;
        }
        match Self::persist(&next) {
            Ok(()) => {
                self.accounts = next;
                self.generation += 1;
                self.refresh(ctx);
            }
            Err(error) => self.status = error.into(),
        }
        ctx.notify();
    }
    pub(crate) fn remove(&mut self, id: &str, ctx: &mut ModelContext<Self>) {
        if self.configuration_error {
            return;
        }
        let has_credential = self.accounts.iter().any(|a| a.id == id && !a.cli_login);
        let mut next = self.accounts.clone();
        next.retain(|a| a.id != id);
        match Self::persist(&next) {
            Ok(()) => {
                self.accounts = next;
                self.readings.remove(id);
                self.generation += 1;
                if has_credential && ctx.secure_storage().remove_value(&key(id)).is_err() {
                    self.status =
                        "Account removed; OS credential cleanup could not be confirmed".into();
                }
            }
            Err(error) => self.status = error.into(),
        }
        ctx.notify();
    }
    pub(crate) fn refresh(&mut self, ctx: &mut ModelContext<Self>) {
        self.refresh_with_qoder_bypass(ctx, false);
    }
    pub(crate) fn refresh_manual(&mut self, ctx: &mut ModelContext<Self>) {
        self.refresh_with_qoder_bypass(ctx, true);
    }
    fn refresh_with_qoder_bypass(
        &mut self,
        ctx: &mut ModelContext<Self>,
        manual: bool,
    ) {
        #[cfg(debug_assertions)]
        if self.capture_preview {
            return;
        }
        if self.checking {
            return;
        }
        let now = Instant::now();
        let requests: Vec<_> = self
            .accounts
            .iter()
            .filter(|a| a.visible)
            .filter(|a| qoder_query_due(a.provider, self.last_qoder_query.get(&a.id), manual))
            .filter(|a| {
                !matches!(
                    self.readings.get(&a.id),
                    Some(Err(providers::AGY_UNSAFE_USAGE))
                )
            })
            .map(|account| {
                let token = if account.cli_login {
                    None
                } else {
                    ctx.secure_storage().read_value(&key(&account.id)).ok()
                };
                (account.clone(), token)
            })
            .collect();
        if requests.is_empty() {
            return;
        }
        for (account, _) in &requests {
            if matches!(account.provider, Provider::Qoder | Provider::QoderCN) {
                self.last_qoder_query.insert(account.id.clone(), now);
            }
        }
        self.checking = true;
        let generation = self.generation;
        ctx.notify();
        ctx.spawn(
            async move {
                futures::future::join_all(requests.into_iter().map(|(account, token)| async move {
                    let reading = async {
                        tokio::time::timeout(Duration::from_secs(40), query(&account, token))
                            .await
                            .unwrap_or(Err("Usage query timed out"))
                    }
                    .compat()
                    .await;
                    (account.id, reading)
                }))
                .await
            },
            move |model, rows, ctx| {
                model.checking = false;
                // A visibility change must not erase evidence of a billed CLI probe.
                for (id, reading) in &rows {
                    if matches!(reading, Err(providers::AGY_UNSAFE_USAGE))
                        && model.accounts.iter().any(|account| &account.id == id)
                    {
                        model.readings.insert(id.clone(), reading.clone());
                    }
                }
                if generation == model.generation {
                    model.readings.extend(rows);
                    model.updated = Some(Instant::now());
                } else {
                    model.refresh(ctx);
                }
                ctx.notify();
            },
        );
    }
    #[cfg(debug_assertions)]
    pub(crate) fn capture_fixture(&mut self, state: &str, ctx: &mut ModelContext<Self>) {
        self.capture_preview = true;
        self.generation += 1;
        self.accounts.clear();
        self.readings.clear();
        self.checking = state == "loading";
        self.status.clear();
        self.updated = Some(Instant::now());
        if state != "empty" {
            for index in 0..if state == "overflow" { 8 } else { 3 } {
                let provider =
                    [Provider::Codex, Provider::Claude, Provider::OpenCodeZen][index % 3];
                let id = uuid::Uuid::from_u128(index as u128 + 1).to_string();
                self.accounts.push(Account {
                    id: id.clone(),
                    provider,
                    label: if state == "overflow" {
                        "A long account label for native clipping verification".into()
                    } else {
                        format!("Sample {}", index + 1)
                    },
                    context: String::new(),
                    visible: true,
                    cli_login: false,
                    detect_5h: true,
                });
                if state != "loading" {
                    self.readings.insert(
                        id,
                        if state == "error" {
                            Err("Sign-in expired; reconnect in Settings")
                        } else if state == "balance" || provider == Provider::OpenCodeZen {
                            Ok(Reading {
                                windows: Vec::new(),
                                balance: Some(27.86),
                            })
                        } else {
                            Ok(Reading {
                                windows: vec![providers::Window {
                                    name: "5h".into(),
                                    used: 37. + index as f64 * 22.,
                                    reset: None,
                                }],
                                balance: None,
                            })
                        },
                    );
                }
            }
        }
        ctx.notify();
    }
}
fn qoder_query_due(provider: Provider, last: Option<&Instant>, manual: bool) -> bool {
    manual
        || !matches!(provider, Provider::Qoder | Provider::QoderCN)
        || last.is_none_or(|last| last.elapsed() >= Duration::from_secs(300))
}
impl Entity for AgentUsage {
    type Event = ();
}
impl SingletonEntity for AgentUsage {}

pub(crate) fn agent_icon(program: &str) -> crate::ui_components::icons::Icon {
    let name = std::path::Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program);
    enum_iterator::all::<crate::terminal::cli_agent::CLIAgent>()
        .find(|agent| {
            agent
                .command_prefixes()
                .iter()
                .any(|prefix| name == *prefix || name.strip_suffix(".exe") == Some(*prefix))
        })
        .and_then(|agent| agent.icon())
        .unwrap_or(crate::ui_components::icons::Icon::Terminal)
}
fn bounded_json(path: PathBuf) -> Result<serde_json::Value, &'static str> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|_| "Sign in with the CLI first")?
        .take(256 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Could not read CLI login")?;
    if bytes.len() > 256 * 1024 {
        return Err("CLI login data too large");
    }
    serde_json::from_slice(&bytes).map_err(|_| "CLI login data unavailable")
}
async fn output(program: &str, args: &[&str], seconds: u64) -> Result<Vec<u8>, &'static str> {
    use tokio::io::AsyncReadExt as _;
    let operation = async {
        let mut command = tokio::process::Command::new(program);
        #[cfg(windows)]
        command.creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0);
        let path = std::env::join_paths(
            std::path::Path::new(program)
                .parent()
                .into_iter()
                .map(std::path::Path::to_owned)
                .chain(std::env::split_paths(
                    &std::env::var_os("PATH").unwrap_or_default(),
                )),
        )
        .map_err(|_| "CLI executable search path unavailable")?;
        let mut child = command
            .args(args)
            .env("PATH", path)
            .kill_on_drop(true)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|_| "CLI is unavailable")?;
        let stdout = child.stdout.take().ok_or("CLI output unavailable")?;
        let mut bytes = Vec::new();
        stdout
            .take(512 * 1024 + 1)
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| "Could not read CLI output")?;
        if bytes.len() > 512 * 1024 {
            return Err("CLI response too large");
        }
        if !child
            .wait()
            .await
            .map_err(|_| "CLI query failed")?
            .success()
        {
            return Err("CLI query failed; sign in again");
        }
        Ok(bytes)
    };
    tokio::time::timeout(Duration::from_secs(seconds), operation)
        .await
        .map_err(|_| "CLI usage query timed out")?
}

async fn qoder_usage(provider: Provider) -> Result<Reading, &'static str> {
    use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt};

    async fn response<R: AsyncBufRead + Unpin>(
        reader: &mut R,
        request_id: &str,
    ) -> Result<serde_json::Value, &'static str> {
        let mut bytes_read = 0;
        for _ in 0..128 {
            let mut line = Vec::new();
            loop {
                let available = reader.fill_buf().await.map_err(|_| "CLI output unavailable")?;
                if available.is_empty() {
                    return Err("CLI closed the usage channel");
                }
                let end = available.iter().position(|byte| *byte == b'\n');
                let take = end.map_or(available.len(), |index| index + 1);
                bytes_read += take;
                if bytes_read > 1024 * 1024 {
                    return Err("CLI usage response too large");
                }
                line.extend_from_slice(&available[..take]);
                reader.consume(take);
                if end.is_some() {
                    break;
                }
            }
            let Ok(message) = serde_json::from_slice::<serde_json::Value>(&line) else {
                continue;
            };
            if message["type"] == "control_response"
                && message["response"]["request_id"] == request_id
            {
                if message["response"]["subtype"] != "success" {
                    return Err("Qoder usage query is unsupported by this CLI version");
                }
                return Ok(message["response"]["response"].clone());
            }
        }
        Err("Qoder usage response unavailable")
    }

    let home = dirs::home_dir().ok_or("CLI home directory unavailable")?;
    let names: &[&str] = match provider {
        Provider::Qoder => &["qodercli", "qoder", "qoder-cli"],
        Provider::QoderCN => &["qoderclicn", "qodercn", "qoder-cn"],
        _ => return Err("Unsupported Qoder provider"),
    };
    let executable = names
        .iter()
        .find_map(|name| crate::agent_communication::setup::executable(name, &[], &home))
        .ok_or("Qoder CLI is unavailable")?;
    let mut auth = tempfile::NamedTempFile::new().map_err(|_| "Could not prepare CLI login")?;
    auth.write_all(br#"{"type":"qodercli"}"#)
        .map_err(|_| "Could not prepare CLI login")?;
    let mut command = tokio::process::Command::new(&executable);
    #[cfg(windows)]
    command.creation_flags(windows::Win32::System::Threading::CREATE_NO_WINDOW.0);
    let path = std::env::join_paths(executable.parent().into_iter().map(PathBuf::from).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .map_err(|_| "CLI executable search path unavailable")?;
    let mut child = command
        .args([
            "--print",
            "--output-format",
            "stream-json",
            "--input-format",
            "stream-json",
            "--no-session-persistence",
            "--tools",
            "default",
            "--disable-builtin-skills",
        ])
        .env("PATH", path)
        // shortcut: follows the 1.0.53 bridge contract; update when the CLI control protocol changes.
        .env("QODER_AGENT_SDK_ENTRYPOINT", "sdk-ts")
        .env("QODER_AGENT_SDK_VERSION", "1.0.53")
        .env("QODER_SDK_AUTH_PAYLOAD_FILE", auth.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "Could not start Qoder CLI usage query")?;
    let mut stdin = child.stdin.take().ok_or("CLI input unavailable")?;
    let mut stdout =
        tokio::io::BufReader::new(child.stdout.take().ok_or("CLI output unavailable")?);
    let exchange = async {
        for (request_id, request_type) in [
            ("warpai-usage-init", "initialize"),
            ("warpai-usage-query", "get_usage_info"),
        ] {
            let request = serde_json::json!({
                "type":"control_request",
                "request_id":request_id,
                "request":{"type":request_type}
            });
            stdin
                .write_all(request.to_string().as_bytes())
                .await
                .map_err(|_| "Could not send Qoder usage request")?;
            stdin
                .write_all(b"\n")
                .await
                .map_err(|_| "Could not send Qoder usage request")?;
            stdin
                .flush()
                .await
                .map_err(|_| "Could not send Qoder usage request")?;
            let result = response(&mut stdout, request_id).await?;
            if request_type == "get_usage_info" {
                if result["usage_error"].is_string() {
                    return Err("Qoder plan quota is unavailable for this account");
                }
                return providers::parse(provider, &result["usage"]);
            }
        }
        Err("Qoder usage response unavailable")
    };
    let result = tokio::time::timeout(Duration::from_secs(25), exchange)
        .await
        .map_err(|_| "Qoder usage query timed out")?;
    drop(stdin);
    let _ = child.kill().await;
    result
}

async fn query(account: &Account, token: Option<String>) -> Result<Reading, &'static str> {
    if account.provider == Provider::Antigravity {
        let home = dirs::home_dir().ok_or("CLI home directory unavailable")?;
        let executable = crate::agent_communication::setup::executable("agy", &[], &home)
            .ok_or("Antigravity CLI is unavailable")?;
        let executable = executable
            .to_str()
            .ok_or("CLI executable path unavailable")?;
        let version = output(executable, &["--version"], 5).await?;
        if !providers::safe_agy_version(&String::from_utf8_lossy(&version)) {
            return Err("Requires agy 1.1.11 or newer for a free usage query");
        }
        let bytes = output(
            executable,
            &[
                "-p",
                "/usage",
                "--output-format",
                "json",
                "--print-timeout",
                "20s",
            ],
            30,
        )
        .await?;
        return providers::parse_agy_stdout(&bytes);
    }
    if matches!(account.provider, Provider::Qoder | Provider::QoderCN) {
        return qoder_usage(account.provider).await;
    }
    let mut context = account.context.clone();
    let token = if account.cli_login {
        let home = dirs::home_dir().ok_or("CLI home directory unavailable")?;
        let data = match account.provider {
            Provider::Codex => bounded_json(
                std::env::var_os("CODEX_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| home.join(".codex"))
                    .join("auth.json"),
            )?,
            Provider::Claude => {
                #[cfg(target_os = "macos")]
                {
                    serde_json::from_slice(
                        &output(
                            "/usr/bin/security",
                            &[
                                "find-generic-password",
                                "-s",
                                "Claude Code-credentials",
                                "-w",
                            ],
                            5,
                        )
                        .await?,
                    )
                    .map_err(|_| "Claude login unavailable")?
                }
                #[cfg(not(target_os = "macos"))]
                {
                    bounded_json(home.join(".claude/.credentials.json"))?
                }
            }
            Provider::Gemini => bounded_json(home.join(".gemini/oauth_creds.json"))?,
            Provider::Kimi => bounded_json(
                std::env::var_os("KIMI_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| home.join(".kimi"))
                    .join("credentials/kimi-code.json"),
            )?,
            Provider::Grok => bounded_json(
                std::env::var_os("GROK_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| home.join(".grok"))
                    .join("auth.json"),
            )?,
            _ => return Err("Connect a credential in Settings"),
        };
        let token = match account.provider {
            Provider::Codex => {
                if context.is_empty() {
                    context = data["tokens"]["account_id"]
                        .as_str()
                        .unwrap_or("")
                        .to_owned();
                }
                data["tokens"]["access_token"].as_str()
            }
            Provider::Claude => data["claudeAiOauth"]["accessToken"].as_str(),
            Provider::Grok => data
                .as_object()
                .and_then(|entries| {
                    entries
                        .iter()
                        .find(|(k, v)| {
                            (k.as_str() == "https://auth.x.ai"
                                || k.starts_with("https://auth.x.ai::"))
                                && v["key"].is_string()
                        })
                        .map(|(_, v)| v)
                })
                .and_then(|v| {
                    if context.is_empty() {
                        context = v["user_id"].as_str().unwrap_or("").to_owned();
                    }
                    v["key"].as_str()
                }),
            _ => data["access_token"].as_str(),
        };
        token
            .ok_or("CLI login unavailable; sign in again")?
            .to_owned()
    } else {
        token.ok_or("Credential unavailable; reconnect in Settings")?
    };
    providers::fetch(account.provider, &token, &context).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_automatic_qoder_usage_queries_are_rate_limited() {
        let now = Instant::now();
        let recent = now - Duration::from_secs(299);
        let due = now - Duration::from_secs(300);
        assert!(qoder_query_due(Provider::QoderCN, None, false));
        assert!(!qoder_query_due(Provider::Qoder, Some(&recent), false));
        assert!(qoder_query_due(Provider::QoderCN, Some(&due), false));
        assert!(qoder_query_due(Provider::QoderCN, Some(&recent), true));
        assert!(qoder_query_due(Provider::Codex, Some(&recent), false));
    }
    #[test]
    fn account_metadata_contains_no_credentials_and_rejects_untrusted_labels() {
        let mut account = Account {
            id: uuid::Uuid::new_v4().to_string(),
            provider: Provider::Codex,
            label: "Current CLI account".into(),
            context: String::new(),
            visible: true,
            cli_login: true,
            detect_5h: true,
        };
        assert!(valid_account(&account));
        let json = serde_json::to_value(&account).unwrap();
        let mut legacy = json.clone();
        legacy.as_object_mut().unwrap().remove("detect_5h");
        assert!(serde_json::from_value::<Account>(legacy).unwrap().detect_5h);
        account.detect_5h = false;
        assert!(!serde_json::from_value::<Account>(serde_json::to_value(&account).unwrap()).unwrap().detect_5h);
        assert!(json.get("token").is_none());
        assert!(json.get("credential").is_none());
        account.label = "bad\nlabel".into();
        assert!(!valid_account(&account));
        account.label = "Valid".into();
        account.id = "../../other".into();
        assert!(!valid_account(&account));
    }
}
