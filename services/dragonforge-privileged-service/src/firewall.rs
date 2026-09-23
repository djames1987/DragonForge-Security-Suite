use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use dragonforge_windows_boundary::{
    FirewallAction, FirewallApplicationIdentity, FirewallMutationResult, FirewallPolicyState,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{Result, ServiceError};

const FIREWALL_STATE_VERSION: u16 = 1;
const MAX_MANAGED_POLICIES: usize = 256;
const MAX_ROLLBACKS: usize = 64;
const MAX_STATE_BYTES: usize = 1024 * 1024;
const MAX_APPLICATION_BYTES: u64 = 1024 * 1024 * 1024;
pub const FIREWALL_RULE_PREFIX: &str = "DragonForge Outbound ";
pub const FIREWALL_RULE_GROUP: &str = "DragonForge Security Suite";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FirewallStateFile {
    version: u16,
    policies: Vec<FirewallPolicyState>,
    rollbacks: Vec<RollbackRecord>,
}

impl Default for FirewallStateFile {
    fn default() -> Self {
        Self {
            version: FIREWALL_STATE_VERSION,
            policies: Vec::new(),
            rollbacks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RollbackRecord {
    token: String,
    rule_name: String,
    previous: Option<FirewallPolicyState>,
}

pub trait FirewallBackend {
    fn exists(&self, rule_name: &str) -> Result<bool>;
    fn apply(&self, policy: &FirewallPolicyState) -> Result<()>;
    fn remove(&self, rule_name: &str) -> Result<()>;
}

#[derive(Debug)]
pub struct FirewallManager<B> {
    state_path: PathBuf,
    backend: B,
}

impl<B: FirewallBackend> FirewallManager<B> {
    #[must_use]
    pub fn new(state_path: impl Into<PathBuf>, backend: B) -> Self {
        Self {
            state_path: state_path.into(),
            backend,
        }
    }

    pub fn status(
        &self,
        identity: &FirewallApplicationIdentity,
    ) -> Result<FirewallMutationResult> {
        identity
            .validate()
            .map_err(|_| ServiceError::RequestRejected)?;
        verify_application_identity(identity)?;
        let state = self.load_state()?;
        let rule_name = rule_name_for_path(&identity.application_path)?;
        let mut policy = state
            .policies
            .into_iter()
            .find(|policy| policy.rule_name == rule_name);
        if let Some(policy) = policy.as_mut() {
            policy.enabled = self.backend.exists(&rule_name)?;
            policy.identity_matches =
                policy.sha256_hex.eq_ignore_ascii_case(&identity.sha256_hex);
        }
        Ok(FirewallMutationResult {
            changed: false,
            rollback_token: None,
            policy,
        })
    }

    pub fn apply(
        &self,
        identity: FirewallApplicationIdentity,
        action: FirewallAction,
        rollback_token: String,
    ) -> Result<FirewallMutationResult> {
        identity
            .validate()
            .map_err(|_| ServiceError::RequestRejected)?;
        verify_application_identity(&identity)?;

        let mut state = self.load_state()?;
        let rule_name = rule_name_for_path(&identity.application_path)?;
        let existing_index = state
            .policies
            .iter()
            .position(|policy| policy.rule_name == rule_name);

        if existing_index.is_none() && self.backend.exists(&rule_name)? {
            return Err(ServiceError::RequestRejected);
        }

        let previous = existing_index.map(|index| state.policies[index].clone());
        let policy = FirewallPolicyState {
            rule_name: rule_name.clone(),
            application_path: identity.application_path,
            sha256_hex: identity.sha256_hex.to_ascii_lowercase(),
            action,
            enabled: true,
            identity_matches: true,
        };

        self.backend.apply(&policy)?;
        match existing_index {
            Some(index) => state.policies[index] = policy.clone(),
            None => {
                if state.policies.len() >= MAX_MANAGED_POLICIES {
                    let _ = self.restore_backend(previous.as_ref(), &rule_name);
                    return Err(ServiceError::RequestRejected);
                }
                state.policies.push(policy.clone());
            }
        }
        push_rollback(
            &mut state,
            RollbackRecord {
                token: rollback_token.clone(),
                rule_name,
                previous,
            },
        );
        if let Err(error) = self.save_state(&state) {
            let record = state.rollbacks.last().cloned();
            if let Some(record) = record {
                let _ = self.restore_backend(record.previous.as_ref(), &record.rule_name);
            }
            return Err(error);
        }

        Ok(FirewallMutationResult {
            changed: true,
            rollback_token: Some(rollback_token),
            policy: Some(policy),
        })
    }

    pub fn remove(
        &self,
        identity: FirewallApplicationIdentity,
        rollback_token: String,
    ) -> Result<FirewallMutationResult> {
        identity
            .validate()
            .map_err(|_| ServiceError::RequestRejected)?;
        verify_application_identity(&identity)?;

        let mut state = self.load_state()?;
        let rule_name = rule_name_for_path(&identity.application_path)?;
        let Some(index) = state
            .policies
            .iter()
            .position(|policy| policy.rule_name == rule_name)
        else {
            return Ok(FirewallMutationResult {
                changed: false,
                rollback_token: None,
                policy: None,
            });
        };
        let previous = state.policies[index].clone();
        if !same_path(&previous.application_path, &identity.application_path) {
            return Err(ServiceError::RequestRejected);
        }

        self.backend.remove(&rule_name)?;
        state.policies.remove(index);
        push_rollback(
            &mut state,
            RollbackRecord {
                token: rollback_token.clone(),
                rule_name: rule_name.clone(),
                previous: Some(previous.clone()),
            },
        );
        if let Err(error) = self.save_state(&state) {
            let _ = self.backend.apply(&previous);
            return Err(error);
        }

        Ok(FirewallMutationResult {
            changed: true,
            rollback_token: Some(rollback_token),
            policy: None,
        })
    }

    pub fn rollback(
        &self,
        identity: FirewallApplicationIdentity,
        rollback_token: &str,
    ) -> Result<FirewallMutationResult> {
        identity
            .validate()
            .map_err(|_| ServiceError::RequestRejected)?;
        verify_application_identity(&identity)?;
        if rollback_token.len() != 32
            || !rollback_token
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(ServiceError::RequestRejected);
        }

        let mut state = self.load_state()?;
        let rule_name = rule_name_for_path(&identity.application_path)?;
        let Some(index) = state
            .rollbacks
            .iter()
            .position(|record| record.token == rollback_token && record.rule_name == rule_name)
        else {
            return Err(ServiceError::RequestRejected);
        };
        let record = state.rollbacks.remove(index);

        match &record.previous {
            Some(previous) => {
                let mut restored = previous.clone();
                restored.sha256_hex = identity.sha256_hex.to_ascii_lowercase();
                restored.identity_matches = true;
                self.backend.apply(&restored)?;
                upsert_policy(&mut state, restored)?;
            }
            None => {
                if self.backend.exists(&record.rule_name)? {
                    self.backend.remove(&record.rule_name)?;
                }
                state
                    .policies
                    .retain(|policy| policy.rule_name != record.rule_name);
            }
        }
        self.save_state(&state)?;

        Ok(FirewallMutationResult {
            changed: true,
            rollback_token: None,
            policy: record.previous,
        })
    }

    fn restore_backend(
        &self,
        previous: Option<&FirewallPolicyState>,
        rule_name: &str,
    ) -> Result<()> {
        match previous {
            Some(policy) => self.backend.apply(policy),
            None => {
                if self.backend.exists(rule_name)? {
                    self.backend.remove(rule_name)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn load_state(&self) -> Result<FirewallStateFile> {
        if !self.state_path.exists() {
            return Ok(FirewallStateFile::default());
        }
        let bytes = fs::read(&self.state_path).map_err(|_| ServiceError::Io)?;
        if bytes.len() > MAX_STATE_BYTES {
            return Err(ServiceError::InvalidConfiguration);
        }
        let state: FirewallStateFile =
            serde_json::from_slice(&bytes).map_err(|_| ServiceError::InvalidConfiguration)?;
        if state.version != FIREWALL_STATE_VERSION
            || state.policies.len() > MAX_MANAGED_POLICIES
            || state.rollbacks.len() > MAX_ROLLBACKS
        {
            return Err(ServiceError::InvalidConfiguration);
        }
        Ok(state)
    }

    fn save_state(&self, state: &FirewallStateFile) -> Result<()> {
        let parent = self
            .state_path
            .parent()
            .ok_or(ServiceError::InvalidConfiguration)?;
        fs::create_dir_all(parent).map_err(|_| ServiceError::Io)?;
        let bytes = serde_json::to_vec_pretty(state).map_err(|_| ServiceError::Io)?;
        if bytes.len() > MAX_STATE_BYTES {
            return Err(ServiceError::Io);
        }
        let temporary = self.state_path.with_extension("json.tmp");
        let backup = self.state_path.with_extension("json.bak");
        fs::write(&temporary, bytes).map_err(|_| ServiceError::Io)?;
        if backup.exists() {
            fs::remove_file(&backup).map_err(|_| ServiceError::Io)?;
        }
        if self.state_path.exists() {
            fs::rename(&self.state_path, &backup).map_err(|_| ServiceError::Io)?;
        }
        if fs::rename(&temporary, &self.state_path).is_err() {
            if backup.exists() && !self.state_path.exists() {
                let _ = fs::rename(&backup, &self.state_path);
            }
            return Err(ServiceError::Io);
        }
        if backup.exists() {
            fs::remove_file(backup).map_err(|_| ServiceError::Io)?;
        }
        Ok(())
    }
}

fn upsert_policy(state: &mut FirewallStateFile, policy: FirewallPolicyState) -> Result<()> {
    if let Some(index) = state
        .policies
        .iter()
        .position(|current| current.rule_name == policy.rule_name)
    {
        state.policies[index] = policy;
        return Ok(());
    }
    if state.policies.len() >= MAX_MANAGED_POLICIES {
        return Err(ServiceError::RequestRejected);
    }
    state.policies.push(policy);
    Ok(())
}

fn push_rollback(state: &mut FirewallStateFile, record: RollbackRecord) {
    state.rollbacks.retain(|item| item.token != record.token);
    state.rollbacks.push(record);
    if state.rollbacks.len() > MAX_ROLLBACKS {
        let remove = state.rollbacks.len() - MAX_ROLLBACKS;
        state.rollbacks.drain(0..remove);
    }
}

pub fn verify_application_identity(identity: &FirewallApplicationIdentity) -> Result<()> {
    let file_name = identity
        .application_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(ServiceError::RequestRejected)?;
    if matches!(
        file_name.to_ascii_lowercase().as_str(),
        "dragonforge-agent.exe" | "dragonforge-privileged-service.exe"
    ) {
        return Err(ServiceError::RequestRejected);
    }

    let metadata = fs::symlink_metadata(&identity.application_path).map_err(|_| ServiceError::Io)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_APPLICATION_BYTES
    {
        return Err(ServiceError::RequestRejected);
    }
    let actual = sha256_file(&identity.application_path)?;
    if !actual.eq_ignore_ascii_case(&identity.sha256_hex) {
        return Err(ServiceError::RequestRejected);
    }
    Ok(())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(|_| ServiceError::Io)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|_| ServiceError::Io)?;
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

pub fn rule_name_for_path(path: &Path) -> Result<String> {
    if !path.is_absolute() {
        return Err(ServiceError::RequestRejected);
    }
    let normalized = path.to_string_lossy().to_ascii_lowercase();
    let digest = Sha256::digest(normalized.as_bytes());
    let suffix = digest
        .iter()
        .take(10)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!("{FIREWALL_RULE_PREFIX}{suffix}"))
}

fn same_path(left: &Path, right: &Path) -> bool {
    left.to_string_lossy()
        .eq_ignore_ascii_case(&right.to_string_lossy())
}

#[cfg(windows)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsFirewallBackend;

#[cfg(windows)]
impl FirewallBackend for WindowsFirewallBackend {
    fn exists(&self, rule_name: &str) -> Result<bool> {
        with_rules(|rules| {
            let name = windows::core::BSTR::from(rule_name);
            Ok(unsafe { rules.Item(&name) }.is_ok())
        })
    }

    fn apply(&self, policy: &FirewallPolicyState) -> Result<()> {
        use windows::Win32::NetworkManagement::WindowsFirewall::{
            INetFwRule, NET_FW_ACTION_ALLOW, NET_FW_ACTION_BLOCK, NET_FW_PROFILE2_ALL,
            NET_FW_RULE_DIR_OUT, NetFwRule,
        };
        use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
        use windows::Win32::System::Variant::VARIANT_TRUE;

        with_rules(|rules| {
            let name = windows::core::BSTR::from(policy.rule_name.as_str());
            if unsafe { rules.Item(&name) }.is_ok() {
                unsafe { rules.Remove(&name) }
                    .ok()
                    .map_err(|_| ServiceError::Platform)?;
            }

            let rule: INetFwRule =
                unsafe { CoCreateInstance(&NetFwRule, None, CLSCTX_INPROC_SERVER) }
                    .map_err(|_| ServiceError::Platform)?;
            let path = windows::core::BSTR::from(policy.application_path.to_string_lossy().as_ref());
            let description = windows::core::BSTR::from(format!(
                "DragonForge managed outbound application policy; SHA-256 {}",
                policy.sha256_hex
            ));
            let grouping = windows::core::BSTR::from(FIREWALL_RULE_GROUP);
            let action = match policy.action {
                FirewallAction::Allow => NET_FW_ACTION_ALLOW,
                FirewallAction::Block => NET_FW_ACTION_BLOCK,
            };

            unsafe {
                rule.SetName(&name).ok().map_err(|_| ServiceError::Platform)?;
                rule.SetDescription(&description)
                    .ok()
                    .map_err(|_| ServiceError::Platform)?;
                rule.SetApplicationName(&path)
                    .ok()
                    .map_err(|_| ServiceError::Platform)?;
                rule.SetDirection(NET_FW_RULE_DIR_OUT)
                    .ok()
                    .map_err(|_| ServiceError::Platform)?;
                rule.SetProfiles(NET_FW_PROFILE2_ALL)
                    .ok()
                    .map_err(|_| ServiceError::Platform)?;
                rule.SetGrouping(&grouping)
                    .ok()
                    .map_err(|_| ServiceError::Platform)?;
                rule.SetEnabled(VARIANT_TRUE)
                    .ok()
                    .map_err(|_| ServiceError::Platform)?;
                rule.SetAction(action)
                    .ok()
                    .map_err(|_| ServiceError::Platform)?;
                rules.Add(&rule).ok().map_err(|_| ServiceError::Platform)?;
            }

            let current = unsafe { rules.Item(&name) }.map_err(|_| ServiceError::Platform)?;
            let current_path = unsafe { current.ApplicationName() }
                .map_err(|_| ServiceError::Platform)?
                .to_string();
            let current_group = unsafe { current.Grouping() }
                .map_err(|_| ServiceError::Platform)?
                .to_string();
            let current_direction =
                unsafe { current.Direction() }.map_err(|_| ServiceError::Platform)?;
            let current_action = unsafe { current.Action() }.map_err(|_| ServiceError::Platform)?;
            let current_profiles =
                unsafe { current.Profiles() }.map_err(|_| ServiceError::Platform)?;
            let current_enabled =
                unsafe { current.Enabled() }.map_err(|_| ServiceError::Platform)?;
            if !same_path(Path::new(&current_path), &policy.application_path)
                || current_group != FIREWALL_RULE_GROUP
                || current_direction != NET_FW_RULE_DIR_OUT
                || current_action != action
                || current_profiles != NET_FW_PROFILE2_ALL
                || current_enabled != VARIANT_TRUE
            {
                let _ = unsafe { rules.Remove(&name) };
                return Err(ServiceError::Platform);
            }
            Ok(())
        })
    }

    fn remove(&self, rule_name: &str) -> Result<()> {
        with_rules(|rules| {
            let name = windows::core::BSTR::from(rule_name);
            let Ok(rule) = (unsafe { rules.Item(&name) }) else {
                return Ok(());
            };
            let grouping = unsafe { rule.Grouping() }
                .map_err(|_| ServiceError::Platform)?
                .to_string();
            if grouping != FIREWALL_RULE_GROUP {
                return Err(ServiceError::RequestRejected);
            }
            unsafe { rules.Remove(&name) }
                .ok()
                .map_err(|_| ServiceError::Platform)
        })
    }
}

#[cfg(windows)]
fn with_rules<T>(
    operation: impl FnOnce(
        &windows::Win32::NetworkManagement::WindowsFirewall::INetFwRules,
    ) -> Result<T>,
) -> Result<T> {
    use windows::Win32::NetworkManagement::WindowsFirewall::{INetFwPolicy2, NetFwPolicy2};
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
    };

    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    if initialized.is_err() {
        return Err(ServiceError::Platform);
    }
    let result = (|| {
        let policy: INetFwPolicy2 =
            unsafe { CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER) }
                .map_err(|_| ServiceError::Platform)?;
        let rules = unsafe { policy.Rules() }.map_err(|_| ServiceError::Platform)?;
        operation(&rules)
    })();
    unsafe {
        CoUninitialize();
    }
    result
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use tempfile::tempdir;

    use super::*;

    #[derive(Debug, Default)]
    struct FakeBackend {
        rules: RefCell<Vec<FirewallPolicyState>>,
    }

    impl FirewallBackend for FakeBackend {
        fn exists(&self, rule_name: &str) -> Result<bool> {
            Ok(self
                .rules
                .borrow()
                .iter()
                .any(|policy| policy.rule_name == rule_name))
        }

        fn apply(&self, policy: &FirewallPolicyState) -> Result<()> {
            let mut rules = self.rules.borrow_mut();
            rules.retain(|current| current.rule_name != policy.rule_name);
            rules.push(policy.clone());
            Ok(())
        }

        fn remove(&self, rule_name: &str) -> Result<()> {
            self.rules
                .borrow_mut()
                .retain(|policy| policy.rule_name != rule_name);
            Ok(())
        }
    }

    fn executable(dir: &Path, name: &str) -> FirewallApplicationIdentity {
        let path = dir.join(name);
        fs::write(&path, b"dragonforge-phase19-test").expect("write exe");
        FirewallApplicationIdentity {
            application_path: path.clone(),
            sha256_hex: sha256_file(&path).expect("hash"),
            display_name: name.to_owned(),
        }
    }

    #[test]
    fn apply_remove_and_rollback_are_bounded_to_managed_rule() {
        let dir = tempdir().expect("tempdir");
        let identity = executable(dir.path(), "test.exe");
        let manager = FirewallManager::new(dir.path().join("firewall.json"), FakeBackend::default());

        let applied = manager
            .apply(
                identity.clone(),
                FirewallAction::Block,
                "00112233445566778899aabbccddeeff".to_owned(),
            )
            .expect("apply");
        assert!(applied.changed);
        assert_eq!(
            applied.policy.as_ref().map(|policy| policy.action),
            Some(FirewallAction::Block)
        );

        let removed = manager
            .remove(
                identity.clone(),
                "10112233445566778899aabbccddeeff".to_owned(),
            )
            .expect("remove");
        assert!(removed.changed);
        assert!(manager.status(&identity).expect("status").policy.is_none());

        let restored = manager
            .rollback(identity.clone(), "10112233445566778899aabbccddeeff")
            .expect("rollback");
        assert_eq!(
            restored.policy.as_ref().map(|policy| policy.action),
            Some(FirewallAction::Block)
        );
    }

    #[test]
    fn hash_mismatch_fails_closed_before_backend_mutation() {
        let dir = tempdir().expect("tempdir");
        let mut identity = executable(dir.path(), "test.exe");
        identity.sha256_hex = "00".repeat(32);
        let manager = FirewallManager::new(dir.path().join("firewall.json"), FakeBackend::default());
        assert!(
            manager
                .apply(
                    identity,
                    FirewallAction::Allow,
                    "20112233445566778899aabbccddeeff".to_owned(),
                )
                .is_err()
        );
    }

    #[test]
    fn control_plane_binaries_are_not_eligible_for_firewall_policy() {
        let dir = tempdir().expect("tempdir");
        for name in ["dragonforge-agent.exe", "dragonforge-privileged-service.exe"] {
            let identity = executable(dir.path(), name);
            assert!(verify_application_identity(&identity).is_err());
        }
    }

    #[test]
    fn rule_name_is_stable_for_windows_path_case() {
        let upper = Path::new(r"C:\Program Files\DragonForge\App.exe");
        let lower = Path::new(r"c:\program files\dragonforge\app.exe");
        assert_eq!(
            rule_name_for_path(upper).expect("upper"),
            rule_name_for_path(lower).expect("lower")
        );
    }
}
