#![forbid(unsafe_code)]

//! Windows-first network visibility for DragonForge Network Guard.
//!
//! Phase 8 inventories bounded TCP/UDP endpoints and the Windows DNS client
//! cache using fixed native probes. Phase 19 adds application identity hashing
//! for typed outbound firewall policy requests routed through the authenticated
//! DragonForge Agent and privileged service. Packet payload capture and
//! connection termination remain out of scope.

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sha2::{Digest, Sha256};

const MAX_CONNECTIONS: usize = 4_096;
const MAX_DNS_ENTRIES: usize = 4_096;
const MAX_PROBE_OUTPUT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NetworkConnection {
    pub protocol: Protocol,
    pub process_id: u32,
    pub process_name: String,
    pub local_address: String,
    pub local_port: u16,
    pub remote_address: Option<String>,
    pub remote_port: Option<u16>,
    pub state: Option<String>,
    pub wildcard_listener: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationIdentity {
    pub process_id: u32,
    pub process_name: String,
    pub application_path: PathBuf,
    pub sha256_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DnsCacheEntry {
    pub name: String,
    pub record_type: String,
    pub data: String,
    pub ttl_seconds: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NetworkSummary {
    pub tcp_connections: usize,
    pub udp_endpoints: usize,
    pub listening_endpoints: usize,
    pub wildcard_listeners: usize,
    pub processes: usize,
    pub dns_entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NetworkSnapshot {
    pub timestamp_ms: u64,
    pub platform: &'static str,
    pub summary: NetworkSummary,
    pub connections: Vec<NetworkConnection>,
    pub dns_cache: Vec<DnsCacheEntry>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum Probe {
    Tcp,
    Udp,
    Dns,
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
            Probe::Tcp => {
                r#"
$names = @{}
Get-Process -ErrorAction SilentlyContinue | ForEach-Object { $names[[int]$_.Id] = $_.ProcessName }
Get-NetTCPConnection -ErrorAction Stop |
  Sort-Object OwningProcess,LocalAddress,LocalPort,RemoteAddress,RemotePort |
  Select-Object -First 4096 |
  ForEach-Object {
    $name = $names[[int]$_.OwningProcess]
    if ([string]::IsNullOrWhiteSpace($name)) { $name = "<unknown>" }
    $state = [string]$_.State
    "$($_.OwningProcess)|$name|$($_.LocalAddress)|$($_.LocalPort)|$($_.RemoteAddress)|$($_.RemotePort)|$state"
  }
"#
            }
            Probe::Udp => {
                r#"
$names = @{}
Get-Process -ErrorAction SilentlyContinue | ForEach-Object { $names[[int]$_.Id] = $_.ProcessName }
Get-NetUDPEndpoint -ErrorAction Stop |
  Sort-Object OwningProcess,LocalAddress,LocalPort |
  Select-Object -First 4096 |
  ForEach-Object {
    $name = $names[[int]$_.OwningProcess]
    if ([string]::IsNullOrWhiteSpace($name)) { $name = "<unknown>" }
    "$($_.OwningProcess)|$name|$($_.LocalAddress)|$($_.LocalPort)"
  }
"#
            }
            Probe::Dns => {
                r#"
Get-DnsClientCache -ErrorAction Stop |
  Sort-Object Entry,RecordType,Data |
  Select-Object -First 4096 |
  ForEach-Object {
    $entry = ([string]$_.Entry) -replace '[
|]+', ' '
    $type = ([string]$_.RecordType) -replace '[
|]+', ' '
    $data = ([string]$_.Data) -replace '[
|]+', ' '
    "$entry|$type|$data|$($_.TimeToLive)"
  }
"#
            }
        };

        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|_| "fixed Network Guard probe could not be started".to_owned())?;

        if !output.status.success() {
            return Err("fixed Network Guard probe did not complete successfully".to_owned());
        }
        if output.stdout.len() > MAX_PROBE_OUTPUT_BYTES {
            return Err("fixed Network Guard probe exceeded the safe output limit".to_owned());
        }

        String::from_utf8(output.stdout)
            .map(|value| value.trim().to_owned())
            .map_err(|_| "fixed Network Guard probe returned invalid text".to_owned())
    }
}

#[cfg(not(target_os = "windows"))]
struct PlatformRunner;

#[cfg(not(target_os = "windows"))]
impl ProbeRunner for PlatformRunner {
    fn run(&self, _probe: Probe) -> Result<String, String> {
        Err("Phase 8 network visibility currently supports Windows".to_owned())
    }
}

