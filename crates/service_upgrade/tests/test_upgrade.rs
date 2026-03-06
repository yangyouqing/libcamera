use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_upgrade::{UpgradeService, UpgradeState};

fn leak_bus() -> &'static InProcessCommBus {
    Box::leak(Box::new(InProcessCommBus::new()))
}

#[test]
fn starts_idle() {
    let bus = leak_bus();
    let mut svc = UpgradeService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    assert_eq!(svc.upgrade_state(), UpgradeState::Idle);
    assert_eq!(svc.health().state, ServiceState::Normal);
}

#[test]
fn start_upgrade_transitions_to_downloading() {
    let bus = leak_bus();
    bus.subscribe(Topic::EvtUpgradeStatus).unwrap();

    let mut svc = UpgradeService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdUpgrade, MethodId::StartUpgrade as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdUpgrade, &req, &[]).unwrap();
    let _ = svc.poll();

    assert_eq!(svc.upgrade_state(), UpgradeState::Downloading);

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
}

#[test]
fn full_upgrade_state_machine() {
    let bus = leak_bus();
    bus.subscribe(Topic::EvtUpgradeStatus).unwrap();

    let mut svc = UpgradeService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdUpgrade, MethodId::StartUpgrade as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdUpgrade, &req, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.upgrade_state(), UpgradeState::Downloading);

    // tick_upgrade runs during poll; progress_pct goes up by 10 each tick
    // Need 10 ticks to reach 100% and transition to Verifying
    for _ in 0..10 {
        let _ = svc.poll();
    }
    assert_eq!(svc.upgrade_state(), UpgradeState::Verifying);

    // Next tick: verification passes (stubs return true) -> Applying
    let _ = svc.poll();
    assert_eq!(svc.upgrade_state(), UpgradeState::Applying);

    // Next tick: apply completes -> Done
    let _ = svc.poll();
    assert_eq!(svc.upgrade_state(), UpgradeState::Done);
}

#[test]
fn check_update_reports_availability() {
    let bus = leak_bus();
    let mut svc = UpgradeService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdUpgrade, MethodId::CheckUpdate as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdUpgrade, &req, &[]).unwrap();
    let _ = svc.poll();

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
    assert_eq!(reply.source, ServiceId::Upgrade as u8);
}

#[test]
fn query_upgrade_status_returns_state() {
    let bus = leak_bus();
    let mut svc = UpgradeService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdUpgrade, MethodId::QueryUpgradeStatus as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdUpgrade, &req, &[]).unwrap();
    let _ = svc.poll();

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
}

#[test]
fn start_upgrade_ignored_when_not_idle() {
    let bus = leak_bus();
    bus.subscribe(Topic::EvtUpgradeStatus).unwrap();

    let mut svc = UpgradeService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    // First start
    let req1 = CtrlMsg::new(Topic::CmdUpgrade, MethodId::StartUpgrade as u16, 1)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdUpgrade, &req1, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.upgrade_state(), UpgradeState::Downloading);

    // Second start should be ignored (state is not Idle)
    let req2 = CtrlMsg::new(Topic::CmdUpgrade, MethodId::StartUpgrade as u16, 2)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdUpgrade, &req2, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.upgrade_state(), UpgradeState::Downloading);
}
