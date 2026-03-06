#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PipelineState {
    Idle = 0,
    Running = 1,
    Error = 2,
}

#[derive(Debug, Clone, Copy)]
struct StreamConfig {
    width: u16,
    height: u16,
    bitrate_kbps: u16,
    fps: u8,
}

impl StreamConfig {
    const fn default_main() -> Self {
        Self { width: 1920, height: 1080, bitrate_kbps: 2048, fps: 25 }
    }
    const fn default_sub() -> Self {
        Self { width: 640, height: 360, bitrate_kbps: 512, fps: 15 }
    }
}

pub struct MediaCoreService {
    pipeline_state: PipelineState,
    main_stream: StreamConfig,
    sub_stream: StreamConfig,
    audio_enabled: bool,
    frame_seq_main: u32,
    frame_seq_sub: u32,
    frame_seq_audio: u32,
    bus: Option<*const dyn CommBus>,
    service_state: ServiceState,
}

unsafe impl Send for MediaCoreService {}
unsafe impl Sync for MediaCoreService {}

impl MediaCoreService {
    pub const fn new() -> Self {
        Self {
            pipeline_state: PipelineState::Idle,
            main_stream: StreamConfig::default_main(),
            sub_stream: StreamConfig::default_sub(),
            audio_enabled: true,
            frame_seq_main: 0,
            frame_seq_sub: 0,
            frame_seq_audio: 0,
            bus: None,
            service_state: ServiceState::Normal,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    pub fn pipeline_state(&self) -> PipelineState {
        self.pipeline_state
    }

    pub fn start_pipeline(&mut self) -> CommResult<()> {
        if self.pipeline_state == PipelineState::Running {
            return Ok(());
        }
        self.pipeline_state = PipelineState::Running;
        self.frame_seq_main = 0;
        self.frame_seq_sub = 0;
        self.frame_seq_audio = 0;
        Ok(())
    }

    pub fn stop_pipeline(&mut self) {
        self.pipeline_state = PipelineState::Idle;
    }

    pub fn produce_frames(&mut self) {
        if self.pipeline_state != PipelineState::Running {
            return;
        }
        self.produce_video_main();
        self.produce_video_sub();
        if self.audio_enabled {
            self.produce_audio();
        }
    }

    fn produce_video_main(&mut self) {
        let is_idr = self.frame_seq_main % 30 == 0;
        let ft = if is_idr {
            frame::FrameType::VideoH264Idr
        } else {
            frame::FrameType::VideoH264P
        };
        let mut hdr = FrameHeader::new(ft, 0, self.frame_seq_main);
        if is_idr {
            hdr.flags |= FrameHeader::FLAG_KEYFRAME;
        }
        let mock_data = [0u8; 64];
        hdr.data_len = mock_data.len() as u32;
        let _ = self.bus().publish_frame(Topic::VideoMainStream, &hdr, &mock_data);
        self.frame_seq_main = self.frame_seq_main.wrapping_add(1);
    }

    fn produce_video_sub(&mut self) {
        let is_idr = self.frame_seq_sub % 30 == 0;
        let ft = if is_idr {
            frame::FrameType::VideoH264Idr
        } else {
            frame::FrameType::VideoH264P
        };
        let mut hdr = FrameHeader::new(ft, 1, self.frame_seq_sub);
        if is_idr {
            hdr.flags |= FrameHeader::FLAG_KEYFRAME;
        }
        let mock_data = [0u8; 32];
        hdr.data_len = mock_data.len() as u32;
        let _ = self.bus().publish_frame(Topic::VideoSubStream, &hdr, &mock_data);
        self.frame_seq_sub = self.frame_seq_sub.wrapping_add(1);
    }

    fn produce_audio(&mut self) {
        let hdr = FrameHeader::new(frame::FrameType::AudioPcm, 0, self.frame_seq_audio)
            .with_data_len(16);
        let mock_data = [0u8; 16];
        let _ = self.bus().publish_frame(Topic::AudioCapture, &hdr, &mock_data);
        self.frame_seq_audio = self.frame_seq_audio.wrapping_add(1);
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg, _payload: &[u8]) {
        let resp = CtrlMsg::new(Topic::CmdMediaCore, msg.method_id, msg.request_id)
            .with_source(ServiceId::MediaCore);
        match msg.method_id {
            x if x == MethodId::SetBitrate as u16 => {
                let _ = self.bus().reply(Topic::CmdMediaCore, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::RequestIdr as u16 => {
                self.frame_seq_main = (self.frame_seq_main / 30) * 30;
                self.frame_seq_sub = (self.frame_seq_sub / 30) * 30;
                let _ = self.bus().reply(Topic::CmdMediaCore, msg.request_id, &resp, &[]);
            }
            x if x == MethodId::SetResolution as u16 => {
                let _ = self.bus().reply(Topic::CmdMediaCore, msg.request_id, &resp, &[]);
            }
            _ => {}
        }
    }

    fn handle_evt_config_changed(&mut self) {
        let _ = self;
    }
}

impl Service for MediaCoreService {
    fn service_id(&self) -> ServiceId {
        ServiceId::MediaCore
    }

    fn dependencies(&self) -> &'static [ServiceId] {
        &[ServiceId::Config]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdMediaCore)?;
        bus.subscribe(Topic::EvtConfigChanged)?;
        Ok(())
    }

    fn start(&mut self) -> CommResult<()> {
        self.service_state = ServiceState::Normal;
        self.start_pipeline()
    }

    fn stop(&mut self) -> CommResult<()> {
        self.stop_pipeline();
        self.service_state = ServiceState::Suspended;
        Ok(())
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            service: ServiceId::MediaCore,
            state: self.service_state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            let payload_len = msg.payload_len as usize;
            let payload = &buf[..payload_len.min(buf.len())];
            if topic == Topic::CmdMediaCore && !msg.is_response() {
                self.handle_cmd(&msg, payload);
                return Ok(true);
            }
            if topic == Topic::EvtConfigChanged {
                self.handle_evt_config_changed();
                return Ok(true);
            }
        }
        self.produce_frames();
        Ok(false)
    }
}