pub fn inspect_process_application(process_id: u32) -> Result<ApplicationIdentity, String> {
    if process_id == 0 {
        return Err("process id must be nonzero".to_owned());
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let script = format!(
            "$p = Get-Process -Id {process_id} -ErrorAction Stop; [Console]::Out.WriteLine($p.ProcessName); [Console]::Out.WriteLine($p.Path)"
        );
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|_| "application identity probe could not be started".to_owned())?;
        if !output.status.success() || output.stdout.len() > 32 * 1024 {
            return Err("application identity probe failed".to_owned());
        }
        let text = String::from_utf8(output.stdout)
            .map_err(|_| "application identity probe returned invalid text".to_owned())?;
        let mut lines = text.lines();
        let process_name = clean(lines.next().unwrap_or_default(), 260);
        let path_text = clean(lines.next().unwrap_or_default(), 2048);
        if process_name.is_empty() || path_text.is_empty() {
            return Err("application executable path is unavailable".to_owned());
        }
        let application_path = PathBuf::from(path_text);
        if !application_path.is_absolute() || !application_path.is_file() {
            return Err("application executable path is invalid".to_owned());
        }
        let sha256_hex = hash_application(&application_path)?;
        Ok(ApplicationIdentity {
            process_id,
            process_name,
            application_path,
            sha256_hex,
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = process_id;
        Err("Phase 19 application identity inspection requires Windows".to_owned())
    }
}

fn hash_application(path: &std::path::Path) -> Result<String, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "application executable metadata is unavailable".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 1024 * 1024 * 1024 {
        return Err("application executable is not eligible for firewall policy".to_owned());
    }
    let mut file = File::open(path)
        .map_err(|_| "application executable could not be opened".to_owned())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| "application executable could not be hashed".to_owned())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[must_use]
pub fn collect_snapshot() -> NetworkSnapshot {
    collect_with_runner(&PlatformRunner)
}

fn collect_with_runner(runner: &impl ProbeRunner) -> NetworkSnapshot {
    let mut warnings = Vec::new();
    let mut connections = Vec::new();
    let mut dns_cache = Vec::new();

    if cfg!(target_os = "windows") {
        match runner.run(Probe::Tcp) {
            Ok(output) => parse_tcp(&output, &mut connections, &mut warnings),
            Err(message) => warnings.push(message),
        }
        match runner.run(Probe::Udp) {
            Ok(output) => parse_udp(&output, &mut connections, &mut warnings),
            Err(message) => warnings.push(message),
        }
        match runner.run(Probe::Dns) {
            Ok(output) => parse_dns(&output, &mut dns_cache, &mut warnings),
            Err(message) => warnings.push(message),
        }
    } else {
        warnings.push("Phase 8 Network Guard is currently Windows-first.".to_owned());
    }

    connections.truncate(MAX_CONNECTIONS * 2);
    dns_cache.truncate(MAX_DNS_ENTRIES);

    let tcp_connections = connections
        .iter()
        .filter(|connection| connection.protocol == Protocol::Tcp)
        .count();
    let udp_endpoints = connections
        .iter()
        .filter(|connection| connection.protocol == Protocol::Udp)
        .count();
    let listening_endpoints = connections
        .iter()
        .filter(|connection| {
            connection.protocol == Protocol::Udp || connection.state.as_deref() == Some("Listen")
        })
        .count();
    let wildcard_listeners = connections
        .iter()
        .filter(|connection| connection.wildcard_listener)
        .count();
    let processes = connections
        .iter()
        .map(|connection| connection.process_id)
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    NetworkSnapshot {
        timestamp_ms: now_ms(),
        platform: if cfg!(target_os = "windows") {
            "windows"
        } else {
            "unsupported"
        },
        summary: NetworkSummary {
            tcp_connections,
            udp_endpoints,
            listening_endpoints,
            wildcard_listeners,
            processes,
            dns_entries: dns_cache.len(),
        },
        connections,
        dns_cache,
        warnings,
    }
}

fn parse_tcp(output: &str, target: &mut Vec<NetworkConnection>, warnings: &mut Vec<String>) {
    for line in output.lines().take(MAX_CONNECTIONS) {
        let fields = line.split('|').collect::<Vec<_>>();
        if fields.len() != 7 {
            warnings.push("A TCP row was skipped because it was malformed.".to_owned());
            continue;
        }

        let Some(process_id) = parse_u32(fields[0]) else {
            warnings.push("A TCP row was skipped because its process id was invalid.".to_owned());
            continue;
        };
        let Some(local_port) = parse_u16(fields[3]) else {
            warnings.push("A TCP row was skipped because its local port was invalid.".to_owned());
            continue;
        };
        let remote_port = parse_u16(fields[5]);
        let state = clean(fields[6], 48);
        let local_address = clean(fields[2], 128);

        target.push(NetworkConnection {
            protocol: Protocol::Tcp,
            process_id,
            process_name: clean(fields[1], 260),
            local_address: local_address.clone(),
            local_port,
            remote_address: nonempty(clean(fields[4], 128)),
            remote_port,
            wildcard_listener: state == "Listen" && is_wildcard(&local_address),
            state: nonempty(state),
        });
    }
}

