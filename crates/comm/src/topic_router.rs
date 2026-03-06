use core_types::{CamError, CommResult, Topic};
use crate::ring_buffer::SpscRingBuf;

const MAX_SUBSCRIBERS_PER_TOPIC: usize = 8;
const MAX_TOPICS: usize = 48; // Must cover all Topic discriminant values

struct TopicEntry {
    subscribers: [Option<*const SpscRingBuf>; MAX_SUBSCRIBERS_PER_TOPIC],
    count: u8,
}

impl TopicEntry {
    const fn new() -> Self {
        Self {
            subscribers: [None; MAX_SUBSCRIBERS_PER_TOPIC],
            count: 0,
        }
    }
}

/// Routes messages from publishers to subscribed ring buffers.
/// Each topic can have up to MAX_SUBSCRIBERS_PER_TOPIC consumers.
pub struct TopicRouter {
    topics: [TopicEntry; MAX_TOPICS],
}

// SAFETY: TopicRouter manages SPSC ring pointers. Subscribe/unsubscribe should
// be called from the orchestrator thread; publish can be called from producer threads.
unsafe impl Send for TopicRouter {}
unsafe impl Sync for TopicRouter {}

impl TopicRouter {
    pub const fn new() -> Self {
        const EMPTY: TopicEntry = TopicEntry::new();
        Self {
            topics: [EMPTY; MAX_TOPICS],
        }
    }

    fn topic_index(topic: Topic) -> usize {
        topic as u8 as usize
    }

    fn entry_for(topics: &[TopicEntry; MAX_TOPICS], topic: Topic) -> Option<&TopicEntry> {
        let idx = Self::topic_index(topic);
        if idx < MAX_TOPICS { Some(&topics[idx]) } else { None }
    }

    fn entry_for_mut(topics: &mut [TopicEntry; MAX_TOPICS], topic: Topic) -> Option<&mut TopicEntry> {
        let idx = Self::topic_index(topic);
        if idx < MAX_TOPICS { Some(&mut topics[idx]) } else { None }
    }

    /// Subscribe a ring buffer to a topic.
    pub fn subscribe(&mut self, topic: Topic, ring: &SpscRingBuf) -> CommResult<()> {
        let entry = Self::entry_for_mut(&mut self.topics, topic)
            .ok_or(CamError::InvalidParam)?;

        if entry.count as usize >= MAX_SUBSCRIBERS_PER_TOPIC {
            return Err(CamError::ResourceExhausted);
        }

        let ptr = ring as *const SpscRingBuf;

        // Check for duplicate
        for sub in entry.subscribers.iter().flatten() {
            if core::ptr::eq(*sub, ptr) {
                return Err(CamError::AlreadyExists);
            }
        }

        for slot in entry.subscribers.iter_mut() {
            if slot.is_none() {
                *slot = Some(ptr);
                entry.count += 1;
                return Ok(());
            }
        }
        Err(CamError::ResourceExhausted)
    }

    /// Unsubscribe a ring buffer from a topic.
    pub fn unsubscribe(&mut self, topic: Topic, ring: &SpscRingBuf) -> CommResult<()> {
        let entry = Self::entry_for_mut(&mut self.topics, topic)
            .ok_or(CamError::InvalidParam)?;

        let ptr = ring as *const SpscRingBuf;
        for slot in entry.subscribers.iter_mut() {
            if let Some(sub_ptr) = slot {
                if core::ptr::eq(*sub_ptr, ptr) {
                    *slot = None;
                    entry.count -= 1;
                    return Ok(());
                }
            }
        }
        Err(CamError::NotFound)
    }

    /// Route a message to all subscribers of a topic.
    /// Returns the number of subscribers that received the message.
    pub fn route(&self, topic: Topic, data: &[u8]) -> usize {
        let idx = Self::topic_index(topic);
        if idx >= MAX_TOPICS {
            return 0;
        }
        let entry = &self.topics[idx];
        let mut count = 0usize;
        for slot in &entry.subscribers {
            if let Some(ring_ptr) = slot {
                // SAFETY: pointer was set from a valid reference in subscribe()
                let ring = unsafe { &**ring_ptr };
                ring.push(data);
                count += 1;
            }
        }
        count
    }

    /// Get subscriber count for a topic.
    pub fn subscriber_count(&self, topic: Topic) -> usize {
        let idx = Self::topic_index(topic);
        if idx >= MAX_TOPICS { 0 } else { self.topics[idx].count as usize }
    }
}
