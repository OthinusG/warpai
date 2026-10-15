//! Native-grid observations; Herdr rules and Orca composer predicates, see docs/HERDR-NOTICE.md.
use super::{detect_with_progress, Status};
use crate::readiness::Activity;
use regex::Regex;
use std::{sync::OnceLock, time::Duration};

pub(super) fn canonical_program(program: &str) -> &str {
    match program {
        "agy" => "antigravity",
        "agent" | "cursor-agent" => "cursor",
        "hermes-agent" => "hermes",
        "vibe-acp" => "vibe",
        "dsh-tui" => "dsh",
        "traecn" | "trae-cli" | "traecn-cli" | "traecli" => "trae",
        "qodercli" | "qoder-cli" | "qodercn" | "qoderclicn" => "qoder",
        other => other,
    }
}

/// Unknown chrome is not evidence of completion. Text is never retained or logged.
pub fn activity(program: &str, screen: &str, title: &str, quiet: Duration) -> Option<Activity> {
    activity_at_cursor(program, screen, title, quiet, None)
}

/// The visible native cursor binds typed drafts to a composer, never a quoted prompt or menu.
pub fn activity_at_cursor(program: &str, screen: &str, title: &str, quiet: Duration, cursor: Option<usize>) -> Option<Activity> {
    activity_with_progress(program, screen, title, "", quiet, cursor)
}

pub fn activity_with_progress(program: &str, screen: &str, title: &str, progress: &str,
    quiet: Duration, cursor: Option<usize>) -> Option<Activity> {
    if screen.len() > 128 * 1024 || title.len() > 4096 || progress.len() > 256 {
        return None;
    }
    let agent = canonical_program(program);
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
    let detection = detect_with_progress(agent, chrome.as_deref().unwrap_or(screen), title, progress, None);
    let title_activity = title_state(agent, title);
    if let Some(detection) = detection {
        match detection.status {
            Some(Status::Waiting)
                if !matches!(
                    detection.rule,
                    "input_prompt" | "ready_footer" | "cursor_line"
                ) =>
            {
                return Some(if detection.rule.starts_with("question_") {
                    Activity::WaitingInput
                } else { Activity::WaitingApproval })
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
    if quiet < Duration::from_millis(700) {
        return None;
    }
    // Herdr's known-Agent fallback: a settled screen with no rule is Idle.
    if detection.is_some() {
        Some(Activity::Idle)
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
    // dsh-tui 0.14 uses a whale title without the executable's name.
    if agent == "dsh" && title.starts_with("✦ 🐋 ") {
        return Some(Activity::Idle);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_managed_agent_alias_has_idle_fallback_and_bounded_evidence() {
        let source = include_str!("../../../../app/src/terminal/cli_agent.rs");
        let aliases = Regex::new(r#"CLIAgent::\w+ => &\[([^\n]+)\]"#).unwrap();
        let mut checked = 0;
        for row in aliases.captures_iter(source) {
            for alias in row[1].split(',').filter_map(|part| part.trim().strip_prefix('"').and_then(|part| part.strip_suffix('"'))) {
                assert!(super::super::has_manifest(alias), "missing native rules for {alias}");
                assert_eq!(activity(alias, "Unrecognized current UI", "", Duration::from_secs(2)), Some(Activity::Idle), "{alias}");
                assert_eq!(activity(alias, "Unrecognized current UI", "", Duration::ZERO), None, "input/output must settle: {alias}");
                checked += 1;
            }
        }
        assert!(checked >= 30);
        assert_eq!(activity("custom", "Unrecognized current UI", "", Duration::from_secs(2)), None);
        assert_eq!(activity("codex", &"x".repeat(128 * 1024 + 1), "", Duration::from_secs(2)), None);
    }

    #[test]
    fn herdr_rules_cover_working_blocked_and_return_to_idle_for_priority_clients() {
        let quiet = Duration::from_secs(2);
        let cases = [
            ("codex", "Thinking (2s • esc to interrupt)", "", Activity::Working),
            ("agy", "↑/↓ Navigate · tab Amend · ctrl+g edit/expand command\nesc to cancel", "", Activity::WaitingApproval),
            ("agy", "──────── esc to cancel", "", Activity::Working),
            ("agy", "────────\n>\n────────\n? for shortcuts", "", Activity::Idle),
            ("qoder", "permission required", "◇ Qoder CLI", Activity::WaitingApproval),
            ("qodercn", "⠋ Thinking (esc to cancel, 2s)", "", Activity::Working),
            ("qoderclicn", "Type your message or @path/to/file", "◇ Qoder CLI CN", Activity::Idle),
            ("dsh-tui", "", "⠂ 🐋 会话标题", Activity::Working),
            ("dsh-tui", "", "✦ 🐋 会话标题", Activity::Idle),
            ("dsh-tui", "❯ 1. Allow once\n2. Deny", "✦ 🐋 会话标题", Activity::WaitingApproval),
            ("dsh-tui", "允许（仅本次）\n拒绝\n↑/↓ 选择 · Enter 确认 · Esc 拒绝", "", Activity::WaitingApproval),
            ("dsh-tui", "Enter submit · Ctrl+C cancel batch", "", Activity::WaitingInput),
            ("dsh-tui", "正在等你回答", "", Activity::WaitingInput),
            ("traecli", "Allow once\nDeny", "", Activity::WaitingApproval),
            ("goose", "Allow\nDeny", "", Activity::WaitingApproval),
        ];
        for (program, screen, title, expected) in cases {
            assert_eq!(activity(program, screen, title, quiet), Some(expected), "{program}: {screen}");
        }
        assert_eq!(activity_with_progress("grok", "", "", "4;1;-1", quiet, None), Some(Activity::Working));
        assert_eq!(activity_with_progress("grok", "", "", "4;0", quiet, None), Some(Activity::Idle));
    }
}
