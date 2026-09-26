# DragonForge DNS Shield — AI-Optimized Development Roadmap

**Project status:** Future project / planning only  
**Repository:** `djames1987/DragonForge-Security-Suite`  
**Planned product name:** DragonForge DNS Shield  
**Planned language:** Rust  
**Planned UI:** Tauri desktop application where appropriate  
**Primary design target:** Windows-first, cross-platform-capable DNS service architecture  
**Document purpose:** Canonical implementation roadmap and continuation contract for human developers and AI coding agents

---

# 1. AI CONTINUATION CONTRACT

This document is intentionally written so an AI agent can resume development without relying on conversational memory.

## 1.1 Mandatory interpretation rules

Any AI or developer continuing this project MUST:

1. Read this entire roadmap before modifying code.
2. Read the current DragonForge Security Suite architecture and security-boundary documentation before integration work.
3. Treat all phases marked future/unstarted as design intent, not completed functionality.
4. Never claim a phase is complete until its acceptance criteria and required validation pass.
5. Preserve standalone operation. DNS Shield must never require the full Security Suite merely to provide DNS service.
6. Preserve managed-node offline operation. Loss of management connectivity must not interrupt DNS resolution.
7. Preserve least privilege. The DNS runtime must not be merged into the DragonForge Privileged Service.
8. Preserve narrow typed privileged operations. Never add generic shell, arbitrary command, arbitrary firewall, arbitrary registry, or arbitrary service-control execution.
9. Prefer mature DNS protocol libraries over hand-written DNS packet parsing.
10. Keep DNS request handling independent from the UI.
11. Keep the management plane logically separate from the DNS resolution plane.
12. Use immutable, versioned policy revisions with validation and atomic activation.
13. Retain a last-known-good policy and support rollback.
14. Never silently weaken security checks to make tests pass.
15. Update this roadmap when implementation decisions materially change.
16. Add tests with every phase.
17. Add a dedicated Windows validation script for every implementation phase that requires Windows behavior.
18. Maintain redaction-safe logs and never record secrets by default.
19. Do not add DNS Shield to the current suite release package until the roadmap reaches the explicit promotion/integration phase.
20. At phase boundaries, document exactly what changed, what remains deferred, known limitations, and the next phase.

## 1.2 Required phase-completion record

Every completed phase should record:

- phase number and name;
- implementation commit or merge commit;
- files/components added;
- security-sensitive changes;
- schema/protocol versions changed;
- tests added;
- validation command;
- authoritative validation result;
- log filename and SHA-256 where applicable;
- known limitations;
- explicit next phase.

---

# 2. PRODUCT VISION

DragonForge DNS Shield is a self-hosted DNS filtering and security platform that can operate in three roles:

1. **Standalone Node** — protects a single computer or a small network and is fully locally managed.
2. **Primary / Controller** — central administrative management plane that publishes DNS policy to managed DNS nodes.
3. **Managed Secondary Node** — provides local DNS service using centrally controlled policy while continuing to operate if the Primary is unavailable.

The terms **Primary** and **Secondary** are the canonical product terminology. "Master/slave" may appear in historical discussion but must not be used in new UI or documentation.

The system should cover the practical use case of Pi-hole-style DNS filtering while adding DragonForge-specific capabilities:

- secure central administration;
- signed/versioned policy deployment;
- offline-safe managed nodes;
- atomic updates and rollback;
- hierarchical organization/site/node policy;
- Security Center integration;
- redaction-safe security events;
- future Network Guard correlation;
- standalone packaging;
- Windows service deployment;
- future Linux/systemd and container deployment.

---

# 3. NON-NEGOTIABLE ARCHITECTURE

## 3.1 Planned component boundaries

When promoted to active development, the intended structure is approximately:

```text
apps/
  dns-shield/                     # Standalone/admin UI

crates/
  dragonforge-dns-shield/         # Core policy/filter/config/domain logic
  dragonforge-dns-management/     # Shared management protocol/types if needed

services/
  dragonforge-dns/                # Always-running DNS resolver/filter service
  dragonforge-dns-controller/     # Optional Primary management service

docs/
  dns-shield/

future-projects/
  DragonForge-DNS-Shield/
    ROADMAP.md                     # This planning document
```

Exact crate boundaries may evolve, but the following separation is mandatory:

