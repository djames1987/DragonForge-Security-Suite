use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

use dragonforge_update::{
    MAX_MANIFEST_BYTES, UpdateChannel, VerifiedUpdateManifest, sha256_hex, verify_manifest,
};
use reqwest::blocking::Client;
use semver::Version;
use serde::{Deserialize, Serialize};

const UPDATE_KEY_ID: &str = match option_env!("DRAGONFORGE_UPDATE_KEY_ID") {
    Some(value) => value,
    None => "dragonforge-release-v1",
};
const UPDATE_PUBLIC_KEY_HEX: Option<&str> = option_env!("DRAGONFORGE_UPDATE_PUBLIC_KEY_HEX");
const ALPHA_FEED: Option<&str> = option_env!("DRAGONFORGE_UPDATE_FEED_ALPHA");
const BETA_FEED: Option<&str> = option_env!("DRAGONFORGE_UPDATE_FEED_BETA");
const STABLE_FEED: Option<&str> = option_env!("DRAGONFORGE_UPDATE_FEED_STABLE");
const DEFAULT_RELEASES_API: &str =
    "https://api.github.com/repos/djames1987/DragonForge-Security-Suite/releases?per_page=50";
const MAX_RELEASE_DISCOVERY_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    draft: bool,
    assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatus {
    pub configured: bool,
    pub channel: UpdateChannel,
    pub current_version: String,
    pub available: bool,
    pub version: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PreparedUpdateStatus {
    pub version: String,
    pub artifact_name: String,
    pub ready: bool,
}

#[derive(Debug, Clone)]
struct PreparedUpdate {
    version: String,
    artifact_name: String,
    path: PathBuf,
    sha256: String,
}

pub struct UpdateManager {
    client: Client,
    prepared: Mutex<Option<PreparedUpdate>>,
}

impl UpdateManager {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("DragonForge-Security-Center/0.1")
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.url().scheme() == "https" && attempt.previous().len() < 5 {
                    attempt.follow()
                } else {
                    attempt.stop()
                }
            }))
            .build()
            .map_err(|_| "unable to initialize secure update HTTP client".to_owned())?;
        Ok(Self {
            client,
            prepared: Mutex::new(None),
        })
    }

    pub fn check(&self, channel: UpdateChannel) -> Result<UpdateStatus, String> {
        let current_version = env!("CARGO_PKG_VERSION").to_owned();
        if UPDATE_PUBLIC_KEY_HEX.is_none() {
            return Ok(UpdateStatus {
                configured: false,
                channel,
                current_version,
                available: false,
                version: None,
                detail: "Secure update feed is not configured in this build.".to_owned(),
            });
        }

        let verified = self.fetch_verified(channel)?;
        let current = Version::parse(&current_version)
            .map_err(|_| "installed DragonForge version is invalid".to_owned())?;
        let available = verified.candidate_version > current;
        Ok(UpdateStatus {
            configured: true,
            channel,
            current_version,
            available,
            version: Some(verified.payload.version.clone()),
            detail: if available {
                "A cryptographically verified update is available.".to_owned()
            } else {
                "This installation is current for the selected update channel.".to_owned()
            },
        })
    }

    pub fn prepare(&self, channel: UpdateChannel) -> Result<PreparedUpdateStatus, String> {
        let verified = self.fetch_verified(channel)?;
        let current = Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|_| "installed DragonForge version is invalid".to_owned())?;
        if verified.candidate_version <= current {
            return Err("no newer update is available for the selected channel".to_owned());
        }

        let artifact = verified
            .installer()
            .ok_or_else(|| "verified update manifest has no Windows installer".to_owned())?;
        let response = self
            .client
            .get(&artifact.url)
            .send()
            .map_err(|_| "unable to download the verified update installer".to_owned())?;
        if !response.status().is_success() || response.url().scheme() != "https" {
            return Err(
                "update installer download did not remain on successful HTTPS".to_owned(),
            );
        }
        if response
            .content_length()
            .is_some_and(|length| length != artifact.bytes)
        {
            return Err(
                "update installer content length does not match signed metadata".to_owned(),
            );
        }
        let bytes = response
            .bytes()
            .map_err(|_| "unable to read the update installer response".to_owned())?;
        if bytes.len() as u64 != artifact.bytes {
            return Err("downloaded update installer size does not match signed metadata".to_owned());
        }
        let actual_hash = sha256_hex(&bytes);
        if !actual_hash.eq_ignore_ascii_case(&artifact.sha256) {
            return Err("downloaded update installer SHA-256 does not match signed metadata".to_owned());
        }

        let directory = std::env::temp_dir().join("DragonForge").join("updates");
        fs::create_dir_all(&directory)
            .map_err(|_| "unable to create the secure update staging directory".to_owned())?;
        let final_path = directory.join(&artifact.name);
        let partial_path = directory.join(format!("{}.part", artifact.name));
        if partial_path.exists() {
            let _ = fs::remove_file(&partial_path);
        }
        {
            let mut file = fs::File::create(&partial_path)
                .map_err(|_| "unable to create the staged update installer".to_owned())?;
            file.write_all(&bytes)
                .map_err(|_| "unable to write the staged update installer".to_owned())?;
            file.sync_all()
                .map_err(|_| "unable to flush the staged update installer".to_owned())?;
        }
        if final_path.exists() {
            fs::remove_file(&final_path).map_err(|_| {
                "unable to replace the previously staged update installer".to_owned()
            })?;
        }
        fs::rename(&partial_path, &final_path)
            .map_err(|_| "unable to finalize the staged update installer".to_owned())?;

        verify_authenticode(&final_path).map_err(|error| {
            let _ = fs::remove_file(&final_path);
            error
        })?;

        let prepared = PreparedUpdate {
            version: verified.payload.version.clone(),
            artifact_name: artifact.name.clone(),
            path: final_path,
            sha256: artifact.sha256.clone(),
        };
        let status = PreparedUpdateStatus {
            version: prepared.version.clone(),
            artifact_name: prepared.artifact_name.clone(),
            ready: true,
        };
        *self
            .prepared
            .lock()
            .map_err(|_| "secure update state is unavailable".to_owned())? = Some(prepared);
        Ok(status)
    }

    pub fn install_prepared(&self) -> Result<(), String> {
        let prepared = self
            .prepared
            .lock()
            .map_err(|_| "secure update state is unavailable".to_owned())?
            .clone()
            .ok_or_else(|| "no verified update installer is prepared".to_owned())?;

        let bytes = fs::read(&prepared.path)
            .map_err(|_| "prepared update installer is no longer available".to_owned())?;
        if !sha256_hex(&bytes).eq_ignore_ascii_case(&prepared.sha256) {
            return Err("prepared update installer changed after verification".to_owned());
        }
        verify_authenticode(&prepared.path)?;

        Command::new(&prepared.path)
            .spawn()
            .map_err(|_| "unable to launch the verified update installer".to_owned())?;
        *self
            .prepared
            .lock()
            .map_err(|_| "secure update state is unavailable".to_owned())? = None;
        Ok(())
    }

    fn fetch_verified(&self, channel: UpdateChannel) -> Result<VerifiedUpdateManifest, String> {
        let public_key = UPDATE_PUBLIC_KEY_HEX
            .ok_or_else(|| "secure update public key is not configured in this build".to_owned())?;
        let bytes = self.fetch_manifest_bytes(channel)?;
        verify_manifest(
            &bytes,
            UPDATE_KEY_ID,
            public_key,
            env!("CARGO_PKG_VERSION"),
            channel,
        )
    }

    fn fetch_manifest_bytes(&self, channel: UpdateChannel) -> Result<Vec<u8>, String> {
        if let Some(feed) = feed_for(channel) {
            return self.fetch_bounded_https(feed, MAX_MANIFEST_BYTES as u64);
        }

        let discovery = self.fetch_bounded_https(DEFAULT_RELEASES_API, MAX_RELEASE_DISCOVERY_BYTES)?;
        let releases: Vec<GitHubRelease> = serde_json::from_slice(&discovery)
            .map_err(|_| "release discovery response is invalid JSON".to_owned())?;
        let expected_name = channel_asset_name(channel);
        let manifest_url = releases
            .iter()
            .filter(|release| !release.draft)
            .flat_map(|release| release.assets.iter())
            .find(|asset| asset.name == expected_name)
            .map(|asset| asset.browser_download_url.as_str())
            .ok_or_else(|| "no signed update manifest is published for the selected channel".to_owned())?;
        self.fetch_bounded_https(manifest_url, MAX_MANIFEST_BYTES as u64)
    }

    fn fetch_bounded_https(&self, url: &str, maximum_bytes: u64) -> Result<Vec<u8>, String> {
        if !url.starts_with("https://") || url.len() > 2_048 {
            return Err("secure update source must use bounded HTTPS".to_owned());
        }
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|_| "unable to retrieve secure update metadata".to_owned())?;
        if !response.status().is_success() || response.url().scheme() != "https" {
            return Err("secure update request did not remain on successful HTTPS".to_owned());
        }
        if response
            .content_length()
            .is_some_and(|length| length > maximum_bytes)
        {
            return Err("secure update response exceeds the maximum allowed size".to_owned());
        }
        let bytes = response
            .bytes()
            .map_err(|_| "unable to read secure update metadata".to_owned())?;
        if bytes.len() as u64 > maximum_bytes {
            return Err("secure update response exceeds the maximum allowed size".to_owned());
        }
        Ok(bytes.to_vec())
    }
}

