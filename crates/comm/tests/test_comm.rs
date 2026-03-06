use comm::ring_buffer::SpscRingBuf;
use comm::fan_out::{FanOutPublisher, RefCountedSlot, MAX_FANOUT};
use comm::topic_router::TopicRouter;
use comm::request_reply::RequestReplyEngine;
use comm::in_process::InProcessCommBus;
use core_types::*;
use core_interfaces::{CommBus, PendingReply};

// ══════════════════════════════════════════════
// Step 06: SPSC RingBuffer tests
// ══════════════════════════════════════════════

#[test]
fn spsc_push_pop_basic() {
    let mut backing = vec![0u8; 64 * 4];
    let ring = SpscRingBuf::new(&mut backing, 64);

    assert!(ring.is_empty());
    assert_eq!(ring.slot_count(), 4);

    let data = [0xAAu8; 64];
    assert!(ring.push(&data));
    assert!(!ring.is_empty());

    let mut out = [0u8; 64];
    assert_eq!(ring.pop(&mut out), Some(64));
    assert_eq!(out[0], 0xAA);
    assert!(ring.is_empty());
}

#[test]
fn spsc_full_backpressure_drops_oldest() {
    let mut backing = vec![0u8; 16 * 4];
    let ring = SpscRingBuf::new(&mut backing, 16);

    // Fill up (3 usable slots in a 4-slot circular buffer)
    ring.push(&[1u8; 16]);
    ring.push(&[2u8; 16]);
    ring.push(&[3u8; 16]);
    assert!(ring.is_full());

    // Push one more should overwrite the oldest
    ring.push(&[4u8; 16]);

    let mut out = [0u8; 16];
    // Oldest should now be [2] since [1] was overwritten
    assert_eq!(ring.pop(&mut out), Some(16));
    assert_eq!(out[0], 2);
}

#[test]
fn spsc_empty_pop_returns_none() {
    let mut backing = vec![0u8; 32 * 2];
    let ring = SpscRingBuf::new(&mut backing, 32);
    let mut out = [0u8; 32];
    assert_eq!(ring.pop(&mut out), None);
}

#[test]
fn spsc_capacity_boundary() {
    let mut backing = vec![0u8; 8 * 2]; // 2 slots, 1 usable
    let ring = SpscRingBuf::new(&mut backing, 8);

    ring.push(&[10u8; 8]);
    assert!(ring.is_full());

    let mut out = [0u8; 8];
    assert_eq!(ring.pop(&mut out), Some(8));
    assert_eq!(out[0], 10);
    assert!(ring.is_empty());
}

// ══════════════════════════════════════════════
// Step 08: Fan-out tests
// ══════════════════════════════════════════════

#[test]
fn fanout_1_to_3() {
    let mut b1 = vec![0u8; 32 * 4];
    let mut b2 = vec![0u8; 32 * 4];
    let mut b3 = vec![0u8; 32 * 4];

    let r1 = SpscRingBuf::new(&mut b1, 32);
    let r2 = SpscRingBuf::new(&mut b2, 32);
    let r3 = SpscRingBuf::new(&mut b3, 32);

    let mut fanout = FanOutPublisher::new();
    assert_eq!(fanout.add_consumer(&r1), Some(0));
    assert_eq!(fanout.add_consumer(&r2), Some(1));
    assert_eq!(fanout.add_consumer(&r3), Some(2));
    assert_eq!(fanout.active_count(), 3);

    let data = [0x42u8; 32];
    assert_eq!(fanout.publish(&data), 3);

    let mut out = [0u8; 32];
    assert_eq!(r1.pop(&mut out), Some(32));
    assert_eq!(out[0], 0x42);
    assert_eq!(r2.pop(&mut out), Some(32));
    assert_eq!(out[0], 0x42);
    assert_eq!(r3.pop(&mut out), Some(32));
    assert_eq!(out[0], 0x42);
}

#[test]
fn fanout_add_remove_consumer() {
    let mut b1 = vec![0u8; 16 * 4];
    let mut b2 = vec![0u8; 16 * 4];

    let r1 = SpscRingBuf::new(&mut b1, 16);
    let r2 = SpscRingBuf::new(&mut b2, 16);

    let mut fanout = FanOutPublisher::new();
    let idx1 = fanout.add_consumer(&r1).unwrap();
    let idx2 = fanout.add_consumer(&r2).unwrap();
    assert_eq!(fanout.active_count(), 2);

    assert!(fanout.remove_consumer(idx1));
    assert_eq!(fanout.active_count(), 1);

    assert_eq!(fanout.publish(&[0xFF; 16]), 1);

    // r1 should not receive (removed)
    let mut out = [0u8; 16];
    assert_eq!(r1.pop(&mut out), None);
    // r2 should receive
    assert_eq!(r2.pop(&mut out), Some(16));
    assert_eq!(out[0], 0xFF);

    assert!(fanout.remove_consumer(idx2));
    assert_eq!(fanout.active_count(), 0);
}

