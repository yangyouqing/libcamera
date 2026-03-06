use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_talk::{TalkService, TalkState, DuplexMode};

fn leak_bus() -> &'static InProcessCommBus {
    Box::leak(Box::new(InProcessCommBus::new()))
}

#[test]
fn starts_idle_half_duplex() {
    let bus = leak_bus();
    let mut svc = TalkService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    assert_eq!(svc.talk_state(), TalkState::Idle);
    assert_eq!(svc.duplex_mode(), DuplexMode::HalfDuplex);
}

#[test]
fn start_talk_activates_session() {
    let bus = leak_bus();
    let mut svc = TalkService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdTalk, MethodId::StartTalk as u16, 5)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdTalk, &req, &[]).unwrap();

    let _ = svc.poll();

    assert_eq!(svc.talk_state(), TalkState::Active);

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
    assert_eq!(reply.source, ServiceId::Talk as u8);
}

#[test]
fn stop_talk_returns_to_idle() {
    let bus = leak_bus();
    let mut svc = TalkService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let start_req = CtrlMsg::new(Topic::CmdTalk, MethodId::StartTalk as u16, 5)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdTalk, &start_req, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.talk_state(), TalkState::Active);
    // Drain the StartTalk reply
    let _ = svc.poll();

    let stop_req = CtrlMsg::new(Topic::CmdTalk, MethodId::StopTalk as u16, 6)
        .with_source(ServiceId::ControlGateway);
    let _ = bus.send_request(Topic::CmdTalk, &stop_req, &[]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.talk_state(), TalkState::Idle);
}

#[test]
fn set_talk_mode_switches_to_full_duplex() {
    let bus = leak_bus();
    let mut svc = TalkService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    assert_eq!(svc.duplex_mode(), DuplexMode::HalfDuplex);

    let req = CtrlMsg::new(Topic::CmdTalk, MethodId::SetTalkMode as u16, 7)
        .with_source(ServiceId::ControlGateway)
        .with_payload_len(1);
    let _ = bus.send_request(Topic::CmdTalk, &req, &[1u8]).unwrap();
    let _ = svc.poll();

    assert_eq!(svc.duplex_mode(), DuplexMode::FullDuplex);
}

#[test]
fn set_talk_mode_switches_back_to_half_duplex() {
    let bus = leak_bus();
    let mut svc = TalkService::new();
    svc.init(bus).unwrap();
    svc.start().unwrap();

    let req_full = CtrlMsg::new(Topic::CmdTalk, MethodId::SetTalkMode as u16, 7)
        .with_source(ServiceId::ControlGateway)
        .with_payload_len(1);
    let _ = bus.send_request(Topic::CmdTalk, &req_full, &[1u8]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.duplex_mode(), DuplexMode::FullDuplex);
    // Drain the first SetTalkMode reply
    let _ = svc.poll();

    let req_half = CtrlMsg::new(Topic::CmdTalk, MethodId::SetTalkMode as u16, 8)
        .with_source(ServiceId::ControlGateway)
        .with_payload_len(1);
    let _ = bus.send_request(Topic::CmdTalk, &req_half, &[0u8]).unwrap();
    let _ = svc.poll();
    assert_eq!(svc.duplex_mode(), DuplexMode::HalfDuplex);
}
