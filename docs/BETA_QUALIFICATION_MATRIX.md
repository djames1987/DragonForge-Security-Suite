# DragonForge Beta Qualification Matrix

Phase 13 turns the Phase 12 external-test matrix into explicit beta evidence. A single passing development machine is necessary but is not sufficient to claim the full beta matrix is complete.

## Required qualification scenarios

| ID | Windows | Machine | User context | Profile | WebView2 | Package | Reboot | Required |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| BQ-01 | Windows 11 supported/current | Physical | Standard user | Established | Present | Installer | Yes | Yes |
| BQ-02 | Windows 11 supported/current | VM or second physical | Standard user | Fresh | Present | Installer | Yes | Yes |
| BQ-03 | Windows 10 22H2 | VM or physical | Standard user | Fresh or established | Present | Portable | Yes | Yes |
| BQ-04 | Windows 11 supported/current | Disposable VM | Standard user | Fresh | Missing/unavailable | Installer or portable | No | Yes |
| BQ-05 | Windows 11 supported/current | Physical or VM | Administrator account, apps not elevated | Established | Present | Installer | Yes | Yes |
| BQ-06 | Windows 10/11 | VM or physical | Standard user | Established | Present | Portable in alternate writable drive/folder | No | Yes |

## Evidence required for every scenario

Each scenario must retain:
- the Phase 13 automated qualification `.log` and matching `.sha256`;
- a machine-readable qualification JSON produced by `scripts/new-beta-qualification-record.ps1`;
- exact DragonForge commit or release tag;
- Windows edition/version/build;
- architecture;
- package kind and location class;
- user context without recording usernames;
- fresh/established profile classification;
- physical/VM classification;
- WebView2 present/missing classification;
- Agent start/reconnect/stop result;
- all nine desktop component launch result;
- reboot/sign-in result when the row requires it;
- installer fresh-install/upgrade/uninstall result where applicable;
- confirmation that disposable data only was used.

## Release-blocking failures

A beta candidate is blocked by:
- reproducible data loss;
- confirmed secret leakage;
- package checksum or Authenticode failure when signing is required;
- failure to launch Security Center or any integrated sibling app on a required supported scenario;
- Agent authentication/lifecycle failure that persists after clean reinstall/reboot;
- installer upgrade deleting or corrupting user data;
- uninstall deleting user-created DragonForge data outside the program directory;
- malformed/tampered encrypted package acceptance;
- release artifact/tag/commit identity mismatch;
- a current applicable RustSec vulnerability without documented mitigation.

## Advisory findings

These do not automatically block beta unless they prevent ordinary use:
- cosmetic layout defects;
- SmartScreen reputation warnings with otherwise valid signing;
- expected WebView2 missing-runtime warning;
- visibility probes returning documented Unknown/Unavailable states;
- known non-claims already documented in the security model.

## Completion rule

Phase 13 implementation can be verified on one authoritative Windows development machine. **Beta qualification itself is complete only when all required BQ rows have evidence and no release-blocking defect remains open.**