- UI != DNS runtime
- DNS runtime != privileged service
- management plane != DNS query path
- persistent configuration != hot-path lookup representation
- policy publication != policy activation
- central administration != mandatory dependency for DNS availability

## 3.2 Existing suite components to integrate later

DNS Shield should eventually integrate with, but must not tightly couple itself to:

- DragonForge Security Center;
- DragonForge Agent;
- DragonForge Privileged Service;
- Phase 19 typed firewall policy infrastructure;
- Phase 20 persistent event hub;
- scheduled automation infrastructure;
- backup/recovery;
- diagnostics;
- signed update/release engineering.

## 3.3 DNS library strategy

Do not implement DNS wire parsing from scratch unless a documented gap forces it.

Preferred foundation: current maintained **Hickory DNS** crates or an equivalent mature Rust DNS implementation after a dependency/security review.

Before implementation, confirm:

- current maintenance status;
- license compatibility;
- supported DNS transports;
- server/resolver APIs;
- DNSSEC support;
- async runtime compatibility;
- known security advisories;
- fuzzing posture;
- blocklist/forwarding capabilities;
- Windows compatibility.

A dependency decision must be documented in an ADR before Phase 1 is marked complete.

---

# 4. OPERATING MODES

## 4.1 Standalone Node

A Standalone Node:

- runs a local DNS service;
- stores and edits its own configuration;
- manages local blocklists/allowlists/deny rules;
- may serve only the host or an entire LAN;
- does not require a controller;
- may later enroll into central management without reinstallation.

## 4.2 Primary / Controller

A Primary is the authoritative **management** service, not necessarily the DNS resolver for clients.

It manages:

- organizations;
- sites;
- node enrollment;
- node identities;
- policy revisions;
- blocklist source definitions;
- allow/deny policy;
- local DNS records;
- upstream resolver policy;
- client groups;
- privacy/logging policy;
- deployment rings;
- rollback;
- node health/synchronization state;
- certificate/node revocation.

A Primary may also run a DNS node on the same machine, but the roles remain logically separate.

## 4.3 Managed Secondary Node

A Secondary:

- resolves DNS locally;
- caches locally;
- applies its local compiled copy of the effective policy;
- reports health/status to the Primary;
- receives signed versioned policy updates;
- must continue DNS service during Primary outages;
- must retain the last known-good policy;
- must not accept unauthorized local overrides of centrally enforced settings.

---

# 5. CORE RELIABILITY INVARIANTS

These invariants are release-blocking.

1. Loss of Primary connectivity MUST NOT stop DNS resolution.
2. A bad policy update MUST NOT replace the current valid policy.
3. Policy activation MUST be atomic.
4. Policy revisions MUST be immutable once published.
5. The active policy MUST be identifiable by revision plus cryptographic digest.
6. Secondaries MUST verify integrity/authenticity before activation.
7. A managed node MUST retain at least one known-good rollback target.
8. DNS service MUST not depend on the desktop UI being open.
9. DNS query handling MUST avoid database queries on the normal hot path.
10. Blocklist refresh failure MUST leave the previous valid compiled rules active.
11. Configuration corruption MUST be detected and surfaced.
12. Service restarts MUST preserve valid active policy.
13. Management-plane compromise resistance MUST be designed separately from DNS-plane packet handling.
14. Central telemetry failure MUST NOT become a DNS outage.
15. Update/restart operations should minimize DNS downtime and be measurable.

---

# 6. SECURITY MODEL

## 6.1 Threat classes

Design and test for:

- malformed DNS packets;
- oversized DNS messages;
- DNS amplification misuse;
- cache poisoning attempts;
- malicious upstream behavior;
- malicious blocklist content;
- blocklist download tampering;
- management impersonation;
- secondary impersonation;
- replayed management messages;
- stolen enrollment token;
- stale policy rollback attack;
- compromised local user attempting to alter enforced policy;
- path/symlink attacks on policy files;
- corrupted persistent state;
- resource exhaustion;
- high-query-rate clients;
- decompression or parsing bombs if compressed lists are supported;
- unauthorized LAN exposure;
- accidental open resolver exposure to the public Internet;
- certificate/key theft;
- sensitive DNS history leakage;
- log injection;
- downgrade attacks on management protocol/schema.

## 6.2 Privilege separation

The DNS runtime should run using the least privileged practical service identity.

Administrative elevation may be required for:

- service registration/removal;
- binding/configuration changes where OS policy requires it;
- protected ACL setup;
- DragonForge-owned firewall rules;
- installer operations.

