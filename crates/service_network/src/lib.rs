#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

const MAX_RETRY_DELAY_MS: u32 = 60_000;
const INITIAL_RETRY_DELAY_MS: u32 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NetworkState {
    Disconnected = 0,
    Connecting = 1,
    Connected = 2,
    Online = 3,
}

pub struct NetworkManager {
    pub net_state: NetworkState,
    pub retry_count: u8,
    pub retry_delay_ms: u32,
    retry_timer_ms: u32,
    pub signal_rssi: i8,
    bus: Option<*const dyn CommBus>,
    pub service_state: ServiceState,
    poll_count: u32,
}

// SAFETY: bus pointer is set once at init, used read-only afterward
unsafe impl Send for NetworkManager {}
unsafe impl Sync for NetworkManager {}

impl NetworkManager {
    pub const fn new() -> Self {
        Self {
            net_state: NetworkState::Disconnected,
            retry_count: 0,
            retry_delay_ms: INITIAL_RETRY_DELAY_MS,
            retry_timer_ms: 0,
            signal_rssi: 0,
            bus: None,
            service_state: ServiceState::Normal,
            poll_count: 0,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    fn compute_next_retry_delay(&self) -> u32 {
        let delay = INITIAL_RETRY_DELAY_MS.saturating_mul(1 << self.retry_count.min(6));
        if delay > MAX_RETRY_DELAY_MS {
            MAX_RETRY_DELAY_MS
        } else {
            delay
        }
    }

    fn transition_to(&mut self, new_state: NetworkState) {
        if self.net_state != new_state {
            self.net_state = new_state;
            self.publish_status();
        }
    }

    fn publish_status(&self) {
        if let Some(bus_ptr) = self.bus {
            let bus = unsafe { &*bus_ptr };
            let msg = CtrlMsg::new(Topic::EvtNetworkStatus, self.net_state as u16, 0)
                .with_source(ServiceId::Network);
            let payload = [self.signal_rssi as u8];
            let _ = bus.publish_ctrl(Topic::EvtNetworkStatus, &msg, &payload);
        }
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg, payload: &[u8]) {
        let resp = CtrlMsg::new(Topic::CmdNetwork, msg.method_id, msg.request_id)
            .with_source(ServiceId::Network);
        match msg.method_id {
            x if x == MethodId::ScanWifi as u16 => {
                let _ = self.bus().reply(Topic::CmdNetwork, msg.request_id, &resp, payload);
            }
            x if x == MethodId::ConnectWifi as u16 => {
                self.transition_to(NetworkState::Connecting);
                self.retry_count = 0;
                self.retry_delay_ms = INITIAL_RETRY_DELAY_MS;
                let _ = self.bus().reply(Topic::CmdNetwork, msg.request_id, &resp, payload);
            }
            x if x == MethodId::GetNetworkStatus as u16 => {
                let status = [self.net_state as u8, self.signal_rssi as u8];
                let _ = self.bus().reply(Topic::CmdNetwork, msg.request_id, &resp, &status);
            }
            _ => {}
        }
    }

    fn handle_evt_config_changed(&mut self) {
        if self.net_state == NetworkState::Disconnected {
            self.retry_count = 0;
            self.retry_delay_ms = INITIAL_RETRY_DELAY_MS;
            self.retry_timer_ms = 0;
            self.transition_to(NetworkState::Connecting);
        }
    }

    fn tick_reconnect(&mut self) {
        const POLL_INTERVAL_MS: u32 = 100;
        match self.net_state {
            NetworkState::Disconnected => {
                if self.retry_timer_ms == 0 {
                    return;
                }
                self.retry_timer_ms = self.retry_timer_ms.saturating_sub(POLL_INTERVAL_MS);
                if self.retry_timer_ms == 0 {
                    self.transition_to(NetworkState::Connecting);
                }
            }
            NetworkState::Connecting => {
                self.transition_to(NetworkState::Connected);
                self.transition_to(NetworkState::Online);
                self.signal_rssi = -50;
            }
            NetworkState::Connected | NetworkState::Online => {
                self.signal_rssi = -50;
                self.poll_count = self.poll_count.saturating_add(1);
                if self.poll_count >= 50 {
                    self.poll_count = 0;
                    self.on_connection_failure();
                }
            }
        }
    }

    fn on_connection_failure(&mut self) {
        self.transition_to(NetworkState::Disconnected);
        self.retry_count = self.retry_count.saturating_add(1);
        self.retry_delay_ms = self.compute_next_retry_delay();
        self.retry_timer_ms = self.retry_delay_ms;
    }
}

impl Service for NetworkManager {
    fn service_id(&self) -> ServiceId {
        ServiceId::Network
    }

    fn dependencies(&self) -> &'static [ServiceId] {
        &[ServiceId::Config]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdNetwork)?;
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
            service: ServiceId::Network,
            state: self.service_state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            let payload_len = msg.payload_len as usize;
            let payload = &buf[..payload_len.min(buf.len())];
            if topic == Topic::CmdNetwork && !msg.is_response() {
                self.handle_cmd(&msg, payload);
                return Ok(true);
            }
            if topic == Topic::EvtConfigChanged {
                self.handle_evt_config_changed();
                return Ok(true);
            }
        }
        self.tick_reconnect();
        Ok(false)
    }
}
