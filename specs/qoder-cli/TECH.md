# Qoder CLI Agent Detection — Technical Plan

## Qoder/QoderCN communication and usage update (2026-10-17)

### Acceptance

- Treat `qoder-cn` as an additional launcher alias for the distinct QoderCN CLI; keep `qoder` and QoderCN separate identities.
- A QoderCN TUI launched through `qoder-cn` receives Warpai's existing native MCP communication setup and screen/activity monitoring.
- Add separate Qoder and QoderCN current-CLI-login usage accounts in the existing Data usage settings and footer.
- Query plan, add-on, and organization quota from each CLI's `get_usage_info` control request. Show available quota buckets as independent usage bars; do not mix credits between products.
- Use the CLI's existing login state and JSONL control channel. Ship no npm/Python SDK or runtime, and read no credential files or secret values in Warpai.
- Usage polling sends only `initialize` and `get_usage_info` control requests; never submit a prompt or start a model turn.
- Return an explicit unavailable/version-unsupported state when CLI login, control capability, or quota response is missing. Never turn missing quota into zero.
- Limit automatic Qoder and QoderCN usage queries to one per account every five minutes. Explicit manual refresh bypasses this limit.

### Scope and verification

- Update command aliases in launch detection, discovery, session routing, terminal observation, and CLI fixtures.
- Add a bounded CLI control-channel query and quota response parser for both Qoder products; wire it into the existing provider account/settings/footer flow.
- Verify alias coverage, response parsing (including missing/invalid buckets), CLI protocol request/response, and Rust build where feasible.
- Probe confirmed QoderCN CLI 1.1.73 returns plan and organization buckets through this JSONL channel; the response parser passed checks against that live response and missing/malformed fixtures. No quota amounts or identity fields were retained.
- Known compatibility boundary: the `get_usage_info` control request follows the Qoder SDK bridge protocol. Failed or unsupported requests must remain visible as unavailable; automatic polling is limited to one query per account every five minutes.

## Implementation

- Add `CLIAgent::Qoder` and exhaustive fallbacks.
- Accept `qoder`, `qodercli`, `qoder-cli`, `qodercn`, `qoderclicn`, and `qoder-cn` command prefixes with arguments.
- Use the existing no-listener, no-plugin, and inline-submit paths.
- Add `qoder` and `qodercn` to the input classifier shell keyword set.
- Maintain the implementation directly in repository source; no patch replay or restoration script is required.
- Bundle a transparent RGBA Qoder PNG and render it as an original-color image in CLI Agent tab/status circles instead of passing it through the monochrome icon tint path.

## Verification

- Run focused CLI Agent detection tests.
- Validate native integration tests, formatting and diff hygiene.