fn parse_udp(output: &str, target: &mut Vec<NetworkConnection>, warnings: &mut Vec<String>) {
    for line in output.lines().take(MAX_CONNECTIONS) {
        let fields = line.split('|').collect::<Vec<_>>();
        if fields.len() != 4 {
            warnings.push("A UDP row was skipped because it was malformed.".to_owned());
            continue;
        }

        let Some(process_id) = parse_u32(fields[0]) else {
            warnings.push("A UDP row was skipped because its process id was invalid.".to_owned());
            continue;
        };
        let Some(local_port) = parse_u16(fields[3]) else {
            warnings.push("A UDP row was skipped because its local port was invalid.".to_owned());
            continue;
        };
        let local_address = clean(fields[2], 128);

        target.push(NetworkConnection {
            protocol: Protocol::Udp,
            process_id,
            process_name: clean(fields[1], 260),
            local_address: local_address.clone(),
            local_port,
            remote_address: None,
            remote_port: None,
            state: None,
            wildcard_listener: is_wildcard(&local_address),
        });
    }
}

fn parse_dns(output: &str, target: &mut Vec<DnsCacheEntry>, warnings: &mut Vec<String>) {
    for line in output.lines().take(MAX_DNS_ENTRIES) {
        let fields = line.split('|').collect::<Vec<_>>();
        if fields.len() != 4 {
            warnings.push("A DNS cache row was skipped because it was malformed.".to_owned());
            continue;
        }

        let name = clean(fields[0], 512);
        if name.is_empty() {
            continue;
        }

        target.push(DnsCacheEntry {
            name,
            record_type: clean(fields[1], 64),
            data: clean(fields[2], 512),
            ttl_seconds: parse_u32(fields[3]),
        });
    }
}

fn clean(value: &str, max_chars: usize) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(max_chars)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn nonempty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn parse_u16(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok()
}

fn parse_u32(value: &str) -> Option<u32> {
    value.trim().parse::<u32>().ok()
}

fn is_wildcard(address: &str) -> bool {
    matches!(address.trim(), "0.0.0.0" | "::" | "[::]" | "*")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().min(u128::from(u64::MAX)) as u64
        })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        NetworkConnection, Probe, ProbeRunner, Protocol, collect_with_runner, parse_dns, parse_tcp,
        parse_udp,
    };

    #[derive(Default)]
    struct FakeRunner {
        values: HashMap<&'static str, Result<String, String>>,
    }

    impl FakeRunner {
        fn with(mut self, key: &'static str, value: &str) -> Self {
            self.values.insert(key, Ok(value.to_owned()));
            self
        }
    }

    impl ProbeRunner for FakeRunner {
        fn run(&self, probe: Probe) -> Result<String, String> {
            let key = match probe {
                Probe::Tcp => "tcp",
                Probe::Udp => "udp",
                Probe::Dns => "dns",
            };
            self.values
                .get(key)
                .cloned()
                .unwrap_or_else(|| Err("probe unavailable".to_owned()))
        }
    }

    #[test]
    fn tcp_parser_marks_wildcard_listener() {
        let mut rows = Vec::<NetworkConnection>::new();
        let mut warnings = Vec::new();
        parse_tcp(
            "4242|server|0.0.0.0|8080|0.0.0.0|0|Listen",
            &mut rows,
            &mut warnings,
        );
        assert!(warnings.is_empty());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].protocol, Protocol::Tcp);
        assert!(rows[0].wildcard_listener);
    }

    #[test]
    fn udp_parser_marks_wildcard_endpoint() {
        let mut rows = Vec::<NetworkConnection>::new();
        let mut warnings = Vec::new();
        parse_udp("51|dns-service|::|5353", &mut rows, &mut warnings);
        assert!(warnings.is_empty());
        assert!(rows[0].wildcard_listener);
    }

    #[test]
    fn dns_parser_keeps_name_type_data_and_ttl() {
        let mut rows = Vec::new();
        let mut warnings = Vec::new();
        parse_dns("example.test|A|192.0.2.10|120", &mut rows, &mut warnings);
        assert!(warnings.is_empty());
        assert_eq!(rows[0].name, "example.test");
        assert_eq!(rows[0].ttl_seconds, Some(120));
    }

    #[test]
    fn malformed_rows_are_skipped_without_panicking() {
        let mut rows = Vec::<NetworkConnection>::new();
        let mut warnings = Vec::new();
        parse_tcp("bad-row", &mut rows, &mut warnings);
        assert!(rows.is_empty());
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn snapshot_summarizes_process_visibility() {
        let runner = FakeRunner::default()
            .with(
                "tcp",
                "100|browser|127.0.0.1|50000|203.0.113.10|443|Established\n200|server|0.0.0.0|8080|0.0.0.0|0|Listen",
            )
            .with("udp", "200|server|0.0.0.0|5353")
            .with("dns", "example.test|A|192.0.2.10|60");

        let snapshot = collect_with_runner(&runner);
        if cfg!(target_os = "windows") {
            assert_eq!(snapshot.summary.tcp_connections, 2);
            assert_eq!(snapshot.summary.udp_endpoints, 1);
            assert_eq!(snapshot.summary.wildcard_listeners, 2);
            assert_eq!(snapshot.summary.processes, 2);
            assert_eq!(snapshot.summary.dns_entries, 1);
        } else {
            assert!(!snapshot.warnings.is_empty());
        }
    }
}
