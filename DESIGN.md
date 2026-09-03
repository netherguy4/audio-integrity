---
name: Audio Integrity
description: A calm, read-only verification desk for live audio-library evidence.
colors:
  primary: "#2563eb"
  primary-hover: "#1d4ed8"
  primary-soft: "#eaf1ff"
  on-accent: "#ffffff"
  page: "#f7f8fb"
  surface: "#ffffff"
  surface-subtle: "#f1f4f9"
  text: "#172033"
  text-muted: "#687386"
  border: "#dfe4ec"
  success: "#15803d"
  success-soft: "#eaf8ef"
  warning: "#a15c05"
  warning-soft: "#fff6df"
  danger: "#c42c3a"
  danger-soft: "#fff0f1"
  dark-primary: "#6290ff"
  dark-primary-hover: "#79a1ff"
  dark-primary-soft: "#1e315b"
  dark-page: "#10141d"
  dark-surface: "#181e29"
  dark-surface-subtle: "#202735"
  dark-text: "#edf2fa"
  dark-text-muted: "#9ca8bb"
  dark-border: "#2d3747"
  dark-success: "#60cf86"
  dark-success-soft: "#173426"
  dark-warning: "#f2b857"
  dark-warning-soft: "#3a2a13"
  dark-danger: "#ff7180"
  dark-danger-soft: "#452027"
typography:
  display:
    fontFamily: "Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: "1.75rem"
    fontWeight: 700
    lineHeight: 1.5
    letterSpacing: "-0.025em"
  title:
    fontFamily: "Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: "1.12rem"
    fontWeight: 700
    lineHeight: 1.5
    letterSpacing: "-0.01em"
  body:
    fontFamily: "Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: "0.84rem"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: "0.73rem"
    fontWeight: 700
    lineHeight: 1.5
  mono:
    fontFamily: "ui-monospace, SFMono-Regular, Consolas, monospace"
    fontSize: "0.68rem"
    fontWeight: 500
    lineHeight: 1.35
rounded:
  pill: "99px"
  control: "8px"
  field: "9px"
  compact-panel: "12px"
  panel: "14px"
  login: "18px"
spacing:
  xxs: "4px"
  xs: "8px"
  sm: "12px"
  md: "16px"
  lg: "24px"
  xl: "32px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-accent}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "9px 13px"
    height: "38px"
  button-secondary:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "9px 13px"
    height: "38px"
  panel:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    rounded: "{rounded.panel}"
    padding: "24px"
  input:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "8px 10px"
  status-badge:
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "3px 8px"
---

# Design System: Audio Integrity

## Overview

**Creative North Star: "The Live Verification Desk"**

Audio Integrity is a compact operations surface, not a decorative analytics dashboard. Its AudioMuse-informed frame is calm and familiar: neutral working surfaces, restrained blue actions, precise status hues, and persistent navigation. The scan panel remains the visual and functional center of gravity.

The interface is evidence-first. Flat telemetry, byte progress, the current path, validator identity, and the event stream stay visually connected so a long-running job can be understood at a glance. Every authenticated browser observes the same server-owned scan through a session-authenticated WebSocket; a reconnecting client exposes that state and rehydrates authoritative state over HTTP.

Motion is sparse and operational: a running dot pulses, a four-bar meter signals active work, and progress width eases over 400ms. Reduced-motion mode removes animation and smooth scrolling. Light and dark themes preserve hierarchy through purpose-built semantic substitutions rather than inversion.

**Key Characteristics:**

- Flat, border-led operational surfaces with no decorative shadows.
- One restrained blue action voice; green, amber, and red carry explicit textual meaning.
- Live shared telemetry with visible connection and recovery states.
- Dense evidence that remains legible in Russian and English on desktop and phone.
- Integrity and authenticity stay visibly and behaviorally distinct.

## Colors

The palette is cool, quiet, and utilitarian. Light mode places white work surfaces on a pale blue-gray page; dark mode uses blue-charcoal layers with brighter semantic foregrounds.

### Primary

- **Verification Blue** (`primary`, `dark-primary`): primary scan action, active navigation, progress, focus, caret, and live activity.
- **Pressed Verification Blue** (`primary-hover`, `dark-primary-hover`): hover feedback for the primary action only.
- **Blue Evidence Wash** (`primary-soft`, `dark-primary-soft`): active navigation and the halo around a running state.

### Neutral

