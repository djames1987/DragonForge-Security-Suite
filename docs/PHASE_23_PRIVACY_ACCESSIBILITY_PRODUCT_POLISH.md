# Phase 23 — Privacy, Accessibility & Product Polish

**Status: Verified Complete**

Phase 23 makes privacy, accessibility, localization readiness, and consistent product presentation explicit suite-wide contracts rather than per-application conventions.

## Delivered

- shared Phase 23 accessibility runtime in all nine desktop UIs;
- skip-to-main-content support;
- visible `:focus-visible` keyboard focus treatment;
- arrow/Home/End keyboard navigation for suite navigation surfaces;
- ARIA current-page tracking for active navigation;
- polite/assertive screen-reader live-region support;
- automatic status/result/toast live-region upgrades;
- reusable `DragonForgeUX` helpers for announcements, busy state, and consistent empty/loading/error/status rendering;
- Windows forced-colors/high-contrast support;
- `prefers-reduced-motion` support;
- DPI/text-scaling resilience and minimum interactive-control sizing;
- Security Center first-run privacy/accessibility guidance;
- persistent Privacy & Accessibility guidance in Security Center settings;
- localization-ready Phase 23 shell string catalog and declared `en-US` locale metadata;
- approved DragonForge branding paths integrated into Tauri application bundle metadata;
- migrated Password Manager icon preserved rather than replacing its existing identity;
- accessible installer privacy/data-retention notice shown before installation;
- suite-wide visible milestone metadata advanced to Suite Phase 23;
- Phase 23 CI coverage and an authoritative Windows verifier.

## Accessibility contract

All DragonForge desktop webviews now load `phase23.css` and `phase23.js`.

The shared runtime:

- assigns a main landmark when one is available;
- injects a keyboard-visible skip link;
- labels primary navigation where a label is missing;
- keeps `aria-current="page"` synchronized with active navigation;
- upgrades dynamic status/result/toast regions for screen readers;
- provides a hidden live region for explicit announcements;
- supports keyboard navigation within navigation groups;
- respects reduced-motion preferences;
- uses Windows forced-colors rather than depending solely on DragonForge colors;
- exposes standardized accessible loading, empty, error, and status states.

The implementation does not claim formal WCAG certification or replace hands-on testing with NVDA, Narrator, high-contrast themes, keyboard-only navigation, and 200%+ zoom. Those remain release-audit acceptance work for Phase 24.

## Privacy contract

DragonForge remains local-first.

Phase 23 user guidance explicitly states:

- support bundles are not uploaded automatically;
- diagnostic logs are not uploaded automatically;
- vault contents, credentials, recovery packages, and private application data remain local unless the user performs an explicit feature action;
- network behavior is confined to features that require it, including signed update checks and configured Password Manager synchronization;
- diagnostic export remains an explicit user action.

Phase 23 also verifies that desktop UI HTML does not introduce remote script, stylesheet, or image dependencies.

This phase does not change the existing signed-update or Password Manager synchronization trust models.

## First-run guidance

Security Center shows a one-time accessible first-run dialog containing privacy and accessibility guidance. The acknowledgement flag is stored only in the webview's local storage under:

`dragonforge.phase23.firstRunSeen`

No identifier, timestamp, analytics event, or network request is associated with this acknowledgement.

The same guidance remains available permanently in Security Center settings.

## Localization readiness

The Phase 23 shared runtime declares:

- locale: `en-US`;
- document language fallback: `en`;
- `data-l10n-ready="true"`;
- a keyed string catalog for all new Phase 23 shared-shell copy;
- a stable `t(key)` lookup exposed through `DragonForgeUX`.

Existing product-specific English copy is intentionally not machine-translated or silently rewritten in this phase. Future locale packs can migrate product strings behind the same key-based boundary without changing security-sensitive Rust logic.

## Consistent dynamic states

`window.DragonForgeUX` exposes:

- `announce(message, urgent)`;
- `setBusy(element, busy, message)`;
- `renderState(host, kind, message)`;
- `t(key)`;
- the current locale and string catalog.

Supported shared state kinds:

- `loading`;
- `empty`;
- `error`;
- `status`.

These helpers are intentionally presentation-only and do not perform privileged actions, filesystem access, process execution, or network requests.

## Branding integration

Approved artwork remains canonical under:

`assets/branding/logos`

Phase 23 connects product artwork to Tauri bundle icon metadata:

- Security Center -> `dragonforge-security-center.png`;
- File Vault -> `dragonforge-file-vault.png`;
- Authenticator -> `dragonforge-authenticator.png`;
- Security Scanner -> `dragonforge-security-scanner.png`;
- Integrity Monitor -> `dragonforge-integrity-monitor.png`;
- Network Guard -> `dragonforge-network-guard.png`;
- Backup & Recovery -> `dragonforge-backup-and-recovery.png`;
- Secure Share -> `dragonforge-secure-share.png`.

Password Manager keeps its migrated `apps/password-manager/icons/icon.png` because the branding source README explicitly calls for preserving that existing product logo.

The suite-wide and Agent logos remain available for installer/suite and background-service branding where a square/icon-specific derivative is appropriate. Phase 23 does not distort the horizontal source artwork or fabricate new emblem crops.

## Installer polish

The Inno Setup flow now presents `installer/PHASE23-PRIVACY.txt` before installation. It explains the local-first model, preserved user data on uninstall, optional privileged-service approval, and suite accessibility behavior.

The installer continues to use its existing executable/icon and signing pipeline. The suite-wide source logo is currently a PNG horizontal lockup rather than an installer-ready ICO, so Phase 23 does not silently manufacture a distorted installer icon.

## Security boundary

Phase 23 does not:

- add telemetry or analytics;
- upload diagnostics automatically;
- add remote fonts, scripts, stylesheets, or images to desktop UIs;
- weaken the Content Security Policy;
- expose security-sensitive values to ARIA labels or live regions;
- alter cryptography, vault formats, sync protocols, privileged-service commands, firewall policy, recovery formats, or Agent authentication;
- make accessibility preferences a security decision.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase23-privacy-accessibility-product-polish-tests.ps1
```

The authoritative Windows verification passed on 2026-09-23 on `DRACO` (Microsoft Windows NT 10.0.26200.0).

Verified coverage included:
- `cargo fmt --all --check`;
- locked Cargo metadata;
- `cargo check --workspace --all-targets --all-features --locked`;
- strict `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`;
- complete workspace tests and doc-tests;
- JavaScript syntax checks for application and Phase 23 runtime files;
- all nine Phase 23 HTML/CSS/JS accessibility/privacy invariants;
- approved Tauri branding-path verification;
- installer privacy-notice verification;
- PowerShell syntax validation;
- all expected Windows suite application builds.

Verified log: `dragonforge-phase23-privacy-accessibility-product-polish-20260923-232110.log`

Verifier-reported log SHA-256: `C2EA7882638E66075F7D5398A59FEB16757820A94FCEA77DBB4ED277112841C2`

The PASS and SHA-256 were supplied from the authoritative DRACO verifier output. The log/sidecar were not re-uploaded for an independent second hash calculation in this chat because the upload limit had been reached.
