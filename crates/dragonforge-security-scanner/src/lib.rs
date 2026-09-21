#![forbid(unsafe_code)]

//! Read-only local security posture assessment for DragonForge Security Scanner.
//!
//! The scanner executes only fixed platform probes. It does not accept shell
//! fragments from callers, modify system configuration, or require the future
//! privileged DragonForge Agent.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

const MAX_LISTENERS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Pass,
    Attention,
    Unknown,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Probe {
    Firewall,
    BitLocker,
    UpdateService,
    LatestHotfix,
    Defender,
    SecureBoot,
    Uac,
    Smb1,
    RemoteDesktop,
    Listeners,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub id: &'static str,
    pub category: &'static str,
    pub title: &'static str,
    pub severity: FindingSeverity,
    pub status: FindingStatus,
    pub summary: String,
    pub evidence: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanSummary {
    pub pass: usize,
    pub attention: usize,
    pub unknown: usize,
    pub informational: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanReport {
    pub scanner_version: u16,
    pub platform: &'static str,
    pub timestamp_ms: u64,
    pub summary: ScanSummary,
    pub findings: Vec<Finding>,
}

trait ProbeRunner {
    fn run(&self, probe: Probe) -> Result<String, String>;
}

#[cfg(target_os = "windows")]
struct PlatformRunner;

#[cfg(target_os = "windows")]
impl ProbeRunner for PlatformRunner {
    fn run(&self, probe: Probe) -> Result<String, String> {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        let script = match probe {
            Probe::Firewall => {
                r#"Get-NetFirewallProfile | ForEach-Object { "$($_.Name)=$($_.Enabled)" }"#
            }
            Probe::BitLocker => {
                r#"try { (Get-BitLockerVolume -MountPoint $env:SystemDrive).ProtectionStatus } catch { "Unknown" }"#
            }
            Probe::UpdateService => {
                r#"try { Get-Service wuauserv | ForEach-Object { "$($_.Status)|$($_.StartType)" } } catch { "Unknown" }"#
            }
            Probe::LatestHotfix => {
                r#"try { Get-HotFix | Sort-Object InstalledOn -Descending | Select-Object -First 1 | ForEach-Object { "$($_.HotFixID)|$($_.InstalledOn.ToString('yyyy-MM-dd'))" } } catch { "Unknown" }"#
            }
            Probe::Defender => {
                r#"try { Get-MpComputerStatus | ForEach-Object { "$($_.AntivirusEnabled)|$($_.RealTimeProtectionEnabled)|$($_.BehaviorMonitorEnabled)" } } catch { "Unknown" }"#
            }
            Probe::SecureBoot => {
                r#"try { if (Confirm-SecureBootUEFI) { "True" } else { "False" } } catch { "Unknown" }"#
            }
            Probe::Uac => {
                r#"try { (Get-ItemProperty 'HKLM:SOFTWAREMicrosoftWindowsCurrentVersionPoliciesSystem' -Name EnableLUA).EnableLUA } catch { "Unknown" }"#
            }
            Probe::Smb1 => {
                r#"try { (Get-WindowsOptionalFeature -Online -FeatureName SMB1Protocol).State } catch { "Unknown" }"#
            }
            Probe::RemoteDesktop => {
                r#"try { (Get-ItemProperty 'HKLM:SYSTEMCurrentControlSetControlTerminal Server' -Name fDenyTSConnections).fDenyTSConnections } catch { "Unknown" }"#
            }
            Probe::Listeners => {
                r#"try { Get-NetTCPConnection -State Listen | Sort-Object LocalPort,LocalAddress -Unique | ForEach-Object { "$($_.LocalAddress)|$($_.LocalPort)" } } catch { "Unknown" }"#
            }
        };

        let output = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|_| "platform probe could not be started".to_owned())?;

        if !output.status.success() {
            return Err("platform probe did not complete successfully".to_owned());
        }

        String::from_utf8(output.stdout)
            .map(|value| value.trim().to_owned())
            .map_err(|_| "platform probe returned invalid text".to_owned())
    }
}

#[cfg(not(target_os = "windows"))]
struct PlatformRunner;

#[cfg(not(target_os = "windows"))]
impl ProbeRunner for PlatformRunner {
    fn run(&self, _probe: Probe) -> Result<String, String> {
        Err("this Phase 6 scanner currently supports Windows posture probes".to_owned())
    }
}

#[must_use]
pub fn scan_system() -> ScanReport {
    scan_with_runner(&PlatformRunner)
}

fn scan_with_runner(runner: &impl ProbeRunner) -> ScanReport {
    let mut findings = Vec::new();

    if cfg!(target_os = "windows") {
        findings.push(check_firewall(runner));
        findings.push(check_bitlocker(runner));
        findings.push(check_update_service(runner));
        findings.push(check_latest_hotfix(runner));
        findings.push(check_defender(runner));
        findings.push(check_secure_boot(runner));
        findings.push(check_uac(runner));
        findings.push(check_smb1(runner));
        findings.push(check_remote_desktop(runner));
        findings.push(check_listeners(runner));
    } else {
        findings.push(Finding {
            id: "platform.unsupported",
            category: "Platform",
            title: "Platform support",
            severity: FindingSeverity::Info,
            status: FindingStatus::Unknown,
            summary: "Phase 6 posture probes are currently Windows-first.".to_owned(),
            evidence: "No platform-specific probes were executed.".to_owned(),
            recommendation: "Use the scanner on Windows; additional platform probes are planned later."
                .to_owned(),
        });
    }

    ScanReport {
        scanner_version: 1,
        platform: if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "unknown"
        },
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis().min(u128::from(u64::MAX)) as u64),
        summary: summarize(&findings),
        findings,
    }
}

