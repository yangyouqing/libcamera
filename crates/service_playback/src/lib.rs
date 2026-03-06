#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlaybackState {
    Idle = 0,
    Playing = 1,
    Paused = 2,
    Seeking = 3,
}

#[derive(Debug, Clone, Copy)]
struct TimelineEntry {
    start_ms: u64,
    end_ms: u64,
}

const MAX_TIMELINE_ENTRIES: usize = 64;

pub struct PlaybackService {
    state: PlaybackState,
    timeline: FixedVec<TimelineEntry, MAX_TIMELINE_ENTRIES>,
    current_pos_ms: u64,
    playback_seq: u32,
    bus: Option<*const dyn CommBus>,
    service_state: ServiceState,
}

unsafe impl Send for PlaybackService {}
unsafe impl Sync for PlaybackService {}

impl PlaybackService {
    pub const fn new() -> Self {
        Self {
            state: PlaybackState::Idle,
            timeline: FixedVec::new(),
            current_pos_ms: 0,
            playback_seq: 0,
            bus: None,
            service_state: ServiceState::Normal,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    pub fn playback_state(&self) -> PlaybackState {
        self.state
    }

    fn start_playback(&mut self, start_ms: u64) {
        self.state = PlaybackState::Playing;
        self.current_pos_ms = start_ms;
        self.playback_seq = 0;
    }

    fn stop_playback(&mut self) {
        self.state = PlaybackState::Idle;
    }

    fn seek(&mut self, target_ms: u64) {
        self.state = PlaybackState::Seeking;
        self.current_pos_ms = target_ms;
        // After seek, resume playing
        self.state = PlaybackState::Playing;
    }

    fn query_timeline(&self, msg: &CtrlMsg) {
        let count = self.timeline.len() as u16;
        let resp = CtrlMsg::new(Topic::CmdPlayback, msg.method_id, msg.request_id)
            .with_source(ServiceId::Playback)
            .with_payload_len(2);
        let payload = count.to_le_bytes();
        let _ = self.bus().reply(Topic::CmdPlayback, msg.request_id, &resp, &payload);
    }

    fn produce_playback_frame(&mut self) {
        if self.state != PlaybackState::Playing {
            return;
        }
        // Demux stub: produce a mock frame from the current position
        let hdr = FrameHeader::new(frame::FrameType::VideoH264P, 0, self.playback_seq)
            .with_pts(self.current_pos_ms)
            .with_data_len(32);
        let mock_data = [0u8; 32];
        let _ = self.bus().publish_frame(Topic::PlaybackStream, &hdr, &mock_data);
        self.playback_seq = self.playback_seq.wrapping_add(1);
        self.current_pos_ms += 40; // ~25fps
    }

    fn handle_storage_status(&mut self, msg: &CtrlMsg) {
        // method_id encodes storage alert: 2 = Removed
        if msg.method_id == 2 {
            self.stop_playback();
            self.service_state = ServiceState::Suspended;
        } else if self.service_state == ServiceState::Suspended && msg.method_id != 2 {
            self.service_state = ServiceState::Normal;
        }
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg, _payload: &[u8]) {
        let resp = CtrlMsg::new(Topic::CmdPlayback, msg.method_id, msg.request_id)
            .with_source(ServiceId::Playback);
        match msg.method_id {
            x if x == MethodId::StartPlayback as u16 => {
                let start_ms = if _payload.len() >= 8 {
                    u64::from_le_bytes([
                        _payload[0], _payload[1], _payload[2], _payload[3],
                        _payload[4], _payload[5], _payload[6], _payload[7],
                    ])
                } else {
                    0
                };
                self.start_playback(start_ms);
                let _ = self.bus().reply(Topic::CmdPlayback, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::StopPlayback as u16 => {
                self.stop_playback();
                let _ = self.bus().reply(Topic::CmdPlayback, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::QueryTimeline as u16 => {
                self.query_timeline(msg);
            }
            x if x == MethodId::SeekPlayback as u16 => {
                let target = if _payload.len() >= 8 {
                    u64::from_le_bytes([
                        _payload[0], _payload[1], _payload[2], _payload[3],
                        _payload[4], _payload[5], _payload[6], _payload[7],
                    ])
                } else {
                    0
                };
                self.seek(target);
                let _ = self.bus().reply(Topic::CmdPlayback, msg.request_id, &resp, &[]);
            }
            _ => {}
        }
    }
}

impl Service for PlaybackService {
    fn service_id(&self) -> ServiceId {
        ServiceId::Playback
    }

    fn dependencies(&self) -> &'static [ServiceId] {
        &[ServiceId::Storage]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdPlayback)?;
        bus.subscribe(Topic::EvtStorageStatus)?;
        Ok(())
    }

    fn start(&mut self) -> CommResult<()> {
        self.service_state = ServiceState::Normal;
        Ok(())
    }

    fn stop(&mut self) -> CommResult<()> {
        self.stop_playback();
        self.service_state = ServiceState::Suspended;
        Ok(())
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            service: ServiceId::Playback,
            state: self.service_state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            if topic == Topic::CmdPlayback && !msg.is_response() {
                let payload_len = msg.payload_len as usize;
                self.handle_cmd(&msg, &buf[..payload_len.min(buf.len())]);
                return Ok(true);
            }
            if topic == Topic::EvtStorageStatus {
                self.handle_storage_status(&msg);
                return Ok(true);
            }
        }
        if self.service_state != ServiceState::Suspended {
            self.produce_playback_frame();
        }
        Ok(false)
    }
}