Ongoing DNS request processing must not require the DragonForge Privileged Service.

## 6.3 Privileged Service integration

If integration requires privileged operations, add narrow typed capabilities such as:

- install/remove DNS service;
- query DNS service installation state;
- apply/remove DragonForge-owned DNS listener firewall rules;
- protected service configuration updates.

Never add:

- arbitrary service control;
- arbitrary firewall expressions;
- raw PowerShell/cmd execution;
- arbitrary executable launch;
- arbitrary registry editing.

## 6.4 Management transport

Managed nodes should use authenticated encrypted transport.

Target design:

- TLS 1.3 where available;
- mutual TLS for established nodes;
- short-lived enrollment secret used only during enrollment;
- per-node identity;
- revocation support;
- bounded message sizes;
- freshness/timestamp checks;
- nonces/replay protection where needed;
- protocol version negotiation without insecure downgrade;
- certificate/key storage using OS-protected mechanisms where practical.

Exact PKI design requires an ADR before implementation.

## 6.5 Signed policy

Policy bundles should include:

- organization identifier;
- intended scope;
- immutable revision number;
- schema version;
- creation timestamp;
- artifact digests;
- complete policy digest;
- signer/key identifier;
- signature.

A node must reject:

- invalid signature;
- wrong organization;
- wrong scope;
- malformed schema;
- missing required artifact;
- digest mismatch;
- unsupported mandatory feature;
- unauthorized downgrade;
- impossible revision dependency.

---

# 7. POLICY MODEL

## 7.1 Hierarchy

Effective policy should support:

```text
Organization policy
       +
Site policy
       +
Node policy
       +
Allowed local settings
       =
Effective node policy
```

## 7.2 Ownership classes

Each setting should be classified:

- **ENFORCED** — centrally controlled, local modification forbidden.
- **DEFAULT** — controller supplies value but node may override.
- **LOCAL** — controller does not manage this setting.

The effective policy compiler must deterministically resolve conflicts.

## 7.3 Rule precedence

Initial target precedence:

1. required system/bootstrap behavior;
2. local authoritative DNS records;
3. explicit allow;
4. explicit deny;
5. centrally enforced rule;
6. wildcard/regex/category rule;
7. compiled external blocklist;
8. cache;
9. upstream forwarding.

Before Phase 2 completion, exact precedence must be documented and unit tested.

---

# 8. DATA AND STORAGE

## 8.1 Persistent state

SQLite is the preferred initial store for node operational state unless implementation evidence favors another embedded database.

Candidate data:

- configuration;
- local DNS records;
- blocklist sources;
- allow/deny rules;
- client metadata;
- client groups;
- policy metadata;
- active/previous policy revision pointers;
- list update metadata;
- aggregate statistics;
- query history when enabled;
- controller/node synchronization metadata.

## 8.2 Hot-path state

DNS query evaluation should use compiled in-memory structures.

No normal query should require repeated SQL lookups.

Potential structures:

- normalized domain hash sets;
- suffix trees/radix structures;
- compact trie;
- bloom filter only as an optimization before authoritative lookup;
- bounded DNS cache.

The exact representation must be benchmarked.

## 8.3 Privacy modes

Support at least:

- **Full** — query history with client identity;
- **Anonymized** — reduced/pseudonymous client identity;
- **Aggregate** — metrics only;
- **Disabled** — no query history.

Retention must be bounded. Unlimited default retention is prohibited.

---

# 9. MANAGEMENT SYNCHRONIZATION MODEL

## 9.1 Revision model

Each published configuration is an immutable revision:

```text
revision: 184
policy_id: <UUID>
schema_version: N
sha256: <digest>
created_at: <timestamp>
```

Nodes report:

- current revision;
- current digest;
- last successful sync;
- last attempted sync;
- activation result;
- health;
- stale/offline duration.

## 9.2 Push + pull

Use both:

- push notification for low-latency update awareness;
- periodic pull/reconciliation for reliability.

A missed push must self-heal on later reconciliation.

## 9.3 Atomic activation flow

Required activation flow:

```text
discover revision
-> download manifest
-> authenticate source
-> download required artifacts
-> verify artifact hashes
-> verify policy signature
-> validate schema
-> compile effective policy
-> run pre-activation checks
-> write staged state
-> atomic active-policy swap
-> post-activation health check
-> report success
```

