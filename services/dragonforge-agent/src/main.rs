#![forbid(unsafe_code)]

use std::env;
use std::process::ExitCode;

use dragonforge_agent::{AgentClient, AgentServer};
use dragonforge_core::{Component, ComponentLogger, install_safe_panic_hook};

fn main() -> ExitCode {
    if let Ok(logger) = ComponentLogger::discover(Component::Agent, false) {
        install_safe_panic_hook(logger, Component::Agent);
    }
    match env::args().nth(1).as_deref() {
        Some("--health") => health(),
        Some("--stop") => stop(),
        Some("--serve") | None => serve(),
        Some(_) => {
            eprintln!("Usage: dragonforge-agent [--serve|--health|--stop]");
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
