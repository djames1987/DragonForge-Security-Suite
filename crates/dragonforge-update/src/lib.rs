#![forbid(unsafe_code)]

//! Fail-closed update manifest verification for DragonForge Security Suite.
//!
//! Transport is not a trust boundary. A candidate is usable only after its
//! ML-DSA-65 manifest signature, release channel, version policy, artifact
//! metadata, and SHA-256 metadata validate.

use std::fmt;

use dragonforge_crypto::{MlDsa65KeyPair, MlDsa65VerifyingKey};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const UPDATE_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const UPDATE_SIGNATURE_ALGORITHM: &str = "ML-DSA-65";
pub const MAX_MANIFEST_BYTES: usize = 256 * 1024;
pub const MAX_UPDATE_ARTIFACT_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    Alpha,
    Beta,
    #[default]
    Stable,
}

impl UpdateChannel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Stable => "stable",
        }
    }
}

impl fmt::Display for UpdateChannel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateArtifact {
    pub name: String,
    pub kind: String,
    pub url: String,
    pub sha256: String,
    pub bytes: u64,
    pub authenticode_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifestPayload {
    pub schema_version: u32,
    pub channel: UpdateChannel,
    pub version: String,
    pub tag: String,
    pub commit: String,
    pub published_utc: String,
    pub artifacts: Vec<UpdateArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifestSignature {
    pub algorithm: String,
    pub key_id: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedUpdateManifest {
    pub payload: UpdateManifestPayload,
    pub signature: UpdateManifestSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedUpdateManifest {
    pub payload: UpdateManifestPayload,
    pub candidate_version: Version,
}

impl VerifiedUpdateManifest {
    #[must_use]
    pub fn installer(&self) -> Option<&UpdateArtifact> {
        self.payload
            .artifacts
            .iter()
            .find(|artifact| artifact.kind == "windows-installer")
    }
}

pub fn sign_manifest(
    payload: UpdateManifestPayload,
    key_id: impl Into<String>,
    key_pair: &MlDsa65KeyPair,
) -> Result<SignedUpdateManifest, String> {
    validate_payload_shape(&payload)?;
    let bytes = signing_bytes(&payload)?;
    Ok(SignedUpdateManifest {
        payload,
        signature: UpdateManifestSignature {
            algorithm: UPDATE_SIGNATURE_ALGORITHM.to_owned(),
            key_id: key_id.into(),
            signature_hex: hex::encode_upper(key_pair.sign(&bytes)),
        },
    })
}

pub fn verify_manifest(
    manifest_bytes: &[u8],
    expected_key_id: &str,
    public_key_hex: &str,
    current_version: &str,
    selected_channel: UpdateChannel,
) -> Result<VerifiedUpdateManifest, String> {
    if manifest_bytes.len() > MAX_MANIFEST_BYTES {
        return Err("update manifest exceeds the maximum allowed size".to_owned());
    }
    let manifest: SignedUpdateManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|_| "update manifest is invalid JSON".to_owned())?;
    validate_payload_shape(&manifest.payload)?;

    if manifest.signature.algorithm != UPDATE_SIGNATURE_ALGORITHM {
        return Err("update manifest uses an unsupported signature algorithm".to_owned());
    }
    if manifest.signature.key_id != expected_key_id {
        return Err("update manifest key identity does not match the pinned release key".to_owned());
    }

    let public_key = hex::decode(public_key_hex)
        .map_err(|_| "configured update public key is not valid hexadecimal".to_owned())?;
    let verifying_key = MlDsa65VerifyingKey::from_bytes(&public_key)
        .map_err(|_| "configured update public key is invalid".to_owned())?;
    let signature = hex::decode(&manifest.signature.signature_hex)
        .map_err(|_| "update manifest signature is not valid hexadecimal".to_owned())?;
    let bytes = signing_bytes(&manifest.payload)?;
    verifying_key
        .verify(&bytes, &signature)
        .map_err(|_| "update manifest signature verification failed".to_owned())?;

    if manifest.payload.channel != selected_channel {
        return Err("update manifest channel does not match the selected channel".to_owned());
    }

    let current = Version::parse(current_version)
        .map_err(|_| "installed DragonForge version is not valid SemVer".to_owned())?;
    let candidate = Version::parse(&manifest.payload.version)
        .map_err(|_| "update manifest version is not valid SemVer".to_owned())?;
    if candidate < current {
        return Err("update candidate would downgrade the installed version".to_owned());
    }
    if !version_matches_channel(&candidate, selected_channel) {
        return Err("update version prerelease label does not match the selected channel".to_owned());
    }

    Ok(VerifiedUpdateManifest {
        payload: manifest.payload,
        candidate_version: candidate,
    })
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode_upper(Sha256::digest(bytes))
}

fn signing_bytes(payload: &UpdateManifestPayload) -> Result<Vec<u8>, String> {
    serde_json::to_vec(payload).map_err(|_| "unable to serialize update manifest payload".to_owned())
}

fn validate_payload_shape(payload: &UpdateManifestPayload) -> Result<(), String> {
    if payload.schema_version != UPDATE_MANIFEST_SCHEMA_VERSION {
        return Err("unsupported update manifest schema version".to_owned());
    }
    if payload.tag != format!("v{}", payload.version) {
        return Err("update manifest tag/version identity mismatch".to_owned());
    }
    if payload.commit.len() != 40 || !payload.commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("update manifest commit must be a full Git commit SHA".to_owned());
    }
    if payload.published_utc.trim().is_empty() {
        return Err("update manifest publication time is missing".to_owned());
    }
    if payload.artifacts.is_empty() {
        return Err("update manifest contains no artifacts".to_owned());
    }

    let mut has_installer = false;
    for artifact in &payload.artifacts {
        validate_artifact(artifact)?;
        if artifact.kind == "windows-installer" {
            if !artifact.authenticode_required {
                return Err("Windows update installer must require Authenticode".to_owned());
            }
            has_installer = true;
        }
    }
    if !has_installer {
        return Err("update manifest does not contain a Windows installer".to_owned());
    }
    Ok(())
}

fn validate_artifact(artifact: &UpdateArtifact) -> Result<(), String> {
    if artifact.name.is_empty()
        || artifact.name.len() > 160
        || !artifact
            .name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err("update artifact has an unsafe file name".to_owned());
    }
    if !artifact.url.starts_with("https://") || artifact.url.len() > 2_048 {
        return Err("update artifact URL must use bounded HTTPS".to_owned());
    }
    if artifact.sha256.len() != 64
        || !artifact.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("update artifact SHA-256 is invalid".to_owned());
    }
    if artifact.bytes == 0 || artifact.bytes > MAX_UPDATE_ARTIFACT_BYTES {
        return Err("update artifact size is outside the allowed range".to_owned());
    }
    Ok(())
}

