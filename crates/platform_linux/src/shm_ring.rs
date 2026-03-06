use core::sync::atomic::{AtomicUsize, Ordering};
use std::io;

/// Shared-memory backed SPSC ring buffer for Linux multi-process data plane.
/// Uses anonymous mmap + fd passing via SCM_RIGHTS.
pub struct ShmRingBuf {
    ptr: *mut u8,
    size: usize,
    slot_size: usize,
    slot_count: usize,
    fd: i32,
}

unsafe impl Send for ShmRingBuf {}
unsafe impl Sync for ShmRingBuf {}

/// Header stored at the beginning of the shared memory region.
#[repr(C)]
struct ShmHeader {
    head: AtomicUsize,
    tail: AtomicUsize,
    slot_size: usize,
    slot_count: usize,
}

const SHM_HEADER_SIZE: usize = core::mem::size_of::<ShmHeader>();

impl ShmRingBuf {
    /// Create a new shared memory ring buffer.
    pub fn create(slot_size: usize, slot_count: usize) -> io::Result<Self> {
        let data_size = slot_size * slot_count;
        let total_size = SHM_HEADER_SIZE + data_size;

        let fd = unsafe {
            libc::memfd_create(
                b"cam_shm\0".as_ptr() as *const libc::c_char,
                libc::MFD_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        if unsafe { libc::ftruncate(fd, total_size as libc::off_t) } != 0 {
            unsafe { libc::close(fd); }
            return Err(io::Error::last_os_error());
        }

        let ptr = unsafe {
            libc::mmap(
                core::ptr::null_mut(),
                total_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            unsafe { libc::close(fd); }
            return Err(io::Error::last_os_error());
        }

        // Initialize header
        let header = ptr as *mut ShmHeader;
        unsafe {
            (*header).head = AtomicUsize::new(0);
            (*header).tail = AtomicUsize::new(0);
            (*header).slot_size = slot_size;
            (*header).slot_count = slot_count;
        }

        Ok(Self {
            ptr: ptr as *mut u8,
            size: total_size,
            slot_size,
            slot_count,
            fd,
        })
    }

    /// Open an existing shared memory ring buffer from a file descriptor.
    pub fn from_fd(fd: i32, slot_size: usize, slot_count: usize) -> io::Result<Self> {
        let total_size = SHM_HEADER_SIZE + slot_size * slot_count;
        let ptr = unsafe {
            libc::mmap(
                core::ptr::null_mut(),
                total_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            ptr: ptr as *mut u8,
            size: total_size,
            slot_size,
            slot_count,
            fd,
        })
    }

    pub fn fd(&self) -> i32 {
        self.fd
    }

    fn header(&self) -> &ShmHeader {
        unsafe { &*(self.ptr as *const ShmHeader) }
    }

    fn data_base(&self) -> *mut u8 {
        unsafe { self.ptr.add(SHM_HEADER_SIZE) }
    }

    pub fn push(&self, data: &[u8]) -> bool {
        let hdr = self.header();
        let write_len = data.len().min(self.slot_size);
        let head = hdr.head.load(Ordering::Relaxed);
        let next_head = (head + 1) % self.slot_count;

        let tail = hdr.tail.load(Ordering::Acquire);
        if next_head == tail {
            hdr.tail.store((tail + 1) % self.slot_count, Ordering::Release);
        }

        let offset = head * self.slot_size;
        unsafe {
            let dst = self.data_base().add(offset);
            core::ptr::copy_nonoverlapping(data.as_ptr(), dst, write_len);
            if write_len < self.slot_size {
                core::ptr::write_bytes(dst.add(write_len), 0, self.slot_size - write_len);
            }
        }

        hdr.head.store(next_head, Ordering::Release);
        true
    }

    pub fn pop(&self, out: &mut [u8]) -> Option<usize> {
        let hdr = self.header();
        let tail = hdr.tail.load(Ordering::Relaxed);
        let head = hdr.head.load(Ordering::Acquire);

        if tail == head {
            return None;
        }

        let offset = tail * self.slot_size;
        let copy_len = out.len().min(self.slot_size);

        unsafe {
            let src = self.data_base().add(offset);
            core::ptr::copy_nonoverlapping(src, out.as_mut_ptr(), copy_len);
        }

        hdr.tail.store((tail + 1) % self.slot_count, Ordering::Release);
        Some(copy_len)
    }

    pub fn is_empty(&self) -> bool {
        let hdr = self.header();
        hdr.head.load(Ordering::Acquire) == hdr.tail.load(Ordering::Acquire)
    }
}

impl Drop for ShmRingBuf {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr as *mut libc::c_void, self.size);
            libc::close(self.fd);
        }
    }
}
