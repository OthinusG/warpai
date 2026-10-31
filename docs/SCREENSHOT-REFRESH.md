# Workspace screenshot refresh

## Scope and source

Use the owner-provided `warpai-window-clean-original-resolution.png` unchanged
as the visual source for both main-branch READMEs, both documentation overviews,
and both homepage download sections. Preserve existing typography, colors,
download links, and the interactive workbench elsewhere on the homepage.

## Implementation and acceptance

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
| Asset | Owner's 3836 × 2114 PNG | All three repository copies have the same SHA-256 as the source. |
| Layout | Existing download copy and controls | Text stays left of the decorative crop on desktop; mobile stacks the crop below all controls. |
| Typography | Existing Archivo/Inter/monospace styles | Existing fonts retained; balanced heading wrapping keeps Chinese phrases together, and commands wrap within the download section. |
| Palette | Existing ink/paper theme tokens | Edge masks blend into each theme without recoloring the screenshot. |
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
