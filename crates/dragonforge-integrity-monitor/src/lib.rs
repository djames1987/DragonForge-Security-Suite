#![forbid(unsafe_code)]

//! Local integrity baseline and change detection for DragonForge Integrity Monitor.
//!
//! Phase 7 records bounded identifiers and SHA-256 fingerprints for selected
//! Windows persistence/configuration surfaces. It does not store command
//! contents, file contents, or secret material in the baseline.

mod continuous;
pub use continuous::*;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const BASELINE_VERSION: u16 = 1;
const MAX_BASELINE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;
const MAX_STARTUP_FILES: usize = 512;
const MAX_HASHED_FILE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_HOSTS_FILE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_PROBE_LINES: usize = 8_000;
const MAX_PROBE_OUTPUT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug)]
pub enum IntegrityError {
    Io(io::Error),
    InvalidBaseline(&'static str),
    BaselineExists,
    BaselineMissing,
    SymlinkNotAllowed,
    LimitExceeded(&'static str),
}

impl std::fmt::Display for IntegrityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(_) => write!(formatter, "integrity monitor storage operation failed"),
            Self::InvalidBaseline(message) => write!(formatter, "{message}"),
            Self::BaselineExists => write!(formatter, "an integrity baseline already exists"),
            Self::BaselineMissing => write!(formatter, "no integrity baseline exists"),
            Self::SymlinkNotAllowed => {
                write!(formatter, "symbolic-link baseline paths are not allowed")
            }
            Self::LimitExceeded(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for IntegrityError {}

impl From<io::Error> for IntegrityError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub type IntegrityResult<T> = Result<T, IntegrityError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceKind {
    Startup,
    RegistryPersistence,
    Services,
    ScheduledTasks,
    HostsFile,
    SystemConfiguration,
}

impl SurfaceKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::RegistryPersistence => "registry_persistence",
            Self::Services => "services",
            Self::ScheduledTasks => "scheduled_tasks",
            Self::HostsFile => "hosts_file",
            Self::SystemConfiguration => "system_configuration",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineEntry {
    pub surface: SurfaceKind,
    pub key: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityBaseline {
    pub version: u16,
    pub created_at_ms: u64,
    pub entries: Vec<BaselineEntry>,
    pub unavailable_surfaces: Vec<SurfaceKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SnapshotSummary {
    pub timestamp_ms: u64,
    pub entries: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Removed,
    Changed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IntegrityChange {
    pub surface: SurfaceKind,
    pub key: String,
    pub kind: ChangeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComparisonSummary {
    pub unchanged: usize,
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComparisonReport {
    pub baseline_created_at_ms: u64,
    pub checked_at_ms: u64,
    pub summary: ComparisonSummary,
    pub changes: Vec<IntegrityChange>,
    pub unavailable_surfaces: Vec<SurfaceKind>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct CollectedSnapshot {
    timestamp_ms: u64,
    entries: Vec<BaselineEntry>,
    unavailable_surfaces: BTreeSet<SurfaceKind>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum Probe {
    RegistryPersistence,
    Services,
    ScheduledTasks,
    SystemConfiguration,
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
            Probe::RegistryPersistence => {
                r#"
$paths = @(
  'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run',
  'HKCU:\Software\Microsoft\Windows\CurrentVersion\RunOnce',
  'HKLM:\Software\Microsoft\Windows\CurrentVersion\Run',
  'HKLM:\Software\Microsoft\Windows\CurrentVersion\RunOnce',
  'HKCU:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run',
  'HKCU:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\RunOnce',
  'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run',
  'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\RunOnce'
)
foreach ($path in $paths) {
  if (Test-Path $path) {
    $item = Get-ItemProperty -Path $path
    foreach ($property in $item.PSObject.Properties) {
      if ($property.Name -notmatch '^PS') {
        $value = ([string]$property.Value) -replace '[\r\n]+', ' '
        "${path}::$($property.Name)|$value"
      }
    }
  }
}
"#
            }
            Probe::Services => {
                r#"
Get-CimInstance Win32_Service |
  Sort-Object Name |
  ForEach-Object {
    $value = "$($_.StartMode);$($_.StartName);$($_.PathName)" -replace '[\r\n]+', ' '
    "$($_.Name)|$value"
  }
"#
            }
            Probe::ScheduledTasks => {
                r#"
Get-ScheduledTask |
  Sort-Object TaskPath,TaskName |
  ForEach-Object {
    $actions = ($_.Actions | ForEach-Object { "$($_.Execute);$($_.Arguments);$($_.WorkingDirectory)" }) -join ';'
    $triggers = ($_.Triggers | ConvertTo-Json -Compress -Depth 5)
    $value = "$($_.Principal.UserId);$($_.Principal.LogonType);$($_.Principal.RunLevel);$actions;$triggers;$($_.Settings.Enabled);$($_.Settings.Hidden)" -replace '[\r\n]+', ' '
    "$($_.TaskPath)$($_.TaskName)|$value"
  }
"#
            }
            Probe::SystemConfiguration => {
                r#"
try {
  $uac = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System' -Name EnableLUA).EnableLUA
  "uac.enable_lua|$uac"
} catch {}
try {
  $rdp = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Terminal Server' -Name fDenyTSConnections).fDenyTSConnections
  "rdp.deny_connections|$rdp"
} catch {}
try {
  $smb = (Get-WindowsOptionalFeature -Online -FeatureName SMB1Protocol).State
  "smb1.state|$smb"
} catch {}
try {
  Get-NetFirewallProfile | Sort-Object Name | ForEach-Object {
    "firewall.$($_.Name)|$($_.Enabled);$($_.DefaultInboundAction);$($_.DefaultOutboundAction)"
  }
} catch {}
"#
            }
        };

        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|_| "fixed integrity probe could not be started".to_owned())?;

        if !output.status.success() {
            return Err("fixed integrity probe did not complete successfully".to_owned());
        }
        if output.stdout.len() > MAX_PROBE_OUTPUT_BYTES {
            return Err("fixed integrity probe exceeded the safe output limit".to_owned());
        }

        String::from_utf8(output.stdout)
            .map(|value| value.trim().to_owned())
            .map_err(|_| "fixed integrity probe returned invalid text".to_owned())
    }
}

#[cfg(not(target_os = "windows"))]
struct PlatformRunner;

#[cfg(not(target_os = "windows"))]
impl ProbeRunner for PlatformRunner {
    fn run(&self, _probe: Probe) -> Result<String, String> {
        Err("Phase 7 fixed integrity probes currently support Windows".to_owned())
    }
}

#[must_use]
pub fn collect_snapshot() -> SnapshotSummary {
    let snapshot = collect_with_runner(&PlatformRunner);
    SnapshotSummary {
        timestamp_ms: snapshot.timestamp_ms,
        entries: snapshot.entries.len(),
        warnings: snapshot.warnings,
    }
}

pub fn create_baseline(path: &Path, replace: bool) -> IntegrityResult<SnapshotSummary> {
    validate_baseline_path(path)?;
    if path.exists() && !replace {
        return Err(IntegrityError::BaselineExists);
    }

    let snapshot = collect_with_runner(&PlatformRunner);
    let baseline = IntegrityBaseline {
        version: BASELINE_VERSION,
        created_at_ms: snapshot.timestamp_ms,
        entries: snapshot.entries.clone(),
        unavailable_surfaces: snapshot.unavailable_surfaces.iter().copied().collect(),
    };
    write_baseline(path, &baseline, replace)?;

    Ok(SnapshotSummary {
        timestamp_ms: snapshot.timestamp_ms,
        entries: snapshot.entries.len(),
        warnings: snapshot.warnings,
    })
}

pub fn baseline_summary(path: &Path) -> IntegrityResult<Option<SnapshotSummary>> {
    validate_baseline_path(path)?;
    if !path.exists() {
        return Ok(None);
    }
    let baseline = read_baseline(path)?;
    let warnings = baseline
        .unavailable_surfaces
        .iter()
        .map(|surface| {
            format!(
                "{} was unavailable when this baseline was created.",
                surface.as_str()
            )
        })
        .collect();
    Ok(Some(SnapshotSummary {
        timestamp_ms: baseline.created_at_ms,
        entries: baseline.entries.len(),
        warnings,
    }))
}

pub fn compare_to_baseline(path: &Path) -> IntegrityResult<ComparisonReport> {
    validate_baseline_path(path)?;
    if !path.exists() {
        return Err(IntegrityError::BaselineMissing);
    }
    let baseline = read_baseline(path)?;
    let current = collect_with_runner(&PlatformRunner);
    Ok(compare(&baseline, current))
}

fn collect_with_runner(runner: &impl ProbeRunner) -> CollectedSnapshot {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    let mut unavailable_surfaces = BTreeSet::new();

    if cfg!(target_os = "windows") {
        if !collect_startup_entries(&mut entries, &mut warnings) {
            unavailable_surfaces.insert(SurfaceKind::Startup);
        }
        if !collect_hosts_entry(&mut entries, &mut warnings) {
            unavailable_surfaces.insert(SurfaceKind::HostsFile);
        }
        if !collect_probe_entries(
            runner,
            Probe::RegistryPersistence,
            SurfaceKind::RegistryPersistence,
            &mut entries,
            &mut warnings,
        ) {
            unavailable_surfaces.insert(SurfaceKind::RegistryPersistence);
        }
        if !collect_probe_entries(
            runner,
            Probe::Services,
            SurfaceKind::Services,
            &mut entries,
            &mut warnings,
        ) {
            unavailable_surfaces.insert(SurfaceKind::Services);
        }
        if !collect_probe_entries(
            runner,
            Probe::ScheduledTasks,
            SurfaceKind::ScheduledTasks,
            &mut entries,
            &mut warnings,
        ) {
            unavailable_surfaces.insert(SurfaceKind::ScheduledTasks);
        }
        if !collect_probe_entries(
            runner,
            Probe::SystemConfiguration,
            SurfaceKind::SystemConfiguration,
            &mut entries,
            &mut warnings,
        ) {
            unavailable_surfaces.insert(SurfaceKind::SystemConfiguration);
        }
    } else {
        unavailable_surfaces.extend([
            SurfaceKind::Startup,
            SurfaceKind::RegistryPersistence,
            SurfaceKind::Services,
            SurfaceKind::ScheduledTasks,
            SurfaceKind::HostsFile,
            SurfaceKind::SystemConfiguration,
        ]);
        warnings.push("Phase 7 integrity collection is currently Windows-first.".to_owned());
    }

    entries.sort_by(|left, right| {
        (left.surface, left.key.as_str()).cmp(&(right.surface, right.key.as_str()))
    });
    entries.dedup_by(|left, right| left.surface == right.surface && left.key == right.key);
    if entries.len() > MAX_ENTRIES {
        entries.truncate(MAX_ENTRIES);
        unavailable_surfaces.extend([
            SurfaceKind::Startup,
            SurfaceKind::RegistryPersistence,
            SurfaceKind::Services,
            SurfaceKind::ScheduledTasks,
            SurfaceKind::HostsFile,
            SurfaceKind::SystemConfiguration,
        ]);
        warnings.push(
            "Integrity entry limit reached; comparison surfaces were marked unavailable."
                .to_owned(),
        );
    }

    CollectedSnapshot {
        timestamp_ms: now_ms(),
        entries,
        unavailable_surfaces,
        warnings,
    }
}

fn collect_probe_entries(
    runner: &impl ProbeRunner,
    probe: Probe,
    surface: SurfaceKind,
    entries: &mut Vec<BaselineEntry>,
    warnings: &mut Vec<String>,
) -> bool {
    let output = match runner.run(probe) {
        Ok(output) => output,
        Err(message) => {
            warnings.push(message);
            return false;
        }
    };

    let mut seen = 0usize;
    let mut complete = true;
    for line in output.lines() {
        if seen >= MAX_PROBE_LINES {
            warnings.push(format!(
                "{} probe line limit reached; additional entries were omitted.",
                surface.as_str()
            ));
            complete = false;
            break;
        }
        let Some((key, value)) = line.split_once('|') else {
            continue;
        };
        let key = sanitize_key(key);
        if key.is_empty() {
            continue;
        }
        entries.push(BaselineEntry {
            surface,
            key,
            fingerprint: fingerprint(value.as_bytes()),
        });
        seen += 1;
    }
    complete
}

fn collect_startup_entries(entries: &mut Vec<BaselineEntry>, warnings: &mut Vec<String>) -> bool {
    let mut roots = Vec::new();
    if let Some(appdata) = env::var_os("APPDATA") {
        roots.push((
            "user",
            PathBuf::from(appdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Startup"),
        ));
    }
    if let Some(programdata) = env::var_os("PROGRAMDATA") {
        roots.push((
            "common",
            PathBuf::from(programdata)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("StartUp"),
        ));
    }

    let mut complete = !roots.is_empty();
    if roots.is_empty() {
        warnings.push("Windows Startup folder locations are unavailable.".to_owned());
    }

    for (scope, root) in roots {
        let mut files = Vec::new();
        if let Err(error) = walk_files(&root, &root, &mut files, 0) {
            warnings.push(format!(
                "Startup folder {scope} could not be fully inspected: {error}."
            ));
            complete = false;
            continue;
        }
        files.sort();
        if files.len() > MAX_STARTUP_FILES {
            complete = false;
            warnings.push(format!(
                "Startup folder {scope} exceeded the item limit; that surface will be excluded from comparison."
            ));
            files.truncate(MAX_STARTUP_FILES);
        }
        for relative in files {
            let full = root.join(&relative);
            match hash_file(&full, MAX_HASHED_FILE_BYTES) {
                Ok(hash) => entries.push(BaselineEntry {
                    surface: SurfaceKind::Startup,
                    key: sanitize_key(&format!("{scope}/{}", relative.to_string_lossy())),
                    fingerprint: hash,
                }),
                Err(_) => {
                    complete = false;
                    warnings.push(format!(
                        "Startup item {scope}/{} could not be fingerprinted.",
                        relative.to_string_lossy()
                    ));
                }
            }
        }
    }
    complete
}

fn walk_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
    depth: usize,
) -> io::Result<()> {
    if depth > 8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "startup traversal depth limit reached",
        ));
    }
    if files.len() > MAX_STARTUP_FILES {
        return Ok(());
    }
    if !current.exists() {
        return Ok(());
    }

    for item in fs::read_dir(current)? {
        if files.len() > MAX_STARTUP_FILES {
            break;
        }
        let item = item?;
        let metadata = fs::symlink_metadata(item.path())?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            walk_files(root, &item.path(), files, depth + 1)?;
        } else if metadata.is_file()
            && let Ok(relative) = item.path().strip_prefix(root)
        {
            files.push(relative.to_path_buf());
        }
    }
    Ok(())
}

fn collect_hosts_entry(entries: &mut Vec<BaselineEntry>, warnings: &mut Vec<String>) -> bool {
    let Some(system_root) = env::var_os("SystemRoot") else {
        warnings
            .push("Windows SystemRoot is unavailable; hosts file was not inspected.".to_owned());
        return false;
    };
    let hosts = PathBuf::from(system_root)
        .join("System32")
        .join("drivers")
        .join("etc")
        .join("hosts");

    match hash_file(&hosts, MAX_HOSTS_FILE_BYTES) {
        Ok(hash) => {
            entries.push(BaselineEntry {
                surface: SurfaceKind::HostsFile,
                key: "windows/hosts".to_owned(),
                fingerprint: hash,
            });
            true
        }
        Err(_) => {
            warnings.push("Windows hosts file could not be fingerprinted.".to_owned());
            false
        }
    }
}

fn hash_file(path: &Path, limit: u64) -> IntegrityResult<String> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(IntegrityError::SymlinkNotAllowed);
    }
    if metadata.len() > limit {
        return Err(IntegrityError::LimitExceeded(
            "an integrity-monitored file exceeded the safe hashing limit",
        ));
    }

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 32 * 1024];
    let mut total = 0_u64;
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        total = total.saturating_add(read as u64);
        if total > limit {
            return Err(IntegrityError::LimitExceeded(
                "an integrity-monitored file exceeded the safe hashing limit",
            ));
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    Ok(hex_digest(&digest))
}

fn fingerprint(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    hex_digest(&digest)
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn sanitize_key(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(512)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn validate_baseline_path(path: &Path) -> IntegrityResult<()> {
    if path.exists() && fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(IntegrityError::SymlinkNotAllowed);
    }
    if let Some(parent) = path.parent() {
        if parent.exists() && fs::symlink_metadata(parent)?.file_type().is_symlink() {
            return Err(IntegrityError::SymlinkNotAllowed);
        }
    }
    Ok(())
}

fn read_baseline(path: &Path) -> IntegrityResult<IntegrityBaseline> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(IntegrityError::SymlinkNotAllowed);
    }
    if metadata.len() > MAX_BASELINE_BYTES {
        return Err(IntegrityError::LimitExceeded(
            "integrity baseline exceeded the supported size limit",
        ));
    }

