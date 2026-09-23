# Phase 19 — Network Policy & Firewall Integration

**Status: Verified Complete**

Phase 19 extends the verified Phase 8 Network Guard and Phase 17 privileged-service boundary with controlled Windows Firewall application policy. The phase is deliberately narrow: DragonForge may create, replace, remove, or roll back only its own outbound per-application rules through a fixed typed protocol. It does not expose arbitrary firewall command text or generic privileged execution.

## Delivered

- typed Phase 19 firewall request/response contract in the Windows security boundary;
- protocol minor-version evolution for fixed firewall status/apply/remove/rollback operations;
- only `FirewallPolicyMutation` enabled as a Phase 19 privileged capability;
- exact signed-Agent caller authorization retained from Phase 17;
- Windows Firewall COM integration through `INetFwPolicy2`, `INetFwRules`, and `INetFwRule`;
- fail-closed mutation when Windows reports that local firewall policy changes will not take effect (for example, Group Policy override), while read-only status remains available;
- outbound-only rules across all Windows Firewall profiles;
- explicit allow or block action;
- exact executable path plus SHA-256 application identity;
- privileged-service re-hashing immediately before mutation;
- symbolic-link rejection and bounded executable hashing;
- explicit refusal to target `dragonforge-agent.exe` or `dragonforge-privileged-service.exe`, preserving the local authenticated control/rollback path;
- deterministic DragonForge rule namespace and fixed DragonForge grouping;
- refusal to overwrite a same-name rule that is not already owned by DragonForge state;
- bounded protected service state for at most 256 managed policies and 64 rollback records;
- rollback tokens derived from the already replay-protected 128-bit service request nonce;
- crash-recoverable service-owned policy state under the existing protected ProgramData service directory;
- Network Guard application selection from observed PIDs;
- Network Guard policy UI for status, allow, block, remove, and rollback;
- all mutation traffic routed through the exact sibling DragonForge Agent;
- no direct Network Guard access to the privileged named pipe;
- no packet-payload capture or connection termination.

## Trust path

1. Network Guard observes a process and receives a numeric PID.
2. Network Guard resolves that PID through a fixed Windows process query and hashes the executable locally.
3. Network Guard launches only the exact sibling `dragonforge-agent.exe` with one fixed firewall CLI verb and typed identity arguments.
4. The Agent serializes the request into the Phase 19 privileged protocol.
5. The privileged service authenticates the connected Agent using the Phase 17 operating-system peer identity, exact executable path, Authenticode verification, and pinned publisher.
6. Replay, timestamp, message-size, and request-rate protections are applied.
7. The privileged service validates the typed firewall payload and independently re-hashes the requested executable.
8. Only then does the service use the native Windows Firewall COM API to mutate its own deterministic rule.

The UI never receives a generic privileged command facility.

## Firewall scope

Phase 19 managed rules are intentionally constrained to outbound direction, all Windows Firewall profiles, one absolute executable path, allow or block action, enabled state, and the fixed `DragonForge Security Suite` grouping with the `DragonForge Outbound ` name prefix.

The phase does not change global firewall enabled state, default inbound/outbound action, notification settings, unrelated rule groups, service restrictions, or local firewall defaults. Before a mutation, the service checks `INetFwPolicy2::LocalPolicyModifyState` and refuses the operation unless Windows reports `NET_FW_MODIFY_STATE_OK`.

## Application identity

Network Guard obtains the PID, process name, absolute executable path, and SHA-256 executable hash. The privileged service does not trust the client-supplied hash by itself. It independently verifies that the target is a regular non-symlink file, enforces the file-size ceiling, re-hashes the executable, and requires an exact SHA-256 match before any apply, remove, rollback, or status operation.

Windows Firewall itself scopes this class of rule by executable path rather than continuously enforcing a file hash. DragonForge therefore records the SHA-256 as authorization and drift context, reports whether the currently observed binary still matches the recorded identity, and revalidates the current hash before each privileged policy operation. If the executable changes between client inspection and privileged execution, the operation fails closed. The Agent and privileged-service executables themselves are ineligible for Phase 19 policy so the control/rollback plane cannot be accidentally firewalled by this feature.

## Rule ownership and rollback

The rule name is derived from a SHA-256 digest of the lower-cased application path and uses the DragonForge namespace. If Windows Firewall already contains that deterministic name but service state does not own it, the operation is rejected instead of overwriting it. Removal verifies DragonForge grouping before deletion.

Every successful apply/remove mutation can return a bounded rollback token. Rollback restores the previous DragonForge-managed policy or removes the newly created rule when no previous policy existed. At most 64 rollback records are retained.

## Safety bounds

- managed policies: 256 maximum;
- rollback records: 64 maximum;
- service firewall state: 1 MiB maximum;
- application executable hash input: 1 GiB maximum;
- application path: 2048 characters maximum;
- display name: 120 characters maximum;
- SHA-256: exactly 64 hexadecimal characters;
- rollback token: exactly 32 hexadecimal characters;
- existing Phase 17 message, freshness, replay, and request-rate limits remain active.

## Privileged capability policy

Phase 19 enables only `FirewallPolicyMutation`. Protected process control, protected file quarantine, protected registry remediation, and system integrity remediation remain denied. Generic shell execution and arbitrary command execution remain prohibited.

## Native API boundary

Firewall mutation uses the Windows Firewall COM API. The privileged service does not invoke `powershell.exe`, `cmd.exe`, `netsh.exe`, arbitrary scripts, or user-supplied command lines for firewall mutation. The native backend verifies the resulting rule's application path, group, direction, profiles, enabled state, and action after creation.

## Non-claims

Phase 19 does not claim packet filtering through a custom WFP kernel callout, packet payload inspection, connection termination, inbound custom rule authoring, arbitrary port/protocol/address rule composition, third-party firewall replacement, anti-malware classification, or management of rules DragonForge does not own.

## Verification

Run on Windows:

`powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase19-network-policy-firewall-tests.ps1`

The verifier writes a transcript and matching SHA-256 sidecar under `test-logs\`. Phase 19 remains **Implementation Complete — Local Verification Pending** until the authoritative Windows verifier reaches its PASS marker and the sidecar is independently checked.


## Authoritative local verification

Phase 19 was verified on 2026-09-23 on DRACO (Microsoft Windows NT 10.0.26200.0) using Windows PowerShell 5.1.26100.9444 with `scripts/run-phase19-network-policy-firewall-tests.ps1`.

The authoritative verifier passed formatting, locked Cargo metadata, full workspace checking, strict Clippy, the complete workspace and doc-test suite, Network Guard JavaScript syntax validation, Phase 19 architecture/security invariants, PowerShell syntax validation, and the complete Windows application build including `dragonforge-agent.exe` and `dragonforge-privileged-service.exe`.

Final verifier marker:

`PHASE 19 NETWORK POLICY & FIREWALL INTEGRATION VERIFICATION: PASS`

Authoritative log SHA-256:

`1BF31EC9E7D4CEB2D935EF9C2EB137F9EBD5074EA7A6AFD82B24CC1902AA6E66`
