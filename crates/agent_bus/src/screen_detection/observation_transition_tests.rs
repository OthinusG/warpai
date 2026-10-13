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
