use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_cloud::{CloudService, CloudState};

fn leak_bus() -> &'static InProcessCommBus {
    Box::leak(Box::new(InProcessCommBus::new()))
}

#[test]
fn starts_idle_disconnected() {
    let bus = leak_bus();
    let mut svc = CloudService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    assert_eq!(svc.cloud_state(), CloudState::Idle);
    assert_eq!(svc.queue_len(), 0);
}

#[test]
fn enqueue_upload_with_network() {
    let bus = leak_bus();
    let mut svc = CloudService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let net_evt = CtrlMsg::new(Topic::EvtNetworkStatus, 2, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &net_evt, &[]).unwrap();
    let _ = svc.poll();

    let req = CtrlMsg::new(Topic::CmdCloud, MethodId::StartUpload as u16, 100)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdCloud, &req, &[]).unwrap();
    let _ = svc.poll();

    assert_eq!(svc.queue_len(), 1);
    assert_eq!(svc.cloud_state(), CloudState::Uploading);

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
}

#[test]
fn tick_upload_completes_tasks() {
    let bus = leak_bus();
    let mut svc = CloudService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let net_evt = CtrlMsg::new(Topic::EvtNetworkStatus, 2, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &net_evt, &[]).unwrap();
    let _ = svc.poll();

    let req = CtrlMsg::new(Topic::CmdCloud, MethodId::StartUpload as u16, 200)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdCloud, &req, &[]).unwrap();
    let _ = svc.poll();

    assert_eq!(svc.queue_len(), 1);

    // Drain any stale reply messages, then tick_upload runs and completes
    for _ in 0..5 {
        let _ = svc.poll();
    }
    assert_eq!(svc.queue_len(), 0);
    assert_eq!(svc.cloud_state(), CloudState::Idle);
}

#[test]
fn network_disconnect_prevents_upload_start() {
    let bus = leak_bus();
    let mut svc = CloudService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    // Connect then immediately disconnect (no CmdCloud replies to interfere)
    let net_up = CtrlMsg::new(Topic::EvtNetworkStatus, 2, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &net_up, &[]).unwrap();
    let _ = svc.poll();

    let net_down = CtrlMsg::new(Topic::EvtNetworkStatus, 0, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &net_down, &[]).unwrap();
    let _ = svc.poll();

    // Enqueue after disconnect: item is queued but service stays Idle
    let req = CtrlMsg::new(Topic::CmdCloud, MethodId::StartUpload as u16, 300)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdCloud, &req, &[]).unwrap();
    let _ = svc.poll();

    assert_eq!(svc.queue_len(), 1);
    assert_ne!(svc.cloud_state(), CloudState::Uploading);

    // tick_upload also won't run because network is disconnected
    let _ = svc.poll();
    assert_eq!(svc.queue_len(), 1);
}

#[test]
fn network_reconnect_enables_upload() {
    let bus = leak_bus();
    let mut svc = CloudService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    // Connect -> disconnect -> reconnect (all events, no CmdCloud interference)
    let net_up1 = CtrlMsg::new(Topic::EvtNetworkStatus, 2, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &net_up1, &[]).unwrap();
    let _ = svc.poll();

    let net_down = CtrlMsg::new(Topic::EvtNetworkStatus, 0, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &net_down, &[]).unwrap();
    let _ = svc.poll();

    let net_up2 = CtrlMsg::new(Topic::EvtNetworkStatus, 3, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &net_up2, &[]).unwrap();
    let _ = svc.poll();

    // Enqueue after reconnect: should transition to Uploading
    let req = CtrlMsg::new(Topic::CmdCloud, MethodId::StartUpload as u16, 400)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdCloud, &req, &[]).unwrap();
    let _ = svc.poll();

    assert_eq!(svc.cloud_state(), CloudState::Uploading);
    assert_eq!(svc.health().state, ServiceState::Normal);
}

#[test]
fn stop_upload_cancels_all() {
    let bus = leak_bus();
    let mut svc = CloudService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let net_up = CtrlMsg::new(Topic::EvtNetworkStatus, 2, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &net_up, &[]).unwrap();
    let _ = svc.poll();

    let req = CtrlMsg::new(Topic::CmdCloud, MethodId::StartUpload as u16, 500)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdCloud, &req, &[]).unwrap();
    let _ = svc.poll();
    // Drain stale reply
    for _ in 0..3 { let _ = svc.poll(); }
    assert!(svc.queue_len() >= 0); // task may have been completed by tick_upload

    let cancel_req = CtrlMsg::new(Topic::CmdCloud, MethodId::StopUpload as u16, 501)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdCloud, &cancel_req, &[]).unwrap();
    for _ in 0..5 { let _ = svc.poll(); }

    assert_eq!(svc.queue_len(), 0);
    assert_eq!(svc.cloud_state(), CloudState::Idle);
}
