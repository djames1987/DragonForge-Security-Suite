#![forbid(unsafe_code)]

use std::env;
use std::process::ExitCode;

#[cfg(windows)]
use dragonforge_agent::PrivilegedServiceClient;
use dragonforge_agent::{AgentClient, AgentIntegrityRuntime, AgentServer};
use dragonforge_core::{Component, ComponentLogger, install_safe_panic_hook};

fn main() -> ExitCode {
    if let Ok(logger) = ComponentLogger::discover(Component::Agent, false) {
        install_safe_panic_hook(logger, Component::Agent);
    }
    match env::args().nth(1).as_deref() {
        Some("--health") => health(),
        Some("--stop") => stop(),
        Some("--integrity-status") => integrity_status(),
        Some("--integrity-events") => integrity_events(),
        #[cfg(windows)]
        Some("--privileged-health") => privileged_health(),
        #[cfg(windows)]
        Some("--privileged-policy") => privileged_policy(),
        Some("--serve") | None => serve(),
        Some(_) => {
            eprintln!(
                "Usage: dragonforge-agent [--serve|--health|--stop|--integrity-status|--integrity-events|--privileged-health|--privileged-policy]"
            );
            ExitCode::from(2)
        }
    }
}

fn serve() -> ExitCode {
    let server = match AgentServer::discover() {
        Ok(server) => server,
        Err(error) => {
            eprintln!("DragonForge Agent initialization failed: {error}");
            return ExitCode::from(1);
        }
    };

    match server.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("DragonForge Agent stopped: {error}");
            ExitCode::from(1)
        }
    }
}

fn health() -> ExitCode {
    let client = match AgentClient::discover() {
        Ok(client) => client,
        Err(error) => {
            eprintln!("DragonForge Agent health discovery failed: {error}");
            return ExitCode::from(1);
        }
    };
    match client.health() {
        Ok(health) => match serde_json::to_string_pretty(&health) {
            Ok(encoded) => {
                println!("{encoded}");
                ExitCode::SUCCESS
            }
            Err(_) => ExitCode::from(1),
        },
        Err(error) => {
            eprintln!("DragonForge Agent is unavailable: {error}");
            ExitCode::from(1)
        }
    }
}

fn stop() -> ExitCode {
    let client = match AgentClient::discover() {
        Ok(client) => client,
        Err(error) => {
            eprintln!("DragonForge Agent stop discovery failed: {error}");
            return ExitCode::from(1);
        }
    };
    match client.shutdown() {
        Ok(()) => {
            println!("DragonForge Agent graceful shutdown requested.");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("DragonForge Agent could not be stopped gracefully: {error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(windows)]
fn privileged_health() -> ExitCode {
    match PrivilegedServiceClient::new().health() {
        Ok(health) => match serde_json::to_string_pretty(&health) {
            Ok(encoded) => {
                println!("{encoded}");
                ExitCode::SUCCESS
            }
            Err(_) => ExitCode::from(1),
        },
        Err(error) => {
            eprintln!("DragonForge Privileged Service is unavailable: {error}");
            ExitCode::from(1)
        }
    }
}

#[cfg(windows)]
fn privileged_policy() -> ExitCode {
    match PrivilegedServiceClient::new().describe_policy() {
        Ok(policy) => match serde_json::to_string_pretty(&policy) {
            Ok(encoded) => {
                println!("{encoded}");
                ExitCode::SUCCESS
            }
            Err(_) => ExitCode::from(1),
        },
        Err(error) => {
            eprintln!("DragonForge Privileged Service policy is unavailable: {error}");
            ExitCode::from(1)
        }
    }
}

fn integrity_status() -> ExitCode {
    let runtime = match AgentIntegrityRuntime::discover() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("Continuous integrity runtime is unavailable: {error}");
            return ExitCode::from(1);
        }
    };
    match runtime.status() {
        Ok(status) => match serde_json::to_string_pretty(&status) {
            Ok(encoded) => {
                println!("{encoded}");
                ExitCode::SUCCESS
            }
            Err(_) => ExitCode::from(1),
        },
        Err(error) => {
            eprintln!("Continuous integrity status is unavailable: {error}");
            ExitCode::from(1)
        }
    }
}

fn integrity_events() -> ExitCode {
    let runtime = match AgentIntegrityRuntime::discover() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("Continuous integrity runtime is unavailable: {error}");
            return ExitCode::from(1);
        }
    };
    match runtime.events(0, 100) {
        Ok(events) => match serde_json::to_string_pretty(&events) {
            Ok(encoded) => {
                println!("{encoded}");
                ExitCode::SUCCESS
            }
            Err(_) => ExitCode::from(1),
        },
        Err(error) => {
            eprintln!("Continuous integrity events are unavailable: {error}");
            ExitCode::from(1)
        }
    }
}
