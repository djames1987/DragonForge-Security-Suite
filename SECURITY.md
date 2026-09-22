# DragonForge Security Suite Security Policy

DragonForge Security Suite is security-sensitive software. Security reports should not be disclosed publicly before a fix is available.

## Supported status

The suite is currently in pre-release architecture development. No production security guarantees are made yet.

## Reporting a vulnerability

Please report suspected vulnerabilities privately to the repository owner through an appropriate private GitHub contact/security channel. Do not include secrets, real credentials, recovery keys, or production vault data in reports.

A useful report includes:

- affected component and version/commit;
- reproduction steps;
- expected versus observed behavior;
- security impact;
- relevant logs with sensitive values removed;
- suggested mitigation, if known.

## Security development principles

- No plaintext secrets in logs.
- No secrets committed to source control.
- Cryptographic changes require dedicated tests and review.
- Authentication, encryption, serialization, and storage formats must be versioned.
- Security boundaries must fail closed where practical.
- Memory containing secrets should have an explicit lifecycle.
- Unsafe Rust is forbidden at the workspace level unless a future component receives an explicit, documented exception.
- Dependencies should be minimized and audited.
- Local Agent IPC is authenticated and fail-closed; future IPC/network/update channels must provide equivalent authenticated peer/artifact verification before carrying security-sensitive actions.

## Scope

This policy applies to all applications, services, libraries, extensions, scripts, and build/release infrastructure in this repository.

## Repeatable dependency auditing

Rust dependencies are checked against the RustSec advisory database by the scheduled and dependency-change GitHub workflow in `.github/workflows/security-audit.yml`. Developers can run the same class of check locally with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-dependency-audit.ps1
```

If `cargo-audit` is not installed, use the script's `-InstallIfMissing` switch or install it explicitly with `cargo install cargo-audit --locked`. Advisory suppressions must be documented with applicability analysis rather than silently ignored.