#[test]
fn fanout_max_capacity_rejected() {
    let mut backings: Vec<Vec<u8>> = (0..MAX_FANOUT + 1)
        .map(|_| vec![0u8; 8 * 2])
        .collect();
    let mut rings: Vec<SpscRingBuf> = backings
        .iter_mut()
        .map(|b| SpscRingBuf::new(b, 8))
        .collect();

    let mut fanout = FanOutPublisher::new();
    for i in 0..MAX_FANOUT {
        assert!(fanout.add_consumer(&rings[i]).is_some());
    }
    assert!(fanout.add_consumer(&rings[MAX_FANOUT]).is_none());
}

#[test]
fn refcounted_slot_lifecycle() {
    let slot = RefCountedSlot::new();
    assert_eq!(slot.count(), 0);

    assert_eq!(slot.acquire(), 1);
    assert_eq!(slot.acquire(), 2);
    assert_eq!(slot.acquire(), 3);
    assert_eq!(slot.count(), 3);

    assert_eq!(slot.release(), 2);
    assert_eq!(slot.release(), 1);
    assert_eq!(slot.release(), 0);
    assert_eq!(slot.count(), 0);
}

// ══════════════════════════════════════════════
// Step 10: TopicRouter tests
// ══════════════════════════════════════════════

#[test]
fn router_subscribe_and_route() {
    let mut b1 = vec![0u8; 64 * 4];
    let mut b2 = vec![0u8; 64 * 4];
    let r1 = SpscRingBuf::new(&mut b1, 64);
    let r2 = SpscRingBuf::new(&mut b2, 64);

    let mut router = TopicRouter::new();
    assert!(router.subscribe(Topic::EvtConfigChanged, &r1).is_ok());
    assert!(router.subscribe(Topic::EvtConfigChanged, &r2).is_ok());
    assert_eq!(router.subscriber_count(Topic::EvtConfigChanged), 2);

    let data = [0xBBu8; 64];
    assert_eq!(router.route(Topic::EvtConfigChanged, &data), 2);

    let mut out = [0u8; 64];
    assert_eq!(r1.pop(&mut out), Some(64));
    assert_eq!(out[0], 0xBB);
    assert_eq!(r2.pop(&mut out), Some(64));
    assert_eq!(out[0], 0xBB);
}

#[test]
fn router_unsubscribe() {
    let mut b1 = vec![0u8; 32 * 4];
    let r1 = SpscRingBuf::new(&mut b1, 32);

    let mut router = TopicRouter::new();
    router.subscribe(Topic::CmdConfig, &r1).unwrap();
    assert_eq!(router.subscriber_count(Topic::CmdConfig), 1);

    router.unsubscribe(Topic::CmdConfig, &r1).unwrap();
    assert_eq!(router.subscriber_count(Topic::CmdConfig), 0);

    assert_eq!(router.route(Topic::CmdConfig, &[0u8; 32]), 0);
}

#[test]
fn router_unsubscribed_topic_drops_message() {
    let router = TopicRouter::new();
    assert_eq!(router.route(Topic::CmdLive, &[0u8; 32]), 0);
}

#[test]
fn router_multi_subscriber_fanout() {
    let mut backings: Vec<Vec<u8>> = (0..4).map(|_| vec![0u8; 16 * 4]).collect();
    let rings: Vec<SpscRingBuf> = backings
        .iter_mut()
        .map(|b| SpscRingBuf::new(b, 16))
        .collect();

    let mut router = TopicRouter::new();
    for r in &rings {
        router.subscribe(Topic::EvtAlarm, r).unwrap();
    }

    assert_eq!(router.route(Topic::EvtAlarm, &[0xCC; 16]), 4);
    for r in &rings {
        let mut out = [0u8; 16];
        assert_eq!(r.pop(&mut out), Some(16));
        assert_eq!(out[0], 0xCC);
    }
}

// ══════════════════════════════════════════════
// Step 12: Request/Reply tests
// ══════════════════════════════════════════════

#[test]
fn rr_send_poll_basic() {
    let mut engine = RequestReplyEngine::new();
    let pending = engine.create_pending(Topic::CmdConfig, 1000).unwrap();
    assert_eq!(engine.pending_count(), 1);

    // No response yet
    assert!(engine.poll(&pending).is_none());

    // Deliver a matching response
    let mut resp = CtrlMsg::new(Topic::CmdConfig, 0x0700, pending.request_id);
    resp.flags |= CtrlMsg::FLAG_RESPONSE;
    engine.deliver_response(&resp);

    let result = engine.poll(&pending).unwrap();
    assert_eq!(result.request_id, pending.request_id);
    assert!(result.is_response());
    assert_eq!(engine.pending_count(), 0);
}

#[test]
fn rr_cancel() {
    let mut engine = RequestReplyEngine::new();
    let pending = engine.create_pending(Topic::CmdLive, 2000).unwrap();
    assert_eq!(engine.pending_count(), 1);

    engine.cancel(&pending).unwrap();
    assert_eq!(engine.pending_count(), 0);
}

