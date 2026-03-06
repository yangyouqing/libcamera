use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_storage::{StorageManager, StorageState, StorageAlert};

#[test]
fn state_machine_no_card_to_ready() {
    let bus = InProcessCommBus::new();
    let mut service = StorageManager::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    assert_eq!(service.storage_state(), StorageState::NoCard);

    service.simulate_card_insert(1_000_000, 100_000);
    assert_eq!(service.storage_state(), StorageState::Ready);
}

#[test]
fn capacity_alert_low_and_full() {
    let bus = InProcessCommBus::new();
    let mut service = StorageManager::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    bus.subscribe(Topic::EvtStorageStatus).unwrap();

    service.simulate_card_insert(100_000, 0);
    assert_eq!(service.alert(), StorageAlert::Normal);

    service.update_used(90_000);
    assert_eq!(service.alert(), StorageAlert::Low);

    service.update_used(98_000);
    assert_eq!(service.alert(), StorageAlert::Full);
}

#[test]
fn simulate_card_remove_removed_alert() {
    let bus = InProcessCommBus::new();
    let mut service = StorageManager::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    service.simulate_card_insert(1_000_000, 500_000);
    service.simulate_card_remove();

    assert_eq!(service.storage_state(), StorageState::NoCard);
    assert_eq!(service.alert(), StorageAlert::Removed);
}

#[test]
fn cmd_storage_reply() {
    let bus = InProcessCommBus::new();
    let mut service = StorageManager::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdStorage, MethodId::QueryCapacity as u16, 0)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdStorage, &req, &[]).unwrap();

    let _ = service.poll();

    let mut buf = [0u8; 256];
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
    assert_eq!(reply.source, ServiceId::Storage as u8);
}
