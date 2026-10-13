//! Native-grid observations; Orca MIT predicates, see docs/ORCA-LICENSE.
use super::{detect, Status};
use crate::readiness::Activity;
use regex::Regex;
use std::{sync::OnceLock, time::Duration};

/// Unknown chrome is not evidence of completion. Text is never retained or logged.
pub fn activity(program: &str, screen: &str, title: &str, quiet: Duration) -> Option<Activity> {
    if screen.len() > 128 * 1024 || title.len() > 4096 {
        return None;
    }
    let agent = match program {
        "agy" => "antigravity",
        "agent" | "cursor-agent" => "cursor",
        "hermes-agent" => "hermes",
        "vibe-acp" => "vibe",
        "qodercli" | "qoder-cli" | "qodercn" | "qoderclicn" => "qoder",
        other => other,
    };
    let detection = detect(agent, screen, title, None);
    let title_activity = title_state(agent, title);
    if let Some(detection) = detection {
        match detection.status {
            Some(Status::Waiting)
                if !matches!(
                    detection.rule,
                    "input_prompt" | "ready_footer" | "cursor_line"
                ) =>
            {
                return Some(Activity::WaitingApproval)
            }
            Some(Status::Error) => return Some(Activity::Error),
            _ => {}
        }
    }
    if matches!(
        title_activity,
        Some(Activity::Working | Activity::WaitingApproval)
    ) {
        return title_activity;
    }
    if detection.is_some_and(|d| d.status == Some(Status::Running)) {
        return Some(Activity::Working);
    }
    // A composer can remain painted during submission; require an actual output/input quiet window.
    if quiet < Duration::from_millis(1000) {
        return None;
    }
    let ready = match agent {
        "codex" => codex_ready(screen) || detection.is_some_and(|d| d.status == Some(Status::Idle)),
        "antigravity" => antigravity_ready(screen),
        "qoder" => {
            let screen = screen.to_lowercase();
            screen.contains("type your message or @path/to/file")
                && !screen.contains("don't trust and exit")
        }
        _ => detection.is_some_and(|d| {
            d.status == Some(Status::Idle)
                || (d.status == Some(Status::Waiting)
                    && matches!(d.rule, "input_prompt" | "ready_footer" | "cursor_line"))
        }),
    };
    if ready {
        Some(Activity::Idle)
    } else if matches!(agent, "antigravity" | "qoder") {
        None
    } else {
        title_activity
    }
}

fn title_state(agent: &str, title: &str) -> Option<Activity> {
    let lower = title.to_lowercase();
    if !agent.is_empty()
        && title.chars().any(|c| {
            ('\u{2800}'..='\u{28ff}').contains(&c) || ('\u{25d0}'..='\u{25d3}').contains(&c)
        })
    {
        return Some(Activity::Working);
    }
    if matches!(agent, "pi" | "omp") {
        if title.starts_with("π - ") {
            return Some(Activity::Idle);
        }
        static PI: OnceLock<Regex> = OnceLock::new();
        let pi =
            PI.get_or_init(|| Regex::new(r"(?:^|[\s|])(π|Pi|OMP)[ \t]+([:!>])(?:\s|$)").unwrap());
        if let Some(marker) = pi.captures(title) {
            return Some(match &marker[2] {
                ">" => Activity::Idle,
                "!" => Activity::WaitingApproval,
                _ => Activity::Working,
            });
        }
    }
    if agent == "qoder" {
        match title.trim_start().chars().next() {
            Some('✦') => return Some(Activity::Working),
            Some('▲') => return Some(Activity::WaitingApproval),
            _ => {}
        }
    }
    let names: &[&str] = match agent {
        "cursor" => &["cursor", "agent", "cursor-agent"],
        "antigravity" => &["antigravity", "agy"],
        "omp" => &["oh my pi", "omp"],
        "pi" => &["pi", "π"],
        "dsh" | "dsh-tui" => &["dsh", "deepseek"],
        "qoder" | "qodercli" | "qoder-cli" | "qodercn" | "qoderclicn" => &["qoder"],
        "trae" | "traecn" | "trae-cli" | "traecn-cli" | "traecli" => &["trae"],
        _ => &[agent],
    };
    if agent.is_empty()
        || !names.iter().any(|name| {
            lower
                .split(|c: char| !c.is_alphanumeric() && c != 'π')
                .any(|word| word == *name)
                || lower.starts_with(name)
        })
    {
        return None;
    }
    if agent == "gemini" {
        if title.contains('✋') {
            return Some(Activity::WaitingApproval);
        }
        if title.contains(['✦', '⏲']) {
            return Some(Activity::Working);
        }
        if title.contains('◇') {
            return Some(Activity::Idle);
        }
    }
    if agent == "claude" && title.starts_with('✳') {
        return Some(Activity::Idle);
    }
    static STATES: OnceLock<Regex> = OnceLock::new();
    let states = STATES.get_or_init(|| {
        Regex::new(r"(?i)(?:^|\s)(ready|idle|done|working|thinking|action required)(?:\s|$|[.!?])")
            .unwrap()
    });
    let state = states.captures(title)?.get(1)?.as_str().to_lowercase();
    Some(match state.as_str() {
        "ready" | "idle" | "done" => Activity::Idle,
        "action required" => Activity::WaitingApproval,
        _ => Activity::Working,
    })
}

