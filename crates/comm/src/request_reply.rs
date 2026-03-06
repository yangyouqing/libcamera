use core::sync::atomic::{AtomicU16, Ordering};
use core_types::{CamError, CommResult, CtrlMsg, Topic};
use core_interfaces::PendingReply;

const MAX_PENDING: usize = 16;

struct PendingEntry {
    request_id: u16,
    topic: Topic,
    active: bool,
    response: Option<CtrlMsg>,
}

/// Non-blocking Request/Reply engine.
/// Tracks pending requests and matches incoming replies by request_id.
pub struct RequestReplyEngine {
    next_id: AtomicU16,
    pending: [PendingEntry; MAX_PENDING],
}

impl RequestReplyEngine {
    pub const fn new() -> Self {
        const EMPTY: PendingEntry = PendingEntry {
            request_id: 0,
            topic: Topic::CmdConfig,
            active: false,
            response: None,
        };
        Self {
            next_id: AtomicU16::new(1),
            pending: [EMPTY; MAX_PENDING],
        }
    }

    /// Allocate a new request ID and register a pending request.
    pub fn create_pending(&mut self, topic: Topic, timestamp_ms: u64) -> CommResult<PendingReply> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        if id == 0 {
            // Skip 0, it's reserved
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            return self.register_pending(id, topic, timestamp_ms);
        }
        self.register_pending(id, topic, timestamp_ms)
    }

    fn register_pending(&mut self, id: u16, topic: Topic, timestamp_ms: u64) -> CommResult<PendingReply> {
        for entry in self.pending.iter_mut() {
            if !entry.active {
                entry.request_id = id;
                entry.topic = topic;
                entry.active = true;
                entry.response = None;
                return Ok(PendingReply {
                    request_id: id,
                    topic,
                    sent_at_ms: timestamp_ms,
                });
            }
        }
        Err(CamError::ResourceExhausted)
    }

    /// Deliver an incoming response. Matches by request_id.
    pub fn deliver_response(&mut self, msg: &CtrlMsg) {
        for entry in self.pending.iter_mut() {
            if entry.active && entry.request_id == msg.request_id {
                entry.response = Some(*msg);
                return;
            }
        }
    }

    /// Poll for a response to a pending request.
    pub fn poll(&mut self, pending: &PendingReply) -> Option<CtrlMsg> {
        for entry in self.pending.iter_mut() {
            if entry.active && entry.request_id == pending.request_id {
                if let Some(resp) = entry.response.take() {
                    entry.active = false;
                    return Some(resp);
                }
                return None;
            }
        }
        None
    }

    /// Cancel a pending request.
    pub fn cancel(&mut self, pending: &PendingReply) -> CommResult<()> {
        for entry in self.pending.iter_mut() {
            if entry.active && entry.request_id == pending.request_id {
                entry.active = false;
                entry.response = None;
                return Ok(());
            }
        }
        Err(CamError::NotFound)
    }

    pub fn pending_count(&self) -> usize {
        self.pending.iter().filter(|e| e.active).count()
    }
}