    let bytes = fs::read(path)?;
    let baseline: IntegrityBaseline = serde_json::from_slice(&bytes)
        .map_err(|_| IntegrityError::InvalidBaseline("integrity baseline is invalid"))?;
    validate_baseline(&baseline)?;
    Ok(baseline)
}

fn validate_baseline(baseline: &IntegrityBaseline) -> IntegrityResult<()> {
    if baseline.version != BASELINE_VERSION {
        return Err(IntegrityError::InvalidBaseline(
            "integrity baseline version is unsupported",
        ));
    }
    if baseline.entries.len() > MAX_ENTRIES {
        return Err(IntegrityError::LimitExceeded(
            "integrity baseline contains too many entries",
        ));
    }
    let unavailable = baseline
        .unavailable_surfaces
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if unavailable.len() != baseline.unavailable_surfaces.len() {
        return Err(IntegrityError::InvalidBaseline(
            "integrity baseline contains duplicate unavailable surfaces",
        ));
    }

    let mut keys = BTreeSet::new();
    for entry in &baseline.entries {
        if entry.key.is_empty()
            || entry.key.len() > 512
            || entry.fingerprint.len() != 64
            || !entry
                .fingerprint
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(IntegrityError::InvalidBaseline(
                "integrity baseline contains an invalid entry",
            ));
        }
        if !keys.insert((entry.surface, entry.key.as_str())) {
            return Err(IntegrityError::InvalidBaseline(
                "integrity baseline contains duplicate entries",
            ));
        }
    }
    Ok(())
}

