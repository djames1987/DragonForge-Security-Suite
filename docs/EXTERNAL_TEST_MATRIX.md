# Phase 12.0 External Windows Test Matrix

Phase 12.0 does not claim every matrix cell has already been executed. This matrix defines the external coverage required during the alpha cycle.

| Dimension | Required coverage |
| --- | --- |
| Windows | Windows 10 22H2; Windows 11 current supported release |
| Hardware | At least one physical machine and one VM |
| User context | Standard user; administrator user running apps without elevation |
| Profile | Fresh Windows user profile; established profile |
| WebView2 | Present/current; missing or intentionally unavailable in disposable VM |
| Package location | User Downloads/extracted folder; Desktop; alternate writable drive/folder |
| Agent | Start; reconnect; Security Center close/reopen; stop; stale-runtime recovery |
| Reboot | Reboot with Agent stopped; reboot after Agent was running |
| Networking | Offline; normal Internet; Password Manager loopback sync; HTTPS remote sync when available |
| Display | 100%; 125%+ scaling |
| Data paths | Normal writable user path; controlled failure on unavailable/unwritable target |
| Upgrade lineage | Fresh alpha install/extract; later alpha side-by-side testing rather than overwriting a frozen release |

## Exit criteria for Phase 12 stabilization cycle

Before moving to a stable beta/installable channel:

- no reproducible data-loss defect remains open;
- no confirmed secret-leakage defect remains open;
- every suite executable launches from Security Center on supported Windows test machines;
- Agent lifecycle survives repeat close/reopen and stale-runtime scenarios;
- portable package integrity verification passes after download;
- diagnostic reports remain redaction-safe;
- critical package/container parsers retain regression coverage for malformed/tampered input;
- known limitations are reflected in release notes.


## Phase 13 qualification

The Phase 12 matrix remains the broad testing baseline. Phase 13 makes the required beta subset explicit in [BETA_QUALIFICATION_MATRIX.md](BETA_QUALIFICATION_MATRIX.md) and requires retained automated logs, SHA-256 sidecars, and machine-readable qualification records for each required scenario before a beta label is used.
