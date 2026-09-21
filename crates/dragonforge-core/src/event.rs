//! Shared security and operational event metadata.

use std::time::SystemTime;

use crate::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Severity {
    Debug,
    Info,
    Notice,
    Warning,
    Critical,
}

impl Severity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Notice => "notice",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    Lifecycle,
    Security,
    Configuration,
    Ipc,
    Health,
    Audit,
}

impl EventKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lifecycle => "lifecycle",
            Self::Security => "security",
            Self::Configuration => "configuration",
            Self::Ipc => "ipc",
            Self::Health => "health",
            Self::Audit => "audit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    event_id: u128,
    timestamp: SystemTime,
    component: Component,
    kind: EventKind,
    severity: Severity,
    code: String,
    safe_summary: String,
}

impl EventRecord {
    #[must_use]
    pub fn new(
        event_id: u128,
        component: Component,
        kind: EventKind,
        severity: Severity,
        code: impl Into<String>,
        safe_summary: impl Into<String>,
    ) -> Self {
        Self {
            event_id,
            timestamp: SystemTime::now(),
            component,
            kind,
            severity,
            code: code.into(),
            safe_summary: safe_summary.into(),
        }
    }

    #[must_use]
    pub const fn event_id(&self) -> u128 {
        self.event_id
    }

    #[must_use]
    pub const fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    #[must_use]
    pub const fn component(&self) -> Component {
        self.component
    }

    #[must_use]
    pub const fn kind(&self) -> EventKind {
        self.kind
    }

    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    #[must_use]
    pub fn safe_summary(&self) -> &str {
        &self.safe_summary
    }
}

#[cfg(test)]
mod tests {
    use super::{EventKind, EventRecord, Severity};
    use crate::Component;

    #[test]
    fn event_metadata_is_stable() {
        let event = EventRecord::new(
            7,
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.started",
            "Security Center started",
        );

        assert_eq!(event.event_id(), 7);
        assert_eq!(event.component(), Component::SecurityCenter);
        assert_eq!(event.kind().as_str(), "lifecycle");
        assert_eq!(event.severity().as_str(), "info");
        assert_eq!(event.code(), "security-center.started");
    }
}
