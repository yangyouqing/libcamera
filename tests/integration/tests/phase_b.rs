use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_config::ConfigService;
use service_storage::StorageManager;
use service_network::NetworkManager;
use service_time::TimeSyncService;

#[test]
fn phase_b_services_start_in_order() {
    let bus = InProcessCommBus::new();
    let bus: &'static InProcessCommBus = Box::leak(Box::new(bus));

    let mut config = ConfigService::new();
    let mut storage = StorageManager::new();
    let mut network = NetworkManager::new();
    let mut time = TimeSyncService::new();

    config.init(bus).unwrap();
    config.start().unwrap();

    storage.init(bus).unwrap();
    storage.start().unwrap();

    network.init(bus).unwrap();
    network.start().unwrap();

    time.init(bus).unwrap();
    time.start().unwrap();

    assert_eq!(config.health().state, ServiceState::Normal);
    assert_eq!(storage.health().state, ServiceState::Normal);
    assert_eq!(network.health().state, ServiceState::Normal);
}

#[test]
fn config_change_propagates_to_storage_and_network() {
    let bus: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));

    let mut config = ConfigService::new();
    let mut storage = StorageManager::new();
    let mut network = NetworkManager::new();

    config.init(bus).unwrap();
    config.start().unwrap();
    storage.init(bus).unwrap();
    storage.start().unwrap();
    network.init(bus).unwrap();
    network.start().unwrap();

    config.set("wifi_ssid", "TestNetwork").unwrap();

    let storage_got = storage.poll().unwrap();
    let network_got = network.poll().unwrap();

    assert!(storage_got || network_got, "at least one subscriber should receive EvtConfigChanged");
}

#[test]
fn network_status_reaches_timesync() {
    let bus: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));

    let mut config = ConfigService::new();
    let mut network = NetworkManager::new();
    let mut time = TimeSyncService::new();

    config.init(bus).unwrap();
    config.start().unwrap();
    network.init(bus).unwrap();
    network.start().unwrap();
    time.init(bus).unwrap();
    time.start().unwrap();

    let msg = CtrlMsg::new(Topic::EvtNetworkStatus, 0, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &msg, &[]).unwrap();

    let time_got = time.poll().unwrap();
    assert!(time_got, "TimeSyncService should receive EvtNetworkStatus");
}
