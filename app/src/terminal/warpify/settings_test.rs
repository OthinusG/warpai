#[cfg(windows)]
use super::WarpifySettings;

#[cfg(windows)]
#[test]
fn test_wsl_subshell_detection_success() {
    [
        "wsl",
        "wsl.exe",
        "wsl -d Ubuntu",
        "wsl --distribution Ubuntu",
        "wsl -u user",
        "wsl --cd /home/user",
        "wsl --system",
        "wsl --shell-type login",
        "wsl -d Ubuntu --cd /home/user -u username",
        "wsl.exe -d Ubuntu --cd /home/user -u username",
    ]
    .iter()
    .for_each(|cmd| {
        assert!(
            WarpifySettings::is_built_in_subshell_match(cmd),
            "{} failed to match",
            *cmd
        )
    });
}

#[cfg(windows)]
#[test]
fn test_wsl_subshell_detection_fail() {
    [
        "wsl --install",
        "wsl --status",
        "wsl --list",
        "wsl --export Ubuntu file.tar",
        "wsl --uninstall",
        "wsl --shutdown",
        "wslfetch",
        "nowsl",
        "wsl --help",
        "wsl --version",
        "wsl --terminate Ubuntu",
        "wsl --unregister Ubuntu",
        "wsl --update",
        "wsl --import-in-place Ubuntu",
        "wsl --default-user root",
        "wsl --mount \\device",
    ]
    .iter()
    .for_each(|cmd| {
        assert!(
            !WarpifySettings::is_built_in_subshell_match(cmd),
            "{} accidentally matched",
            *cmd
        )
    });
}

#[cfg(windows)]
    #[test]
    fn automatic_wsl_integration_accepts_only_literal_interactive_launches() {
        for command in ["wsl", "wsl.exe -d Ubuntu -u guest", "wsl --distribution 'Custom Linux' --cd /home/guest"] { assert!(WarpifySettings::is_interactive_wsl_command(command)); }
        for command in ["wsl --status", "wsl --exec codex", "wsl -e bash", "wsl --system", "wsl -d", "wsl -u -root", "wsl; codex", "wsl -d $name", "wsl -d $(whoami)", "ssh host wsl", "wslfetch"] { assert!(!WarpifySettings::is_interactive_wsl_command(command)); }
    }
