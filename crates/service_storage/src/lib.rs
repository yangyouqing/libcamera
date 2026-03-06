#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageState {
    NoCard,
    Mounting,
    Ready,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StorageAlert {
    Normal = 0,
    Low = 1,
    Full = 2,
    Removed = 3,
}

pub struct StorageManager {
    storage_state: StorageState,
    alert: StorageAlert,
    total_bytes: u64,
    used_bytes: u64,
    low_threshold_pct: u8,
    full_threshold_pct: u8,
    bus: Option<*const dyn CommBus>,
    service_state: ServiceState,
}

unsafe impl Send for StorageManager {}
unsafe impl Sync for StorageManager {}

impl StorageManager {
    pub const fn new() -> Self {
        Self {
            storage_state: StorageState::NoCard,
            alert: StorageAlert::Removed,
            total_bytes: 0,
            used_bytes: 0,
            low_threshold_pct: 90,
            full_threshold_pct: 98,
            bus: None,
            service_state: ServiceState::Normal,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    pub fn storage_state(&self) -> StorageState {
        self.storage_state
    }
    pub fn alert(&self) -> StorageAlert {
        self.alert
    }

    pub fn simulate_card_insert(&mut self, total: u64, used: u64) {
        self.storage_state = StorageState::Ready;
        self.total_bytes = total;
        self.used_bytes = used;
        self.check_capacity();
    }

    pub fn simulate_card_remove(&mut self) {
        self.storage_state = StorageState::NoCard;
        self.alert = StorageAlert::Removed;
        self.publish_status();
    }

    pub fn update_used(&mut self, used: u64) {
        self.used_bytes = used;
        self.check_capacity();
    }

    fn check_capacity(&mut self) {
        if self.total_bytes == 0 {
            return;
        }
        let pct = (self.used_bytes * 100 / self.total_bytes) as u8;
        let new_alert = if pct >= self.full_threshold_pct {
            StorageAlert::Full
        } else if pct >= self.low_threshold_pct {
            StorageAlert::Low
        } else {
            StorageAlert::Normal
        };
        if new_alert != self.alert {
            self.alert = new_alert;
            self.publish_status();
        }
    }

    fn publish_status(&self) {
        if let Some(bus_ptr) = self.bus {
            let bus = unsafe { &*bus_ptr };
            let msg = CtrlMsg::new(Topic::EvtStorageStatus, self.alert as u16, 0)
                .with_source(ServiceId::Storage);
            let _ = bus.publish_ctrl(Topic::EvtStorageStatus, &msg, &[]);
        }
    }

    fn handle_cmd(&self, msg: &CtrlMsg) {
        let resp = CtrlMsg::new(Topic::CmdStorage, msg.method_id, msg.request_id)
            .with_source(ServiceId::Storage);
        let _ = self.bus().reply(Topic::CmdStorage, msg.request_id, &resp, &[]);
    }
}

impl Service for StorageManager {
    fn service_id(&self) -> ServiceId {
        ServiceId::Storage
    }
    fn dependencies(&self) -> &'static [ServiceId] {
        &[ServiceId::Config]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        // SAFETY: Caller must ensure bus outlives this StorageManager
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdStorage)?;
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
            service: ServiceId::Storage,
            state: self.service_state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((_topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            if msg.topic == Topic::CmdStorage as u8 && !msg.is_response() {
                self.handle_cmd(&msg);
                return Ok(true);
            }
        }
        Ok(false)
    }
}
