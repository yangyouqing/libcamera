use core_types::*;
use core_interfaces::{CommBus, Service};
use comm::in_process::InProcessCommBus;
use service_time::{TimeSyncService, construct_ntp_request, parse_ntp_response, calculate_offset};

#[test]
fn ntp_request_packet_48_bytes_correct_li_vn_mode() {
    let packet = construct_ntp_request();
    assert_eq!(packet.len(), 48);

    let first_byte = packet[0];
    let li = (first_byte >> 6) & 0x3;
    let vn = (first_byte >> 3) & 0x7;
    let mode = first_byte & 0x7;

    assert_eq!(li, 0, "LI should be 0");
    assert_eq!(vn, 4, "VN should be 4");
    assert_eq!(mode, 3, "Mode should be 3 (client)");
}

#[test]
fn ntp_response_parsing_extracts_timestamp() {
    let mut data = [0u8; 48];
    let ntp_seconds = 2208988803u32;
    let ntp_fraction = 0u32;
    data[40..44].copy_from_slice(&ntp_seconds.to_be_bytes());
    data[44..48].copy_from_slice(&ntp_fraction.to_be_bytes());

    let epoch_ms = parse_ntp_response(&data).unwrap();
    assert_eq!(epoch_ms, 3000);
}

#[test]
fn offset_calculation_correct() {
    let t1 = 1000u64;
    let t2 = 1100u64;
    let t3 = 1200u64;
    let t4 = 1150u64;

    let offset = calculate_offset(t1, t2, t3, t4);
    assert_eq!(offset, 75);
}

#[test]
fn sync_now_succeeds_health_remains_normal() {
    let bus = InProcessCommBus::new();
    let mut service = TimeSyncService::new();
    service.init(&bus).unwrap();
    service.start().unwrap();

    let req = CtrlMsg::new(Topic::CmdTime, MethodId::SyncNow as u16, 0)
        .with_source(ServiceId::ControlGateway);
    bus.publish_ctrl(Topic::CmdTime, &req, &[]).unwrap();
    let _ = service.poll(); // handle SyncNow
    for _ in 0..5 {
        let _ = service.poll(); // tick_sync runs, transitions to Synced on first success
    }

    assert_eq!(service.health().state, core_types::ServiceState::Normal);
}

#[test]
fn consecutive_failures_cause_degradation() {
    let bus = InProcessCommBus::new();
    let mut service = TimeSyncService::new();
    service.init(&bus).unwrap();
    service.start().unwrap();
    assert_eq!(service.health().state, core_types::ServiceState::Normal);

    // MAX_CONSECUTIVE_FAILURES = 3. After 4 failures: consecutive_failures > 3 -> Degraded.
    service.simulate_sync_failure();
    assert_eq!(service.health().state, core_types::ServiceState::Normal);
    service.simulate_sync_failure();
    assert_eq!(service.health().state, core_types::ServiceState::Normal);
    service.simulate_sync_failure();
    assert_eq!(service.health().state, core_types::ServiceState::Normal);
    service.simulate_sync_failure(); // 4 > 3 -> Degraded
    assert_eq!(service.health().state, core_types::ServiceState::Degraded);
}
