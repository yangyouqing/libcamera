//! Cross-platform consistency tests.
//!
//! Verifies that core business logic running on InProcessCommBus (same mode as RTOS multi-thread)
//! produces identical results across multiple runs. Validates platform-independent behavior of all services.

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

/// Topological order: Level 0 -> Level 5
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
    config.init(bus).unwrap();
    config.start().unwrap();
    storage.init(bus).unwrap();
    storage.start().unwrap();
    network.init(bus).unwrap();
    network.start().unwrap();
    media_core.init(bus).unwrap();
    media_core.start().unwrap();
    time.init(bus).unwrap();
    time.start().unwrap();
    record.init(bus).unwrap();
    record.start().unwrap();
    playback.init(bus).unwrap();
    playback.start().unwrap();
    live.init(bus).unwrap();
    live.start().unwrap();
    talk.init(bus).unwrap();
    talk.start().unwrap();
    cloud.init(bus).unwrap();
    cloud.start().unwrap();
    upgrade.init(bus).unwrap();
    upgrade.start().unwrap();
    control.init(bus).unwrap();
    control.start().unwrap();
}

fn stop_all(
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
    control.stop().unwrap();
    upgrade.stop().unwrap();
    cloud.stop().unwrap();
    talk.stop().unwrap();
    live.stop().unwrap();
    playback.stop().unwrap();
    record.stop().unwrap();
    time.stop().unwrap();
    media_core.stop().unwrap();
    network.stop().unwrap();
    storage.stop().unwrap();
    config.stop().unwrap();
}

#[test]
fn cross_platform_service_state_consistency() {
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

    // All services report Normal when started
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

    stop_all(
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

    // All services report Suspended when stopped
    assert_eq!(config.health().state, ServiceState::Suspended);
    assert_eq!(storage.health().state, ServiceState::Suspended);
    assert_eq!(network.health().state, ServiceState::Suspended);
    assert_eq!(time.health().state, ServiceState::Suspended);
    assert_eq!(media_core.health().state, ServiceState::Suspended);
    assert_eq!(live.health().state, ServiceState::Suspended);
    assert_eq!(talk.health().state, ServiceState::Suspended);
    assert_eq!(record.health().state, ServiceState::Suspended);
    assert_eq!(playback.health().state, ServiceState::Suspended);
    assert_eq!(cloud.health().state, ServiceState::Suspended);
    assert_eq!(upgrade.health().state, ServiceState::Suspended);
    assert_eq!(control.health().state, ServiceState::Suspended);
}

#[test]
fn cross_platform_pubsub_deterministic() {
    let bus: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));

    // Subscribe to EvtAlarm (no service subscribes in minimal setup) to receive our sequence
    bus.subscribe(Topic::EvtAlarm).unwrap();

    // Publish a known sequence: method_id 1, 2, 3, 4, 5
    let sequence: [u16; 5] = [1, 2, 3, 4, 5];
    for &seq in &sequence {
        let msg = CtrlMsg::new(Topic::EvtAlarm, seq, 0).with_source(ServiceId::Config);
        bus.publish_ctrl(Topic::EvtAlarm, &msg, &[]).unwrap();
    }

    // Poll and verify messages arrive in correct order
    let mut received = Vec::new();
    let mut buf = [0u8; 256];
    for _ in 0..10 {
        if let Ok(Some((topic, msg))) = bus.poll_ctrl(&mut buf) {
            if topic == Topic::EvtAlarm {
                received.push(msg.method_id);
            }
        }
        if received.len() >= 5 {
            break;
        }
    }

    assert_eq!(received.len(), 5, "expected 5 messages");
    assert_eq!(
        received.as_slice(),
        &[1, 2, 3, 4, 5],
        "messages must arrive in published order"
    );
}

