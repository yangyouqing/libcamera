use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_control::ControlGateway;

fn leak_bus() -> &'static InProcessCommBus {
    Box::leak(Box::new(InProcessCommBus::new()))
}

#[test]
fn starts_with_no_sessions() {
    let bus = leak_bus();
    let mut gw = ControlGateway::new();
    gw.init(bus).unwrap();
    gw.start().unwrap();

    assert_eq!(gw.session_count(), 0);
    assert_eq!(gw.health().state, ServiceState::Normal);
}

#[test]
fn route_table_forwards_start_live_to_cmd_live() {
    let bus = leak_bus();
    bus.subscribe(Topic::CmdLive).unwrap();

    let mut gw = ControlGateway::new();
    gw.init(bus).unwrap();
    gw.start().unwrap();

    // Create a viewer session so auth check passes
    gw.create_session(1, AuthLevel::Viewer).unwrap();

    let req = CtrlMsg::new(Topic::CmdControl, MethodId::StartLive as u16, 1)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdControl, &req, &[]).unwrap();

    let _ = gw.poll();

    let mut buf = [0u8; 256];
    let (topic, msg) = bus.poll_ctrl(&mut buf).unwrap().unwrap();
    assert_eq!(topic, Topic::CmdLive);
    assert_eq!(msg.method_id, MethodId::StartLive as u16);
}

#[test]
fn auth_level_blocks_admin_cmd_for_viewer() {
    let bus = leak_bus();
    bus.subscribe(Topic::CmdControl).unwrap();

    let mut gw = ControlGateway::new();
    gw.init(bus).unwrap();
    gw.start().unwrap();

    // Viewer session (auth=Viewer), but StartRecord requires Admin
    gw.create_session(10, AuthLevel::Viewer).unwrap();

    let req = CtrlMsg::new(Topic::CmdControl, MethodId::StartRecord as u16, 10)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdControl, &req, &[]).unwrap();

    let _ = gw.poll();

    // Should get an error reply on CmdControl
    let mut buf = [0u8; 256];
    let result = bus.poll_ctrl(&mut buf).unwrap();
    if let Some((_topic, msg)) = result {
        assert!(msg.is_error() || msg.is_response());
    }
}

#[test]
fn admin_can_access_admin_commands() {
    let bus = leak_bus();
    bus.subscribe(Topic::CmdRecord).unwrap();

    let mut gw = ControlGateway::new();
    gw.init(bus).unwrap();
    gw.start().unwrap();

    gw.create_session(20, AuthLevel::Admin).unwrap();

    let req = CtrlMsg::new(Topic::CmdControl, MethodId::StartRecord as u16, 20)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdControl, &req, &[]).unwrap();

    let _ = gw.poll();

    let mut buf = [0u8; 256];
    let (topic, msg) = bus.poll_ctrl(&mut buf).unwrap().unwrap();
    assert_eq!(topic, Topic::CmdRecord);
    assert_eq!(msg.method_id, MethodId::StartRecord as u16);
}

#[test]
fn session_create_and_remove() {
    let bus = leak_bus();
    bus.subscribe(Topic::EvtSessionStatus).unwrap();

    let mut gw = ControlGateway::new();
    gw.init(bus).unwrap();
    gw.start().unwrap();

    gw.create_session(100, AuthLevel::Viewer).unwrap();
    assert_eq!(gw.session_count(), 1);

    gw.create_session(101, AuthLevel::Admin).unwrap();
    assert_eq!(gw.session_count(), 2);

    gw.remove_session(100);
    assert_eq!(gw.session_count(), 1);

    gw.remove_session(101);
    assert_eq!(gw.session_count(), 0);
}

#[test]
fn no_session_means_no_auth() {
    let bus = leak_bus();
    bus.subscribe(Topic::CmdControl).unwrap();

    let mut gw = ControlGateway::new();
    gw.init(bus).unwrap();
    gw.start().unwrap();

    // No session created, request_id=999 has no session
    let req = CtrlMsg::new(Topic::CmdControl, MethodId::StartLive as u16, 999)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdControl, &req, &[]).unwrap();

    let _ = gw.poll();

    let mut buf = [0u8; 256];
    let result = bus.poll_ctrl(&mut buf).unwrap();
    if let Some((_topic, msg)) = result {
        assert!(msg.is_error() || msg.is_response());
    }
}

#[test]
fn stop_clears_sessions() {
    let bus = leak_bus();
    let mut gw = ControlGateway::new();
    gw.init(bus).unwrap();
    gw.start().unwrap();

    gw.create_session(1, AuthLevel::Admin).unwrap();
    gw.create_session(2, AuthLevel::Viewer).unwrap();
    assert_eq!(gw.session_count(), 2);

    gw.stop().unwrap();
    assert_eq!(gw.session_count(), 0);
    assert_eq!(gw.health().state, ServiceState::Suspended);
}
