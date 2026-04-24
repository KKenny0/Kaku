---
version: alpha
name: Kaku
description: >
  Terminal emulator for macOS. Light-only warm parchment design with deep green
  accent. Terminal blocks stay dark for semantic correctness.

colors:
  bg:            "#f5f4ed"
  bg-elevated:   "#faf9f5"
  bg-card:       "#ffffff"
  border:        "#e8e5da"
  border-strong: "#d5d1c4"
  primary:       "#141413"
  secondary:     "#5e5d59"
  muted:         "#87867f"
  accent:        "#0d7d4d"
  accent-dim:    "#0a6640"
  error:         "#c93c3c"
  warning:       "#b8860b"
  terminal-bg:      "#0e0e0e"
  terminal-bar:     "#1a1a1a"
  terminal-border:  "#2a2a2a"

typography:
  display:
    fontFamily:    JetBrains Mono
    fontSize:      3rem
    fontWeight:    700
    lineHeight:    1.1
    letterSpacing: -0.02em
  display-mobile:
    fontFamily:    JetBrains Mono
    fontSize:      2rem
    fontWeight:    700
    lineHeight:    1.1
    letterSpacing: -0.02em
  body-md:
    fontFamily: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif
    fontSize:   1rem
    fontWeight: 400
    lineHeight: 1.6
  body-sm:
    fontFamily: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif
    fontSize:   0.875rem
    fontWeight: 400
    lineHeight: 1.5
  code:
    fontFamily: JetBrains Mono, ui-monospace, "SF Mono", Menlo, Consolas, monospace
    fontSize:   0.8125rem
    fontWeight: 400
    lineHeight: 1.6
  label:
    fontFamily:    JetBrains Mono, ui-monospace, "SF Mono", Menlo, Consolas, monospace
    fontSize:      0.6875rem
    fontWeight:    400
    lineHeight:    1
    letterSpacing: 0.04em

rounded:
  sm: 3px
  md: 6px
  lg: 10px

spacing:
  1:  4px
  2:  8px
  3:  12px
  4:  16px
  6:  24px
  8:  32px
  12: 48px
  16: 64px

components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor:       "{colors.bg}"
    rounded:         "{rounded.md}"
    padding:         12px 20px
    typography:      code
  button-primary-hover:
    backgroundColor: "{colors.accent-dim}"
  button-ghost:
    backgroundColor: transparent
    textColor:       "{colors.primary}"
    rounded:         "{rounded.md}"
    padding:         11px 20px
  button-ghost-hover:
    textColor: "{colors.accent}"
  nav:
    backgroundColor: "{colors.bg-elevated}"
    textColor:       "{colors.secondary}"
    height:          52px
    padding:         0 24px
  card:
    backgroundColor: "{colors.bg-card}"
    rounded:         "{rounded.lg}"
    padding:         24px
  card-hover:
    borderColor: "{colors.border-strong}"
  divider:
    backgroundColor: "{colors.border}"
    height:          1px
  divider-strong:
    backgroundColor: "{colors.border-strong}"
    height:          1px
  hint:
    textColor: "{colors.muted}"
    typography: label
  alert-error:
    backgroundColor: "rgba(201,60,60,0.08)"
    textColor:       "{colors.error}"
    rounded:         "{rounded.md}"
    padding:         12px 16px
  alert-warning:
    backgroundColor: "rgba(184,134,11,0.08)"
    textColor:       "{colors.warning}"
    rounded:         "{rounded.md}"
    padding:         12px 16px
  terminal:
    backgroundColor: "{colors.terminal-bg}"
    textColor:       "#00cc7a"
    rounded:         "{rounded.md}"
    padding:         16px
  badge:
    backgroundColor: "rgba(13,125,77,0.08)"
    textColor:       "{colors.accent}"
    rounded:         "{rounded.sm}"
    padding:         3px 8px
  focus-ring:
    textColor: "{colors.accent}"
    rounded:   "{rounded.sm}"
---

## Overview

Kaku is a GPU-accelerated terminal emulator for macOS with built-in AI. The visual
language is **warm terminal**: parchment surfaces with a deep green accent that
connects the brand to terminal output. Terminal code blocks remain dark, creating
a signature contrast against the warm light page.

The tone is terse and technical. Copy is written for developers who already know
what a terminal is. Decorative elements are removed in favour of motion that
demonstrates the product (animated terminal replays, tab switching, copy feedback).

