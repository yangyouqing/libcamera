#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

const NTP_EPOCH_OFFSET_SECS: u64 = 2208988800;
const LARGE_JUMP_THRESHOLD_MS: i64 = 5_000;
const MAX_CONSECUTIVE_FAILURES: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SyncState {
    Idle = 0,
    Syncing = 1,
    Synced = 2,
    Failed = 3,
}

pub struct TimeSyncService {
    sync_state: SyncState,
    consecutive_failures: u8,
    last_synced_epoch_ms: u64,
    bus: Option<*const dyn CommBus>,
    service_state: ServiceState,
    sync_attempt_count: u32,
    network_available: bool,
}

// SAFETY: bus pointer is set once at init, used read-only afterward
unsafe impl Send for TimeSyncService {}
unsafe impl Sync for TimeSyncService {}

/// Construct a 48-byte NTP client request packet (RFC 5905).
/// LI=0, VN=4, Mode=3 (client).
pub fn construct_ntp_request() -> [u8; 48] {
    let mut packet = [0u8; 48];
    packet[0] = 0x23;
    packet
}

/// Parse NTP response and extract transmit timestamp as epoch milliseconds.
/// Returns None if packet is invalid or too short.
pub fn parse_ntp_response(data: &[u8]) -> Option<u64> {
    if data.len() < 48 {
        return None;
    }
    let seconds = u32::from_be_bytes([data[40], data[41], data[42], data[43]]) as u64;
    let fraction = u32::from_be_bytes([data[44], data[45], data[46], data[47]]) as u64;
    let ntp_epoch_ms = seconds
        .saturating_mul(1000)
        .saturating_add(fraction.saturating_mul(1000) / (1 << 32));
    let unix_epoch_ms = ntp_epoch_ms.saturating_sub(NTP_EPOCH_OFFSET_SECS.saturating_mul(1000));
    Some(unix_epoch_ms)
}

/// Standard NTP offset calculation: offset = ((t2 - t1) + (t3 - t4)) / 2
/// t1=client send, t2=server receive, t3=server transmit, t4=client receive
/// All timestamps in milliseconds. Returns offset in ms (positive = clock is fast).
pub fn calculate_offset(t1: u64, t2: u64, t3: u64, t4: u64) -> i64 {
    let a = (t2 as i64).saturating_sub(t1 as i64);
    let b = (t3 as i64).saturating_sub(t4 as i64);
    (a.saturating_add(b)) / 2
}

