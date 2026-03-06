use core::sync::atomic::{AtomicU32, Ordering};
use crate::ring_buffer::SpscRingBuf;

pub const MAX_FANOUT: usize = 8;

/// A slot that holds a reference-counted shared buffer for large frames.
/// Avoids copying video frames to each consumer's ring buffer.
pub struct RefCountedSlot {
    pub refcount: AtomicU32,
    pub data_len: AtomicU32,
}

impl RefCountedSlot {
    pub const fn new() -> Self {
        Self {
            refcount: AtomicU32::new(0),
            data_len: AtomicU32::new(0),
        }
    }

    pub fn acquire(&self) -> u32 {
        self.refcount.fetch_add(1, Ordering::AcqRel) + 1
    }

    pub fn release(&self) -> u32 {
        let prev = self.refcount.fetch_sub(1, Ordering::AcqRel);
        prev - 1
    }

    pub fn count(&self) -> u32 {
        self.refcount.load(Ordering::Acquire)
    }

    pub fn reset(&self) {
        self.refcount.store(0, Ordering::Release);
        self.data_len.store(0, Ordering::Release);
    }
}

/// Fan-out publisher: 1 producer writes to N independent SPSC RingBuffers.
/// Supports runtime add/remove of consumers up to MAX_FANOUT.
pub struct FanOutPublisher {
    /// Each slot is an independent SPSC ring buffer, or None if inactive.
    /// We store raw pointers because the ring buffers are owned externally
    /// (typically in shared memory or static allocations).
    rings: [Option<*const SpscRingBuf>; MAX_FANOUT],
    active_count: u8,
}

// SAFETY: FanOutPublisher coordinates N SPSC buffers. Each ring is
// accessed only by the publisher (push) and its designated consumer (pop).
unsafe impl Send for FanOutPublisher {}
unsafe impl Sync for FanOutPublisher {}

impl FanOutPublisher {
    pub const fn new() -> Self {
        Self {
            rings: [None; MAX_FANOUT],
            active_count: 0,
        }
    }

    /// Add a consumer ring buffer. Returns the consumer index, or None if at capacity.
    pub fn add_consumer(&mut self, ring: &SpscRingBuf) -> Option<usize> {
        for (i, slot) in self.rings.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(ring as *const SpscRingBuf);
                self.active_count += 1;
                return Some(i);
            }
        }
        None
    }

    /// Remove a consumer by index.
    pub fn remove_consumer(&mut self, index: usize) -> bool {
        if index >= MAX_FANOUT {
            return false;
        }
        if self.rings[index].is_some() {
            self.rings[index] = None;
            self.active_count -= 1;
            true
        } else {
            false
        }
    }

    /// Number of currently active consumers.
    pub fn active_count(&self) -> u8 {
        self.active_count
    }

    /// Publish data to all active consumers. Returns number of consumers written to.
    pub fn publish(&self, data: &[u8]) -> usize {
        let mut count = 0usize;
        for slot in &self.rings {
            if let Some(ring_ptr) = slot {
                // SAFETY: ring_ptr was set via add_consumer from a valid reference
                let ring = unsafe { &**ring_ptr };
                ring.push(data);
                count += 1;
            }
        }
        count
    }
}
