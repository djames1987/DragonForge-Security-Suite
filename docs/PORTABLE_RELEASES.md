# Windows Release Workflow

DragonForge Security Suite supports both a portable Windows ZIP and a suite-level per-user installer.

Phase 12.3 makes releases **tag-bound and verified before publication**. Direct ad-hoc packaging scripts remain implementation building blocks, but publishable releases should use the workflow below.

## 1. Prepare the version

Start from a clean working tree:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\prepare-release.ps1 -Version 0.1.0-alpha.3
~~~

This stamps the workspace/Tauri/UI/installer versions, refreshes `Cargo.lock`, runs a workspace check, and verifies version consistency.

Review and commit the changes. Then create and push an annotated tag on that exact commit:

~~~powershell
git tag -a v0.1.0-alpha.3 -m "DragonForge Security Suite v0.1.0-alpha.3"
git push origin HEAD
git push origin v0.1.0-alpha.3
~~~

## 2. Build and verify the exact tag

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-tagged-windows-release.ps1 -Tag v0.1.0-alpha.3
~~~

The tagged builder requires:
- Windows x64 build prerequisites;
- Inno Setup 6 for installer builds;
- a clean working tree;
- `HEAD` exactly equal to the requested tag commit;
- tracked and valid `Cargo.lock`;
- all stamped versions matching the tag.

It produces the portable ZIP, installer, SHA-256 sidecars, generated release notes, and a release manifest. The artifact verifier extracts the ZIP, checks all ten executables, validates internal checksums, and confirms BUILD-INFO tag/commit identity.

Use `-NoInstaller` only for an intentionally portable-only release.

## 3. Publish verified assets

GitHub CLI must be installed and authenticated:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\publish-windows-release.ps1 -Tag v0.1.0-alpha.3
~~~

The publisher reruns the tagged build/verifier, uses GitHub's existing tag verification, uploads only verified assets, and refuses to mutate an existing release.

Tags with a prerelease suffix are published as prereleases; plain semantic versions are treated as stable-channel releases.

## Artifact set

A normal tagged Windows build produces:
- `DragonForge-Security-Suite-v<version>-win-x64.zip`;
- ZIP `.sha256`;
- `DragonForge-Security-Suite-v<version>-win-x64-setup.exe`;
- installer `.sha256`;
- `release-notes-v<version>.md`;
- `release-manifest-v<version>.json`;
- release-manifest `.sha256`.

## Test machine requirements

The packaged applications need no Rust, Cargo, Node.js, Git, or source checkout. They require 64-bit Windows 10/11 and Microsoft Edge WebView2 Runtime.

The Phase 12.3 pipeline is still unsigned. SmartScreen/unknown-publisher warnings remain expected until Phase 12.4 code signing is implemented.

Existing published releases such as `v0.1.0-alpha.1` remain immutable.