Failure at any pre-activation step must preserve the current policy.

## 9.4 Differential/content-addressed artifacts

Large blocklists should eventually be content-addressed.

A revision manifest references artifact hashes. Nodes download only missing/changed artifacts.

Full-policy transfer may be used initially, but the protocol must leave room for content-addressed artifacts without breaking identity semantics.

---

# 10. DNS FEATURE TARGETS

## Core 1.0 target

- UDP DNS;
- TCP DNS;
- IPv4 and IPv6;
- forwarding resolver;
- bounded cache;
- configurable upstreams;
- health-aware upstream failover;
- allowlist;
- denylist;
- external domain blocklists;
- local DNS records;
- client visibility;
- query metrics;
- privacy/retention controls;
- safe temporary disable;
- blocklist update/rollback;
- standalone operation;
- managed-secondary operation;
- Primary administration;
- Windows service support.

## Later targets

- DNS-over-TLS upstream;
- DNS-over-HTTPS upstream;
- DNS-over-QUIC if mature/justified;
- DNSSEC validation;
- richer authoritative/local zones;
- DHCP integration;
- Linux systemd service;
- Docker/OCI image;
- high-availability controller;
- distributed administration;
- threat-intelligence feeds;
- Security Center/Network Guard correlation.

A full recursive resolver is not a 1.0 requirement.

---

# 11. BLOCKLIST ENGINE

## Requirements

- multiple sources;
- plain domain list;
- hosts-file-style input;
- canonical domain normalization;
- IDN/punycode handling policy;
- comments/blank lines;
- bounded line length;
- bounded file size;
- source-level maximum entry counts;
- duplicate elimination;
- safe update timeout;
- TLS validation;
- optional pinned source digest/signature in future;
- staging;
- compilation;
- atomic activation;
- previous-known-good preservation.

Do not activate partially parsed lists.

## Explainability

For every block decision, the engine should be capable of identifying:

- matched rule/domain;
- rule source;
- policy layer;
- category if known;
- effective action;
- whether an allow rule overrode another rule.

This metadata should be bounded and generated without slowing the query hot path excessively.

---

# 12. LOCAL DNS

Initial local record support:

- A;
- AAAA;
- CNAME;
- PTR/reverse mappings where practical.

Later:

- SRV;
- TXT;
- additional authoritative-zone features.

Local records require strict validation and deterministic precedence over external forwarding.

---

# 13. UPSTREAM RESOLVERS

Support multiple upstreams with:

- ordered or health-aware selection;
- per-upstream timeout;
- health state;
- failure counters;
- latency metrics;
- optional fallback;
- explicit bootstrap behavior for encrypted DNS endpoint names.

A controller may centrally enforce approved upstreams.

Managed nodes must not silently bypass an enforced upstream policy unless a documented emergency-fallback policy explicitly allows it.

---

# 14. CLIENTS AND GROUPS

Nodes should identify clients primarily by network source identity available to the resolver.

Support:

- IP-based client identity;
- friendly names;
- site/group assignment;
- policy group assignment;
- last-seen;
- aggregate query counts.

Do not promise process-level attribution from network DNS alone.

Process correlation on the local Windows host is a future Network Guard integration project and requires separate evidence.

---

# 15. ADMINISTRATION UX

## Standalone UI

Planned areas:

- Dashboard
- Queries
- Clients
- Blocklists
- Allowlist
- Denylist
- Local DNS
- Upstreams
- Privacy
- Service
- Settings
- Diagnostics

## Primary UI

Additional areas:

- Organizations
- Sites
- Nodes
- Enrollment
- Policy editor
- Policy revisions
- Deployment
- Rollback
- Certificates/identity
- Node health
- Synchronization status
- Audit history

## Managed Secondary UI

Must clearly show:

- Managed status;
- organization;
- Primary identity;
- connection status;
- active revision;
- last sync;
- stale duration;
- locally editable settings;
- centrally locked settings.

Locked settings must be visibly identified rather than silently ignored.

---

# 16. ENROLLMENT AND NODE LIFECYCLE

## Enrollment

Target flow:

1. Administrator creates node/enrollment record.
2. Controller issues short-lived single-use enrollment secret.
3. Node connects to expected controller.
4. Controller and node establish authenticated identity.
5. Per-node certificate/credential is provisioned.
6. Enrollment secret becomes unusable.
7. Initial policy is downloaded, validated, compiled, and atomically activated.
8. Node appears healthy/synchronized.

