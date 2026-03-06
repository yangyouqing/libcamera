/// Frame types for the data plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    VideoH264Idr = 0,
    VideoH264P = 1,
    VideoH265Idr = 2,
    VideoH265P = 3,
    AudioPcm = 10,
    AudioAac = 11,
    AudioG711a = 12,
    AudioG711u = 13,
}

/// Data plane frame header. Binary, naturally aligned, for shm transmission.
/// Frame payload follows immediately after this header in the RingBuffer slot.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FrameHeader {
    pub frame_type: u8,
    pub stream_id: u8,
    pub flags: u8,
    pub _pad: u8,
    pub seq: u32,
    pub pts_ms: u64,
    pub dts_ms: u64,
    pub data_len: u32,
    pub _reserved: u32,
}

impl FrameHeader {
    pub const HEADER_SIZE: usize = core::mem::size_of::<Self>();

    pub const fn new(frame_type: FrameType, stream_id: u8, seq: u32) -> Self {
        Self {
            frame_type: frame_type as u8,
            stream_id,
            flags: 0,
            _pad: 0,
            seq,
            pts_ms: 0,
            dts_ms: 0,
            data_len: 0,
            _reserved: 0,
        }
    }

    pub const fn with_pts(mut self, pts: u64) -> Self {
        self.pts_ms = pts;
        self
    }

    pub const fn with_dts(mut self, dts: u64) -> Self {
        self.dts_ms = dts;
        self
    }

    pub const fn with_data_len(mut self, len: u32) -> Self {
        self.data_len = len;
        self
    }

    pub const FLAG_KEYFRAME: u8 = 0x01;
    pub const FLAG_EOS: u8 = 0x02;

    pub const fn is_keyframe(&self) -> bool {
        self.flags & Self::FLAG_KEYFRAME != 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: repr(C), naturally aligned, plain data
        unsafe {
            core::slice::from_raw_parts(self as *const Self as *const u8, Self::HEADER_SIZE)
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<&Self> {
        if bytes.len() < Self::HEADER_SIZE {
            return None;
        }
        let ptr = bytes.as_ptr();
        if ptr.align_offset(core::mem::align_of::<Self>()) != 0 {
            return None;
        }
        // SAFETY: alignment and size checked
        Some(unsafe { &*(ptr as *const Self) })
    }
}

impl core::fmt::Debug for FrameHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FrameHeader")
            .field("frame_type", &self.frame_type)
            .field("stream_id", &self.stream_id)
            .field("seq", &self.seq)
            .field("pts_ms", &self.pts_ms)
            .field("dts_ms", &self.dts_ms)
            .field("data_len", &self.data_len)
            .finish()
    }
}
