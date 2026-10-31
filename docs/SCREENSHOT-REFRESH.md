# Workspace screenshot refresh

## Scope and source

Use the owner-provided `warpai-window-clean-original-resolution.png` unchanged
for light previews and as the source for the approved derived dark preview.
Both documentation overviews and both homepage download sections follow the
website theme; main-branch READMEs retain the original. Preserve typography, colors,
download links, and the interactive workbench elsewhere on the homepage.

## Implementation and acceptance

### Dark variant (owner correction, 2026-10-31)

- Keep the approved full inversion except for the original macOS traffic lights,
  Spotify/Monitor/Codex sidebar icons (including the Codex status badge), and
  album artwork. Restore these regions directly from the original capture.
- Preserve the 3836 × 2114 canvas and verify every pixel outside those regions
  matches the approved inverted preview.
- Select the corrected dark asset with the site's existing ink/paper theme in
  both homepages and both documentation overviews. Keep the light assets intact.
- Validate asset selection on initial load and manual theme changes, desktop and
  mobile, plus the existing blur reveal/reduced-motion behavior.

- Replace existing hero screenshot assets; synchronize English/Chinese captions
  with the visible vertical tabs, Spotify/btop split terminals, and right Agent panel.
- Keep the entire screenshot visible in READMEs and documentation.
- Add a decorative enlarged crop to the right of the homepage download copy,
  fading at its edges and revealing from blur when scrolled into view.
- Use existing CSS tokens and native CSS/IntersectionObserver; add no dependencies.
- Keep download controls unobscured; stack the image below the copy on mobile.
- Show a static image without JavaScript or with reduced motion enabled.

## Verification

Build Jekyll with `/warpai` baseurl. Check both locales, light/dark themes,
desktop/mobile overflow, image loading, download controls, reveal behavior,
reduced motion, and matching image checksums. Record screenshot QA below.

## Risks

The homepage intentionally crops the decorative image. The complete reference
remains available in both documentation overviews. Main and gh-pages assets
must stay synchronized when the screenshot changes again.

## Visual fidelity ledger

| Point | Source / expected | Rendered result |
| --- | --- | --- |
| Asset | Owner's 3836 × 2114 PNG | All original light copies match the source; the dark copy retains the same canvas. |
| Layout | Existing download copy and controls | Text stays left of the decorative crop on desktop; mobile stacks the crop below all controls. |
| Typography | Existing Archivo/Inter/monospace styles | Existing fonts retained; balanced heading wrapping keeps Chinese phrases together, and commands wrap within the download section. |
| Palette | Existing ink/paper theme tokens | Edge masks blend into each theme; ink uses the approved selective inversion, paper uses the original. |
| Spacing | Existing section gutters | Gutters and download control spacing retained at desktop/mobile sizes. |
| Crop | Full image in documentation; enlarged detail in CTA | Overview retains the full aspect ratio; CTA intentionally emphasizes btop and the right Agent panel. |
| Motion | Blur-to-clear reveal, static reduced-motion fallback | One reveal when entering the viewport; default image remains visible without JavaScript. |

## Verification results (2026-10-31)

- Jekyll 3.9.5 production-baseurl build passed. The local Ruby environment
  requires selecting installed `mercenary ~> 0.3.3` before loading Jekyll.
- All local link and asset targets across 34 generated HTML pages passed.
- Headless Edge/Playwright checks passed for both locales at 1440px and 390px:
  image dimensions, download-control hit testing, download-section text bounds,
  one-time reveal, reduced motion and no-JavaScript fallback.
- Inspected desktop/mobile CTA screenshots in ink/paper and both mobile
  Overview figures. Native screenshot copies are byte-identical to the source.
- JavaScript syntax and `git diff --check` passed. No Rust sources changed.

### Dark correction verification

- Native canvas restored 64,892 pixels in the approved regions. Checks found
  zero changed channels outside those regions, zero mismatches inside them,
  and zero changed channels after encoding/decoding the final PNG.
- Restore masks in source pixels: traffic-light circles `(52,55,12)`,
  `(92,55,12)`, `(132,55,12)`; Spotify `(74,238,24)`; Codex `(70,458,20)`
  and status badge `(87,475,16)` (center x/y/radius); Monitor rounded rectangle
  `(50,313)-(98,361)`, radius 4; album rectangle `(530,140)-(770,380)`.
- Final dark PNG SHA-256:
  `0567c54264b0a330483e7e4b582781f49ea75eba24e6f66fa47f404751967b84`.
- Browser checks passed for both homepages and Overviews at 1440px/390px,
  switching ink → paper → ink, image dimensions, download controls, text
  bounds and the existing reveal. Inspected dark and light screenshots.
- Jekyll build, all 34 pages' local link/asset targets and diff checks passed.