fn write_baseline(path: &Path, baseline: &IntegrityBaseline, replace: bool) -> IntegrityResult<()> {
    validate_baseline(baseline)?;
    let encoded = serde_json::to_vec_pretty(baseline)
        .map_err(|_| IntegrityError::InvalidBaseline("integrity baseline could not be encoded"))?;
    if encoded.len() as u64 > MAX_BASELINE_BYTES {
        return Err(IntegrityError::LimitExceeded(
            "integrity baseline exceeded the supported size limit",
        ));
    }

    let parent = path.parent().ok_or(IntegrityError::InvalidBaseline(
        "baseline path has no parent",
    ))?;
    fs::create_dir_all(parent)?;
    validate_baseline_path(path)?;

    if path.exists() && !replace {
        return Err(IntegrityError::BaselineExists);
    }

    let stamp = now_ms();
    let temporary = parent.join(format!(".baseline-{stamp}-{}.tmp", std::process::id()));
    let backup = parent.join(format!(".baseline-{stamp}-{}.bak", std::process::id()));

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    if let Err(error) = file.write_all(&encoded).and_then(|()| file.sync_all()) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }
    drop(file);

    if path.exists() {
        fs::rename(path, &backup)?;
        match fs::rename(&temporary, path) {
            Ok(()) => {
                let _ = fs::remove_file(&backup);
            }
            Err(error) => {
                let _ = fs::rename(&backup, path);
                let _ = fs::remove_file(&temporary);
                return Err(error.into());
            }
        }
    } else if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error.into());
    }

    Ok(())
}

