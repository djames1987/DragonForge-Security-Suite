# DragonForge Security Suite UI Style Guide

## Purpose

This document defines the shared visual baseline for DragonForge desktop applications. Phase 3 Security Center and the existing Password Manager are the reference implementations.

## Core palette

```css
--bg: #090b10;
--surface: #11141b;
--surface-2: #171b24;
--surface-3: #202632;
--border: rgba(255, 255, 255, 0.08);
--border-strong: rgba(255, 255, 255, 0.14);
--text: #f6f7fb;
--muted: #8d96a8;
--muted-2: #667085;
--accent: #ff8a2b;
--accent-2: #ffb45e;
--accent-soft: rgba(255, 138, 43, 0.12);
--success: #57d39b;
--danger: #ff6577;
```

Orange is the DragonForge brand accent. It is not a warning color.

## Typography

Preferred stack:

```css
Inter, ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif
```

Do not distribute font files with the repository unless licensing is explicitly reviewed.

Use system fallbacks when Inter is unavailable.

## Shape language

- primary panels: 12–16 px radius;
- controls: 8–12 px radius;
- brand tile: approximately 12–14 px radius;
- pills/status chips: fully rounded;
- shadows: dark and restrained;
- borders: subtle neutral white alpha.

## Brand mark

The compact application mark uses:

- orange gradient from approximately `#ffab4d` to `#f36b15`;
- dark DF lettering;
- modest orange glow/shadow.

Full application logos live under:

`assets/branding/logos/`

Product packaging can derive OS-specific square icons separately. Do not stretch horizontal branding into a low-quality application icon.

## Buttons

Primary actions:

- orange gradient;
- dark foreground;
- strong readable label;
- one primary action per focused area where practical.

Secondary actions:

- dark surface;
- subtle border;
- white/neutral text.

Destructive actions:

- red only when the action is actually destructive;
- confirm high-impact destructive actions.

## Status language

Preferred labels:

- Active
- Integrated
- Protected
- Attention
- Action Required
- Planned
- Unavailable

Do not claim:

- 100% secure
- perfectly protected
- guaranteed safe

Do not use red simply to make the interface look more security-oriented.

## Security Center layout

Security Center establishes the suite dashboard pattern:

- persistent left navigation;
- sticky top header;
- overview health card;
- component cards;
- activity feed;
- settings page;
- explicit distinction between current and future capabilities.

## Accessibility

- maintain WCAG AA contrast where practical;
- keep visible keyboard focus;
- do not encode status only by color;
- controls require readable text/labels;
- respect reduced-motion preferences when substantial animation is introduced.

## Motion

Suggested timing:

- micro-interactions: 120–180 ms;
- panel/view transitions: 200–240 ms;
- avoid decorative motion in security warnings or confirmation flows.

## Security UX

- show privilege/elevation requirements before actions;
- do not hide destructive consequences;
- avoid fake alarm states for planned/unavailable products;
- clearly distinguish local-only state from server/cloud state;
- never display secrets in logs, toast messages, event summaries, or error details.
