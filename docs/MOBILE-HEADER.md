# Mobile documentation header

## Scope and visual reference

Use the existing bilingual documentation layout, assets, fonts, ink/paper tokens
and desktop geometry at source `2e746b05` as the visual reference. At 390 px the
existing header extends to 605 px in English and 539 px in Chinese. The product
requirement is to keep navigation, language and theme controls accessible without
horizontal page scrolling, including while the mobile sidebar is open.

## Implementation and acceptance

- Reuse the current controls and CSS; no dependencies or replacement navigation.
- Use two header rows on narrow screens and share the resulting height with the
  sticky sidebar offset. Preserve the desktop single-row header.
- Verify English/Chinese, ink/paper, 320/375/390/768/1440 px, sidebar open/closed,
  language destinations and both theme buttons with the real static Jekyll output.
- Header controls must remain inside the viewport, readable and non-overlapping;
  compare desktop geometry against the baseline. Existing tables/code blocks
  retain their scoped scrolling.
- Build Jekyll, check the focused diff, deploy gh-pages and verify live CSS.

## Fidelity ledger

- Layout: baseline 390-px header exceeded the viewport (EN 605 px, ZH 539 px).
  A two-row header at <=700 px now fits at 320/375/390/700 px; the 701/768/1440-px
  header stays one row. The shared height keeps an open drawer below the header.
- Typography: existing font families/weights remain; narrow navigation uses 11 px
  text and nonwrapping labels. Long inline commands wrap within the article;
  preformatted command blocks retain their independent scrolling.
- Palette: existing ink/paper tokens and pressed-button styling are unchanged.
- Spacing: 16-px narrow header padding and 4/8/12-px gaps retain every control;
  navigation/theme hit areas are at least 32 px high. No clipping or overlap.
- Assets and copy: existing logo, icons, labels, language targets and navigation
  remain intact. No replacement assets, hidden controls or new dependencies.
- Interaction: 112 bilingual page/width/theme combinations passed bounds,
  overlap, sidebar-offset and language-target assertions. Real pointer clicks
  open the drawer; outside clicks close it. Both theme buttons work. English
  ink and Chinese paper screenshots were inspected at 320 px.
- Local Jekyll build and focused whitespace review passed. Production source
  8eb59796 deployed successfully. Eight live bilingual 320/390-px theme checks,
  language targets and real-pointer drawer interactions passed on the public site.

The final 320-px check also constrains controls to the header content box, not
merely the viewport. A 4-px second-row gap and 2-px navigation-label side padding
preserve the full 16-px outer padding in English and Chinese; all 112 checks pass.
