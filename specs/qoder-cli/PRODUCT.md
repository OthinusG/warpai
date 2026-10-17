# Qoder CLI Agent Detection

## Goal

Recognize Qoder CLI commands as CLI Agent sessions so Warp Lite shows its existing toolbar, pane management, and rich input.

## Behavior

- Recognize `qoder`, `qodercli`, `qoder-cli`, `qodercn`, `qoderclicn`, `qoder-cn`, and commands with additional arguments.
- Let users add separate Qoder and QoderCN usage accounts using their existing CLI login, without installing an SDK; show available plan, add-on, and organization quotas separately.
- Keep usage polling free of model turns. Automatic polling runs at most once per account every five minutes; manual refresh bypasses the interval.
- Preserve environment-assignment and alias resolution before detection.
- Display `Qoder` and submit rich input inline.

## Boundaries

- No structured status listener, notification plugin, Warp AI, Cloud, login, collaboration, or telemetry upload.

## Acceptance Criteria

- Positive detection cases and alias/env-var cases have regression coverage.
- `qoder` and `qodercn` are classified as shell command keywords.
- The customization is reapplied by the existing upstream-sync restoration script.