## Revocation

Controller must support revoking an individual node without rotating every node credential.

Revoked nodes must not receive new managed policy.

## Unenrollment behavior

Initial safe default: **Freeze**.

- retain last known-good policy;
- stop management synchronization;
- continue DNS service;
- visibly report unmanaged/frozen state.

Future optional modes may include converting to Standalone or organization-enforced shutdown after an explicit grace period.

---

# 17. CONTROLLER FAILURE AND OFFLINE MODE

Secondaries must continue serving DNS using local resources.

Node states:

- Connected / current;
- Connected / updating;
- Connected / update failed;
- Offline / current-as-of-last-contact;
- Offline / stale;
- Revoked;
- Frozen;
- Local recovery required.

Staleness must be visible but must not by itself break DNS.

Policies may define warnings at durations such as:

- 1 hour;
- 24 hours;
- 7 days.

Do not automatically delete the active policy due only to age.

---

# 18. DEPLOYMENT AND ROLLBACK

Initial deployment:

- publish revision;
- all targeted nodes reconcile.

Later deployment rings:

- canary;
- percentage;
- named sites;
- named node groups;
- full rollout.

Rollback should support:

- organization scope;
- site scope;
- individual node;
- automatic node-local rollback after defined activation-health failure.

Automatic rollback rules must be conservative and observable.

---

# 19. WINDOWS SERVICE DESIGN

The DNS service must:

- start independently of the UI;
- survive UI closure/logoff as appropriate;
- use controlled service identity;
- use protected configuration directories;
- detect port conflicts;
- expose health through authenticated local management;
- support graceful stop/restart;
- retain active policy across restart.

Required listeners:

- UDP/53;
- TCP/53.

The service must detect and report conflicts rather than killing or disabling another process.

---

# 20. FIREWALL DESIGN

For LAN operation, DragonForge may need inbound rules for TCP/UDP port 53.

Do not reuse Phase 19 outbound application rules as if they represented this requirement.

Create a new narrowly typed DNS-listener policy capability if needed, constrained to:

- exact signed DragonForge DNS service;
- fixed local port 53;
- TCP and/or UDP;
- DragonForge-owned rule namespace;
- explicit network scope such as local subnet/private profile;
- deterministic ownership;
- reversible mutation.

Public-Internet exposure must never be enabled silently.

---

# 21. OBSERVABILITY

Per node:

- service uptime;
- queries/sec;
- total queries;
- cache hit rate;
- blocked count;
- upstream latency;
- upstream failures;
- active policy revision;
- blocklist age;
- synchronization health;
- database/storage health;
- memory usage;
- bounded diagnostic events.

Controller:

- node inventory;
- node health;
- current revision;
- stale nodes;
- failed deployments;
- certificate expiry/revocation state;
- rollout progress.

Logs must remain redaction-safe.

---

# 22. SECURITY CENTER INTEGRATION

DNS Shield should eventually appear as a component in Security Center.

Security Center should receive only significant bounded events, for example:

- service stopped unexpectedly;
- blocklist update failed repeatedly;
- managed node lost controller connectivity;
- policy activation failed;
- automatic rollback occurred;
- upstream resolver outage;
- known-malicious-domain block aggregate;
- certificate nearing expiry.

Do not forward every DNS query or ordinary ad block into the global event hub.

---

# 23. BACKUP AND RECOVERY

Backup targets:

- standalone configuration;
- controller organization/site/policy metadata;
- locally authored lists/rules;
- local DNS records;
- enrollment/controller configuration excluding export-prohibited keys;
- policy history;
- database schema/version metadata.

Recovery must distinguish:

- restore configuration;
- restore management database;
- restore node identity;
- re-enroll node.

Private keys may require OS-protected backup semantics and should never be placed unencrypted into normal support bundles.

---

# 24. PERFORMANCE TARGETS

Exact targets must be benchmarked, but Phase 1 must establish a baseline.

Measure:

- cached-query latency;
- forwarded-query latency overhead;
- queries/sec;
- memory at 100k, 1M, and multi-million-domain rule sets;
- policy compile time;
- policy activation time;
- blocklist update time;
- startup recovery time;
- controller synchronization time.

Performance regression tests should be introduced once stable baselines exist.

Correctness and security take priority over synthetic benchmark numbers.

---

# 25. FUZZING AND ADVERSARIAL TESTING

