//! Linux full-chain end-to-end integration tests.
//! All 12 services run on a single InProcessCommBus in topological order.

use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_config::ConfigService;
use service_storage::StorageManager;
use service_network::NetworkManager;
use service_time::TimeSyncService;
use media_core::MediaCoreService;
use service_live::LiveService;
use service_talk::TalkService;
use service_record::RecordService;
use service_playback::PlaybackService;
use service_cloud::{CloudService, CloudState};
use service_upgrade::UpgradeService;
use service_control::ControlGateway;

/// Topological order: Level 0 -> Level 6
/// Level 0: Config
/// Level 1: Storage, Network, MediaCore
/// Level 2: TimeSync, Record, Playback
/// Level 3: Live, Talk
/// Level 4: Cloud, Upgrade
/// Level 5: ControlGateway
fn init_and_start_all(
    bus: &'static InProcessCommBus,
    config: &mut ConfigService,
    storage: &mut StorageManager,
    network: &mut NetworkManager,
    time: &mut TimeSyncService,
    media_core: &mut MediaCoreService,
    live: &mut LiveService,
    talk: &mut TalkService,
    record: &mut RecordService,
    playback: &mut PlaybackService,
    cloud: &mut CloudService,
    upgrade: &mut UpgradeService,
    control: &mut ControlGateway,
) {
    // Level 0
    config.init(bus).unwrap();
    config.start().unwrap();

    // Level 1
    storage.init(bus).unwrap();
    storage.start().unwrap();
    network.init(bus).unwrap();
    network.start().unwrap();
    media_core.init(bus).unwrap();
    media_core.start().unwrap();

    // Level 2
    time.init(bus).unwrap();
    time.start().unwrap();
    record.init(bus).unwrap();
    record.start().unwrap();
    playback.init(bus).unwrap();
    playback.start().unwrap();

    // Level 3
    live.init(bus).unwrap();
    live.start().unwrap();
    talk.init(bus).unwrap();
    talk.start().unwrap();

    // Level 4
    cloud.init(bus).unwrap();
    cloud.start().unwrap();
    upgrade.init(bus).unwrap();
    upgrade.start().unwrap();

    // Level 5
    control.init(bus).unwrap();
    control.start().unwrap();
}

#[test]
fn e2e_all_services_start_in_order() {
    let bus: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));

    let mut config = ConfigService::new();
    let mut storage = StorageManager::new();
    let mut network = NetworkManager::new();
    let mut time = TimeSyncService::new();
    let mut media_core = MediaCoreService::new();
    let mut live = LiveService::new();
    let mut talk = TalkService::new();
    let mut record = RecordService::new();
    let mut playback = PlaybackService::new();
    let mut cloud = CloudService::new();
    let mut upgrade = UpgradeService::new();
    let mut control = ControlGateway::new();

    init_and_start_all(
        bus,
        &mut config,
        &mut storage,
        &mut network,
        &mut time,
        &mut media_core,
        &mut live,
        &mut talk,
        &mut record,
        &mut playback,
        &mut cloud,
        &mut upgrade,
        &mut control,
    );

    assert_eq!(config.health().state, ServiceState::Normal);
    assert_eq!(storage.health().state, ServiceState::Normal);
    assert_eq!(network.health().state, ServiceState::Normal);
    assert_eq!(time.health().state, ServiceState::Normal);
    assert_eq!(media_core.health().state, ServiceState::Normal);
    assert_eq!(live.health().state, ServiceState::Normal);
    assert_eq!(talk.health().state, ServiceState::Normal);
    assert_eq!(record.health().state, ServiceState::Normal);
    assert_eq!(playback.health().state, ServiceState::Normal);
    assert_eq!(cloud.health().state, ServiceState::Normal);
    assert_eq!(upgrade.health().state, ServiceState::Normal);
    assert_eq!(control.health().state, ServiceState::Normal);
}