fn summarize(findings: &[Finding]) -> ScanSummary {
    let mut summary = ScanSummary {
        pass: 0,
        attention: 0,
        unknown: 0,
        informational: 0,
    };
    for finding in findings {
        match finding.status {
            FindingStatus::Pass => summary.pass += 1,
            FindingStatus::Attention => summary.attention += 1,
            FindingStatus::Unknown => summary.unknown += 1,
            FindingStatus::Info => summary.informational += 1,
        }
    }
    summary
}

fn output(runner: &impl ProbeRunner, probe: Probe) -> Option<String> {
    runner
        .run(probe)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("unknown"))
}

fn unknown(
    id: &'static str,
    category: &'static str,
    title: &'static str,
    recommendation: &'static str,
) -> Finding {
    Finding {
        id,
        category,
        title,
        severity: FindingSeverity::Info,
        status: FindingStatus::Unknown,
        summary: "This posture check could not be determined with the current user context.".to_owned(),
        evidence: "The fixed local probe returned no usable result.".to_owned(),
        recommendation: recommendation.to_owned(),
    }
}

fn check_firewall(runner: &impl ProbeRunner) -> Finding {
    let Some(value) = output(runner, Probe::Firewall) else {
        return unknown(
            "firewall.profiles",
            "Firewall",
            "Windows Firewall profiles",
            "Review Windows Defender Firewall profile state manually.",
        );
    };
    let profiles: Vec<_> = value
        .lines()
        .filter_map(|line| line.split_once('='))
        .collect();
    if profiles.is_empty() {
        return unknown(
            "firewall.profiles",
            "Firewall",
            "Windows Firewall profiles",
            "Review Windows Defender Firewall profile state manually.",
        );
    }
    let disabled: Vec<_> = profiles
        .iter()
        .filter(|(_, enabled)| !enabled.trim().eq_ignore_ascii_case("true"))
        .map(|(name, _)| name.trim())
        .collect();
    if disabled.is_empty() {
        Finding {
            id: "firewall.profiles",
            category: "Firewall",
            title: "Windows Firewall profiles",
            severity: FindingSeverity::Low,
            status: FindingStatus::Pass,
            summary: "All reported Windows Firewall profiles are enabled.".to_owned(),
            evidence: format!("Profiles checked: {}.", profiles.len()),
            recommendation: "Keep firewall profiles enabled and review inbound rules periodically."
                .to_owned(),
        }
    } else {
        Finding {
            id: "firewall.profiles",
            category: "Firewall",
            title: "Windows Firewall profiles",
            severity: FindingSeverity::High,
            status: FindingStatus::Attention,
            summary: "One or more Windows Firewall profiles are disabled.".to_owned(),
            evidence: format!("Disabled profiles: {}.", disabled.join(", ")),
            recommendation: "Enable the disabled firewall profiles unless a documented network policy requires otherwise."
                .to_owned(),
        }
    }
}

