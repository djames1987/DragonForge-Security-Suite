# Portable Windows Test Releases

DragonForge Security Suite uses a portable ZIP for the initial external-test release channel.

## Why portable first

The suite contains multiple sibling desktop executables plus DragonForge Agent. Security Center intentionally launches suite components by exact sibling path. Keeping the verified binaries together in one extracted directory provides a small, auditable release surface while installer and updater design remain future work.

## Create the package

From a clean Windows checkout:

~~~powershell
git checkout main
git pull
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\package-windows-release.ps1
~~~

The script performs release-profile builds, verifies all ten expected executables, stages only runnable binaries and tester helpers, writes build commit/version information, generates SHA-256 hashes, creates a ZIP, and emits a SHA-256 sidecar.

Artifacts are written under dist/.

## Publish the alpha

Publishing uses GitHub CLI so the tag, release, and uploaded assets are created from the exact clean main commit:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\publish-windows-release.ps1
~~~

Requirements on the release-building machine:

- normal project development prerequisites;
- GitHub CLI (gh);
- an authenticated GitHub CLI session with permission to publish releases to this repository.

The published release is marked pre-release.

## Test machine requirements

The extracted package needs no Rust, Cargo, Node.js, Git, or source checkout. Testers can run Check-Prerequisites.cmd and Launch-Security-Center.cmd directly from the extracted folder.

It requires 64-bit Windows 10/11 and Microsoft Edge WebView2 Runtime for the Tauri desktop applications.

The release remains unsigned during this alpha stage, so SmartScreen warnings are expected.
