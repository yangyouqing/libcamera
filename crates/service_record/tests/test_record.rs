use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_record::{RecordService, RecordState};

fn leak_bus() -> &'static InProcessCommBus {
    Box::leak(Box::new(InProcessCommBus::new()))
}

#[test]
fn starts_idle() {
    let bus = leak_bus();
    let mut svc = RecordService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    assert_eq!(svc.record_state(), RecordState::Idle);
    assert_eq!(svc.health().state, ServiceState::Normal);
}

#[test]
fn start_record_via_cmd() {
    let bus = leak_bus();
    let mut svc = RecordService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdRecord, MethodId::StartRecord as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdRecord, &req, &[]).unwrap();

    let _ = svc.poll();

    assert_eq!(svc.record_state(), RecordState::Recording);

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
    assert_eq!(reply.source, ServiceId::Record as u8);
}

#[test]
fn stop_record_via_cmd() {
    let bus = leak_bus();
    let mut svc = RecordService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let start_req = CtrlMsg::new(Topic::CmdRecord, MethodId::StartRecord as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdRecord, &start_req, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.record_state(), RecordState::Recording);
    // Drain the StartRecord reply
    let _ = svc.poll();

    let stop_req = CtrlMsg::new(Topic::CmdRecord, MethodId::StopRecord as u16, 2)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdRecord, &stop_req, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.record_state(), RecordState::Idle);
}

#[test]
fn storage_full_degrades_to_paused() {
    let bus = leak_bus();
    let mut svc = RecordService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let start_req = CtrlMsg::new(Topic::CmdRecord, MethodId::StartRecord as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdRecord, &start_req, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.record_state(), RecordState::Recording);
    // Drain the CmdRecord reply before publishing the event
    let _ = svc.poll();

    // method_id=2 means storage Full
    let evt = CtrlMsg::new(Topic::EvtStorageStatus, 2, 0)
        .with_source(ServiceId::Storage);
    bus.publish_ctrl(Topic::EvtStorageStatus, &evt, &[]).unwrap();

    let _ = svc.poll();
    assert_eq!(svc.record_state(), RecordState::Paused);
    assert_eq!(svc.health().state, ServiceState::Degraded);
}

#[test]
fn storage_normal_resumes_from_paused() {
    let bus = leak_bus();
    let mut svc = RecordService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let start_req = CtrlMsg::new(Topic::CmdRecord, MethodId::StartRecord as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdRecord, &start_req, &[]).unwrap();
    let _ = svc.poll();
    // Drain CmdRecord reply
    let _ = svc.poll();

    let full_evt = CtrlMsg::new(Topic::EvtStorageStatus, 2, 0)
        .with_source(ServiceId::Storage);
    bus.publish_ctrl(Topic::EvtStorageStatus, &full_evt, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.record_state(), RecordState::Paused);

    // method_id=0 means Normal
    let normal_evt = CtrlMsg::new(Topic::EvtStorageStatus, 0, 0)
        .with_source(ServiceId::Storage);
    bus.publish_ctrl(Topic::EvtStorageStatus, &normal_evt, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.record_state(), RecordState::Recording);
    assert_eq!(svc.health().state, ServiceState::Normal);
}

#[test]
fn storage_removed_suspends() {
    let bus = leak_bus();
    let mut svc = RecordService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let start_req = CtrlMsg::new(Topic::CmdRecord, MethodId::StartRecord as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdRecord, &start_req, &[]).unwrap();
    let _ = svc.poll();
    // Drain CmdRecord reply
    let _ = svc.poll();

    // method_id=3 means Removed
    let evt = CtrlMsg::new(Topic::EvtStorageStatus, 3, 0)
        .with_source(ServiceId::Storage);
    bus.publish_ctrl(Topic::EvtStorageStatus, &evt, &[]).unwrap();
    let _ = svc.poll();

    assert_eq!(svc.record_state(), RecordState::Error);
    assert_eq!(svc.health().state, ServiceState::Suspended);
}
