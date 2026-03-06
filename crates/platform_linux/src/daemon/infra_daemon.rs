/// infra-daemon: infrastructure services process.
/// Hosts: ConfigService, UpgradeService, NetworkManager, StorageManager, TimeSyncService.
pub struct InfraDaemon {
    running: bool,
}

impl InfraDaemon {
    pub fn new() -> Self {
        Self { running: false }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}
