# Phase 12.4 — Code Signing

## Status

**Verified Complete**

Phase 12.4 adds Windows Authenticode signing to the verified Phase 12.3 exact-tag release pipeline.

## Security goals

- sign all ten DragonForge release executables before portable-package hashing;
- sign the compiled Windows installer before its SHA-256 sidecar is written;
- use SHA-256 Authenticode file digests;
- require RFC 3161 timestamping with SHA-256 for signed release artifacts;
- verify signatures after signing and again during final artifact verification;
- keep signing credentials outside the repository and normal developer configuration;
- record signer identity metadata in the machine-readable release manifest;
- require signed stable releases while allowing explicitly unsigned prerelease/development builds;
- preserve exact-tag provenance, Cargo.lock enforcement, release immutability, and all Phase 12.3 checks.

## Signing identity sources

The signing helper supports exactly one of these identity sources.

### Windows certificate store — preferred

Set:

~~~powershell
$env:DRAGONFORGE_SIGN_CERT_THUMBPRINT = "<certificate thumbprint>"
$env:DRAGONFORGE_SIGN_CERT_STORE_LOCATION = "CurrentUser"
$env:DRAGONFORGE_SIGN_TIMESTAMP_URL = "<RFC3161 timestamp URL>"
~~~

Use `LocalMachine` only when the signing environment is deliberately provisioned for it.

The private key remains managed by Windows or the underlying hardware/provider and is never read by DragonForge scripts.

### PFX — CI/controlled fallback

Set:

~~~powershell
$env:DRAGONFORGE_SIGN_PFX_PATH = "C:\secure\dragonforge-signing.pfx"
$env:DRAGONFORGE_SIGN_PFX_PASSWORD = "<secret>"
$env:DRAGONFORGE_SIGN_TIMESTAMP_URL = "<RFC3161 timestamp URL>"
~~~

PFX/P12/PVK/SNK files are ignored by Git.

The PFX password is redacted from DragonForge console logging. Environment-variable PFX credentials are still less desirable than a protected certificate-store/HSM-backed key because process-level access on the signing host may expose secrets.

Do not configure both a certificate thumbprint and a PFX path.

## SignTool

Phase 12.4 uses Microsoft's `SignTool.exe`. The helper searches PATH first, then Windows SDK `bin\<version>\x64\signtool.exe` locations.

Signing uses:

- `/fd SHA256`;
- `/tr <RFC3161 URL>`;
- `/td SHA256`.

Verification uses the default Authenticode policy with all embedded signatures and requires timestamp presence for release signing.

## Tagged release usage

For a prerelease that should remain unsigned:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-tagged-windows-release.ps1 -Tag v0.1.0-alpha.3
~~~

For a signed prerelease or any stable release:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-tagged-windows-release.ps1 -Tag v0.1.0-alpha.3 -SignRelease
~~~

Stable tags such as `v1.0.0` fail closed unless `-SignRelease` is supplied.

Publishing follows the same rule:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\publish-windows-release.ps1 -Tag v0.1.0-alpha.3 -SignRelease
~~~

## Release-manifest signing metadata

A signed release manifest records:

- signing state;
- whether signing was required;
- SHA-256 file digest;
- RFC 3161 timestamp protocol;
- SHA-256 timestamp digest;
- signer certificate subject;
- signer certificate thumbprint.

Private keys, PFX passwords, certificate-store PINs, tokens, and timestamp-service credentials are never written to the manifest.

Unsigned prerelease/development builds are explicitly marked `unsigned-development`.

## Certificate rotation

Normal rotation:

1. obtain/provision the replacement certificate and protected private key;
2. validate its Code Signing EKU and trust chain on a dedicated signing host;
3. update only the external signing environment/thumbprint;
4. create a signed test prerelease and verify all ten EXEs plus installer;
5. publish new releases with the new certificate;
6. retain historical release manifests and hashes so old artifacts remain attributable to their original signer.

Do not rewrite or resign already-published immutable GitHub releases.

## Revocation and compromised-key response

If a signing key may be compromised:

1. stop signing and release publication immediately;
2. remove/restrict access to the affected key on signing hosts and CI;
3. contact the certificate authority/provider and request certificate revocation;
4. rotate all associated credentials/tokens/PINs and provision a replacement key;
5. identify every DragonForge release manifest containing the compromised certificate thumbprint;
6. publish a security notice identifying affected release tags/hashes;
7. build new releases from reviewed source with the replacement certificate;
8. never silently replace existing release assets.

Timestamped signatures help distinguish artifacts signed while a certificate was valid, but revocation/incident interpretation depends on Windows trust policy and CA status.

## Timestamp outage policy

A signed DragonForge release is not considered publishable when RFC 3161 timestamping fails.

Do not bypass timestamping to meet a release deadline. Retry later or use an approved alternate timestamp service configured outside the repository.

## SmartScreen expectations

Authenticode establishes publisher identity and file integrity. It does not guarantee immediate Microsoft Defender SmartScreen reputation; reputation can still depend on certificate/provider history and download prevalence.

Unsigned prereleases must continue to state that unknown-publisher/SmartScreen warnings are expected.

## Phase boundary

Phase 12.4 does not:
- obtain or purchase a certificate automatically;
- store private keys in GitHub;
- create a remote signing service;
- weaken Phase 12.3 tag/provenance validation;
- claim that signing alone makes binaries safe.

## Verification

Run:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase12.4-code-signing-tests.ps1
~~~

The verifier does not require a production signing certificate. It validates release-script syntax, signing configuration fail-closed behavior, stable-release signing enforcement, hash/signing ordering, signature verification wiring, manifest metadata, and repository secret-exclusion controls.

## Verified Windows result

Phase 12.4 completed authoritative Windows verification on `DRACO`.

- Machine: `DRACO`
- Windows: `Microsoft Windows NT 10.0.26200.0`
- Result: **PASS**
- Verified log: `dragonforge-phase12.4-code-signing-20260922-113515.log`
- Log SHA-256: `EE98BCF2D6E0AD1780197698E43AE655BAEE7FDED078C1621B2C100E2B903DA4`

The passing run covered rustfmt, targeted compile checks, strict Clippy, all 9 Agent tests, all 22 Security Center tests, JavaScript syntax validation, PowerShell 5.1 syntax validation for the signing and release pipeline, signing-before-hashing ordering, installer signing order, stable-release signing enforcement, signature-verification wiring, release-manifest signing metadata, and repository ignore rules for private-key containers.

This verifier validates the Phase 12.4 implementation without requiring a production signing certificate. Actual production signing still depends on provisioning a trusted code-signing certificate/private key and an approved RFC 3161 timestamp service in the release environment.
