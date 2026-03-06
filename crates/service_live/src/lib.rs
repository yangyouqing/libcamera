#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

const MAX_VIEWERS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FlowState {
    Normal = 0,
    SlowConsumer = 1,
    Suspended = 2,
}

#[derive(Debug, Clone, Copy)]
struct Viewer {
    session_id: u16,
    active: bool,
    flow: FlowState,
    dropped_frames: u32,
}

impl Viewer {
    const fn empty() -> Self {
        Self {
            session_id: 0,
            active: false,
            flow: FlowState::Normal,
            dropped_frames: 0,
        }
    }
}

pub struct LiveService {
    viewers: [Viewer; MAX_VIEWERS],
    viewer_count: u8,
    bus: Option<*const dyn CommBus>,
    service_state: ServiceState,
}

unsafe impl Send for LiveService {}
unsafe impl Sync for LiveService {}

impl LiveService {
    pub const fn new() -> Self {
        Self {
            viewers: [Viewer::empty(); MAX_VIEWERS],
            viewer_count: 0,
            bus: None,
            service_state: ServiceState::Normal,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    pub fn viewer_count(&self) -> u8 {
        self.viewer_count
    }

    fn add_viewer(&mut self, session_id: u16) -> CommResult<()> {
        for v in self.viewers.iter_mut() {
            if !v.active {
                *v = Viewer {
                    session_id,
                    active: true,
                    flow: FlowState::Normal,
                    dropped_frames: 0,
                };
                self.viewer_count += 1;
                return Ok(());
            }
        }
        Err(CamError::ResourceExhausted)
    }

    fn remove_viewer(&mut self, session_id: u16) {
        for v in self.viewers.iter_mut() {
            if v.active && v.session_id == session_id {
                v.active = false;
                self.viewer_count = self.viewer_count.saturating_sub(1);
                return;
            }
        }
    }

    fn fan_out_frame(&mut self, _hdr: &FrameHeader, _data: &[u8]) {
        for v in self.viewers.iter_mut() {
            if !v.active {
                continue;
            }
            match v.flow {
                FlowState::Normal => {}
                FlowState::SlowConsumer => {
                    v.dropped_frames += 1;
                    if v.dropped_frames > 100 {
                        v.flow = FlowState::Suspended;
                    }
                    continue;
                }
                FlowState::Suspended => continue,
            }
        }
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg, _payload: &[u8]) {
        let resp = CtrlMsg::new(Topic::CmdLive, msg.method_id, msg.request_id)
            .with_source(ServiceId::Live);
        match msg.method_id {
            x if x == MethodId::StartLive as u16 => {
                let sid = msg.request_id;
                let _ = self.add_viewer(sid);
                let _ = self.bus().reply(Topic::CmdLive, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::StopLive as u16 => {
                let sid = msg.request_id;
                self.remove_viewer(sid);
                let _ = self.bus().reply(Topic::CmdLive, msg.request_id, &resp, &[]);
            }
            _ => {}
        }
    }
}

impl Service for LiveService {
    fn service_id(&self) -> ServiceId {
        ServiceId::Live
    }

    fn dependencies(&self) -> &'static [ServiceId] {
        &[ServiceId::MediaCore]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdLive)?;
        bus.subscribe(Topic::VideoSubStream)?;
        bus.subscribe(Topic::AudioCapture)?;
        bus.subscribe(Topic::EvtSessionStatus)?;
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
            service: ServiceId::Live,
            state: self.service_state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            if topic == Topic::CmdLive && !msg.is_response() {
                let payload_len = msg.payload_len as usize;
                self.handle_cmd(&msg, &buf[..payload_len.min(buf.len())]);
                return Ok(true);
            }
            if topic == Topic::EvtSessionStatus {
                // No active sessions: suspend to save CPU
                if self.viewer_count == 0 {
                    self.service_state = ServiceState::Suspended;
                } else {
                    self.service_state = ServiceState::Normal;
                }
                return Ok(true);
            }
        }

        if self.viewer_count > 0 {
            let mut hdr = FrameHeader::new(frame::FrameType::VideoH264P, 1, 0);
            let mut data_buf = [0u8; 4096];
            if let Ok(Some(n)) = self.bus().poll_frame(Topic::VideoSubStream, &mut hdr, &mut data_buf) {
                self.fan_out_frame(&hdr, &data_buf[..n]);
                return Ok(true);
            }
            let mut audio_hdr = FrameHeader::new(frame::FrameType::AudioPcm, 0, 0);
            if let Ok(Some(n)) = self.bus().poll_frame(Topic::AudioCapture, &mut audio_hdr, &mut data_buf) {
                self.fan_out_frame(&audio_hdr, &data_buf[..n]);
                return Ok(true);
            }
        }

        Ok(false)
    }
}
