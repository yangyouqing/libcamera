use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_playback::{PlaybackService, PlaybackState};

fn leak_bus() -> &'static InProcessCommBus {
    Box::leak(Box::new(InProcessCommBus::new()))
}

#[test]
fn starts_idle() {
    let bus = leak_bus();
    let mut svc = PlaybackService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    assert_eq!(svc.playback_state(), PlaybackState::Idle);
}

#[test]
fn query_timeline_returns_reply() {
    let bus = leak_bus();
    let mut svc = PlaybackService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdPlayback, MethodId::QueryTimeline as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdPlayback, &req, &[]).unwrap();

    let _ = svc.poll();

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
    assert_eq!(reply.source, ServiceId::Playback as u8);
}

#[test]
fn start_playback_transitions_to_playing() {
    let bus = leak_bus();
    let mut svc = PlaybackService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let start_ms: u64 = 1000;
    let payload = start_ms.to_le_bytes();
    let req = CtrlMsg::new(Topic::CmdPlayback, MethodId::StartPlayback as u16, 1)
        .with_source(ServiceId::ControlGateway)
        .with_payload_len(8);
    let _ = bus.send_request(Topic::CmdPlayback, &req, &payload).unwrap();
    let _ = svc.poll();

    assert_eq!(svc.playback_state(), PlaybackState::Playing);
}

#[test]
fn seek_keeps_playing() {
    let bus = leak_bus();
    let mut svc = PlaybackService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let start_ms: u64 = 0;
    let start_payload = start_ms.to_le_bytes();
    let start_req = CtrlMsg::new(Topic::CmdPlayback, MethodId::StartPlayback as u16, 1)
        .with_source(ServiceId::ControlGateway)
        .with_payload_len(8);
    let _ = bus.send_request(Topic::CmdPlayback, &start_req, &start_payload).unwrap();
    let _ = svc.poll();
    // Drain the StartPlayback reply
    let _ = svc.poll();

    let target_ms: u64 = 5000;
    let seek_payload = target_ms.to_le_bytes();
    let seek_req = CtrlMsg::new(Topic::CmdPlayback, MethodId::SeekPlayback as u16, 2)
        .with_source(ServiceId::ControlGateway)
        .with_payload_len(8);
    let _ = bus.send_request(Topic::CmdPlayback, &seek_req, &seek_payload).unwrap();
    let _ = svc.poll();

    assert_eq!(svc.playback_state(), PlaybackState::Playing);
}

#[test]
fn stop_playback_returns_to_idle() {
    let bus = leak_bus();
    let mut svc = PlaybackService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let start_ms: u64 = 0;
    let start_payload = start_ms.to_le_bytes();
    let start_req = CtrlMsg::new(Topic::CmdPlayback, MethodId::StartPlayback as u16, 1)
        .with_source(ServiceId::ControlGateway)
        .with_payload_len(8);
    let _ = bus.send_request(Topic::CmdPlayback, &start_req, &start_payload).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.playback_state(), PlaybackState::Playing);
    // Drain the StartPlayback reply
    let _ = svc.poll();

    let stop_req = CtrlMsg::new(Topic::CmdPlayback, MethodId::StopPlayback as u16, 2)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdPlayback, &stop_req, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.playback_state(), PlaybackState::Idle);
}

#[test]
fn service_stop_suspends() {
    let bus = leak_bus();
    let mut svc = PlaybackService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    svc.stop().unwrap();
    assert_eq!(svc.health().state, ServiceState::Suspended);
    assert_eq!(svc.playback_state(), PlaybackState::Idle);
}