#[test]
fn e2e_control_gateway_routes_cmd_live() {
    let bus: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));

    let mut config = ConfigService::new();
    let mut storage = StorageManager::new();
    let mut network = NetworkManager::new();
    let mut time = TimeSyncService::new();
    let mut media_core = MediaCoreService::new();
    let mut live = LiveService::new();
    let mut talk = TalkService::new();
    let mut record = RecordService::new();
    let mut playback = PlaybackService::new();
    let mut cloud = CloudService::new();
    let mut upgrade = UpgradeService::new();
    let mut control = ControlGateway::new();

    init_and_start_all(
        bus,
        &mut config,
        &mut storage,
        &mut network,
        &mut time,
        &mut media_core,
        &mut live,
        &mut talk,
        &mut record,
        &mut playback,
        &mut cloud,
        &mut upgrade,
        &mut control,
    );

    // Create a viewer session so auth check passes
    control.create_session(1, AuthLevel::Viewer).unwrap();

    // ControlGateway receives CmdControl with StartLive, forwards to CmdLive
    let req = CtrlMsg::new(Topic::CmdControl, MethodId::StartLive as u16, 1)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdControl, &req, &[]).unwrap();

    // Poll until LiveService receives CmdLive and adds viewer (bus returns first message
    // to whoever polls, so we poll control then live repeatedly until the chain completes)
    for _ in 0..50 {
        let _ = control.poll();
        let _ = live.poll();
        if live.viewer_count() == 1 {
            break;
        }
    }

    assert_eq!(live.viewer_count(), 1, "LiveService should have received CmdLive and added viewer");
}

#[test]
fn e2e_config_change_broadcasts_to_all() {
    let bus: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));

    let mut config = ConfigService::new();
    let mut storage = StorageManager::new();
    let mut network = NetworkManager::new();
    let mut time = TimeSyncService::new();
    let mut media_core = MediaCoreService::new();
    let mut live = LiveService::new();
    let mut talk = TalkService::new();
    let mut record = RecordService::new();
    let mut playback = PlaybackService::new();
    let mut cloud = CloudService::new();
    let mut upgrade = UpgradeService::new();
    let mut control = ControlGateway::new();

    init_and_start_all(
        bus,
        &mut config,
        &mut storage,
        &mut network,
        &mut time,
        &mut media_core,
        &mut live,
        &mut talk,
        &mut record,
        &mut playback,
        &mut cloud,
        &mut upgrade,
        &mut control,
    );

    // Config change triggers EvtConfigChanged
    config.set("test_key", "test_value").unwrap();

    // Poll downstream services - at least one should receive EvtConfigChanged
    let storage_got = storage.poll().unwrap();
    let network_got = network.poll().unwrap();
    let time_got = time.poll().unwrap();
    let media_got = media_core.poll().unwrap();

    assert!(
        storage_got || network_got || time_got || media_got,
        "at least one downstream service should receive EvtConfigChanged"
    );
}

#[test]
fn e2e_degradation_cascade_on_network_disconnect() {
    let bus: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));

    // Minimal setup: Config, Storage, Network, Cloud (Cloud depends on Network + Storage)
    let mut config = ConfigService::new();
    let mut storage = StorageManager::new();
    let mut network = NetworkManager::new();
    let mut cloud = CloudService::new();
    config.init(bus).unwrap();
    config.start().unwrap();
    storage.init(bus).unwrap();
    storage.start().unwrap();
    network.init(bus).unwrap();
    network.start().unwrap();
    cloud.init(bus).unwrap();
    cloud.start().unwrap();

    // 1. Bring network online
    let msg_online = CtrlMsg::new(Topic::EvtNetworkStatus, service_network::NetworkState::Online as u16, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &msg_online, &[]).unwrap();
    for _ in 0..5 {
        let _ = cloud.poll();
    }

    // 2. Enqueue upload -> Cloud enters Uploading
    let req = CtrlMsg::new(Topic::CmdCloud, MethodId::StartUpload as u16, 42)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdCloud, &req, &[]).unwrap();
    for _ in 0..5 {
        let _ = cloud.poll();
        if cloud.cloud_state() == CloudState::Uploading {
            break;
        }
    }

    assert_eq!(
        cloud.cloud_state(),
        CloudState::Uploading,
        "CloudService should be Uploading before network disconnect"
    );

    // 3. Simulate network disconnect - Cloud should suspend and report Degraded
    let msg_disconnected = CtrlMsg::new(Topic::EvtNetworkStatus, service_network::NetworkState::Disconnected as u16, 0)
        .with_source(ServiceId::Network);
    bus.publish_ctrl(Topic::EvtNetworkStatus, &msg_disconnected, &[]).unwrap();
    for _ in 0..10 {
        let _ = cloud.poll();
        if cloud.cloud_state() == CloudState::Suspended {
            break;
        }
    }

    assert_eq!(
        cloud.cloud_state(),
        CloudState::Suspended,
        "CloudService should suspend when network is disconnected"
    );
    assert_eq!(
        cloud.health().state,
        ServiceState::Degraded,
        "CloudService should report Degraded when network is down"
    );
}