Required fuzz/test areas:

- DNS packet parsing through chosen library integration;
- domain normalization;
- blocklist parser;
- hosts parser;
- wildcard/regex handling;
- policy manifest parser;
- policy signature validation;
- management protocol;
- configuration parser;
- corrupted SQLite/state recovery;
- malformed local records;
- hostile Unicode/IDN input;
- oversized artifacts;
- rollback/replay attempts.

Use property tests where practical.

---

# 26. PHASE ROADMAP

## Phase 0 — Architecture, Threat Model, Dependency Review

**Goal:** Freeze core architecture before executable DNS work.

Deliver:

- project ADR set;
- Hickory/equivalent dependency review;
- threat model;
- standalone/Primary/Secondary role specification;
- initial configuration schema;
- initial policy schema;
- management protocol design;
- PKI/enrollment design;
- storage design;
- privacy model;
- least-privilege/service design;
- initial repository layout proposal;
- test strategy;
- promotion plan out of `future-projects`.

Acceptance:

- no production DNS listener required;
- architecture documentation internally consistent;
- clear security boundaries;
- no dependency-license conflict;
- all unresolved high-risk decisions explicitly listed.

## Phase 0.1 — Protocol and Schema Prototypes

Deliver:

- Rust types for policy manifest;
- revision identity;
- node identity;
- effective-policy model;
- validation tests;
- serialization compatibility tests;
- no network deployment yet.

Acceptance:

- malformed manifests rejected;
- deterministic serialization strategy documented;
- schema version policy documented.

## Phase 1 — Core DNS Runtime

Deliver:

- DNS service executable;
- UDP/53;
- TCP/53;
- forwarding;
- bounded cache;
- upstream configuration;
- graceful start/stop;
- health endpoint/IPC;
- basic metrics;
- port-conflict diagnostics.

Acceptance:

- standard A/AAAA queries pass;
- UDP/TCP behavior verified;
- timeout/fallback behavior tested;
- malformed input does not crash service;
- restart preserves configuration;
- benchmark baseline captured.

## Phase 1.1 — Runtime Hardening

Deliver:

- concurrency bounds;
- per-client defensive rate controls;
- message/time bounds;
- cache limits;
- DNS amplification/open-resolver exposure review;
- fuzz targets;
- service crash-recovery behavior.

Acceptance:

- abuse tests pass;
- no unbounded state identified in reviewed paths;
- Windows validation passes.

## Phase 2 — Filtering and Effective Policy Engine

Deliver:

- allowlist;
- denylist;
- external compiled block rules;
- deterministic precedence;
- NULL/NXDOMAIN or selected blocking mode;
- explainability metadata;
- policy compiler.

Acceptance:

- precedence exhaustively unit tested;
- false allow/deny regression fixtures;
- hot path does not depend on SQL;
- million-domain benchmark recorded.

## Phase 2.1 — Local DNS

Deliver:

- A/AAAA/CNAME;
- reverse/PTR support where appropriate;
- validation;
- collision/precedence rules.

Acceptance:

- local-zone fixtures pass;
- invalid records rejected;
- forwarding fallback behavior verified.

## Phase 3 — Blocklist Lifecycle

Deliver:

- remote source configuration;
- safe downloader;
- formats;
- normalization;
- deduplication;
- size/line bounds;
- staging;
- compile;
- atomic activation;
- previous-known-good retention;
- update history.

Acceptance:

- partial/corrupt downloads never replace active rules;
- hostile parser fixtures pass;
- update rollback works.

## Phase 3.1 — Scheduled Refresh

Deliver:

- scheduled updates;
- jitter/backoff;
- failure counters;
- health reporting.

Acceptance:

- repeated source outage does not impact DNS;
- scheduler has bounded retry behavior.

## Phase 4 — Immutable Policy Revision System

Deliver:

- revision IDs;
- policy digests;
- manifest;
- artifact digests;
- staged activation;
- active/previous pointers;
- rollback.

Acceptance:

- atomicity tested under simulated crash points;
- corrupt staged revision ignored;
- rollback verified.

## Phase 4.1 — Signed Policy Bundles

Deliver:

- signing format;
- signer identity;
- verification;
- anti-downgrade rules;
- key-rotation design.

Acceptance:

- modified policy rejected;
- wrong signer rejected;
- wrong organization/scope rejected;
- authorized rollback path remains possible.

## Phase 5 — Primary Management Service

