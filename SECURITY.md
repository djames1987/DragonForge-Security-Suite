# DragonForge Security Suite Security Policy

DragonForge Security Suite is security-sensitive software. Security reports should not be disclosed publicly before a fix is available.

## Supported status

DragonForge Security Suite 1.0 establishes the first stable support line. The repository may contain a 1.0 release-candidate baseline before the signed `v1.0.0` artifacts are published; stable support begins with that immutable signed release.

The current stable 1.x line receives security/correctness fixes according to [docs/SUPPORT_POLICY.md](docs/SUPPORT_POLICY.md). Compatibility commitments are documented in [docs/1_0_COMPATIBILITY_POLICY.md](docs/1_0_COMPATIBILITY_POLICY.md).

## Reporting a vulnerability

Use GitHub's private security-advisory reporting flow when available:

`https://github.com/djames1987/DragonForge-Security-Suite/security/advisories/new`

If that private flow is unavailable, contact the repository owner through a private GitHub channel. Do **not** open a public issue containing exploit details or sensitive evidence. The coordinated response process is documented in [docs/VULNERABILITY_RESPONSE.md](docs/VULNERABILITY_RESPONSE.md).

Do not include real credentials, recovery keys, Account Secrets, OTP seeds, private signing keys, production vaults, tokens, personal data, or other live secrets. Use synthetic test material and sanitized evidence.

A useful private report includes:

- affected component and version/commit;
- reproduction steps using synthetic data;
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