fn check_bitlocker(runner: &impl ProbeRunner) -> Finding {
    let Some(value) = output(runner, Probe::BitLocker) else {
        return unknown(
            "encryption.system-volume",
            "Disk encryption",
            "System volume protection",
            "Confirm full-disk encryption status in Windows settings or BitLocker management.",
        );
    };
    let protected = value.eq_ignore_ascii_case("on")
        || value.eq_ignore_ascii_case("1")
        || value.eq_ignore_ascii_case("protectionon");
    Finding {
        id: "encryption.system-volume",
        category: "Disk encryption",
        title: "System volume protection",
        severity: if protected {
            FindingSeverity::Low
        } else {
            FindingSeverity::High
        },
        status: if protected {
            FindingStatus::Pass
        } else {
            FindingStatus::Attention
        },
        summary: if protected {
            "The system volume reports BitLocker protection enabled."
        } else {
            "The system volume does not report active BitLocker protection."
        }
        .to_owned(),
        evidence: format!("Protection status: {value}."),
        recommendation: if protected {
            "Keep recovery material protected and verify recovery procedures."
        } else {
            "Consider enabling supported Windows device encryption or BitLocker after confirming recovery-key storage."
        }
        .to_owned(),
    }
}

fn check_update_service(runner: &impl ProbeRunner) -> Finding {
    let Some(value) = output(runner, Probe::UpdateService) else {
        return unknown(
            "updates.service",
            "Updates",
            "Windows Update service",
            "Confirm Windows Update policy and service configuration manually.",
        );
    };
    let parts: Vec<_> = value.split('|').map(str::trim).collect();
    let disabled = parts
        .get(1)
        .is_some_and(|start_type| start_type.eq_ignore_ascii_case("disabled"));
    Finding {
        id: "updates.service",
        category: "Updates",
        title: "Windows Update service",
        severity: if disabled {
            FindingSeverity::Medium
        } else {
            FindingSeverity::Low
        },
        status: if disabled {
            FindingStatus::Attention
        } else {
            FindingStatus::Pass
        },
        summary: if disabled {
            "Windows Update is configured with a disabled service startup type."
        } else {
            "Windows Update is not reported as administratively disabled."
        }
        .to_owned(),
        evidence: format!("Service state: {value}."),
        recommendation: if disabled {
            "Review update policy and restore Windows Update unless it is intentionally managed another way."
        } else {
            "Continue installing security updates on a regular schedule."
        }
        .to_owned(),
    }
}

fn check_latest_hotfix(runner: &impl ProbeRunner) -> Finding {
    let Some(value) = output(runner, Probe::LatestHotfix) else {
        return unknown(
            "updates.latest-hotfix",
            "Updates",
            "Latest installed hotfix",
            "Open Windows Update and review update history and pending updates.",
        );
    };
    Finding {
        id: "updates.latest-hotfix",
        category: "Updates",
        title: "Latest installed hotfix",
        severity: FindingSeverity::Info,
        status: FindingStatus::Info,
        summary: "The latest installed hotfix was discovered for review.".to_owned(),
        evidence: value,
        recommendation:
            "Compare this information with Windows Update; this check does not claim that no newer update is available."
                .to_owned(),
    }
}

fn check_defender(runner: &impl ProbeRunner) -> Finding {
    let Some(value) = output(runner, Probe::Defender) else {
        return unknown(
            "defender.realtime",
            "Malware protection",
            "Microsoft Defender protection",
            "Confirm that a supported antimalware provider has active real-time protection.",
        );
    };
    let values: Vec<_> = value.split('|').map(str::trim).collect();
    let all_enabled = values.len() >= 3
        && values
            .iter()
            .take(3)
            .all(|item| item.eq_ignore_ascii_case("true"));
    Finding {
        id: "defender.realtime",
        category: "Malware protection",
        title: "Microsoft Defender protection",
        severity: if all_enabled {
            FindingSeverity::Low
        } else {
            FindingSeverity::High
        },
        status: if all_enabled {
            FindingStatus::Pass
        } else {
            FindingStatus::Attention
        },
        summary: if all_enabled {
            "Microsoft Defender reports antivirus, real-time, and behavior monitoring enabled."
        } else {
            "One or more Microsoft Defender protection signals are not enabled."
        }
        .to_owned(),
        evidence: format!("Protection signals: {value}."),
        recommendation: if all_enabled {
            "Keep the installed antimalware provider updated."
        } else {
            "Confirm whether another security provider is intentionally active; otherwise restore real-time protection."
        }
        .to_owned(),
    }
}