#[test]
fn rr_multiple_concurrent() {
    let mut engine = RequestReplyEngine::new();
    let p1 = engine.create_pending(Topic::CmdConfig, 100).unwrap();
    let p2 = engine.create_pending(Topic::CmdLive, 200).unwrap();
    let p3 = engine.create_pending(Topic::CmdRecord, 300).unwrap();
    assert_eq!(engine.pending_count(), 3);

    // Respond to p2 first
    let mut resp2 = CtrlMsg::new(Topic::CmdLive, 0x0100, p2.request_id);
    resp2.flags |= CtrlMsg::FLAG_RESPONSE;
    engine.deliver_response(&resp2);

    assert!(engine.poll(&p1).is_none()); // p1 not yet responded
    assert!(engine.poll(&p2).is_some()); // p2 responded
    assert!(engine.poll(&p3).is_none()); // p3 not yet responded

    assert_eq!(engine.pending_count(), 2);
}

#[test]
fn rr_mismatched_id_returns_none() {
    let mut engine = RequestReplyEngine::new();
    let pending = engine.create_pending(Topic::CmdConfig, 500).unwrap();

    // Deliver response with wrong request_id
    let mut resp = CtrlMsg::new(Topic::CmdConfig, 0x0700, pending.request_id + 100);
    resp.flags |= CtrlMsg::FLAG_RESPONSE;
    engine.deliver_response(&resp);

    assert!(engine.poll(&pending).is_none());
}

// ══════════════════════════════════════════════
// Step 14: InProcessCommBus tests
// ══════════════════════════════════════════════

#[test]
fn inprocess_ctrl_pubsub() {
    let bus = InProcessCommBus::new();
    bus.subscribe(Topic::EvtConfigChanged).unwrap();

    let msg = CtrlMsg::new(Topic::EvtConfigChanged, 0, 0)
        .with_source(ServiceId::Config);
    bus.publish_ctrl(Topic::EvtConfigChanged, &msg, &[]).unwrap();

    let mut buf = [0u8; 256];
    let (topic, received) = bus.poll_ctrl(&mut buf).unwrap().unwrap();
    assert_eq!(topic, Topic::EvtConfigChanged);
    assert_eq!(received.source, ServiceId::Config as u8);
}

#[test]
fn inprocess_frame_pubsub() {
    let bus = InProcessCommBus::new();
    bus.subscribe(Topic::VideoMainStream).unwrap();

    let hdr = FrameHeader::new(frame::FrameType::VideoH264Idr, 0, 1)
        .with_pts(1000)
        .with_data_len(4);
    let data = [0xDE, 0xAD, 0xBE, 0xEF];
    bus.publish_frame(Topic::VideoMainStream, &hdr, &data).unwrap();

    let mut recv_hdr = FrameHeader::new(frame::FrameType::AudioPcm, 0, 0);
    let mut recv_data = [0u8; 1024];
    let len = bus.poll_frame(Topic::VideoMainStream, &mut recv_hdr, &mut recv_data)
        .unwrap()
        .unwrap();
    assert_eq!(len, 4);
    assert_eq!(recv_hdr.seq, 1);
    assert_eq!(recv_hdr.pts_ms, 1000);
    assert_eq!(&recv_data[..4], &[0xDE, 0xAD, 0xBE, 0xEF]);
}

#[test]
fn inprocess_request_reply() {
    let bus = InProcessCommBus::new();
    bus.subscribe(Topic::CmdConfig).unwrap();

    let req = CtrlMsg::new(Topic::CmdConfig, MethodId::GetConfig as u16, 0)
        .with_source(ServiceId::ControlGateway);
    let pending = bus.send_request(Topic::CmdConfig, &req, &[]).unwrap();

    // Simulate service processing: poll the request
    let mut buf = [0u8; 256];
    let (_topic, received) = bus.poll_ctrl(&mut buf).unwrap().unwrap();
    assert_eq!(received.method_id, MethodId::GetConfig as u16);

    // Service sends reply
    let resp = CtrlMsg::new(Topic::CmdConfig, MethodId::GetConfig as u16, received.request_id)
        .with_source(ServiceId::Config);
    bus.reply(Topic::CmdConfig, received.request_id, &resp, &[]).unwrap();

    // Requester polls for reply
    let reply = bus.poll_reply(&pending, &mut buf).unwrap().unwrap();
    assert!(reply.is_response());
    assert_eq!(reply.source, ServiceId::Config as u8);
}

#[test]
fn inprocess_multi_thread_pubsub() {
    use std::sync::Arc;
    use std::thread;

    let bus = Arc::new(InProcessCommBus::new());
    bus.subscribe(Topic::EvtNetworkStatus).unwrap();

    let bus_pub = Arc::clone(&bus);
    let publisher = thread::spawn(move || {
        for i in 0..10u16 {
            let msg = CtrlMsg::new(Topic::EvtNetworkStatus, i, 0)
                .with_source(ServiceId::Network);
            bus_pub.publish_ctrl(Topic::EvtNetworkStatus, &msg, &[]).unwrap();
        }
    });

    publisher.join().unwrap();

    let mut buf = [0u8; 256];
    let mut received_count = 0;
    while let Ok(Some(_)) = bus.poll_ctrl(&mut buf) {
        received_count += 1;
    }
    assert_eq!(received_count, 10);
}

use core_types::frame;