fn compare(baseline: &IntegrityBaseline, current: CollectedSnapshot) -> ComparisonReport {
    let baseline_map = baseline
        .entries
        .iter()
        .map(|entry| {
            (
                (entry.surface, entry.key.as_str()),
                entry.fingerprint.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let current_map = current
        .entries
        .iter()
        .map(|entry| {
            (
                (entry.surface, entry.key.as_str()),
                entry.fingerprint.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let unavailable_surfaces = baseline
        .unavailable_surfaces
        .iter()
        .copied()
        .chain(current.unavailable_surfaces.iter().copied())
        .collect::<BTreeSet<_>>();

    let mut keys = BTreeSet::new();
    keys.extend(baseline_map.keys().copied());
    keys.extend(current_map.keys().copied());

    let mut changes = Vec::new();
    let mut unchanged = 0usize;
    let mut added = 0usize;
    let mut removed = 0usize;
    let mut changed = 0usize;

    for (surface, key) in keys {
        if unavailable_surfaces.contains(&surface) {
            continue;
        }
        match (
            baseline_map.get(&(surface, key)),
            current_map.get(&(surface, key)),
        ) {
            (None, Some(_)) => {
                added += 1;
                changes.push(IntegrityChange {
                    surface,
                    key: key.to_owned(),
                    kind: ChangeKind::Added,
                });
            }
            (Some(_), None) => {
                removed += 1;
                changes.push(IntegrityChange {
                    surface,
                    key: key.to_owned(),
                    kind: ChangeKind::Removed,
                });
            }
            (Some(left), Some(right)) if left != right => {
                changed += 1;
                changes.push(IntegrityChange {
                    surface,
                    key: key.to_owned(),
                    kind: ChangeKind::Changed,
                });
            }
            (Some(_), Some(_)) => unchanged += 1,
            (None, None) => {}
        }
    }

    let mut warnings = current.warnings;
    for surface in &unavailable_surfaces {
        warnings.push(format!(
            "{} was unavailable in the baseline or current snapshot and was excluded from comparison.",
            surface.as_str()
        ));
    }

    ComparisonReport {
        baseline_created_at_ms: baseline.created_at_ms,
        checked_at_ms: current.timestamp_ms,
        summary: ComparisonSummary {
            unchanged,
            added,
            removed,
            changed,
        },
        changes,
        unavailable_surfaces: unavailable_surfaces.into_iter().collect(),
        warnings,
    }
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
    use std::collections::BTreeSet;
    use std::fs;

    use tempfile::tempdir;

    use super::{
        BASELINE_VERSION, BaselineEntry, ChangeKind, CollectedSnapshot, IntegrityBaseline,
        SurfaceKind, baseline_summary, compare, read_baseline, write_baseline,
    };

    fn entry(surface: SurfaceKind, key: &str, fingerprint: &str) -> BaselineEntry {
        BaselineEntry {
            surface,
            key: key.to_owned(),
            fingerprint: fingerprint.to_owned(),
        }
    }

    fn hash(character: char) -> String {
        std::iter::repeat_n(character, 64).collect()
    }

    #[test]
    fn comparison_reports_added_removed_changed_and_unchanged() {
        let baseline = IntegrityBaseline {
            version: BASELINE_VERSION,
            created_at_ms: 10,
            entries: vec![
                entry(SurfaceKind::Services, "same", &hash('a')),
                entry(SurfaceKind::Services, "changed", &hash('b')),
                entry(SurfaceKind::Startup, "removed", &hash('c')),
            ],
            unavailable_surfaces: Vec::new(),
        };
        let current = CollectedSnapshot {
            timestamp_ms: 20,
            entries: vec![
                entry(SurfaceKind::Services, "same", &hash('a')),
                entry(SurfaceKind::Services, "changed", &hash('d')),
                entry(SurfaceKind::ScheduledTasks, "added", &hash('e')),
            ],
            unavailable_surfaces: BTreeSet::new(),
            warnings: Vec::new(),
        };

        let report = compare(&baseline, current);
        assert_eq!(report.summary.unchanged, 1);
        assert_eq!(report.summary.added, 1);
        assert_eq!(report.summary.removed, 1);
        assert_eq!(report.summary.changed, 1);
        assert!(
            report
                .changes
                .iter()
                .any(|change| { change.key == "added" && change.kind == ChangeKind::Added })
        );
    }

    #[test]
    fn unavailable_surface_does_not_create_false_removal() {
        let baseline = IntegrityBaseline {
            version: BASELINE_VERSION,
            created_at_ms: 10,
            entries: vec![entry(SurfaceKind::Services, "service-a", &hash('a'))],
            unavailable_surfaces: Vec::new(),
        };
        let mut unavailable = BTreeSet::new();
        unavailable.insert(SurfaceKind::Services);
        let current = CollectedSnapshot {
            timestamp_ms: 20,
            entries: Vec::new(),
            unavailable_surfaces: unavailable,
            warnings: vec!["services unavailable".to_owned()],
        };

        let report = compare(&baseline, current);
        assert_eq!(report.summary.removed, 0);
        assert!(report.changes.is_empty());
        assert_eq!(report.unavailable_surfaces, vec![SurfaceKind::Services]);
    }

    #[test]
    fn baseline_round_trip_preserves_fingerprints() {
        let dir = tempdir().expect("temporary directory");
        let path = dir.path().join("baseline.json");
        let baseline = IntegrityBaseline {
            version: BASELINE_VERSION,
            created_at_ms: 42,
            entries: vec![entry(SurfaceKind::HostsFile, "windows/hosts", &hash('f'))],
            unavailable_surfaces: Vec::new(),
        };

        write_baseline(&path, &baseline, false).expect("write baseline");
        assert_eq!(read_baseline(&path).expect("read baseline"), baseline);
        let summary = baseline_summary(&path)
            .expect("baseline summary")
            .expect("baseline exists");
        assert_eq!(summary.timestamp_ms, 42);
        assert_eq!(summary.entries, 1);
    }

    #[test]
    fn baseline_replacement_requires_explicit_replace() {
        let dir = tempdir().expect("temporary directory");
        let path = dir.path().join("baseline.json");
        let first = IntegrityBaseline {
            version: BASELINE_VERSION,
            created_at_ms: 1,
            entries: Vec::new(),
            unavailable_surfaces: Vec::new(),
        };
        let second = IntegrityBaseline {
            version: BASELINE_VERSION,
            created_at_ms: 2,
            entries: Vec::new(),
            unavailable_surfaces: Vec::new(),
        };

        write_baseline(&path, &first, false).expect("initial baseline");
        assert!(write_baseline(&path, &second, false).is_err());
        write_baseline(&path, &second, true).expect("replace baseline");
        assert_eq!(read_baseline(&path).expect("updated").created_at_ms, 2);
    }

    #[test]
    fn invalid_baseline_version_is_rejected() {
        let dir = tempdir().expect("temporary directory");
        let path = dir.path().join("baseline.json");
        fs::write(
            &path,
            br#"{"version":99,"created_at_ms":0,"entries":[],"unavailable_surfaces":[]}"#,
        )
        .expect("write invalid");
        assert!(read_baseline(&path).is_err());
    }
}
