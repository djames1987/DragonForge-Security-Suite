#[cfg(windows)]
fn main() -> std::process::ExitCode {
    match dragonforge_privileged_service::windows::run_dispatcher() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::from(1),
    }
}

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("dragonforge-privileged-service is Windows-only");
    std::process::ExitCode::from(1)
}
