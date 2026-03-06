use core_types::*;

#[test]
fn fixed_vec_push_pop() {
    let mut v: FixedVec<u32, 4> = FixedVec::new();
    assert!(v.is_empty());
    assert_eq!(v.len(), 0);

    assert!(v.push(10).is_ok());
    assert!(v.push(20).is_ok());
    assert!(v.push(30).is_ok());
    assert_eq!(v.len(), 3);

    assert_eq!(v.pop(), Some(30));
    assert_eq!(v.pop(), Some(20));
    assert_eq!(v.pop(), Some(10));
    assert_eq!(v.pop(), None);
    assert!(v.is_empty());
}

#[test]
fn fixed_vec_full_returns_err() {
    let mut v: FixedVec<u8, 2> = FixedVec::new();
    assert!(v.push(1).is_ok());
    assert!(v.push(2).is_ok());
    assert!(v.is_full());
    assert_eq!(v.push(3), Err(3));
    assert_eq!(v.len(), 2);
}

#[test]
fn fixed_vec_iter() {
    let mut v: FixedVec<i32, 8> = FixedVec::new();
    for i in 0..5 {
        let _ = v.push(i * 10);
    }
    let collected: Vec<i32> = v.iter().copied().collect();
    assert_eq!(collected, vec![0, 10, 20, 30, 40]);
}

#[test]
fn fixed_vec_get_and_remove() {
    let mut v: FixedVec<&str, 4> = FixedVec::new();
    let _ = v.push("a");
    let _ = v.push("b");
    let _ = v.push("c");

    assert_eq!(v.get(1), Some(&"b"));
    assert_eq!(v.get(5), None);

    assert_eq!(v.remove(1), Some("b"));
    assert_eq!(v.len(), 2);
    assert_eq!(v.as_slice(), &["a", "c"]);
}

#[test]
fn fixed_string_from_str() {
    let s = FixedString::<16>::from_str("hello");
    assert_eq!(s.as_str(), "hello");
    assert_eq!(s.len(), 5);
}

#[test]
fn fixed_string_truncate() {
    let mut s = FixedString::<16>::from_str("hello world!");
    s.truncate(5);
    assert_eq!(s.as_str(), "hello");

    // Truncate beyond length is no-op
    s.truncate(100);
    assert_eq!(s.as_str(), "hello");
}

#[test]
fn fixed_string_from_bytes() {
    let valid = FixedString::<8>::from_bytes(b"OK");
    assert!(valid.is_some());
    assert_eq!(valid.unwrap().as_str(), "OK");

    let invalid = FixedString::<8>::from_bytes(&[0xFF, 0xFE]);
    assert!(invalid.is_none());
}

#[test]
fn fixed_string_capacity_overflow() {
    let s = FixedString::<4>::from_str("toolong");
    assert_eq!(s.len(), 4);
    assert_eq!(s.as_str(), "tool");
}

#[test]
fn ctrl_msg_repr_c_size() {
    // CtrlMsg should be exactly 16 bytes: u8+u8+u16+u16+u8+u8+u16+u16+u32
    assert_eq!(CtrlMsg::HEADER_SIZE, 16);
    assert_eq!(core::mem::size_of::<CtrlMsg>(), 16);
}

#[test]
fn ctrl_msg_alignment() {
    assert!(core::mem::align_of::<CtrlMsg>() <= 4);
}

#[test]
fn ctrl_msg_roundtrip() {
    let msg = CtrlMsg::new(Topic::CmdConfig, 0x0700, 42)
        .with_source(ServiceId::ControlGateway)
        .with_payload_len(128)
        .with_timestamp(999);

    let bytes = msg.as_bytes();
    assert_eq!(bytes.len(), 16);

    let parsed = CtrlMsg::from_bytes(bytes).unwrap();
    assert_eq!(parsed.topic, Topic::CmdConfig as u8);
    assert_eq!(parsed.method_id, 0x0700);
    assert_eq!(parsed.request_id, 42);
    assert_eq!(parsed.source, ServiceId::ControlGateway as u8);
    assert_eq!(parsed.payload_len, 128);
    assert_eq!(parsed.timestamp_ms, 999);
}

#[test]
fn frame_header_alignment() {
    // FrameHeader contains u64 fields, so alignment should be 8
    assert_eq!(core::mem::align_of::<FrameHeader>(), 8);
    // Size: u8+u8+u8+u8 + u32 + u64 + u64 + u32 + u32 = 32
    assert_eq!(FrameHeader::HEADER_SIZE, 32);
}

#[test]
fn frame_header_roundtrip() {
    let hdr = FrameHeader::new(frame::FrameType::VideoH264Idr, 0, 100)
        .with_pts(12345)
        .with_dts(12340)
        .with_data_len(65536);

    let bytes = hdr.as_bytes();
    let parsed = FrameHeader::from_bytes(bytes).unwrap();
    assert_eq!(parsed.seq, 100);
    assert_eq!(parsed.pts_ms, 12345);
    assert_eq!(parsed.dts_ms, 12340);
    assert_eq!(parsed.data_len, 65536);
    assert!(parsed.frame_type == frame::FrameType::VideoH264Idr as u8);
}

#[test]
fn topic_classification() {
    assert!(Topic::VideoMainStream.is_data_plane());
    assert!(Topic::AudioCapture.is_data_plane());
    assert!(!Topic::VideoMainStream.is_control_plane());

    assert!(Topic::CmdLive.is_control_plane());
    assert!(Topic::CmdConfig.is_control_plane());
    assert!(!Topic::CmdLive.is_data_plane());
    assert!(!Topic::CmdLive.is_event_plane());

    assert!(Topic::EvtConfigChanged.is_event_plane());
    assert!(Topic::EvtAlarm.is_event_plane());
    assert!(!Topic::EvtConfigChanged.is_control_plane());
}

#[test]
fn service_state_values() {
    assert_eq!(ServiceState::Normal as u8, 0);
    assert_eq!(ServiceState::Degraded as u8, 1);
    assert_eq!(ServiceState::Suspended as u8, 2);
}

// Need to use the frame module for FrameType
use core_types::frame;