- **Library Mist** (`page`, `dark-page`): full-page working field.
- **Evidence Sheet** (`surface`, `dark-surface`): panels, controls, sidebar, and table body.
- **Telemetry Shelf** (`surface-subtle`, `dark-surface-subtle`): progress tracks, table headers, neutral badges, and quiet hover states.
- **Ink** (`text`, `dark-text`): primary copy and numeric evidence.
- **Slate Annotation** (`text-muted`, `dark-text-muted`): metadata, labels, inactive navigation, and supporting copy.
- **Hairline** (`border`, `dark-border`): the structural separator that replaces elevation.

### Tertiary

- **Verified Green** (`success`, `dark-success`) and **Verified Wash** (`success-soft`, `dark-success-soft`): healthy integrity, likely-lossless authenticity, and a live WebSocket connection.
- **Review Amber** (`warning`, `dark-warning`) and **Review Wash** (`warning-soft`, `dark-warning-soft`): likely-lossy authenticity, validation errors, reconnecting status, and full-audit caution.
- **Corruption Red** (`danger`, `dark-danger`) and **Corruption Wash** (`danger-soft`, `dark-danger-soft`): corrupt media, failed scans, offline state, and the stop action.

**The Text-With-Color Rule.** Semantic color never stands alone: every dot, count, or badge is paired with a localized status label or adjacent explanation.

**The Separate Verdicts Rule.** Red describes integrity failure. Amber describes uncertainty, heuristic evidence, or a validation problem; “likely lossy” never inherits the red corruption treatment.

## Typography

**Display Font:** Inter with the native UI sans stack as fallback

**Body Font:** Inter with the native UI sans stack as fallback

**Label/Mono Font:** the native monospace stack for paths, validator evidence, timestamps, and build IDs

**Character:** Typography is compact, neutral, and optimized for scanning. Weight and tabular numerals create hierarchy without oversized dashboard headings; monospace is reserved for evidence that benefits from character-level inspection.

### Hierarchy

- **Display** (`typography.display`): page titles, reduced to 1.4rem on phones.
- **Title** (`typography.title`): scan and section headings.
- **Body** (`typography.body`): descriptions and controls, with natural wrapping for RU/EN expansion.
- **Label** (`typography.label`): status, buttons, table headings, and metric captions.
- **Mono** (`typography.mono`): current file, events, paths, validator output, and version identifiers.

**The Evidence Type Rule.** Use monospace for exact machine evidence, never as a decorative brand face.

**The Numeric Stability Rule.** Progress, counters, throughput, ETA, timestamps, and history totals use tabular numerals so live updates do not jitter.

## Layout

Desktop uses a fixed 238px left sidebar and a fluid content column capped at 1600px, with horizontal padding that grows from 24px to 58px. The wide scan panel leads: header and actions, one continuous progress instrument, a three-column live telemetry strip, event log, and five-column outcome strip. Summary cards and evidence tables follow.

At 960px and below, navigation contracts to a 78px icon rail and scan metrics move to three columns. At 680px and below, navigation becomes a fixed 62px bottom rail with three labeled destinations and content gains an 88px bottom safe area. The scan header stacks; the primary action expands; telemetry becomes two columns with connection state spanning the last row; the final “Read” metric spans both columns. Tables retain state, path, and authenticity while hiding lower-priority format and checked-time columns.

The result table may scroll horizontally. Live paths truncate only in the single-line current-file instrument and keep the complete value in the native title; result paths wrap character-by-character and validator details remain available in an expandable disclosure. Theme and language choices persist locally, and layouts allow controls to wrap before evidence is clipped.

**The One Instrument Rule.** Keep progress, current file, throughput, ETA, connection state, events, and counters inside one bordered scan instrument; do not fragment them into floating statistic cards.

## Elevation & Depth

The system is deliberately flat. Panels, sidebar, telemetry cells, tables, and controls have no resting shadows; hierarchy comes from page/surface tonal contrast, one-pixel Hairline borders, padding, and connected grid seams. Focus is the only shadow-like treatment: a three-pixel translucent blue ring that communicates interaction rather than physical elevation.

**The Flat-by-Default Rule.** Do not add card shadows, glass effects, gradients, or nested elevation. Stronger separation uses the established surface tone and Hairline border.

## Shapes

Controls use compact 8px corners, form fields and caution strips use 9px, summary cards and the brand mark use 12px, and primary panels use 14px. The sign-in card alone receives the softer 18px silhouette. Pills and progress tracks use fully rounded 99px ends. Status dots and the tiny live meter are the only repeated micro-shapes.

