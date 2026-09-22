#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::PathBuf;

use dragonforge_crypto::MlDsa65KeyPair;
use dragonforge_update::{UpdateManifestPayload, sign_manifest};

fn main() {
    if let Err(error) = run() {
        eprintln!("UPDATE MANIFEST SIGNING: FAIL");
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "usage: dragonforge-sign-update-manifest <payload.json> <signed.json>".to_owned())?;
    let output = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "usage: dragonforge-sign-update-manifest <payload.json> <signed.json>".to_owned())?;
    if args.next().is_some() {
        return Err("unexpected extra arguments".to_owned());
    }

    let seed_hex = env::var("DRAGONFORGE_UPDATE_SIGNING_KEY_HEX")
        .map_err(|_| "DRAGONFORGE_UPDATE_SIGNING_KEY_HEX is required".to_owned())?;
    let key_id = env::var("DRAGONFORGE_UPDATE_KEY_ID")
        .map_err(|_| "DRAGONFORGE_UPDATE_KEY_ID is required".to_owned())?;
    if key_id.trim().is_empty() || key_id.len() > 128 {
        return Err("DRAGONFORGE_UPDATE_KEY_ID must be 1-128 characters".to_owned());
    }

    let seed = hex::decode(seed_hex)
        .map_err(|_| "update signing key seed is not valid hexadecimal".to_owned())?;
    let key_pair = MlDsa65KeyPair::from_seed(&seed)
        .map_err(|_| "update signing key seed is invalid".to_owned())?;

    let payload_bytes = fs::read(&input).map_err(|_| "unable to read update payload".to_owned())?;
    let payload: UpdateManifestPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|_| "update payload is invalid JSON".to_owned())?;
    let signed = sign_manifest(payload, key_id, &key_pair)?;
    let encoded = serde_json::to_vec_pretty(&signed)
        .map_err(|_| "unable to serialize signed update manifest".to_owned())?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| "unable to create signed manifest output directory".to_owned())?;
    }
    fs::write(&output, encoded).map_err(|_| "unable to write signed update manifest".to_owned())?;

    println!("UPDATE MANIFEST SIGNING: PASS");
    println!("Manifest: {}", output.display());
    println!(
        "Pinned public key (ML-DSA-65 hex): {}",
        hex::encode_upper(key_pair.verifying_key().as_bytes())
    );
    Ok(())
}
