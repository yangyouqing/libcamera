use core::sync::atomic::{AtomicUsize, Ordering};

/// Single-Producer Single-Consumer lock-free ring buffer.
/// Uses atomic head/tail for thread safety without mutexes.
/// Back-pressure policy: overwrite oldest entry when full.
pub struct SpscRingBuf {
    buf: *mut u8,
    slot_size: usize,
    slot_count: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

// SAFETY: SpscRingBuf is designed for exactly one producer and one consumer
// on separate threads. The atomic head/tail ensure correct synchronization.
unsafe impl Send for SpscRingBuf {}
unsafe impl Sync for SpscRingBuf {}

impl SpscRingBuf {
    /// Create a new SPSC ring buffer with the given slot configuration.
    /// `backing` must be a slice of at least `slot_size * slot_count` bytes
    /// and must live as long as this ring buffer.
    ///
    /// # Safety
    /// Caller must ensure `backing` outlives this ring buffer and is exclusively
    /// owned by it.
    pub unsafe fn from_raw(backing: *mut u8, slot_size: usize, slot_count: usize) -> Self {
        assert!(slot_count > 0 && slot_size > 0);
        Self {
            buf: backing,
            slot_size,
            slot_count,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Create from a mutable slice (takes ownership of layout).
    pub fn new(backing: &mut [u8], slot_size: usize) -> Self {
        let slot_count = backing.len() / slot_size;
        assert!(slot_count > 0, "backing too small for one slot");
        // SAFETY: backing is a valid mutable slice, caller manages its lifetime
        unsafe { Self::from_raw(backing.as_mut_ptr(), slot_size, slot_count) }
    }

    #[inline]
    pub fn slot_size(&self) -> usize {
        self.slot_size
    }

    #[inline]
    pub fn slot_count(&self) -> usize {
        self.slot_count
    }

    #[inline]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if head >= tail { head - tail } else { self.slot_count - (tail - head) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire) == self.tail.load(Ordering::Acquire)
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        (head + 1) % self.slot_count == tail
    }

    /// Push data into the next slot. Returns true on success.
    /// If full, overwrites the oldest slot (advances tail).
    pub fn push(&self, data: &[u8]) -> bool {
        let write_len = data.len().min(self.slot_size);
        let head = self.head.load(Ordering::Relaxed);
        let next_head = (head + 1) % self.slot_count;

        // If full, advance tail (drop oldest)
        let tail = self.tail.load(Ordering::Acquire);
        if next_head == tail {
            self.tail.store((tail + 1) % self.slot_count, Ordering::Release);
        }

        let offset = head * self.slot_size;
        // SAFETY: head < slot_count, offset + slot_size <= capacity
        unsafe {
            let dst = self.buf.add(offset);
            core::ptr::copy_nonoverlapping(data.as_ptr(), dst, write_len);
            // Zero remaining bytes in slot
            if write_len < self.slot_size {
                core::ptr::write_bytes(dst.add(write_len), 0, self.slot_size - write_len);
            }
        }

        self.head.store(next_head, Ordering::Release);
        true
    }

    /// Pop the oldest slot's data. Returns number of bytes copied, or None if empty.
    pub fn pop(&self, out: &mut [u8]) -> Option<usize> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail == head {
            return None;
        }

        let offset = tail * self.slot_size;
        let copy_len = out.len().min(self.slot_size);

        // SAFETY: tail < slot_count, offset + slot_size <= capacity
        unsafe {
            let src = self.buf.add(offset);
            core::ptr::copy_nonoverlapping(src, out.as_mut_ptr(), copy_len);
        }

        self.tail.store((tail + 1) % self.slot_count, Ordering::Release);
        Some(copy_len)
    }

    /// Peek at the oldest slot without consuming it.
    pub fn peek(&self, out: &mut [u8]) -> Option<usize> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail == head {
            return None;
        }

        let offset = tail * self.slot_size;
        let copy_len = out.len().min(self.slot_size);

        unsafe {
            let src = self.buf.add(offset);
            core::ptr::copy_nonoverlapping(src, out.as_mut_ptr(), copy_len);
        }
        Some(copy_len)
    }

    /// Reset the ring buffer, discarding all data.
    pub fn clear(&self) {
        self.tail.store(self.head.load(Ordering::Acquire), Ordering::Release);
    }
}
