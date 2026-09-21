# Applications

Interactive DragonForge applications live here.

Current:
- `security-center/` — suite dashboard/orchestration foundation.
- `password-manager/` — migrated DragonForge Password Manager desktop application.

Future user-facing products include File Vault, Authenticator, Security Scanner, Network Guard UI, and other suite applications.

Application UI/orchestration code should depend downward on product/shared crates and should not own reusable cryptographic primitives.