Deliver:

- controller service;
- organization/site/node model;
- policy authoring API;
- revision publication;
- node inventory;
- synchronization metadata;
- audit records.

Acceptance:

- controller can publish immutable revision;
- no DNS-node dependency required on controller host;
- API authentication model documented/tested.

## Phase 5.1 — Enrollment and Node Identity

Deliver:

- short-lived enrollment token;
- one-time use;
- per-node credential/certificate;
- revocation;
- identity persistence.

Acceptance:

- replayed enrollment rejected;
- expired token rejected;
- revoked node rejected.

## Phase 5.2 — Secure Synchronization

Deliver:

- authenticated encrypted channel;
- push notification;
- periodic pull/reconciliation;
- manifest download;
- policy application/reporting;
- reconnect/backoff.

Acceptance:

- missed push heals through pull;
- Primary outage does not affect DNS;
- reconnect applies current authorized revision;
- malformed management messages bounded/rejected.

## Phase 6 — Managed Policy Hierarchy

Deliver:

- organization/site/node layers;
- ENFORCED/DEFAULT/LOCAL ownership;
- deterministic merge;
- locked-setting metadata.

Acceptance:

- policy merge fixtures;
- unauthorized local override rejected;
- allowed local override persists correctly.

## Phase 6.1 — Offline and Stale Management Behavior

Deliver:

- offline states;
- stale warnings;
- frozen policy behavior;
- controlled unenrollment.

Acceptance:

- extended controller outage test;
- restart while offline retains DNS operation;
- stale policy remains valid unless cryptographically/configuration-invalid.

## Phase 7 — Standalone DNS Shield UI

Deliver:

- dashboard;
- query/metrics view;
- list management;
- local DNS;
- upstream configuration;
- privacy controls;
- service controls;
- diagnostics.

Acceptance:

- UI can close without stopping DNS;
- UI never bypasses validation;
- sensitive data handling reviewed.

## Phase 7.1 — Primary Administration UI

Deliver:

- organizations/sites/nodes;
- enrollment workflow;
- policy editor;
- revision history;
- deployment status;
- rollback;
- audit viewer.

Acceptance:

- admin operations map to typed management APIs;
- no generic remote execution.

## Phase 7.2 — Managed Secondary UI

Deliver:

- managed status;
- controller identity;
- synchronization status;
- locked settings;
- active revision;
- offline/stale visibility.

Acceptance:

- centrally locked settings cannot be changed through UI/API.

## Phase 8 — Windows Service and Installer

Deliver:

- Windows service install;
- least-privilege identity;
- protected directories;
- service lifecycle;
- port conflict diagnostics;
- uninstall behavior;
- standalone installer option.

Acceptance:

- clean install/uninstall test;
- reboot persistence;
- UI-independent DNS;
- no unauthorized ACL access.

## Phase 8.1 — Typed Firewall Integration

Deliver:

- exact DragonForge DNS inbound listener rules;
- TCP/UDP 53;
- restricted scope;
- ownership/rollback;
- privileged boundary review.

Acceptance:

- only DragonForge-owned rules mutated;
- unrelated firewall policy untouched;
- rule removal/rollback verified.

## Phase 9 — Security Suite Integration

Deliver:

- Security Center component registration;
- Agent integration where needed;
- event hub events;
- diagnostics;
- backup/recovery integration;
- release/update integration.

Acceptance:

- standalone packaging still works;
- suite absence does not affect standalone DNS;
- event volume bounded.

## Phase 9.1 — Release Engineering

Deliver:

- component versioning;
- package integrity;
- signed Windows release integration;
- upgrade compatibility;
- migration tests.

Acceptance:

- upgrade does not lose active policy;
- failed upgrade recovers cleanly.

## Phase 10 — Encrypted Upstream DNS and DNSSEC

Deliver as separately reviewed features:

- DoT;
- DoH;
- DNSSEC;
- optional DoQ if dependency maturity is acceptable.

Acceptance:

- bootstrap/certificate behavior tested;
- fallback semantics explicit;
- no silent downgrade when policy forbids it.

## Phase 11 — Advanced Client Policy

Deliver:

- client groups;
- site groups;
- categories;
- wildcard rules;
- carefully bounded regex if retained;
- schedules;
- temporary overrides;
- explainability.

Acceptance:

- deterministic evaluation;
- performance remains acceptable;
- ReDoS risk eliminated/bounded.