fn check_secure_boot(runner: &impl ProbeRunner) -> Finding {
    let Some(value) = output(runner, Probe::SecureBoot) else {
        return unknown(
            "boot.secure-boot",
            "Boot security",
            "Secure Boot",
            "Confirm Secure Boot state in Windows System Information or firmware settings.",
        );
    };
    let enabled = value.eq_ignore_ascii_case("true");
    Finding {
        id: "boot.secure-boot",
        category: "Boot security",
        title: "Secure Boot",
        severity: if enabled {
            FindingSeverity::Low
        } else {
            FindingSeverity::Medium
        },
        status: if enabled {
            FindingStatus::Pass
        } else {
            FindingStatus::Attention
        },
        summary: if enabled {
            "Secure Boot reports enabled."
        } else {
            "Secure Boot reports disabled."
        }
        .to_owned(),
        evidence: format!("Secure Boot: {value}."),
        recommendation: if enabled {
            "Keep firmware and platform security updates current."
        } else {
            "If supported by the device and operating system configuration, consider enabling Secure Boot."
        }
        .to_owned(),
    }
}

fn check_uac(runner: &impl ProbeRunner) -> Finding {
    let Some(value) = output(runner, Probe::Uac) else {
        return unknown(
            "account-control.uac",
            "Account control",
            "User Account Control",
            "Confirm User Account Control is enabled in Windows security settings.",
        );
    };
    let enabled = value.trim() == "1";
    Finding {
        id: "account-control.uac",
        category: "Account control",
        title: "User Account Control",
        severity: if enabled {
            FindingSeverity::Low
        } else {
            FindingSeverity::High
        },
        status: if enabled {
            FindingStatus::Pass
        } else {
            FindingStatus::Attention
        },
        summary: if enabled {
            "User Account Control is enabled."
        } else {
            "User Account Control appears disabled."
        }
        .to_owned(),
        evidence: format!("EnableLUA: {value}."),
        recommendation: if enabled {
            "Keep UAC enabled and use elevation only when necessary."
        } else {
            "Re-enable UAC unless a documented enterprise control provides an equivalent boundary."
        }
        .to_owned(),
    }
}

fn check_smb1(runner: &impl ProbeRunner) -> Finding {
    let Some(value) = output(runner, Probe::Smb1) else {
        return unknown(
            "legacy.smb1",
            "Legacy protocols",
            "SMB1 optional feature",
            "Confirm the SMB1 optional feature is disabled unless legacy interoperability is explicitly required.",
        );
    };
    let enabled = value.eq_ignore_ascii_case("enabled");
    Finding {
        id: "legacy.smb1",
        category: "Legacy protocols",
        title: "SMB1 optional feature",
        severity: if enabled {
            FindingSeverity::High
        } else {
            FindingSeverity::Low
        },
        status: if enabled {
            FindingStatus::Attention
        } else {
            FindingStatus::Pass
        },
        summary: if enabled {
            "The legacy SMB1 optional feature is enabled."
        } else {
            "The SMB1 optional feature is not reported as enabled."
        }
        .to_owned(),
        evidence: format!("SMB1 state: {value}."),
        recommendation: if enabled {
            "Disable SMB1 unless a documented legacy dependency requires it."
        } else {
            "Keep SMB1 disabled."
        }
        .to_owned(),
    }
}

fn check_remote_desktop(runner: &impl ProbeRunner) -> Finding {
    let Some(value) = output(runner, Probe::RemoteDesktop) else {
        return unknown(
            "remote-access.rdp",
            "Remote access",
            "Remote Desktop",
            "Review Remote Desktop exposure and access policy manually.",
        );
    };
    let enabled = value.trim() == "0";
    Finding {
        id: "remote-access.rdp",
        category: "Remote access",
        title: "Remote Desktop",
        severity: if enabled {
            FindingSeverity::Medium
        } else {
            FindingSeverity::Info
        },
        status: if enabled {
            FindingStatus::Attention
        } else {
            FindingStatus::Pass
        },
        summary: if enabled {
            "Remote Desktop is enabled on this system."
        } else {
            "Remote Desktop is disabled."
        }
        .to_owned(),
        evidence: format!("fDenyTSConnections: {value}."),
        recommendation: if enabled {
            "If Remote Desktop is required, restrict reachability, require strong authentication, and keep the host patched."
        } else {
            "Keep Remote Desktop disabled when it is not needed."
        }
        .to_owned(),
    }
}

