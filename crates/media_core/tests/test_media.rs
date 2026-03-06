use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use media_core::{MediaCoreService, PipelineState};

fn leak_bus() -> &'static InProcessCommBus {
    Box::leak(Box::new(InProcessCommBus::new()))
}

#[test]
fn pipeline_starts_and_produces_frames() {
    let bus = leak_bus();
    bus.subscribe(Topic::VideoMainStream).unwrap();
    bus.subscribe(Topic::VideoSubStream).unwrap();
    bus.subscribe(Topic::AudioCapture).unwrap();

    let mut svc = MediaCoreService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    assert_eq!(svc.pipeline_state(), PipelineState::Running);

    svc.produce_frames();

    let mut hdr = FrameHeader::new(frame::FrameType::VideoH264Idr, 0, 0);
    let mut data = [0u8; 4096];

    let n = bus.poll_frame(Topic::VideoMainStream, &mut hdr, &mut data).unwrap().unwrap();
    assert!(n > 0);
    assert!(hdr.is_keyframe());

    let n = bus.poll_frame(Topic::VideoSubStream, &mut hdr, &mut data).unwrap().unwrap();
    assert!(n > 0);

    let n = bus.poll_frame(Topic::AudioCapture, &mut hdr, &mut data).unwrap().unwrap();
    assert!(n > 0);
}

#[test]
fn stop_pipeline_goes_idle() {
    let bus = leak_bus();
    let mut svc = MediaCoreService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    assert_eq!(svc.pipeline_state(), PipelineState::Running);
    svc.stop_pipeline();
    assert_eq!(svc.pipeline_state(), PipelineState::Idle);
}

#[test]
fn set_bitrate_command_replies() {
    let bus = leak_bus();
    let mut svc = MediaCoreService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdMediaCore, MethodId::SetBitrate as u16, 42)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdMediaCore, &req, &[]).unwrap();

    let _ = svc.poll();

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
    assert_eq!(reply.source, ServiceId::MediaCore as u8);
}

#[test]
fn request_idr_command_replies() {
    let bus = leak_bus();
    let mut svc = MediaCoreService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdMediaCore, MethodId::RequestIdr as u16, 99)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdMediaCore, &req, &[]).unwrap();

    let _ = svc.poll();

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
    assert_eq!(reply.source, ServiceId::MediaCore as u8);
}

#[test]
fn health_reports_normal_after_start() {
    let bus = leak_bus();
    let mut svc = MediaCoreService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let health = svc.health();
    assert_eq!(health.service, ServiceId::MediaCore);
    assert_eq!(health.state, ServiceState::Normal);
}
