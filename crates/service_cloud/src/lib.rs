#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

const MAX_UPLOAD_QUEUE: usize = 16;
const INITIAL_RETRY_DELAY_MS: u32 = 1_000;
const MAX_RETRY_DELAY_MS: u32 = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CloudState {
    Idle = 0,
    Uploading = 1,
    Suspended = 2,
}

#[derive(Debug, Clone, Copy)]
struct UploadTask {
    file_id: u32,
    retry_count: u8,
    retry_delay_ms: u32,
    active: bool,
}

impl UploadTask {
    const fn empty() -> Self {
        Self {
            file_id: 0,
            retry_count: 0,
            retry_delay_ms: INITIAL_RETRY_DELAY_MS,
            active: false,
        }
    }
}

pub struct CloudService {
    state: CloudState,
    queue: [UploadTask; MAX_UPLOAD_QUEUE],
    queue_len: u8,
    current_index: u8,
    network_connected: bool,
    bus: Option<*const dyn CommBus>,
    service_state: ServiceState,
}

unsafe impl Send for CloudService {}
unsafe impl Sync for CloudService {}

impl CloudService {
    pub const fn new() -> Self {
        Self {
            state: CloudState::Idle,
            queue: [UploadTask::empty(); MAX_UPLOAD_QUEUE],
            queue_len: 0,
            current_index: 0,
            network_connected: false,
            bus: None,
            service_state: ServiceState::Normal,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    pub fn cloud_state(&self) -> CloudState {
        self.state
    }

    pub fn queue_len(&self) -> u8 {
        self.queue_len
    }

    fn enqueue(&mut self, file_id: u32) -> CommResult<()> {
        for task in self.queue.iter_mut() {
            if !task.active {
                *task = UploadTask {
                    file_id,
                    retry_count: 0,
                    retry_delay_ms: INITIAL_RETRY_DELAY_MS,
                    active: true,
                };
                self.queue_len += 1;
                if self.state == CloudState::Idle && self.network_connected {
                    self.state = CloudState::Uploading;
                }
                return Ok(());
            }
        }
        Err(CamError::ResourceExhausted)
    }

    fn cancel_all(&mut self) {
        for task in self.queue.iter_mut() {
            task.active = false;
        }
        self.queue_len = 0;
        self.state = CloudState::Idle;
    }

    fn tick_upload(&mut self) {
        if self.state != CloudState::Uploading || !self.network_connected {
            return;
        }
        // Find current active task
        for task in self.queue.iter_mut() {
            if task.active {
                // Simulate upload completion
                task.active = false;
                self.queue_len = self.queue_len.saturating_sub(1);
                break;
            }
        }
        if self.queue_len == 0 {
            self.state = CloudState::Idle;
        }
    }

    fn compute_retry_delay(retry_count: u8) -> u32 {
        let delay = INITIAL_RETRY_DELAY_MS.saturating_mul(1 << retry_count.min(6));
        delay.min(MAX_RETRY_DELAY_MS)
    }

    fn handle_network_status(&mut self, method_id: u16) {
        // NetworkState encoding: method_id carries the state
        let connected = method_id >= 2; // Connected or Online
        self.network_connected = connected;
        if !connected {
            if self.state == CloudState::Uploading {
                self.state = CloudState::Suspended;
                self.service_state = ServiceState::Degraded;
            }
        } else if self.state == CloudState::Suspended {
            self.state = if self.queue_len > 0 {
                CloudState::Uploading
            } else {
                CloudState::Idle
            };
            self.service_state = ServiceState::Normal;
        }
    }

    fn handle_storage_status(&mut self, method_id: u16) {
        if method_id >= 2 {
            // Storage full or removed: degrade
            self.service_state = ServiceState::Degraded;
        }
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg, _payload: &[u8]) {
        let resp = CtrlMsg::new(Topic::CmdCloud, msg.method_id, msg.request_id)
            .with_source(ServiceId::Cloud);
        match msg.method_id {
            x if x == MethodId::StartUpload as u16 => {
                let file_id = msg.request_id as u32;
                let _ = self.enqueue(file_id);
                let _ = self.bus().reply(Topic::CmdCloud, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::StopUpload as u16 => {
                self.cancel_all();
                let _ = self.bus().reply(Topic::CmdCloud, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::QueryUploadQueue as u16 => {
                let payload = [self.queue_len, self.state as u8];
                let _ = self.bus().reply(Topic::CmdCloud, msg.request_id, &resp, &payload);
            }
            _ => {}
        }
    }
}

impl Service for CloudService {
    fn service_id(&self) -> ServiceId {
        ServiceId::Cloud
    }

    fn dependencies(&self) -> &'static [ServiceId] {
        &[ServiceId::Network, ServiceId::Storage]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdCloud)?;
        bus.subscribe(Topic::EvtNetworkStatus)?;
        bus.subscribe(Topic::EvtStorageStatus)?;
        Ok(())
    }

    fn start(&mut self) -> CommResult<()> {
        self.service_state = ServiceState::Normal;
        Ok(())
    }

    fn stop(&mut self) -> CommResult<()> {
        self.cancel_all();
        self.service_state = ServiceState::Suspended;
        Ok(())
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            service: ServiceId::Cloud,
            state: self.service_state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            if topic == Topic::CmdCloud && !msg.is_response() {
                let payload_len = msg.payload_len as usize;
                self.handle_cmd(&msg, &buf[..payload_len.min(buf.len())]);
                return Ok(true);
            }
            if topic == Topic::EvtNetworkStatus {
                self.handle_network_status(msg.method_id);
                return Ok(true);
            }
            if topic == Topic::EvtStorageStatus {
                self.handle_storage_status(msg.method_id);
                return Ok(true);
            }
            return Ok(false);
        }
        self.tick_upload();
        Ok(false)
    }
}