#[test]
fn cross_platform_request_reply_semantics() {
    let bus: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));

    let mut config = ConfigService::new();
    config.init(bus).unwrap();
    config.start().unwrap();

    // Send GetConfig request
    let req = CtrlMsg::new(Topic::CmdConfig, MethodId::GetConfig as u16, 0)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdConfig, &req, &[]).unwrap();

    // Poll config until it processes the request and sends reply
    let mut buf = [0u8; 256];
    for _ in 0..50 {
        let _ = config.poll();
        if let Ok(Some(resp)) = bus.poll_reply(&pending, &mut buf) {
            assert!(
                resp.is_response(),
                "reply must have FLAG_RESPONSE set"
            );
            assert_eq!(
                resp.source,
                ServiceId::Config as u8,
                "reply must come from Config service"
            );
            assert_eq!(
                resp.request_id, pending.request_id,
                "reply must match request_id"
            );
            return;
        }
    }
    panic!("request/reply did not complete within 50 poll cycles");
}

#[test]
fn cross_platform_degradation_state_machine() {
    fn run_degradation_sequence(
        bus: &'static InProcessCommBus,
    ) -> Vec<(CloudState, ServiceState)> {
        let mut config = ConfigService::new();
        let mut storage = StorageManager::new();
        let mut network = NetworkManager::new();
        let mut cloud = CloudService::new();

        config.init(bus).unwrap();
        config.start().unwrap();
        storage.init(bus).unwrap();
        storage.start().unwrap();
        storage.simulate_card_insert(1_000_000, 0);
        network.init(bus).unwrap();
        network.start().unwrap();
        cloud.init(bus).unwrap();
        cloud.start().unwrap();

        let mut states = Vec::new();
        states.push((cloud.cloud_state(), cloud.health().state));

        // 1. Bring network online
        let msg_online = CtrlMsg::new(
            Topic::EvtNetworkStatus,
            service_network::NetworkState::Online as u16,
            0,
        )
        .with_source(ServiceId::Network);
        bus.publish_ctrl(Topic::EvtNetworkStatus, &msg_online, &[]).unwrap();
        for _ in 0..5 {
            let _ = cloud.poll();
        }
        states.push((cloud.cloud_state(), cloud.health().state));

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
        states.push((cloud.cloud_state(), cloud.health().state));

        // 3. Network disconnect -> Cloud suspends, reports Degraded
        let msg_disconnected = CtrlMsg::new(
            Topic::EvtNetworkStatus,
            service_network::NetworkState::Disconnected as u16,
            0,
        )
        .with_source(ServiceId::Network);
        bus.publish_ctrl(Topic::EvtNetworkStatus, &msg_disconnected, &[]).unwrap();
        for _ in 0..10 {
            let _ = cloud.poll();
            if cloud.cloud_state() == CloudState::Suspended {
                break;
            }
        }
        states.push((cloud.cloud_state(), cloud.health().state));

        // 4. Network reconnect -> Cloud resumes to Uploading
        bus.publish_ctrl(Topic::EvtNetworkStatus, &msg_online, &[]).unwrap();
        for _ in 0..10 {
            let _ = cloud.poll();
            if cloud.cloud_state() == CloudState::Uploading {
                break;
            }
        }
        states.push((cloud.cloud_state(), cloud.health().state));

        states
    }

    let bus1: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));
    let bus2: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));

    let states1 = run_degradation_sequence(bus1);
    let states2 = run_degradation_sequence(bus2);

    assert_eq!(
        states1.len(),
        states2.len(),
        "both runs must produce same number of state snapshots"
    );
    for (i, (s1, s2)) in states1.iter().zip(states2.iter()).enumerate() {
        assert_eq!(
            s1.0, s2.0,
            "step {}: CloudState must match ({} vs {})",
            i, s1.0 as u8, s2.0 as u8
        );
        assert_eq!(
            s1.1, s2.1,
            "step {}: ServiceState must match ({} vs {})",
            i, s1.1 as u8, s2.1 as u8
        );
    }

    // Verify expected transitions
    assert_eq!(states1[0].0, CloudState::Idle);
    assert_eq!(states1[1].0, CloudState::Idle);
    assert_eq!(states1[2].0, CloudState::Uploading);
    assert_eq!(states1[3].0, CloudState::Suspended);
    assert_eq!(states1[3].1, ServiceState::Degraded);
    assert_eq!(states1[4].0, CloudState::Uploading);
    assert_eq!(states1[4].1, ServiceState::Normal);
}