fn codex_ready(screen: &str) -> bool {
    let lower = screen.to_lowercase();
    let lines: Vec<_> = lower.lines().map(str::trim).collect();
    let Some(composer) = lines
        .iter()
        .rposition(|line| *line == "› ask codex to do anything")
    else {
        return false;
    };
    let mut above = lines[..composer]
        .iter()
        .rev()
        .filter(|line| !line.is_empty());
    let row = above.next().copied().unwrap_or("");
    let row = if row.starts_with("└ tip:") {
        above.next().copied().unwrap_or("")
    } else {
        row
    };
    if row.contains("to interrupt)") {
        return false;
    }
    if let Some(header) = lines.iter().position(|line| line.contains("openai codex")) {
        static LOADING: OnceLock<Regex> = OnceLock::new();
        let loading = LOADING.get_or_init(|| {
            Regex::new(r"(?:model|directory):\s+loading|(?m)^\s*loading\s*$").unwrap()
        });
        let header = lines
            .iter()
            .skip(header)
            .take(6)
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        if loading.is_match(&header) {
            return false;
        }
    }
    true
}

fn antigravity_ready(screen: &str) -> bool {
    let lines: Vec<_> = screen
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .rev()
        .take(4)
        .collect();
    if lines.len() != 4 {
        return false;
    }
    let rule = |line: &str| line.chars().count() >= 8 && line.chars().all(|c| c == '─');
    static MODE: OnceLock<Regex> = OnceLock::new();
    let mode = MODE.get_or_init(|| Regex::new(r"(?i)^>\s+[a-z][a-z-]*\s+mode:\s").unwrap());
    lines[0].to_lowercase().starts_with("? for shortcuts")
        && rule(lines[1])
        && (lines[2] == ">" || mode.is_match(lines[2]))
        && rule(lines[3])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn welcome_is_not_completion_and_real_composers_recover_after_errors() {
        let quiet = Duration::from_secs(2);
        for agent in [
            "codex", "agy", "claude", "amp", "droid", "opencode", "copilot", "pi", "omp", "auggie",
            "agent", "goose", "hermes", "vibe", "grok", "dsh", "qoder", "qodercn", "trae",
            "custom",
        ] {
            assert_eq!(activity(agent, "Welcome", agent, quiet), None, "{agent}");
            if !matches!(agent, "agy" | "qoder" | "qodercn") {
                assert_eq!(
                    activity(agent, "Welcome", &format!("{agent} ready"), quiet),
                    Some(Activity::Idle),
                    "explicit native title: {agent}"
                );
            }
            assert_eq!(
                activity(agent, "Welcome", &format!("⠋ {agent}"), quiet),
                Some(Activity::Working),
                "native busy title: {agent}"
            );
        }
        let codex = "OpenAI Codex\nWarning: update available\n› Ask Codex to do anything\n";
        let agy =
            "Error: previous request failed\n────────────\n>\n────────────\n? for shortcuts\n";
        for (agent, screen) in [("codex", codex), ("agy", agy)] {
            assert_eq!(activity(agent, screen, "", quiet), Some(Activity::Idle));
            assert_eq!(activity(agent, screen, "", Duration::ZERO), None);
        }
        assert_ne!(
            activity(
                "agy",
                &agy.replace("? for shortcuts", "esc to cancel"),
                "",
                quiet
            ),
            Some(Activity::Idle)
        );
        assert_ne!(
            activity(
                "agy",
                &agy.replace("\n>\n", "\n> selected model\n"),
                "",
                quiet
            ),
            Some(Activity::Idle)
        );
        assert_ne!(
            activity(
                "codex",
                "OpenAI Codex\n• Processing (esc to interrupt)\n› Ask Codex to do anything",
                "",
                quiet
            ),
            Some(Activity::Idle)
        );
        assert_ne!(
            activity(
                "codex",
                "OpenAI Codex\nmodel: loading\n› Ask Codex to do anything",
                "",
                quiet
            ),
            Some(Activity::Idle)
        );
        assert_eq!(
            activity("codex", codex, "⠋ Codex", quiet),
            Some(Activity::Working)
        );
        assert_ne!(
            activity(
                "codex",
                "OpenAI Codex\n│ model:     loading │\n› Ask Codex to do anything",
                "",
                quiet
            ),
            Some(Activity::Idle)
        );
        assert_eq!(
            activity(
                "codex",
                "──── Worked for 2s\n› Explain this codebase",
                "",
                quiet
            ),
            Some(Activity::Idle)
        );
        assert_eq!(
            activity("omp", "", "π - session", quiet),
            Some(Activity::Idle)
        );
        assert_eq!(activity("omp", "", "π > cwd", quiet), Some(Activity::Idle));
        assert_eq!(
            activity("omp", "", "π : cwd", quiet),
            Some(Activity::Working)
        );
        assert_eq!(
            activity("omp", "", "π ! cwd", quiet),
            Some(Activity::WaitingApproval)
        );
        assert_eq!(
            activity(
                "qodercn",
                "Type your message or @path/to/file",
                "◇ Qoder CLI CN",
                quiet
            ),
            Some(Activity::Idle)
        );
        assert_ne!(
            activity(
                "qoder",
                "Type your message or @path/to/file\nDon't trust and exit",
                "◇ Qoder CLI",
                quiet
            ),
            Some(Activity::Idle)
        );
        assert_eq!(
            activity("codex", "› 1. Yes\n  2. No\nenter to select", "", quiet),
            Some(Activity::WaitingApproval)
        );
    }
}
