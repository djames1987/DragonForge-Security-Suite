# Future Projects

This directory contains design work and implementation roadmaps for DragonForge projects that are intentionally **not yet part of the active DragonForge Security Suite build or release surface**.

## Rules

- Content under `future-projects/` is planning material until a project is explicitly promoted into active development.
- Future projects must not be added to the workspace, installer, release manifests, Security Center registry, privileged-service capability surface, or CI release gates merely because planning documents exist here.
- Each future project should live in its own folder.
- A future project roadmap should define architecture, security boundaries, phases, dependencies, acceptance criteria, validation requirements, and migration/promotion steps before implementation begins.
- When a project is promoted, preserve this planning history and create an explicit migration record into the active `apps/`, `crates/`, `services/`, `docs/`, and/or installer structure.

## Planned projects

- [DragonForge DNS Shield](DragonForge-DNS-Shield/ROADMAP.md) — standalone or centrally managed self-hosted DNS filtering/protection platform.
