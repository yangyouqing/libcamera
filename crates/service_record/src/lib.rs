#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

const FRAGMENT_DURATION_MS: u64 = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordState {
    Idle = 0,
    Recording = 1,
    Paused = 2,
    Error = 3,
}

pub struct RecordService {
    state: RecordState,
    fragment_start_ms: u64,
    fragment_index: u32,
    frames_written: u64,
    power_fail_marker: bool,
    bus: Option<*const dyn CommBus>,
    service_state: ServiceState,
}

unsafe impl Send for RecordService {}
unsafe impl Sync for RecordService {}

impl RecordService {
    pub const fn new() -> Self {
        Self {
            state: RecordState::Idle,
            fragment_start_ms: 0,
            fragment_index: 0,
            frames_written: 0,
            power_fail_marker: false,
            bus: None,
            service_state: ServiceState::Normal,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    pub fn record_state(&self) -> RecordState {
        self.state
    }

    fn start_recording(&mut self) {
        self.state = RecordState::Recording;
        self.fragment_start_ms = 0;
        self.fragment_index = 0;
        self.frames_written = 0;
        self.power_fail_marker = true;
    }

    fn stop_recording(&mut self) {
        self.finalize_fragment();
        self.state = RecordState::Idle;
        self.power_fail_marker = false;
    }

    fn mux_frame(&mut self, hdr: &FrameHeader, _data: &[u8]) {
        if self.state != RecordState::Recording {
            return;
        }

        if self.fragment_start_ms == 0 {
            self.fragment_start_ms = hdr.pts_ms;
        }

        let elapsed = hdr.pts_ms.saturating_sub(self.fragment_start_ms);
        if elapsed >= FRAGMENT_DURATION_MS && hdr.is_keyframe() {
            self.finalize_fragment();
            self.fragment_index += 1;
            self.fragment_start_ms = hdr.pts_ms;
        }

        self.frames_written += 1;
    }

    fn finalize_fragment(&mut self) {
        // MP4 muxer finalization stub: write moov atom, clear power-fail marker
        self.power_fail_marker = false;
    }

    fn handle_storage_status(&mut self, method_id: u16) {
        // StorageAlert encoding: method_id carries the alert level
        match method_id {
            2 => {
                // Full -> pause recording
                if self.state == RecordState::Recording {
                    self.state = RecordState::Paused;
                    self.service_state = ServiceState::Degraded;
                }
            }
            3 => {
                // Removed -> stop recording
                if self.state == RecordState::Recording || self.state == RecordState::Paused {
                    self.stop_recording();
                    self.state = RecordState::Error;
                    self.service_state = ServiceState::Suspended;
                }
            }
            0 => {
                // Normal -> resume if paused
                if self.state == RecordState::Paused {
                    self.state = RecordState::Recording;
                    self.service_state = ServiceState::Normal;
                }
            }
            _ => {}
        }
    }

    fn handle_alarm(&mut self) {
        if self.state == RecordState::Idle {
            self.start_recording();
        }
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg, _payload: &[u8]) {
        let resp = CtrlMsg::new(Topic::CmdRecord, msg.method_id, msg.request_id)
            .with_source(ServiceId::Record);
        match msg.method_id {
            x if x == MethodId::StartRecord as u16 => {
                self.start_recording();
                let _ = self.bus().reply(Topic::CmdRecord, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::StopRecord as u16 => {
                self.stop_recording();
                let _ = self.bus().reply(Topic::CmdRecord, msg.request_id, &resp, &[]);
            }
            _ => {}
        }
    }
}

impl Service for RecordService {
    fn service_id(&self) -> ServiceId {
        ServiceId::Record
    }

    fn dependencies(&self) -> &'static [ServiceId] {
        &[ServiceId::MediaCore, ServiceId::Storage]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdRecord)?;
        bus.subscribe(Topic::VideoMainStream)?;
        bus.subscribe(Topic::AudioCapture)?;
        bus.subscribe(Topic::EvtStorageStatus)?;
        bus.subscribe(Topic::EvtAlarm)?;
        Ok(())
    }

    fn start(&mut self) -> CommResult<()> {
        self.service_state = ServiceState::Normal;
        Ok(())
    }

    fn stop(&mut self) -> CommResult<()> {
        if self.state == RecordState::Recording {
            self.stop_recording();
        }
        self.service_state = ServiceState::Suspended;
        Ok(())
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            service: ServiceId::Record,
            state: self.service_state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            let payload_len = msg.payload_len as usize;
            let payload = &buf[..payload_len.min(buf.len())];
            if topic == Topic::CmdRecord && !msg.is_response() {
                self.handle_cmd(&msg, payload);
                return Ok(true);
            }
            if topic == Topic::EvtStorageStatus {
                self.handle_storage_status(msg.method_id);
                return Ok(true);
            }
            if topic == Topic::EvtAlarm {
                self.handle_alarm();
                return Ok(true);
            }
        }

        if self.state == RecordState::Recording {
            let mut hdr = FrameHeader::new(frame::FrameType::VideoH264P, 0, 0);
            let mut data_buf = [0u8; 4096];
            if let Ok(Some(n)) = self.bus().poll_frame(Topic::VideoMainStream, &mut hdr, &mut data_buf) {
                self.mux_frame(&hdr, &data_buf[..n]);
                return Ok(true);
            }
        }

        Ok(false)
    }
}
