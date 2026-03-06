use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_live::LiveService;

fn leak_bus() -> &'static InProcessCommBus {
    Box::leak(Box::new(InProcessCommBus::new()))
}

#[test]
fn service_starts_in_normal_state() {
    let bus = leak_bus();
    let mut svc = LiveService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let health = svc.health();
    assert_eq!(health.service, ServiceId::Live);
    assert_eq!(health.state, ServiceState::Normal);
    assert_eq!(svc.viewer_count(), 0);
}

#[test]
fn start_live_adds_viewer() {
    let bus = leak_bus();
    let mut svc = LiveService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdLive, MethodId::StartLive as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdLive, &req, &[]).unwrap();

    let _ = svc.poll();

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
    assert_eq!(reply.source, ServiceId::Live as u8);
    assert_eq!(svc.viewer_count(), 1);
}

#[test]
fn stop_live_removes_viewer() {
    let bus = leak_bus();
    let mut svc = LiveService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    // Use publish_ctrl to preserve session_id (send_request overrides request_id)
    let sid = 10u16;
    let req_start = CtrlMsg::new(Topic::CmdLive, MethodId::StartLive as u16, sid)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdLive, &req_start, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.viewer_count(), 1);
    // Drain the StartLive reply
    let _ = svc.poll();

    let req_stop = CtrlMsg::new(Topic::CmdLive, MethodId::StopLive as u16, sid)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdLive, &req_stop, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.viewer_count(), 0);
}

#[test]
fn multiple_viewers_tracked() {
    let bus = leak_bus();
    let mut svc = LiveService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    for sid in 1..=3u16 {
        let req = CtrlMsg::new(Topic::CmdLive, MethodId::StartLive as u16, sid)
            .with_source(ServiceId::ControlGateway);
        let _ = bus.send_request(Topic::CmdLive, &req, &[]).unwrap();
        let _ = svc.poll();
        // Drain the reply so the next command is processed on the next iteration
        let _ = svc.poll();
    }
    assert_eq!(svc.viewer_count(), 3);
}
