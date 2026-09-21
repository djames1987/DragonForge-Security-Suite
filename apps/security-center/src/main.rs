#![forbid(unsafe_code)]

use dragonforge_core::{Component, EventKind, EventRecord, Platform, Severity, SuitePaths};

fn main() {
    let startup = EventRecord::new(
        1,
        Component::SecurityCenter,
        EventKind::Lifecycle,
        Severity::Info,
        "security-center.started",
        "Security Center foundation started",
    );

    let path_status = if SuitePaths::discover().is_ok() {
        "paths-ready"
    } else {
        "paths-unavailable"
    };

    println!(
        "DragonForge Security Center foundation ({}, {}, {}, {})",
        Component::SecurityCenter.as_str(),
        Platform::current().as_str(),
        startup.code(),
        path_status
    );
}
