#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DuplexMode {
    HalfDuplex = 0,
    FullDuplex = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TalkState {
    Idle = 0,
    Active = 1,
}

pub struct TalkService {
    state: TalkState,
    mode: DuplexMode,
    session_id: u16,
    bus: Option<*const dyn CommBus>,
    service_state: ServiceState,
}

unsafe impl Send for TalkService {}
unsafe impl Sync for TalkService {}

impl TalkService {
    pub const fn new() -> Self {
        Self {
            state: TalkState::Idle,
            mode: DuplexMode::HalfDuplex,
            session_id: 0,
            bus: None,
            service_state: ServiceState::Normal,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    pub fn talk_state(&self) -> TalkState {
        self.state
    }

    pub fn duplex_mode(&self) -> DuplexMode {
        self.mode
    }

    fn start_talk(&mut self, session_id: u16) {
        self.state = TalkState::Active;
        self.session_id = session_id;
    }

    fn stop_talk(&mut self) {
        self.state = TalkState::Idle;
        self.session_id = 0;
    }

    fn process_downlink(&self, _data: &[u8]) {
        if self.state != TalkState::Active {
            return;
        }
        if self.mode == DuplexMode::HalfDuplex {
            // In half-duplex: mute uplink while downlink is playing (stub)
        }
    }

    fn process_uplink(&self, _data: &[u8]) {
        if self.state != TalkState::Active {
            return;
        }
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg, _payload: &[u8]) {
        let resp = CtrlMsg::new(Topic::CmdTalk, msg.method_id, msg.request_id)
            .with_source(ServiceId::Talk);
        match msg.method_id {
            x if x == MethodId::StartTalk as u16 => {
                self.start_talk(msg.request_id);
                let _ = self.bus().reply(Topic::CmdTalk, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::StopTalk as u16 => {
                self.stop_talk();
                let _ = self.bus().reply(Topic::CmdTalk, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::SetTalkMode as u16 => {
                if _payload.first().copied().unwrap_or(0) == 1 {
                    self.mode = DuplexMode::FullDuplex;
                } else {
                    self.mode = DuplexMode::HalfDuplex;
                }
                let _ = self.bus().reply(Topic::CmdTalk, msg.request_id, &resp, &[]);
            }
            _ => {}
        }
    }
}

impl Service for TalkService {
    fn service_id(&self) -> ServiceId {
        ServiceId::Talk
    }

    fn dependencies(&self) -> &'static [ServiceId] {
        &[ServiceId::MediaCore]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdTalk)?;
        bus.subscribe(Topic::TalkDownlink)?;
        bus.subscribe(Topic::AudioCapture)?;
        Ok(())
    }

    fn start(&mut self) -> CommResult<()> {
        self.service_state = ServiceState::Normal;
        Ok(())
    }

    fn stop(&mut self) -> CommResult<()> {
        self.stop_talk();
        self.service_state = ServiceState::Suspended;
        Ok(())
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            service: ServiceId::Talk,
            state: self.service_state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            if topic == Topic::CmdTalk && !msg.is_response() {
                let payload_len = msg.payload_len as usize;
                self.handle_cmd(&msg, &buf[..payload_len.min(buf.len())]);
                return Ok(true);
            }
        }

        if self.state == TalkState::Active {
            let mut hdr = FrameHeader::new(frame::FrameType::AudioPcm, 0, 0);
            let mut data_buf = [0u8; 4096];
            if let Ok(Some(n)) = self.bus().poll_frame(Topic::TalkDownlink, &mut hdr, &mut data_buf) {
                self.process_downlink(&data_buf[..n]);
                return Ok(true);
            }
            if let Ok(Some(n)) = self.bus().poll_frame(Topic::AudioCapture, &mut hdr, &mut data_buf) {
                self.process_uplink(&data_buf[..n]);
                let uplink_hdr = FrameHeader::new(frame::FrameType::AudioPcm, 0, hdr.seq)
                    .with_data_len(n as u32);
                let _ = self.bus().publish_frame(Topic::TalkUplink, &uplink_hdr, &data_buf[..n]);
                return Ok(true);
            }
        }

        Ok(false)
    }
}