fn check_listeners(runner: &impl ProbeRunner) -> Finding {
    let Some(value) = output(runner, Probe::Listeners) else {
        return unknown(
            "network.listeners",
            "Network exposure",
            "Listening TCP endpoints",
            "Review listening services with Windows networking tools.",
        );
    };

    let mut total = 0usize;
    let mut wildcard_ports = Vec::new();
    let mut sensitive_ports = Vec::new();
    for line in value.lines().take(MAX_LISTENERS) {
        let Some((address, port_text)) = line.split_once('|') else {
            continue;
        };
        let Ok(port) = port_text.trim().parse::<u16>() else {
            continue;
        };
        total += 1;
        let address = address.trim();
        let wildcard = matches!(address, "0.0.0.0" | "::" | "[::]" | "*");
        if wildcard {
            wildcard_ports.push(port);
            if matches!(port, 22 | 23 | 445 | 3389 | 5985 | 5986) {
                sensitive_ports.push(port);
            }
        }
    }
    wildcard_ports.sort_unstable();
    wildcard_ports.dedup();
    sensitive_ports.sort_unstable();
    sensitive_ports.dedup();

    let attention = !sensitive_ports.is_empty();
    Finding {
        id: "network.listeners",
        category: "Network exposure",
        title: "Listening TCP endpoints",
        severity: if attention {
            FindingSeverity::Medium
        } else {
            FindingSeverity::Info
        },
        status: if attention {
            FindingStatus::Attention
        } else {
            FindingStatus::Info
        },
        summary: if attention {
            "One or more commonly sensitive service ports are listening on wildcard addresses."
        } else {
            "Listening TCP endpoints were inventoried; no selected remote-management port triggered attention."
        }
        .to_owned(),
        evidence: if attention {
            format!(
                "Endpoints inspected: {total}; wildcard listeners: {}; selected sensitive ports: {}.",
                wildcard_ports.len(),
                sensitive_ports
                    .iter()
                    .map(u16::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            format!(
                "Endpoints inspected: {total}; wildcard listeners: {}.",
                wildcard_ports.len()
            )
        },
        recommendation:
            "Review whether each externally reachable listener is required and restrict exposure with firewall rules where appropriate."
                .to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        FindingStatus, Probe, ProbeRunner, check_bitlocker, check_firewall, check_listeners,
        check_smb1, check_uac, scan_with_runner,
    };

    #[derive(Default)]
    struct FakeRunner {
        values: HashMap<Probe, Result<String, String>>,
    }

    impl FakeRunner {
        fn with(mut self, probe: Probe, value: &str) -> Self {
            self.values.insert(probe, Ok(value.to_owned()));
            self
        }
    }

    impl ProbeRunner for FakeRunner {
        fn run(&self, probe: Probe) -> Result<String, String> {
            self.values
                .get(&probe)
                .cloned()
                .unwrap_or_else(|| Err("not available".to_owned()))
        }
    }

    #[test]
    fn enabled_firewall_profiles_pass() {
        let runner = FakeRunner::default().with(
            Probe::Firewall,
            "Domain=True\nPrivate=True\nPublic=True",
        );
        assert_eq!(check_firewall(&runner).status, FindingStatus::Pass);
    }

    #[test]
    fn disabled_firewall_profile_requires_attention() {
        let runner =
            FakeRunner::default().with(Probe::Firewall, "Domain=True\nPublic=False");
        assert_eq!(
            check_firewall(&runner).status,
            FindingStatus::Attention
        );
    }

    #[test]
    fn unprotected_system_volume_requires_attention() {
        let runner = FakeRunner::default().with(Probe::BitLocker, "Off");
        assert_eq!(
            check_bitlocker(&runner).status,
            FindingStatus::Attention
        );
    }

    #[test]
    fn disabled_uac_requires_attention() {
        let runner = FakeRunner::default().with(Probe::Uac, "0");
        assert_eq!(check_uac(&runner).status, FindingStatus::Attention);
    }

    #[test]
    fn enabled_smb1_requires_attention() {
        let runner = FakeRunner::default().with(Probe::Smb1, "Enabled");
        assert_eq!(check_smb1(&runner).status, FindingStatus::Attention);
    }

    #[test]
    fn wildcard_remote_management_listener_requires_attention() {
        let runner = FakeRunner::default().with(
            Probe::Listeners,
            "0.0.0.0|3389\n127.0.0.1|8080\n::|445",
        );
        let finding = check_listeners(&runner);
        assert_eq!(finding.status, FindingStatus::Attention);
        assert!(finding.evidence.contains("3389"));
    }

    #[test]
    fn failed_probes_are_reported_unknown_not_panics() {
        let report = scan_with_runner(&FakeRunner::default());
        if cfg!(target_os = "windows") {
            assert!(report.summary.unknown > 0);
        } else {
            assert_eq!(report.findings.len(), 1);
        }
    }
}