**The Radius Hierarchy Rule.** Radius expresses containment depth: controls are tighter than panels, and the sign-in card is the single softest container.

## Components

### Buttons

- **Shape:** compact 8px corners, dense padding, and a minimum 38px height.
- **Primary:** Verification Blue with white text; reserved for incremental scan and sign-in.
- **Secondary:** Evidence Sheet with Ink text and Hairline border; full audit and confirmation.
- **Danger / Ghost:** red tonal stop action; transparent muted dismissal.
- **Hover / Focus:** designated blue or Telemetry Shelf hover, plus the shared visible blue focus ring. Disabled actions remain visible at 50% opacity.

### Fields

- **Style:** Hairline border, Evidence Sheet application background, Library Mist sign-in background, and 8–9px corners.
- **Focus:** Verification Blue border plus the shared three-pixel ring.
- **Responsive:** result filters sit beside the heading on wide screens and stack full-width on phones.

### Status badges and dots

- **Integrity:** healthy is green, corrupt is red, and validation error is amber.
- **Authenticity:** likely lossless is green, likely lossy is amber, and unknown/not applicable are neutral.
- **Behavior:** corrupt integrity can block import; likely-lossy authenticity is explicitly heuristic and remains a manual-review signal.
- **Shape:** badges are compact pills; phase dots are 8px circles with a soft semantic halo.

### Scan instrument

- **Container:** one 14px panel with 24px desktop padding and 18px phone padding; lower metrics meet its edges as a connected grid.
- **Progress:** seven-pixel blue bar paired with file count and byte percentage.
- **Current file:** muted label, truncating monospace path, and validator identity; a vertical evidence stack on phones.
- **Live telemetry:** throughput, ETA, and connection use flat Hairline-separated cells. Connection text cycles through localized `connecting`, green `live`, and amber `reconnecting` states.
- **Realtime behavior:** the WebSocket pushes authoritative status to all simultaneous authenticated sessions. On close the UI exposes recovery immediately, reconnects after 1.5 seconds, and uses the red offline banner for failed HTTP refreshes.
- **Event stream:** the latest six events sit in a polite live region. Warnings and errors receive semantic text color without losing the textual message.
- **Motion:** pulse and meter animate only while work is active; reduced-motion mode disables all animation and transition.

### Navigation

- **Desktop:** persistent Evidence Sheet sidebar with icon-and-label links, product lockup, import-policy state, and sign-out below a separator.
- **Active / hover:** blue text and icon on Blue Evidence Wash; Ink on Telemetry Shelf for hover.
- **Tablet / mobile:** a centered 78px icon rail, then a fixed bottom rail with visible icon-and-label destinations.

### Cards and containers

- **Panels:** Evidence Sheet, Hairline border, 14px corners, no shadow.
- **Summary cards:** 12px corners and 19–21px padding; semantic value color always has a muted label.
- **Inline confirmation:** amber tonal strip inside the scan panel, never a modal; it explains cache cost before full audit.

### Results and history

- **Results:** Telemetry Shelf header, Hairline row separators, exact paths, separate integrity/authenticity badges, quiet hover, and native details disclosure.
- **Empty state:** centered outlined success icon, short title, and one next-step sentence.
- **History:** flat rows with status, run metadata, and right-aligned tabular totals; phones retain status and metadata.

## Do's and Don'ts

### Do:

- **Do** make the server-owned scan and WebSocket state obvious in the first viewport.
- **Do** preserve exact paths, validator messages, and timestamps as inspectable evidence.
- **Do** keep integrity and authenticity as separate labeled dimensions.
- **Do** pair semantic color with text and retain visible keyboard focus.
- **Do** verify light/dark, RU/EN expansion, the 960px icon rail, and the 680px bottom-navigation layout.
- **Do** keep actions at least 38px tall and support keyboard input and reduced motion.

### Don't:

- **Don't** turn the surface into disconnected dashboard tiles; live evidence belongs to the scan instrument.
- **Don't** use red for likely-lossy authenticity or imply spectral suspicion is corruption.
- **Don't** hide the only copy of evidence behind hover, color, animation, or a desktop-only column.
- **Don't** add shadows, gradients, glass, oversized display type, or decorative charts.
- **Don't** let a reconnecting or offline client silently appear live.
- **Don't** introduce controls that imply repair, deletion, quarantine, or other mutation of the read-only library.
