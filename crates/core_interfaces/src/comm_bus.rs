use core_types::{CommResult, CtrlMsg, FrameHeader, Topic};

/// Handle returned by send_request for non-blocking request/reply.
#[derive(Debug)]
pub struct PendingReply {
    pub request_id: u16,
    pub topic: Topic,
    pub sent_at_ms: u64,
}

/// Unified communication bus trait: Pub/Sub + non-blocking Request/Reply.
///
/// Data plane (frames) goes through shm RingBuffer on Linux.
/// Control/event plane goes through UDS on Linux.
/// InProcessCommBus provides a single-process implementation for testing and RTOS.
pub trait CommBus {
    // ── Pub/Sub: Control & Event plane ──

    /// Publish a control/event message to a topic.
    fn publish_ctrl(&self, topic: Topic, msg: &CtrlMsg, payload: &[u8]) -> CommResult<()>;

    /// Poll for control/event messages on subscribed topics.
    fn poll_ctrl(&self, buf: &mut [u8]) -> CommResult<Option<(Topic, CtrlMsg)>>;

    /// Subscribe to a control/event topic.
    fn subscribe(&self, topic: Topic) -> CommResult<()>;

    /// Unsubscribe from a control/event topic.
    fn unsubscribe(&self, topic: Topic) -> CommResult<()>;

    // ── Pub/Sub: Data plane (frames) ──

    /// Publish a media frame to a data plane topic.
    fn publish_frame(&self, topic: Topic, header: &FrameHeader, data: &[u8]) -> CommResult<()>;

    /// Poll for media frames on subscribed data topics.
    fn poll_frame(&self, topic: Topic, hdr_buf: &mut FrameHeader, data_buf: &mut [u8])
        -> CommResult<Option<usize>>;

    // ── Request/Reply (non-blocking) ──

    /// Send a request and get a PendingReply handle (non-blocking).
    fn send_request(&self, topic: Topic, msg: &CtrlMsg, payload: &[u8]) -> CommResult<PendingReply>;

    /// Poll for a matching reply to a pending request (non-blocking).
    fn poll_reply(&self, pending: &PendingReply, buf: &mut [u8])
        -> CommResult<Option<CtrlMsg>>;

    /// Cancel a pending request.
    fn cancel_request(&self, pending: PendingReply) -> CommResult<()>;

    /// Send a reply to a received request.
    fn reply(&self, topic: Topic, request_id: u16, msg: &CtrlMsg, payload: &[u8]) -> CommResult<()>;
}
