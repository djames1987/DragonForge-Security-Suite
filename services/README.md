# Services

Long-running local or server-side DragonForge processes live here.

Current:
- `password-manager-sync/` — migrated Password Manager zero-knowledge sync service.

Planned:
- `dragonforge-agent/` — future local background monitoring/protection service.

Background services should expose narrow authenticated interfaces and run with the least privilege required.
