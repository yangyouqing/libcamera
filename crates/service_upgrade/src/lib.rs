#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UpgradeState {
    Idle = 0,
    Downloading = 1,
    Verifying = 2,
    Applying = 3,
    Done = 4,
    Failed = 5,
}

pub struct UpgradeService {
    state: UpgradeState,
    progress_pct: u8,
    error_code: u8,
    bus: Option<*const dyn CommBus>,
    service_state: ServiceState,
    network_connected: bool,
    storage_available: bool,
}

unsafe impl Send for UpgradeService {}
unsafe impl Sync for UpgradeService {}

impl UpgradeService {
    pub const fn new() -> Self {
        Self {
            state: UpgradeState::Idle,
            progress_pct: 0,
            error_code: 0,
            bus: None,
            service_state: ServiceState::Normal,
            network_connected: true,
            storage_available: true,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    pub fn upgrade_state(&self) -> UpgradeState {
        self.state
    }

    fn transition_to(&mut self, new_state: UpgradeState) {
        self.state = new_state;
        self.publish_status();
    }

    fn publish_status(&self) {
        if let Some(bus_ptr) = self.bus {
            let bus = unsafe { &*bus_ptr };
            let msg = CtrlMsg::new(Topic::EvtUpgradeStatus, self.state as u16, 0)
                .with_source(ServiceId::Upgrade);
            let payload = [self.progress_pct, self.error_code];
            let _ = bus.publish_ctrl(Topic::EvtUpgradeStatus, &msg, &payload);
        }
    }

    fn start_upgrade(&mut self) {
        if self.state != UpgradeState::Idle {
            return;
        }
        self.progress_pct = 0;
        self.error_code = 0;
        self.transition_to(UpgradeState::Downloading);
    }

    fn verify_signature_stub(&self, _data: &[u8]) -> bool {
        // ed25519 signature verification stub
        true
    }

    fn verify_integrity_stub(&self, _data: &[u8]) -> bool {
        // SHA2 integrity check stub
        true
    }

    fn tick_upgrade(&mut self) {
        match self.state {
            UpgradeState::Downloading => {
                self.progress_pct = self.progress_pct.saturating_add(10);
                if self.progress_pct >= 100 {
                    self.progress_pct = 100;
                    self.transition_to(UpgradeState::Verifying);
                }
            }
            UpgradeState::Verifying => {
                let sig_ok = self.verify_signature_stub(&[]);
                let integrity_ok = self.verify_integrity_stub(&[]);
                if sig_ok && integrity_ok {
                    self.transition_to(UpgradeState::Applying);
                } else {
                    self.error_code = 1;
                    self.transition_to(UpgradeState::Failed);
                }
            }
            UpgradeState::Applying => {
                self.transition_to(UpgradeState::Done);
            }
            _ => {}
        }
    }

    fn handle_network_status(&mut self, msg: &CtrlMsg) {
        // method_id encodes network state: 0 = Disconnected
        self.network_connected = msg.method_id != 0;
        self.update_degradation();
    }

    fn handle_storage_status(&mut self, msg: &CtrlMsg) {
        // method_id encodes storage alert: 1 = Full, 2 = Removed
        self.storage_available = msg.method_id == 0;
        self.update_degradation();
    }

    fn update_degradation(&mut self) {
        if !self.network_connected || !self.storage_available {
            if self.state == UpgradeState::Downloading {
                self.transition_to(UpgradeState::Failed);
                self.error_code = 2;
            }
            self.service_state = ServiceState::Suspended;
        } else if self.service_state == ServiceState::Suspended {
            self.service_state = ServiceState::Normal;
        }
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg, _payload: &[u8]) {
        let resp = CtrlMsg::new(Topic::CmdUpgrade, msg.method_id, msg.request_id)
            .with_source(ServiceId::Upgrade);
        match msg.method_id {
            x if x == MethodId::CheckUpdate as u16 => {
                let available: u8 = if self.state == UpgradeState::Idle { 1 } else { 0 };
                let _ = self.bus().reply(Topic::CmdUpgrade, msg.request_id, &resp, &[available]);
            }
            x if x == MethodId::StartUpgrade as u16 => {
                self.start_upgrade();
                let _ = self.bus().reply(Topic::CmdUpgrade, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::QueryUpgradeStatus as u16 => {
                let payload = [self.state as u8, self.progress_pct, self.error_code];
                let _ = self.bus().reply(Topic::CmdUpgrade, msg.request_id, &resp, &payload);
            }
            _ => {}
        }
    }
}

impl Service for UpgradeService {
    fn service_id(&self) -> ServiceId {
        ServiceId::Upgrade
    }

    fn dependencies(&self) -> &'static [ServiceId] {
        &[ServiceId::Network]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdUpgrade)?;
        bus.subscribe(Topic::EvtNetworkStatus)?;
        bus.subscribe(Topic::EvtStorageStatus)?;
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
            service: ServiceId::Upgrade,
            state: self.service_state,
            error_code: self.error_code,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            if topic == Topic::CmdUpgrade && !msg.is_response() {
                let payload_len = msg.payload_len as usize;
                self.handle_cmd(&msg, &buf[..payload_len.min(buf.len())]);
                return Ok(true);
            }
            if topic == Topic::EvtNetworkStatus {
                self.handle_network_status(&msg);
                return Ok(true);
            }
            if topic == Topic::EvtStorageStatus {
                self.handle_storage_status(&msg);
                return Ok(true);
            }
        }
        if self.service_state != ServiceState::Suspended {
            self.tick_upgrade();
        }
        Ok(false)
    }
}
