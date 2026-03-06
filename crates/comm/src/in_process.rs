extern crate alloc;

use alloc::vec::Vec;
use core_types::{CamError, CommResult, CtrlMsg, FrameHeader, Topic};
use core_interfaces::{CommBus, PendingReply};
use crate::ring_buffer::SpscRingBuf;
use crate::request_reply::RequestReplyEngine;
use crate::spin_mutex::SpinMutex;

const CTRL_SLOT_SIZE: usize = 256;
const CTRL_SLOT_COUNT: usize = 64;
const FRAME_SLOT_SIZE: usize = 256 * 1024;
const FRAME_SLOT_COUNT: usize = 8;
const MAX_SUBSCRIBERS: usize = 8;
const TOPIC_TABLE_SIZE: usize = 64;

/// Single-process, multi-thread CommBus implementation.
/// Uses SpinMutex for thread-safe interior mutability.
/// Suitable for integration tests and RTOS multi-thread mode.
pub struct InProcessCommBus {
    ctrl_subscriptions: SpinMutex<[TopicSubs; TOPIC_TABLE_SIZE]>,
    frame_subscriptions: SpinMutex<[TopicSubs; TOPIC_TABLE_SIZE]>,
    rr_engine: SpinMutex<RequestReplyEngine>,
}

struct TopicSubs {
    subscribers: [Option<SubscriberRing>; MAX_SUBSCRIBERS],
    count: u8,
}

struct SubscriberRing {
    ring: SpscRingBuf,
    #[allow(dead_code)]
    backing: Vec<u8>,
    id: u32,
}

static NEXT_SUB_ID: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1);

impl TopicSubs {
    fn new() -> Self {
        Self {
            subscribers: Default::default(),
            count: 0,
        }
    }
}

unsafe impl Send for InProcessCommBus {}
unsafe impl Sync for InProcessCommBus {}

impl InProcessCommBus {
    pub fn new() -> Self {
        const EMPTY_SUBS: TopicSubs = TopicSubs {
            subscribers: [None, None, None, None, None, None, None, None],
            count: 0,
        };
        Self {
            ctrl_subscriptions: SpinMutex::new([EMPTY_SUBS; TOPIC_TABLE_SIZE]),
            frame_subscriptions: SpinMutex::new([EMPTY_SUBS; TOPIC_TABLE_SIZE]),
            rr_engine: SpinMutex::new(RequestReplyEngine::new()),
        }
    }

    fn add_subscriber(subs: &mut TopicSubs, slot_size: usize, slot_count: usize) -> CommResult<()> {
        if subs.count as usize >= MAX_SUBSCRIBERS {
            return Err(CamError::ResourceExhausted);
        }
        for slot in subs.subscribers.iter_mut() {
            if slot.is_none() {
                let mut backing = alloc::vec![0u8; slot_size * slot_count];
                let ring = SpscRingBuf::new(&mut backing, slot_size);
                let id = NEXT_SUB_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                *slot = Some(SubscriberRing { ring, backing, id });
                subs.count += 1;
                return Ok(());
            }
        }
        Err(CamError::ResourceExhausted)
    }

    fn publish_to_subs(subs: &TopicSubs, data: &[u8]) -> usize {
        let mut count = 0;
        for sub in subs.subscribers.iter().flatten() {
            sub.ring.push(data);
            count += 1;
        }
        count
    }
}

impl CommBus for InProcessCommBus {
    fn publish_ctrl(&self, topic: Topic, msg: &CtrlMsg, payload: &[u8]) -> CommResult<()> {
        let idx = topic as u8 as usize;
        if idx >= TOPIC_TABLE_SIZE {
            return Err(CamError::InvalidParam);
        }

        let total = CtrlMsg::HEADER_SIZE + payload.len();
        if total > CTRL_SLOT_SIZE {
            return Err(CamError::BufferFull);
        }

        let mut buf = [0u8; CTRL_SLOT_SIZE];
        buf[..CtrlMsg::HEADER_SIZE].copy_from_slice(msg.as_bytes());
        if !payload.is_empty() {
            buf[CtrlMsg::HEADER_SIZE..CtrlMsg::HEADER_SIZE + payload.len()]
                .copy_from_slice(payload);
        }

        if msg.is_response() {
            self.rr_engine.lock().deliver_response(msg);
        }

        let subs = self.ctrl_subscriptions.lock();
        Self::publish_to_subs(&subs[idx], &buf[..total]);
        Ok(())
    }

