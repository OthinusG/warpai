use super::*;

#[test]
fn native_drafts_and_slash_commands_recover_without_hiding_busy_or_dialogs() {
    let quiet = Duration::from_secs(2);
    for draft in ["ordinary draft", "working on this later", "approve this change", "/status", "几个字 👩‍🔬"] {
        let codex = format!("OpenAI Codex\n› {draft}\n\ngpt-5.6 · ~/project");
        let agy = format!("Antigravity CLI\n────────────\n> {draft}\n────────────\n? for shortcuts");
        for (agent, screen, cursor) in [("codex", codex, 1), ("agy", agy, 2)] {
            assert_eq!(activity_at_cursor(agent, &screen, "", quiet, Some(cursor)), Some(Activity::Idle), "{agent}: native draft is not a task");
            assert_eq!(activity_at_cursor(agent, &screen, "", Duration::ZERO, Some(cursor)), None);
            assert_eq!(activity_at_cursor(agent, &screen, &format!("⠋ {agent}"), quiet, Some(cursor)), Some(Activity::Working));
        }
    }
    let multiline = "OpenAI Codex\n› first line\n  second line\n\ngpt-5.6 · ~/project";
    assert_eq!(activity_at_cursor("codex", multiline, "", quiet, Some(2)), Some(Activity::Idle));
    assert_ne!(activity_at_cursor("codex", multiline, "", quiet, Some(0)), Some(Activity::Idle), "a quoted prompt cannot own the cursor");
    for command in ["/status", "/help", "/mcp", "/model", "/warnings"] {
        let returned = format!("OpenAI Codex\n{command}\nSession configuration\nWarning: previous request failed\n› Ask Codex to do anything");
        assert_eq!(activity("codex", &returned, "", quiet), Some(Activity::Idle));
        let busy = returned.replace("› Ask", "• Thinking (esc to interrupt)\n› Ask");
        assert_eq!(activity("codex", &busy, "", quiet), Some(Activity::Working));
    }
    let palette = "OpenAI Codex\n› /model\nSelect a model\nEnter to select · Esc to cancel\ngpt-5.6 · ~/project";
    assert_ne!(activity_at_cursor("codex", palette, "", quiet, Some(1)), Some(Activity::Idle));
    let permission = "OpenAI Codex\nRun command?\n› 1. Allow once\n  2. Deny\nEnter to select · Esc to cancel";
    assert_eq!(activity_at_cursor("codex", permission, "", quiet, Some(2)), Some(Activity::WaitingApproval));
    let agy_palette = "────────────\n> /model\n────────────\nEsc to cancel";
    assert_ne!(activity_at_cursor("agy", agy_palette, "", quiet, Some(1)), Some(Activity::Idle));
}

#[test]
fn live_composer_supersedes_transcript_warnings_and_slash_command_history() {
    let history = "Old output: approve changes (y/n)\n⠋ working on the previous task\nERROR: previous connection failed\n/status\nSession status";
    for model in ["gpt-5.6", "o3", "o4-mini"] {
        let screen = format!("OpenAI Codex\n{history}\n› unsent draft\n{model} · ~/project");
        assert_eq!(activity_at_cursor("codex", &screen, "", Duration::from_secs(2), Some(6)), Some(Activity::Idle));
        let busy = format!("OpenAI Codex\n{history}\n• Thinking (esc to interrupt)\n› draft\n{model} · ~/project");
        assert_eq!(activity_at_cursor("codex", &busy, "", Duration::from_secs(2), Some(7)), Some(Activity::Working));
    }
    let agy = format!("Antigravity CLI\n{history}\n────────────\n> unsent draft\n────────────\n? for shortcuts");
    assert_eq!(activity_at_cursor("agy", &agy, "", Duration::from_secs(2), Some(7)), Some(Activity::Idle));
}

#[test]
fn claude_draft_box_and_ascii_titles_preserve_busy_and_permission_guards() {
    let quiet = Duration::from_secs(2);
    let screen = "Previous output: approve or allow (y/n)\n────────────\n❯ approve this draft later\n────────────\n⏵⏵ accept edits on (shift+tab to cycle)";
    assert_eq!(activity_at_cursor("claude", screen, "* Session topic", quiet, Some(2)), Some(Activity::Idle));
    assert_eq!(activity_at_cursor("claude", screen, ". Session topic", quiet, Some(2)), Some(Activity::Working));
    assert_ne!(activity_at_cursor("claude", screen, "", quiet, None), Some(Activity::Idle));
    let permission = "Would you like to proceed?\n────────────\n❯ 1. Allow once\n────────────\nEnter to select · Esc to cancel";
    assert_eq!(activity_at_cursor("claude", permission, "* Session topic", quiet, Some(2)), Some(Activity::WaitingApproval));
}