## Phase 12 — Deployment Rings and Fleet Operations

Deliver:

- canary rollout;
- staged percentages/groups;
- deployment pause;
- rollout status;
- targeted rollback;
- optional conservative health rollback.

Acceptance:

- failed canary does not continue automatically unless policy explicitly permits;
- rollout state survives controller restart.

## Phase 13 — Cross-Platform Service

Deliver:

- Linux/systemd support;
- filesystem permissions;
- service packaging;
- Linux validation.

Optional separate subphase:

- Docker/OCI packaging.

Acceptance:

- same policy semantics as Windows;
- no Windows-only assumptions in shared core.

## Phase 14 — Threat Intelligence Integration

Deliver:

- known-malicious domain feeds;
- signed/trusted feed metadata;
- bounded categories;
- Security Center aggregate security events.

Acceptance:

- feed outage does not break DNS;
- provenance visible.

## Phase 15 — Network Guard Correlation

Goal: safely combine local process/network observations with DNS events where evidence allows.

Do not claim network-wide process attribution.

Deliver only after a separate privacy/threat analysis.

## Phase 16 — Optional DHCP

DHCP is explicitly deferred.

Before implementation:

- separate threat model;
- conflict detection;
- rollback/recovery plan;
- warning that two DHCP servers can disrupt a network;
- optional component architecture.

## Phase 17 — Controller High Availability

Deferred enterprise feature.

Potential work:

- replicated administration state;
- leader election/consensus;
- certificate authority availability;
- failover.

Do not introduce distributed consensus early.

## Phase 18 — 1.0 Security Audit and Qualification

Deliver:

- repository-wide security audit;
- protocol review;
- privilege review;
- DNS abuse review;
- fuzzing campaign;
- dependency audit;
- recovery exercises;
- upgrade/downgrade tests;
- external testing matrix;
- performance qualification.

## Phase 19 — DragonForge DNS Shield 1.0

Requirements:

- standalone node production-ready;
- Primary management production-ready;
- Managed Secondary production-ready;
- offline behavior validated;
- signed policy validated;
- installer/update/recovery validated;
- documentation complete;
- security audit blockers resolved;
- release artifacts signed and reproducible/traceable per suite standards.

---

# 27. VALIDATION STRATEGY

Each implementation phase should provide:

- `cargo fmt --check`;
- locked dependency checks;
- full relevant workspace compile;
- strict Clippy with warnings denied;
- unit tests;
- integration tests;
- doc tests where applicable;
- platform-specific tests;
- architecture invariant checks;
- PowerShell syntax validation for Windows scripts;
- authoritative validation transcript;
- SHA-256 sidecar for authoritative logs where consistent with suite practice.

DNS-specific fixtures should include known-good packet/query fixtures and malformed/adversarial inputs.

---

# 28. PROMOTION FROM FUTURE PROJECT TO ACTIVE PROJECT

Do not simply move files from this folder into the workspace.

Promotion requires an explicit change that:

1. declares the project active;
2. creates the approved crate/service/app directories;
3. adds only the Phase 0/0.1 workspace members required;
4. adds docs under `docs/dns-shield/`;
5. preserves this roadmap as planning history;
6. updates root architecture documentation;
7. updates roadmap references;
8. adds CI only for implemented components;
9. does not add unfinished binaries to stable release artifacts;
10. does not expand privileged capabilities before their reviewed phase.

---

# 29. CURRENT NEXT ACTION

When development is authorized, begin with:

**Phase 0 — Architecture, Threat Model, Dependency Review**

Do not start by opening port 53.

The first implementation session should convert the design assumptions in this roadmap into explicit ADRs and versioned Rust schema/protocol types, then validate them before any production DNS listener is introduced.

---

# 30. DEFINITION OF SUCCESS

DragonForge DNS Shield succeeds when:

- one user can install it on one PC and receive reliable local DNS filtering;
- a home user can use it as a LAN DNS server;
- an administrator can centrally manage many DNS nodes;
- downstream nodes adapt automatically to authorized policy changes;
- downstream nodes keep resolving DNS if the Primary disappears;
- bad policy cannot silently take down the fleet;
- policy provenance and rollback are clear;
- DNS privacy is configurable and bounded;
- the DNS runtime stays least-privileged;
- Security Suite integration enhances the product without becoming a dependency;
- the codebase remains auditable, testable, and understandable without hidden conversational context.
