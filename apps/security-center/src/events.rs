use std::collections::VecDeque;
use std::time::UNIX_EPOCH;

use dragonforge_core::{Component, EventKind, EventRecord, Severity};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DashboardEvent {
    pub id: u64,
    pub timestamp_ms: u64,
    pub component: &'static str,
    pub kind: &'static str,
    pub severity: &'static str,
    pub code: String,
    pub summary: String,
}

impl From<EventRecord> for DashboardEvent {
    fn from(event: EventRecord) -> Self {
        let timestamp_ms = event
            .timestamp()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX);

        Self {
            id: event.event_id(),
            timestamp_ms,
            component: event.component().as_str(),
            kind: event.kind().as_str(),
            severity: event.severity().as_str(),
            code: event.code().to_owned(),
            summary: event.safe_summary().to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct EventStore {
    events: VecDeque<DashboardEvent>,
    capacity: usize,
    next_id: u64,
}

impl EventStore {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity.min(4096)),
            capacity: capacity.max(1),
            next_id: 1,
        }
    }

    pub fn push(
        &mut self,
        component: Component,
        kind: EventKind,
        severity: Severity,
        code: impl Into<String>,
        safe_summary: impl Into<String>,
    ) {
        if self.events.len() == self.capacity {
            self.events.pop_front();
        }

        let record = EventRecord::new(
            u128::from(self.next_id),
            component,
            kind,
            severity,
            code,
            safe_summary,
        );
        self.next_id = self.next_id.saturating_add(1);
        self.events.push_back(record.into());
    }

    #[must_use]
    pub fn recent(&self, limit: usize) -> Vec<DashboardEvent> {
        self.events
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity.max(1);
        while self.events.len() > self.capacity {
            self.events.pop_front();
        }
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new(250)
    }
}

#[cfg(test)]
mod tests {
    use dragonforge_core::{Component, EventKind, Severity};

    use super::EventStore;

    #[test]
    fn event_store_is_bounded_and_returns_newest_first() {
        let mut store = EventStore::new(2);
        store.push(
            Component::SecurityCenter,
            EventKind::Health,
            Severity::Info,
            "one",
            "one",
        );
        store.push(
            Component::SecurityCenter,
            EventKind::Health,
            Severity::Info,
            "two",
            "two",
        );
        store.push(
            Component::SecurityCenter,
            EventKind::Health,
            Severity::Info,
            "three",
            "three",
        );

        let recent = store.recent(10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].code, "three");
        assert_eq!(recent[1].code, "two");
    }
}
