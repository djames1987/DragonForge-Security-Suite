# Third-Party Notices

This repository is proprietary DragonForge source. The DragonForge `LICENSE` governs original DragonForge material only. Third-party software and material retain their own copyright and license terms; nothing in the DragonForge license restricts rights independently granted by those third parties.

## Rust dependencies

The workspace resolves third-party Rust crates through Cargo. They are not vendored in this repository. Binary distributions may incorporate those crates and must preserve all notices and license obligations that apply to the exact resolved release graph.

The repository includes `scripts/run-license-audit.ps1`, which runs:

```powershell
cargo metadata --locked --format-version 1
```

and fails closed when a registry package has missing license metadata, an unreviewed SPDX identifier, or an unreviewed complex expression. Run this script from a clean checkout before every public binary release and retain its JSON output as release evidence. The script currently recognizes reviewed permissive/license families including MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, BSL-1.0, MPL-2.0, OpenSSL, Python-2.0, Unicode licenses, CC0-1.0, 0BSD, MIT-0, NCSA, Unlicense, and explicitly reviewed SPDX exception expressions. The allowlist is a review control, not a claim that every listed license is present in every dependency graph.

If the resolved graph changes, a passing historical report is not sufficient. Regenerate the report and review any new expression before release.

## Browser-extension and web assets

`extensions/password-manager-browser/package.json` declares no npm runtime or development dependencies. The browser-extension JavaScript, HTML, and CSS in the repository are therefore treated as DragonForge source unless a file states otherwise.

No third-party web font, icon package, or bundled JavaScript library was identified in the current tree during the Phase 3 repository inspection.

## Password Manager migration provenance

The Password Manager source under `apps/password-manager/`, `services/password-manager-sync/`, `extensions/password-manager-browser/`, `crates/dragonforge-crypto/`, `crates/dragonforge-vault/`, and `docs/password-manager/` was imported from the first-party `djames1987/DragonForge-Password-Manager` repository. `docs/password-manager/MIGRATION_RECORD.md` records the frozen source commit and path mapping. This migration does not convert third-party code into DragonForge-owned code; any independently licensed material remains governed by its original terms.

## Visual and documentation assets

The repository contains DragonForge branding PNGs, application icons, and Password Manager documentation SVGs. Repository history and adjacent documentation identify them as project assets, but the files do not contain sufficient creator/source/license metadata to independently prove redistribution rights.

**Publication gate `DF-P3-ASSET-001`:** before making this repository public, the owner must confirm and record that the branding PNGs, application icons, and documentation SVGs were created by DragonForge/the owner, validly commissioned/assigned, or are otherwise redistributable under terms compatible with publication. If any asset came from an external source, record its source and license here and preserve the required attribution; otherwise replace/remove it before publication.

## Release rule

Do not distribute a public binary, installer, archive, or container image until the exact release dependency graph has passed the repository license audit and all required upstream notices/license texts for the distributed artifacts have been collected. Uncertain or missing license metadata is a release blocker, not an implied permission grant.