The design signature: **dark terminal blocks on a warm parchment page**, showing
exactly what the product does while feeling approachable and premium.

## Colors

The palette is deliberately minimal. One warm background family, one accent, two
semantic states, plus a dedicated dark surface for terminal blocks.

**Background family** -- warm parchment surfaces create a layered reading
experience:

| Token | Value | Role |
|-------|-------|------|
| `bg` | `#f5f4ed` | Page background (parchment) |
| `bg-elevated` | `#faf9f5` | Nav, footer, elevated chrome (ivory) |
| `bg-card` | `#ffffff` | Cards, panels (white, subtle lift from parchment) |

Cards are white on parchment. The nav and footer use ivory to float above the
page without competing with card surfaces.

**Borders** -- warm sand tones, never cool gray:

| Token | Value | Role |
|-------|-------|------|
| `border` | `#e8e5da` | Default dividers, card outlines |
| `border-strong` | `#d5d1c4` | Focused, hovered, or active states |

**Text** -- warm near-black, three weights:

| Token | Value | Role |
|-------|-------|------|
| `primary` | `#141413` | Headings, primary copy |
| `secondary` | `#5e5d59` | Nav links, metadata, labels |
| `muted` | `#87867f` | Hints, placeholders, de-emphasised copy |

**Accent** -- deep terminal green. Use it for interactive elements, highlights,
and links. Never use it decoratively; it must always signal "actionable".

| Token | Value | Role |
|-------|-------|------|
| `accent` | `#0d7d4d` | CTAs, links, focus rings |
| `accent-dim` | `#0a6640` | Hover state of accent elements |

**Terminal surface** -- dedicated dark tokens for terminal blocks. These blocks
represent the actual terminal and stay dark regardless of page theme.

| Token | Value | Role |
|-------|-------|------|
| `terminal-bg` | `#0e0e0e` | Terminal window background |
| `terminal-bar` | `#1a1a1a` | Terminal title bar |
| `terminal-border` | `#2a2a2a` | Terminal window border |

Inside terminal blocks, text and accent colors are overridden via the
`.kk-dark-surface` utility class to use light-on-dark values (e.g. `#00cc7a`
for accent, `#e8e8e8` for text).

**Semantic** -- error and warning only:

| Token | Value | Role |
|-------|-------|------|
| `error` | `#c93c3c` | Destructive actions, error states |
| `warning` | `#b8860b` | Non-blocking warnings |

Do not introduce additional colours. If a new state requires a colour, use opacity
on an existing token (e.g. `rgba(13,125,77,0.08)` for the badge background).

## Typography

Two families, one job each.

**JetBrains Mono** is the display font. All headings and hero copy use it. The
choice is intentional: Kaku renders monospace text; the marketing site should
feel continuous with what the app actually produces.

**System UI** (`-apple-system`, `BlinkMacSystemFont`, `Segoe UI`) handles all
body prose. Switching to monospace for long-form copy would hurt readability.
System UI keeps the reading experience fast and native.

Scale in use:

| Token | Size | Family | Weight | Use |
|-------|------|--------|--------|-----|
| `display` | 3rem / 48px | JetBrains Mono | 700 | H1, hero headline |
| `display-mobile` | 2rem / 32px | JetBrains Mono | 700 | H1 at <= 720px |
| `body-md` | 1rem / 16px | System UI | 400 | Section intros, card descriptions |
| `body-sm` | 0.875rem / 14px | System UI | 400 | Feature copy, FAQ answers |
| `code` | 0.8125rem / 13px | JetBrains Mono | 400 | Inline code, CTA labels, nav links |
| `label` | 0.6875rem / 11px | JetBrains Mono | 400 | Badges, stat labels |

Letter-spacing on `display`: `-0.02em`. Tight tracking on large monospace type
prevents the letterforms from reading as too mechanical at heading scale.

## Layout

Two container widths:

| Name | Max-width | Use |
|------|-----------|-----|
| Narrow (`kk-container`) | 720px | Text-heavy sections: hero prose, FAQ, QuickStart |
| Wide (`kk-container-wide`) | 1024px | Grid sections: FeatureGrid, ScreenshotGallery, Download methods |

All containers: `margin: 0 auto`, `padding: 0 24px`.