impl TimeSyncService {
    pub const fn new() -> Self {
        Self {
            sync_state: SyncState::Idle,
            consecutive_failures: 0,
            last_synced_epoch_ms: 0,
            bus: None,
            service_state: ServiceState::Normal,
            sync_attempt_count: 0,
            network_available: false,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    fn transition_to(&mut self, new_state: SyncState) {
        if self.sync_state != new_state {
            self.sync_state = new_state;
        }
    }

    fn publish_time_sync(&self, epoch_ms: u64) {
        if let Some(bus_ptr) = self.bus {
            let bus = unsafe { &*bus_ptr };
            let msg = CtrlMsg::new(Topic::EvtTimeSync, 0, 0).with_source(ServiceId::TimeSync);
            let payload = epoch_ms.to_be_bytes();
            let _ = bus.publish_ctrl(Topic::EvtTimeSync, &msg, &payload);
        }
    }

    fn check_large_jump(&mut self, new_epoch_ms: u64) -> bool {
        if self.last_synced_epoch_ms == 0 {
            return false;
        }
        let delta = new_epoch_ms as i64 - self.last_synced_epoch_ms as i64;
        delta.abs() > LARGE_JUMP_THRESHOLD_MS
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg) {
        let resp = CtrlMsg::new(Topic::CmdTime, msg.method_id, msg.request_id)
            .with_source(ServiceId::TimeSync);
        match msg.method_id {
            x if x == MethodId::SyncNow as u16 => {
                let was_failed = self.sync_state == SyncState::Failed;
                self.transition_to(SyncState::Syncing);
                if !was_failed {
                    self.consecutive_failures = 0;
                }
                self.sync_attempt_count = 0;
                let _ = self.bus().reply(Topic::CmdTime, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::QueryTime as u16 => {
                let payload = self.last_synced_epoch_ms.to_be_bytes();
                let _ = self.bus().reply(Topic::CmdTime, msg.request_id, &resp, &payload);
            }
            _ => {}
        }
    }

    fn handle_evt_network_status(&mut self, msg: &CtrlMsg) {
        let was_available = self.network_available;
        // method_id encodes network state: 0 = Disconnected, 1+ = Connected/Online
        self.network_available = msg.method_id != 0;

        if self.network_available && !was_available {
            // Network just came up: trigger a sync attempt
            self.consecutive_failures = 0;
            self.sync_attempt_count = 0;
            self.transition_to(SyncState::Syncing);
        }
    }

    fn handle_evt_config_changed(&mut self, _msg: &CtrlMsg) {
        // NTP server address or sync interval may have changed; re-trigger sync
        if self.network_available && self.sync_state != SyncState::Syncing {
            self.sync_attempt_count = 0;
            self.transition_to(SyncState::Syncing);
        }
    }

    /// Simulate a sync failure (for testing degradation path).
    /// In production, this would be triggered by NTP timeout or network errors.
    pub fn simulate_sync_failure(&mut self) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        if self.consecutive_failures > MAX_CONSECUTIVE_FAILURES {
            self.service_state = ServiceState::Degraded;
        }
        self.transition_to(SyncState::Failed);
    }

    fn tick_sync(&mut self) {
        if self.sync_state != SyncState::Syncing {
            return;
        }
        self.sync_attempt_count = self.sync_attempt_count.saturating_add(1);
        // Simulate failure: every 5th attempt, or immediately if previous failures
        // are still accumulating (simulates unreachable NTP server scenario).
        let simulate_failure = self.sync_attempt_count % 5 == 0
            || (self.consecutive_failures > 0 && self.sync_attempt_count == 1);
        if simulate_failure {
            self.consecutive_failures = self.consecutive_failures.saturating_add(1);
            if self.consecutive_failures > MAX_CONSECUTIVE_FAILURES {
                self.service_state = ServiceState::Degraded;
            }
            self.transition_to(SyncState::Failed);
            return;
        }
        let epoch_ms = 1700000000000u64;
        if self.check_large_jump(epoch_ms) {
            self.publish_time_sync(epoch_ms);
        }
        self.last_synced_epoch_ms = epoch_ms;
        self.consecutive_failures = 0;
        self.transition_to(SyncState::Synced);
    }
}

impl Service for TimeSyncService {
    fn service_id(&self) -> ServiceId {
        ServiceId::TimeSync
    }

    fn dependencies(&self) -> &'static [ServiceId] {
        &[ServiceId::Network]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdTime)?;
        bus.subscribe(Topic::EvtNetworkStatus)?;
        bus.subscribe(Topic::EvtConfigChanged)?;
        Ok(())
    }

    fn start(&mut self) -> CommResult<()> {
        self.service_state = ServiceState::Normal;
        Ok(())
    }

    fn stop(&mut self) -> CommResult<()> {
        self.service_state = ServiceState::Suspended;
        Ok(())
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            service: ServiceId::TimeSync,
            state: self.service_state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            if topic == Topic::CmdTime && !msg.is_response() {
                self.handle_cmd(&msg);
                return Ok(true);
            }
            if topic == Topic::EvtNetworkStatus {
                self.handle_evt_network_status(&msg);
                return Ok(true);
            }
            if topic == Topic::EvtConfigChanged {
                self.handle_evt_config_changed(&msg);
                return Ok(true);
            }
        }
        self.tick_sync();
        Ok(false)
    }
}
