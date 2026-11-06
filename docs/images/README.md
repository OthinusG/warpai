# README screenshot provenance

**English** | [简体中文](README.zh-CN.md) · [Product introduction](../../README.md)

The README brand cover is a generated marketing image, documented separately in
[Social preview provenance and upload instructions](SOCIAL-PREVIEW.md).
The following provenance applies only to the native application screenshots.

## Workspace heroes (2026-11-06)

[Light](warpai-hero-showcase.png) and [dark](warpai-hero-showcase-dark.png)
are owner-approved local edits of native macOS screenshots captured on
2026-11-06 at 12:02:55 and 12:03:31. Both retain the 4032 × 2304 canvas.
Only the specified sidebar icon/status/path fields, collaboration panel, and
process User column were edited; pixels outside these regions are unchanged.
These are edited product illustrations, not unmodified runtime evidence.
No whole-image generation, resampling, or theme inversion was used.

Both bilingual READMEs and website Overviews use the matching light/dark pair;
the homepage download previews retain their decorative CSS crop and fade.
Copies in main and gh-pages are byte-identical to the approved PNGs.

- Light SHA-256: `60995938b1b1186ead966c8465fda8db18718ec743ca4d276300e197170e480d`.
- Dark SHA-256: `bfc8ee583d57b5bacbf54a20bd4042f6e7a96c81964dbb89f2d509391fa6e8d9`.

The CI provenance below applies only to the older workspace/SSH captures.

## Native review captures

These are unmodified PNG captures from the native macOS application, not mockups
or generated product images. Sample projects, agents and tasks were exercised by
the native review driver; they are not customer projects or authenticated vendor
model sessions.

- Tabs: native macOS vertical tab panel enabled.
- Theme: default **Claude Warm Light**, rendered by the native app.
- Source: `7ebbd3ee45b4652a2e4038f9fb9b16b971ec5fb8`.
- [GitHub desktop run 37349315123](https://github.com/OthinusG/warpai/actions/runs/37349315123).
- Artifact: `Warpai-collaboration-native-captures-macOS`.
- Capture set: `capture-90608/collaboration-static/2026-10-05T18-00-33`.

| Documentation image | Original capture | Shows |
| --- | --- | --- |
| [warpai-workspace.png](warpai-workspace.png) | `live-evidence-file-open.png` | Source editor and a live task detail panel. |
| [warpai-ssh-project.png](warpai-ssh-project.png) | `live-ssh-task.png` | Remote project, agent and task projections in the live SSH panel. |

## Visual review

Compared the raw captures with [the native UI contract](../../specs/agent-communication-v2/UI-CONSISTENCY.md).
The earlier `5883221` frames were rejected for faint toolbar icons; the corrected
source uses full-red opacity masks and native theme colors. The downloaded
macOS archive passed GitHub SHA-256 and ZIP CRC checks, all 163 PNGs decoded, and
its source-matched diagnostic reports exit code 0 with no failed native actions.

| Point | Expected / reviewed result |
| --- | --- |
| Palette | Native Claude Warm Light workspace and SSH frames; no recoloring. |
| Layout | Visible macOS vertical tabs beside the source/terminal and task panel; native scrolling retains long content. |
| Toolbar assets | Clear terminal Settings glyph and four-row Tools glyph, without the supplied enclosing ring/frame; Light/Dark and both tab layouts reviewed. |
| Typography | Primary semibold panel title remains distinct from section headings and secondary guidance. |
| Actions and spacing | Secondary SSH entry, adjacent refresh and wrapping navigation in the 320px/125% panel; no overlapping primary controls. |
| Settings | Agents sub-header, aligned native setting rows and readable descriptions in Claude Warm at 100%/125%, plus Light/Dark at 125%. |

Both documentation PNGs are byte-identical to their original captures at
1440 × 684 pixels. The screenshot-only review build is not a published installer.

These captures include the current branding source. Workspace images do not
show Dock or installer artwork; those are verified by native packaging checks.
The capture driver retains 125% UI zoom for these live checkpoints.

Final capture archive SHA-256:

- macOS: `2dcf99107677b2d1d15ab93ee91373135ee11c550e5162f18c35d64f2c0c432a`.
- Windows: `65f9781f9ae58c8253b7a45667ffd8a43d23eca572c7fc6fed89caf9de30bb20`.

The Windows archive also passed CRC, decoding of all 163 frames and native
action diagnostics. Warm/dark PowerShell banners and 125% settings were reviewed
on both platforms before the release tag was created.
