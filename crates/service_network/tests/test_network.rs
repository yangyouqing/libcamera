use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_network::{NetworkManager, NetworkState};

#[test]
fn state_machine_initial_disconnected() {
    let bus = InProcessCommBus::new();
    let mut service = NetworkManager::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    assert_eq!(service.net_state, NetworkState::Disconnected);
}

#[test]
fn connect_changes_state() {
    let bus = InProcessCommBus::new();
    let mut service = NetworkManager::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdNetwork, MethodId::ConnectWifi as u16, 0)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdNetwork, &req, &[]).unwrap();

    let _ = service.poll();

    assert_eq!(service.net_state, NetworkState::Connecting);

    for _ in 0..5 {
        let _ = service.poll();
    }
    assert_eq!(service.net_state, NetworkState::Online);
}

#[test]
fn disconnect_resets_state() {
    let bus = InProcessCommBus::new();
    let mut service = NetworkManager::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdNetwork, MethodId::ConnectWifi as u16, 0)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdNetwork, &req, &[]).unwrap();

    for _ in 0..60 {
        let _ = service.poll();
    }
    assert_eq!(service.net_state, NetworkState::Disconnected);
}

#[test]
fn evt_network_status_published_on_state_change() {
    let bus = InProcessCommBus::new();
    bus.subscribe(Topic::EvtNetworkStatus).unwrap();

    let mut service = NetworkManager::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdNetwork, MethodId::ConnectWifi as u16, 0)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdNetwork, &req, &[]).unwrap();

    let mut buf = [0u8; 256];
    let _ = service.poll();

    let mut found = false;
    for _ in 0..10 {
        if let Ok(Some((topic, msg))) = bus.poll_ctrl(&mut buf) {
            if topic == Topic::EvtNetworkStatus {
                assert_eq!(msg.source, ServiceId::Network as u8);
                found = true;
                break;
            }
        }
    }
    assert!(found, "EvtNetworkStatus should be published on state change");
}