    fn poll_ctrl(&self, buf: &mut [u8]) -> CommResult<Option<(Topic, CtrlMsg)>> {
        let subs = self.ctrl_subscriptions.lock();
        for idx in 0..TOPIC_TABLE_SIZE {
            if subs[idx].count == 0 {
                continue;
            }
            for sub in subs[idx].subscribers.iter().flatten() {
                let mut slot_buf = [0u8; CTRL_SLOT_SIZE];
                if let Some(n) = sub.ring.pop(&mut slot_buf) {
                    if n >= CtrlMsg::HEADER_SIZE {
                        if let Some(msg) = CtrlMsg::from_bytes(&slot_buf[..n]) {
                            let payload_len = msg.payload_len as usize;
                            if payload_len > 0 && buf.len() >= payload_len {
                                let start = CtrlMsg::HEADER_SIZE;
                                buf[..payload_len].copy_from_slice(
                                    &slot_buf[start..start + payload_len]
                                );
                            }
                            let topic_byte = idx as u8;
                            let topic = unsafe {
                                core::mem::transmute::<u8, Topic>(topic_byte)
                            };
                            return Ok(Some((topic, *msg)));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn subscribe(&self, topic: Topic) -> CommResult<()> {
        let idx = topic as u8 as usize;
        if idx >= TOPIC_TABLE_SIZE {
            return Err(CamError::InvalidParam);
        }

        if topic.is_data_plane() {
            let mut subs = self.frame_subscriptions.lock();
            Self::add_subscriber(&mut subs[idx], FRAME_SLOT_SIZE, FRAME_SLOT_COUNT)
        } else {
            let mut subs = self.ctrl_subscriptions.lock();
            Self::add_subscriber(&mut subs[idx], CTRL_SLOT_SIZE, CTRL_SLOT_COUNT)
        }
    }

    fn unsubscribe(&self, topic: Topic) -> CommResult<()> {
        let idx = topic as u8 as usize;
        if idx >= TOPIC_TABLE_SIZE {
            return Err(CamError::InvalidParam);
        }

        let mut subs = if topic.is_data_plane() {
            self.frame_subscriptions.lock()
        } else {
            self.ctrl_subscriptions.lock()
        };

        // Remove the last subscriber (LIFO order for deterministic behavior)
        for slot in subs[idx].subscribers.iter_mut().rev() {
            if slot.is_some() {
                *slot = None;
                subs[idx].count = subs[idx].count.saturating_sub(1);
                return Ok(());
            }
        }
        Err(CamError::NotFound)
    }

    fn publish_frame(&self, topic: Topic, header: &FrameHeader, data: &[u8]) -> CommResult<()> {
        let idx = topic as u8 as usize;
        if idx >= TOPIC_TABLE_SIZE {
            return Err(CamError::InvalidParam);
        }

        let total = FrameHeader::HEADER_SIZE + data.len();
        if total > FRAME_SLOT_SIZE {
            return Err(CamError::BufferFull);
        }

        let mut hdr = *header;
        hdr.data_len = data.len() as u32;
        let mut buf = alloc::vec![0u8; total];
        buf[..FrameHeader::HEADER_SIZE].copy_from_slice(hdr.as_bytes());
        buf[FrameHeader::HEADER_SIZE..].copy_from_slice(data);

        let subs = self.frame_subscriptions.lock();
        Self::publish_to_subs(&subs[idx], &buf);
        Ok(())
    }

    fn poll_frame(
        &self,
        topic: Topic,
        hdr_buf: &mut FrameHeader,
        data_buf: &mut [u8],
    ) -> CommResult<Option<usize>> {
        let idx = topic as u8 as usize;
        if idx >= TOPIC_TABLE_SIZE {
            return Err(CamError::InvalidParam);
        }

        let subs = self.frame_subscriptions.lock();
        for sub in subs[idx].subscribers.iter().flatten() {
            let mut slot_buf = alloc::vec![0u8; FRAME_SLOT_SIZE];
            if let Some(n) = sub.ring.pop(&mut slot_buf) {
                if n >= FrameHeader::HEADER_SIZE {
                    if let Some(hdr) = FrameHeader::from_bytes(&slot_buf[..n]) {
                        *hdr_buf = *hdr;
                        let actual_data = hdr.data_len as usize;
                        let data_len = actual_data
                            .min(n - FrameHeader::HEADER_SIZE)
                            .min(data_buf.len());
                        data_buf[..data_len].copy_from_slice(
                            &slot_buf[FrameHeader::HEADER_SIZE..FrameHeader::HEADER_SIZE + data_len]
                        );
                        return Ok(Some(data_len));
                    }
                }
            }
        }
        Ok(None)
    }

    fn send_request(&self, topic: Topic, msg: &CtrlMsg, payload: &[u8]) -> CommResult<PendingReply> {
        let pending = self.rr_engine.lock().create_pending(topic, msg.timestamp_ms as u64)?;

        let mut req = *msg;
        req.request_id = pending.request_id;

        self.publish_ctrl(topic, &req, payload)?;
        Ok(pending)
    }

    fn poll_reply(&self, pending: &PendingReply, _buf: &mut [u8]) -> CommResult<Option<CtrlMsg>> {
        Ok(self.rr_engine.lock().poll(pending))
    }

    fn cancel_request(&self, pending: PendingReply) -> CommResult<()> {
        self.rr_engine.lock().cancel(&pending)
    }

    fn reply(&self, topic: Topic, request_id: u16, msg: &CtrlMsg, payload: &[u8]) -> CommResult<()> {
        let mut resp = *msg;
        resp.request_id = request_id;
        resp.flags |= CtrlMsg::FLAG_RESPONSE;
        self.publish_ctrl(topic, &resp, payload)
    }
}
