use core_types::ServiceId;

/// Staged startup: services are started in topological order by level.
/// Level 0: Config
/// Level 1: Network, Storage
/// Level 2: TimeSync
/// Level 3: MediaCore
/// Level 4: Live, Talk, Record, Playback
/// Level 5: Cloud, Upgrade
/// Level 6: ControlGateway
pub const STARTUP_ORDER: &[&[ServiceId]] = &[
    &[ServiceId::Config],
    &[ServiceId::Network, ServiceId::Storage],
    &[ServiceId::TimeSync],
    &[ServiceId::MediaCore],
    &[ServiceId::Live, ServiceId::Talk, ServiceId::Record, ServiceId::Playback],
    &[ServiceId::Cloud, ServiceId::Upgrade],
    &[ServiceId::ControlGateway],
];

/// Application entry point for Linux multi-process mode.
pub struct AppEntry {
    started_levels: usize,
}

impl AppEntry {
    pub fn new() -> Self {
        Self { started_levels: 0 }
    }

    /// Get the next startup level to initialize.
    pub fn next_level(&self) -> Option<&'static [ServiceId]> {
        STARTUP_ORDER.get(self.started_levels).copied()
    }

    /// Mark a level as started.
    pub fn advance(&mut self) {
        if self.started_levels < STARTUP_ORDER.len() {
            self.started_levels += 1;
        }
    }

    pub fn all_started(&self) -> bool {
        self.started_levels >= STARTUP_ORDER.len()
    }

    pub fn current_level(&self) -> usize {
        self.started_levels
    }
}
