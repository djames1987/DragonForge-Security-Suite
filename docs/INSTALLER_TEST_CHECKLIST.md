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

## Upgrade

- Leave the installed suite in place.
- Start DragonForge Agent.
- Run a newer installer build using the same AppId.
- Confirm the installer handles the installed Agent cleanly.
- Confirm the suite still launches after upgrade.
- Confirm user-created data remains available.

## Uninstall

- Uninstall DragonForge Security Suite from Windows Settings or the Start Menu uninstall shortcut.
- Confirm the installed program directory is removed.
- Confirm DragonForge shortcuts are removed.
- Confirm user-created DragonForge data outside the installation directory remains intact.
- Confirm Password Manager vaults stored elsewhere are not deleted.
- Confirm Windows Credential Manager sync credentials are not intentionally removed by uninstall.

Never attach or use production passwords, Account Secrets, recovery kits/codes, OTP seeds, sync/admin tokens, private keys, vault contents, or sensitive personal files during alpha testing.
