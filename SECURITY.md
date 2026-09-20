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
- Updates and inter-process communication must eventually be authenticated.

## Scope

This policy applies to all applications, services, libraries, extensions, scripts, and build/release infrastructure in this repository.
