# DragonForge Security Suite — Phase 24 Feature Freeze

**State: ACTIVE**

Phase 24 begins the DragonForge Security Suite 1.0 release freeze.

Until Phase 25 publishes 1.0, changes merged to the release line should be limited to:

- release-blocking security fixes;
- correctness/regression fixes;
- accessibility/privacy corrections;
- dependency/advisory/license remediation;
- release engineering, signing, packaging, installer, and documentation fixes;
- test/audit improvements needed to prove an existing 1.0 capability.

The following are frozen for 1.0 unless a security defect requires a versioned compatibility change:

- encrypted vault/container/package formats;
- Password Manager sync and recovery protocol semantics;
- Agent IPC protocol semantics;
- privileged-service protocol and capability set;
- firewall policy behavior;
- update manifest trust/signature semantics;
- recovery package format/schema;
- suite component set and user-facing feature scope.

New features and broader privileged capabilities are deferred until after 1.0.

Any exception must:
1. be documented as release-blocking;
2. receive a dedicated security review;
3. update compatibility/security documentation;
4. rerun the complete Phase 24 audit before release.
