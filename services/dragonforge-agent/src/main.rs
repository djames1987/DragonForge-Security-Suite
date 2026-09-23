#![forbid(unsafe_code)]

use std::env;
use std::process::ExitCode;

use dragonforge_agent::{AgentClient, AgentIntegrityRuntime, AgentServer};
#[cfg(windows)]
use dragonforge_agent::{
    FirewallAction, FirewallApplicationIdentity, FirewallMutationResult, PrivilegedServiceClient,
};
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
        #[cfg(windows)]
        Some("--firewall-status") => firewall_command(FirewallCliAction::Status),
        #[cfg(windows)]
        Some("--firewall-allow") => firewall_command(FirewallCliAction::Allow),
        #[cfg(windows)]
        Some("--firewall-block") => firewall_command(FirewallCliAction::Block),
        #[cfg(windows)]
        Some("--firewall-remove") => firewall_command(FirewallCliAction::Remove),
        #[cfg(windows)]
        Some("--firewall-rollback") => firewall_command(FirewallCliAction::Rollback),
        Some("--serve") | None => serve(),
        Some(_) => {
            eprintln!(
                "Usage: dragonforge-agent [--serve|--health|--stop|--integrity-status|--integrity-events|--privileged-health|--privileged-policy|--firewall-status|--firewall-allow|--firewall-block|--firewall-remove|--firewall-rollback]"
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

#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
enum FirewallCliAction {
    Status,
    Allow,
    Block,
    Remove,
    Rollback,
}

#[cfg(windows)]
fn firewall_command(action: FirewallCliAction) -> ExitCode {
    let mut args = env::args().skip(2);
    let rollback_token = if matches!(action, FirewallCliAction::Rollback) {
        match args.next() {
            Some(value) => Some(value),
            None => return firewall_usage(),
        }
    } else {
        None
    };
    let Some(path) = args.next() else {
        return firewall_usage();
    };
    let Some(sha256_hex) = args.next() else {
        return firewall_usage();
    };
    let Some(display_name) = args.next() else {
        return firewall_usage();
    };
    if args.next().is_some() {
        return firewall_usage();
    }

    let identity = FirewallApplicationIdentity {
        application_path: path.into(),
        sha256_hex,
        display_name,
    };
    let client = PrivilegedServiceClient::new();
    let result = match action {
        FirewallCliAction::Status => client.firewall_status(identity),
        FirewallCliAction::Allow => client.firewall_apply(identity, FirewallAction::Allow),
        FirewallCliAction::Block => client.firewall_apply(identity, FirewallAction::Block),
        FirewallCliAction::Remove => client.firewall_remove(identity),
        FirewallCliAction::Rollback => match rollback_token {
            Some(token) => client.firewall_rollback(identity, token),
            None => return firewall_usage(),
        },
    };
    print_firewall_result(result)
}

#[cfg(windows)]
fn firewall_usage() -> ExitCode {
    eprintln!(
        "Firewall usage: --firewall-status <exe> <sha256> <name> | --firewall-allow <exe> <sha256> <name> | --firewall-block <exe> <sha256> <name> | --firewall-remove <exe> <sha256> <name> | --firewall-rollback <token> <exe> <sha256> <name>"
    );
    ExitCode::from(2)
}

#[cfg(windows)]
fn print_firewall_result(result: dragonforge_agent::Result<FirewallMutationResult>) -> ExitCode {
    match result {
        Ok(result) => match serde_json::to_string_pretty(&result) {
            Ok(encoded) => {
                println!("{encoded}");
                ExitCode::SUCCESS
            }
            Err(_) => ExitCode::from(1),
        },
        Err(error) => {
            eprintln!("Firewall policy request failed: {error}");
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
