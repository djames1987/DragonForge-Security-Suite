use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    let commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=DRAGONFORGE_BUILD_COMMIT={commit}");
    tauri_build::build();
}