fn version_matches_channel(version: &Version, channel: UpdateChannel) -> bool {
    if version.pre.is_empty() {
        return channel == UpdateChannel::Stable;
    }
    let prerelease = version.pre.as_str().to_ascii_lowercase();
    match channel {
        UpdateChannel::Alpha => prerelease.starts_with("alpha"),
        UpdateChannel::Beta => prerelease.starts_with("beta"),
        UpdateChannel::Stable => false,
    }
}

#[cfg(test)]
mod tests {
    use dragonforge_crypto::MlDsa65KeyPair;

    use super::{
        SignedUpdateManifest, UpdateArtifact, UpdateChannel, UpdateManifestPayload, sha256_hex,
        sign_manifest, verify_manifest,
    };

    fn payload(version: &str, channel: UpdateChannel) -> UpdateManifestPayload {
        UpdateManifestPayload {
            schema_version: 1,
            channel,
            version: version.to_owned(),
            tag: format!("v{version}"),
            commit: "0123456789abcdef0123456789abcdef01234567".to_owned(),
            published_utc: "2026-09-22T20:00:00Z".to_owned(),
            artifacts: vec![UpdateArtifact {
                name: "DragonForge-Security-Suite-setup.exe".to_owned(),
                kind: "windows-installer".to_owned(),
                url: "https://example.invalid/DragonForge-Security-Suite-setup.exe".to_owned(),
                sha256: "A".repeat(64),
                bytes: 1024,
                authenticode_required: true,
            }],
        }
    }