fn feed_for(channel: UpdateChannel) -> Option<&'static str> {
    match channel {
        UpdateChannel::Alpha => ALPHA_FEED,
        UpdateChannel::Beta => BETA_FEED,
        UpdateChannel::Stable => STABLE_FEED,
    }
}

fn channel_asset_name(channel: UpdateChannel) -> String {
    format!("DragonForge-Security-Suite-update-{channel}.json")
}

#[cfg(windows)]
fn verify_authenticode(path: &Path) -> Result<(), String> {
    let script = "$p=[Environment]::GetEnvironmentVariable('DRAGONFORGE_VERIFY_PATH'); $s=Get-AuthenticodeSignature -LiteralPath $p; if ($s.Status -ne 'Valid') { exit 3 }";
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .env("DRAGONFORGE_VERIFY_PATH", path)
        .status()
        .map_err(|_| "unable to invoke Windows Authenticode verification".to_owned())?;
    if !status.success() {
        return Err("update installer does not have a valid Authenticode signature".to_owned());
    }
    Ok(())
}

#[cfg(not(windows))]
fn verify_authenticode(_path: &Path) -> Result<(), String> {
    Err("secure update installation is supported only on Windows".to_owned())
}

#[cfg(test)]
mod tests {
    use super::channel_asset_name;
    use dragonforge_update::UpdateChannel;

    #[test]
    fn channel_assets_are_explicit_and_distinct() {
        assert_eq!(
            channel_asset_name(UpdateChannel::Alpha),
            "DragonForge-Security-Suite-update-alpha.json"
        );
        assert_eq!(
            channel_asset_name(UpdateChannel::Beta),
            "DragonForge-Security-Suite-update-beta.json"
        );
        assert_eq!(
            channel_asset_name(UpdateChannel::Stable),
            "DragonForge-Security-Suite-update-stable.json"
        );
    }
}
