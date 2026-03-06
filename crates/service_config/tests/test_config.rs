use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_config::ConfigService;

#[test]
fn get_returns_correct_value_after_set() {
    let bus = InProcessCommBus::new();
    let mut service = ConfigService::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    service.set("wifi_ssid", "MyNetwork").unwrap();
    assert_eq!(service.get("wifi_ssid"), Some("MyNetwork"));
}

#[test]
fn set_updates_and_publishes_evt_config_changed() {
    let bus = InProcessCommBus::new();
    bus.subscribe(Topic::EvtConfigChanged).unwrap();

    let mut service = ConfigService::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    service.set("key", "value").unwrap();

    let mut buf = [0u8; 256];
    let (topic, msg) = bus.poll_ctrl(&mut buf).unwrap().unwrap();
    assert_eq!(topic, Topic::EvtConfigChanged);
    assert_eq!(msg.source, ServiceId::Config as u8);
}

#[test]
fn layered_merge_priority_cloud_user_factory() {
    let bus = InProcessCommBus::new();
    let mut service = ConfigService::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    service.set_factory("foo", "factory_val").unwrap();
    service.set("foo", "user_val").unwrap();
    service.set_cloud("foo", "cloud_val").unwrap();
    assert_eq!(service.get("foo"), Some("cloud_val"));

    service.set_factory("baz", "factory_baz").unwrap();
    service.set("baz", "user_baz").unwrap();
    assert_eq!(service.get("baz"), Some("user_baz"));

    service.set_factory("qux", "factory_qux").unwrap();
    assert_eq!(service.get("qux"), Some("factory_qux"));
}

#[test]
fn non_existent_key_returns_none() {
    let bus = InProcessCommBus::new();
    let mut service = ConfigService::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    assert_eq!(service.get("nonexistent"), None);
}
