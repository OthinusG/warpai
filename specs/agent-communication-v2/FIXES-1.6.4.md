# 1.6.4 usage and readiness repairs

## Scope and acceptance

- Antigravity shows only Gemini and shared Claude/GPT quotas. Each pool shows
  its five-hour window when detection is enabled, otherwise its weekly window.
  Other providers with five-hour windows use the same exclusive setting;
  sources without that window retain their reported quota/balance data.
  Preserve reported values, reset times and the existing account setting.
- Bare interactive launches, including Codex permission/model options, become
  idle at native MCP discovery. Startup menu acknowledgements and draft edits
  must not consume that lease. Initial prompts, real submissions, approvals,
  cancellation and replaced runs still prevent automatic execution.
- Sidebar and collaboration panel use the same observed lifecycle. Idle/startup
  must not be represented as running work or generate completion notifications.
- App-owned composer drafts and attachments remain in memory while queued peer
  work is submitted independently through the existing PTY pipeline. Automatic
  submissions must not trigger composer auto-dismiss/clear. Unknown native PTY
  edits remain guarded because the editor cannot be reconstructed safely.
- Preserve run/generation fences, delayed Enter cancellation, private draft data,
  task receipts and MCP acknowledgement semantics. No personal configuration edits.

## Plan and validation

1. Reuse provider parsing/cache and centralize visible window selection; add
   representative five-bucket tests for both setting values.
2. Repair shared startup evidence and Codex argument classification; regress
   startup acknowledgements, nonempty submissions and every managed program.
3. Project broker activity into the existing session model without feeding
   presentation updates back into lifecycle evidence. Preserve native hooks.
4. Permit app-owned drafts during wake and preserve them in the shared submit
   pipeline; test saved/open drafts, attachments, approvals and stale Enter.
5. Run local Rustfmt parsing, diff and locale checks. Run focused Rust tests,
   both desktop builds and native capture on GitHub per project verification rules.

Windows WSL manual testing and public release packaging are outside this fix.