    #[test]
    fn signed_manifest_round_trip_verifies() {
        let key = MlDsa65KeyPair::from_seed(&[7_u8; 32]).expect("test key");
        let signed = sign_manifest(payload("0.2.0-beta.1", UpdateChannel::Beta), "test-key", &key)
            .expect("sign");
        let json = serde_json::to_vec(&signed).expect("json");
        let verified = verify_manifest(
            &json,
            "test-key",
            &hex::encode(key.verifying_key().as_bytes()),
            "0.1.0",
            UpdateChannel::Beta,
        )
        .expect("verify");
        assert_eq!(verified.payload.version, "0.2.0-beta.1");
    }

    #[test]
    fn signature_tampering_is_rejected() {
        let key = MlDsa65KeyPair::from_seed(&[8_u8; 32]).expect("test key");
        let signed = sign_manifest(payload("0.2.0-alpha.1", UpdateChannel::Alpha), "test-key", &key)
            .expect("sign");
        let mut value = serde_json::to_value(&signed).expect("value");
        value["payload"]["commit"] =
            serde_json::Value::String("fedcba9876543210fedcba9876543210fedcba98".to_owned());
        let json = serde_json::to_vec(&value).expect("json");
        assert!(verify_manifest(
            &json,
            "test-key",
            &hex::encode(key.verifying_key().as_bytes()),
            "0.1.0",
            UpdateChannel::Alpha,
        )
        .is_err());
    }

    #[test]
    fn downgrade_and_channel_mismatch_are_rejected() {
        let key = MlDsa65KeyPair::from_seed(&[9_u8; 32]).expect("test key");
        let signed = sign_manifest(payload("0.1.0", UpdateChannel::Stable), "test-key", &key)
            .expect("sign");
        let json = serde_json::to_vec(&signed).expect("json");
        let current = verify_manifest(
            &json,
            "test-key",
            &hex::encode(key.verifying_key().as_bytes()),
            "0.1.0",
            UpdateChannel::Stable,
        )
        .expect("same version is a valid signed manifest");
        assert_eq!(current.payload.version, "0.1.0");

        let older = sign_manifest(payload("0.1.0", UpdateChannel::Stable), "test-key", &key)
            .expect("sign");
        let older_json = serde_json::to_vec(&older).expect("json");
        assert!(verify_manifest(
            &older_json,
            "test-key",
            &hex::encode(key.verifying_key().as_bytes()),
            "0.2.0",
            UpdateChannel::Stable,
        )
        .is_err());

        let newer = sign_manifest(payload("0.2.0-beta.1", UpdateChannel::Beta), "test-key", &key)
            .expect("sign");
        let json = serde_json::to_vec(&newer).expect("json");
        assert!(verify_manifest(
            &json,
            "test-key",
            &hex::encode(key.verifying_key().as_bytes()),
            "0.1.0",
            UpdateChannel::Alpha,
        )
        .is_err());
    }

    #[test]
    fn unsigned_or_non_authenticode_installer_is_rejected() {
        let key = MlDsa65KeyPair::from_seed(&[10_u8; 32]).expect("test key");
        let mut candidate = payload("0.2.0", UpdateChannel::Stable);
        candidate.artifacts[0].authenticode_required = false;
        assert!(sign_manifest(candidate, "test-key", &key).is_err());
    }


    #[test]
    fn wrong_pinned_key_identity_is_rejected() {
        let key = MlDsa65KeyPair::from_seed(&[11_u8; 32]).expect("test key");
        let signed = sign_manifest(payload("0.2.0", UpdateChannel::Stable), "release-key-a", &key)
            .expect("sign");
        let json = serde_json::to_vec(&signed).expect("json");
        assert!(verify_manifest(
            &json,
            "release-key-b",
            &hex::encode(key.verifying_key().as_bytes()),
            "0.1.0",
            UpdateChannel::Stable,
        )
        .is_err());
    }

    #[test]
    fn sha256_helper_is_stable() {
        assert_eq!(
            sha256_hex(b"dragonforge"),
            "C6825A9A106557AA80DE352543522D9BFC31737EBAE2FCE00E269334F2B34D60"
        );
    }

    #[test]
    fn manifest_deserialization_rejects_missing_signature() {
        let json = serde_json::to_vec(&payload("0.2.0", UpdateChannel::Stable)).expect("json");
        assert!(serde_json::from_slice::<SignedUpdateManifest>(&json).is_err());
    }
}
