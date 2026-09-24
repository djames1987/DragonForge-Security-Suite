# DragonForge Windows Installer Test Checklist

Use disposable test data during the alpha cycle.

## Before installation

- Confirm the installer filename/version.
- Verify the installer SHA-256 against its sidecar.
- Record Windows edition/version/build.
- Record whether the account is Standard User or Administrator.
- Record whether WebView2 is detected.

## Fresh install

- Launch the installer without explicitly elevating it.
- Confirm the default path is under `%LOCALAPPDATA%\Programs\DragonForge Security Suite`.
- Confirm no administrator prompt is required for the normal path.
- Confirm the WebView2 warning appears when testing a disposable system without WebView2.
- Complete installation.
- Confirm the Start Menu DragonForge Security Suite folder exists.
- Confirm DragonForge Security Center launches.
- If selected, confirm the desktop shortcut works.

## Installed-suite smoke test

From Security Center, start DragonForge Agent and launch Password Manager, File Vault, Authenticator, Security Scanner, Integrity Monitor, Network Guard, Backup & Recovery, and Secure Share. Confirm all applications remain sibling executables in the same installation directory.

## Agent lifecycle at sign-in

- Confirm the installed Startup shortcut points to the exact installed `dragonforge-agent.exe --serve`.
- Sign out/in or reboot a disposable test system.
- Confirm the Agent starts as the current normal user without elevation.
- Open Security Center and confirm it reconnects to the existing authenticated Agent session.
- Stop the Agent from Security Center and confirm a same-session refresh does not immediately restart it.
- Start or restart the Agent and confirm authenticated health returns.

## Upgrade

- Leave the installed suite in place.
- Start DragonForge Agent.
- Run a newer installer build using the same AppId.
- Confirm the installer handles the installed Agent cleanly.
- Confirm the suite still launches after upgrade.
- Confirm user-created data remains available.

## Uninstall

- Start the installed DragonForge Agent before uninstalling so shutdown behavior is exercised.
- Uninstall DragonForge Security Suite from Windows Settings or the Start Menu uninstall shortcut.
- Confirm the exact installed Agent process is stopped before its executable is removed.
- Confirm the generated Inno Setup uninstaller exits successfully.
- Confirm the installed program directory and installer-managed files are removed.
- Confirm the Start Menu group and DragonForge Agent Startup shortcut are removed.
- If the optional privileged service was never installed, confirm uninstall does not request unnecessary Administrator elevation.
- If the optional privileged service was installed, confirm uninstall requests Administrator approval and removes the managed service/binary while preserving its data unless explicit data removal was requested.
- Confirm user-created DragonForge data outside the installation directory remains intact.
- Confirm Password Manager vaults stored elsewhere are not deleted.
- Confirm Windows Credential Manager sync credentials are not intentionally removed by uninstall.

The automated disposable lifecycle gate is `scripts/test-windows-installer-uninstall.ps1`. It performs a real silent install/uninstall on CI or an explicitly disposable local/VM system and refuses local execution unless `-AllowLocal` is deliberately supplied.

For the dedicated pre-1.0 beta, run the guarded publisher from `release/v1.0.0-beta.1-uninstaller`:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\publish-beta-uninstaller-test.ps1
~~~

The publisher first builds an isolated-AppId installer with isolated Start Menu/Startup shortcut names, runs the full install -> Agent start -> uninstall lifecycle, then rebuilds the public beta with the normal production AppId and publishes it only after the lifecycle passes.

Never attach or use production passwords, Account Secrets, recovery kits/codes, OTP seeds, sync/admin tokens, private keys, vault contents, or sensitive personal files during alpha testing.


## Beta qualification evidence

When this checklist is used for a required Phase 13 BQ scenario, retain the Phase 13 verifier log + sidecar and record the installer lifecycle result in the generated qualification JSON. Upgrade/uninstall preservation failures are release-blocking until resolved or the candidate is withdrawn.
