//! Native-grid observations; Orca MIT predicates, see docs/ORCA-LICENSE.
use super::{detect, Status};
use crate::readiness::Activity;
use regex::Regex;
use std::{sync::OnceLock, time::Duration};

/// Unknown chrome is not evidence of completion. Text is never retained or logged.
pub fn activity(program: &str, screen: &str, title: &str, quiet: Duration) -> Option<Activity> {
    activity_at_cursor(program, screen, title, quiet, None)
}

/// The visible native cursor binds typed drafts to a composer, never a quoted prompt or menu.
pub fn activity_at_cursor(program: &str, screen: &str, title: &str, quiet: Duration, cursor: Option<usize>) -> Option<Activity> {
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
    let composer = composer_rows(agent, screen, cursor);
    // Draft words such as "approve" or "working" are not lifecycle signals.
    let chrome = composer.as_ref().map(|range| {
        let lines: Vec<_> = screen.lines().collect();
        // A cursor-bound composer supersedes transcript warnings and old activity text.
        let live_above = lines[..range.start].iter().rev()
            .find(|line| !line.trim().is_empty() && !line.trim().chars().all(|c| c == '─'))
            .filter(|line| { let lower = line.to_lowercase(); lower.contains("esc to interrupt")
                || lower.contains("ctrl+c to interrupt") || lower.contains("esc to cancel") });
        live_above.into_iter().map(|line| *line).chain(lines[range.end..].iter().copied())
            .collect::<Vec<_>>().join("\n")
    });
    let detection = detect(agent, chrome.as_deref().unwrap_or(screen), title, None);
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
            None if detection.rule != "no_rule" => return None,
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
        "codex" => codex_ready(screen, composer.as_ref().map(|range| range.start)) || detection.is_some_and(|d| d.status == Some(Status::Idle)),
        "antigravity" => antigravity_ready(screen) || composer.is_some(),
        "claude" => composer.is_some() || detection.is_some_and(|d|
            d.status == Some(Status::Idle) && d.rule != "live_prompt_box"),
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
    if agent == "claude" {
        if title.starts_with("* ") { return Some(Activity::Idle); }
        if title.starts_with(". ") { return Some(Activity::Working); }
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

fn model_footer(line: &str) -> bool {
    (line.starts_with("gpt-") || line.starts_with("o3") || line.starts_with("o4")) && line.contains('·')
}

fn codex_ready(screen: &str, cursor: Option<usize>) -> bool {
    let lower = screen.to_lowercase();
    let lines: Vec<_> = lower.lines().map(str::trim).collect();
    let Some(composer) = lines
        .iter()
        .enumerate().rfind(|(row, line)| **line == "› ask codex to do anything"
            || cursor == Some(*row) && line.starts_with("› ")
                && lines.iter().skip(*row + 1).any(|line| model_footer(line)))
        .map(|(row, _)| row)
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

fn composer_rows(agent: &str, screen: &str, cursor: Option<usize>) -> Option<std::ops::Range<usize>> {
    let cursor = cursor?;
    let lines: Vec<_> = screen.lines().map(str::trim).collect();
    if cursor >= lines.len() { return None; }
    if agent == "codex" {
        let start = lines[..=cursor].iter().rposition(|line| line.starts_with("› "))?;
        let end = lines.iter().enumerate().skip(cursor + 1)
            .find(|(_, line)| model_footer(line))?.0;
        if lines[cursor + 1..end].iter().any(|line| {
            let lower = line.to_lowercase();
            lower.contains("esc to cancel") || lower.contains("enter to select") || lower.contains("enter to submit answer")
        }) { return None; }
        return Some(start..end);
    }
    if !matches!(agent, "antigravity" | "claude") { return None; }
    let frame = |line: &&str| line.chars().count() >= 8 && line.chars().all(|c| c == '─');
    let end = lines.iter().rposition(frame)?;
    let start = lines[..end].iter().rposition(frame)?;
    (cursor > start && cursor < end && lines.get(start + 1).is_some_and(|line| line.starts_with(if agent == "claude" { '❯' } else { '>' }))
        && lines.iter().skip(end + 1).find(|line| !line.is_empty())
            .is_some_and(|line| if agent == "claude" { (line.starts_with('⏵') || line.starts_with('⏸')) && line.to_lowercase().contains("shift+tab") }
                else { line.to_lowercase().starts_with("? for shortcuts") }))
        .then_some(start + 1..end)
}

#[cfg(test)]
#[path = "observation_transition_tests.rs"]
mod transition_tests;

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