**Section rhythm**: sections are separated by `48px` vertical padding. Within a
section, the heading-to-content gap is `24px`. Within a card grid, gap is `16px`.

**Grid defaults**: FeatureGrid and Download methods use a 3-column grid on desktop
(`min-width: 721px`), collapsing to 2-column at <= 720px, 1-column at <= 480px.
ScreenshotGallery uses 2-column, collapsing to 1-column at <= 720px.

The nav is sticky at `top: 0`, `height: 52px`, with a `border-bottom: 1px solid
{border}` and a subtle `box-shadow: 0 1px 3px rgba(0,0,0,0.04)`.

## Elevation & Depth

Kaku uses two depth models:

**Page surfaces** use upward depth: parchment (base) -> white cards (lifted).
Cards use `border: 1px solid {border}` to define their boundary. On hover,
`border-color` advances to `{border-strong}`.

**Terminal blocks** use downward depth: they drop below the page background into
near-black, signaling "this is the actual terminal". The `.kk-dark-surface`
utility class resets all color tokens inside the block to light-on-dark values.

Focus rings use `outline: 2px solid {accent}` at `outline-offset: 2px`. This is
the only place where accent appears as a border treatment.

## Shapes

Radius is small and consistent with a developer tool aesthetic.

| Token | Value | Use |
|-------|-------|-----|
| `sm` | 3px | Badges, focus rings, inline code |
| `md` | 6px | Buttons, input fields |
| `lg` | 10px | Cards, modals, terminal windows |

Do not mix radius values within a single component family. All buttons use `md`.
All cards use `lg`. Never use `border-radius: 50%` except for the avatar in
the WhyKaku section.

## Components

### Button -- Primary

Background: `{colors.accent}` / Text: `{colors.bg}` / Radius: `{rounded.md}`

Padding `12px 20px`. Typography: `code` (JetBrains Mono 13px). On hover,
background transitions to `{colors.accent-dim}`. On press: `scale(0.97)`.

### Button -- Ghost

Background: `transparent` / Border: `1px solid {colors.border-strong}` /
Text: `{colors.primary}` / Radius: `{rounded.md}`

Padding `11px 20px` (one pixel less vertical than primary to account for the
border). On hover, `text-color` steps to `{colors.accent}`. Used for secondary
CTAs (e.g. "Read Docs", "View Changelog").

### Nav

Height `52px`, background `{colors.bg-elevated}`, bottom border `{colors.border}`,
subtle shadow `0 1px 3px rgba(0,0,0,0.04)`.
Brand uses `font-weight: 700`, `font-size: 16px`, system sans. Nav links use
`code` typography in `{colors.secondary}`, transitioning to `{colors.accent}`
on hover. Language switcher and GitHub link sit at the right end.

### Card

Background `{colors.bg-card}`, radius `{rounded.lg}`, padding `24px`, border
`1px solid {colors.border}`. Cards are used in FeatureGrid and WhyKaku. They
do not scroll or clip overflow.

### Terminal

Background `{colors.terminal-bg}`, radius `{rounded.md}`, padding `16px`.
Add `kk-dark-surface` class to the container. Text uses `code` typography.
The "bar" uses three macOS traffic-light dots. Cursor is a blinking character
in accent color.

### Badge / Tag

Background `rgba(13,125,77,0.08)`, text `{colors.accent}`, radius `{rounded.sm}`,
padding `3px 8px`. Label typography.

## Do's and Don'ts

**Do:**
- Use `{colors.accent}` exclusively for actionable elements.
- Use JetBrains Mono for all headings and short labels.
- Keep terminal blocks dark with `.kk-dark-surface` class.
- Use warm grays throughout; never use cool blue-gray.
- Use parchment (`#f5f4ed`) as page background, never pure white.
- Apply `active:scale(0.97)` to all buttons and interactive cards.
- Respect `prefers-reduced-motion`: disable CSS animations, keep layout transitions.

**Don't:**
- Don't add a second accent colour. If you need a variant, use `accent-dim` or opacity.
- Don't apply heavy `box-shadow` to cards. Use border and background steps.
- Don't use Inter, Roboto, or any geometric sans for headings.
- Don't use `border-radius > 10px` on any surface.
- Don't use `color.accent` as a section background fill.
- Don't introduce a third container width. Use narrow for text, wide for grids.
- Don't add dark-mode rules. The site is light-only.
