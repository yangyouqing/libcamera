/// Identifies a service in the system. Used for startup dependency graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ServiceId {
    Config = 0,
    Network = 1,
    Storage = 2,
    TimeSync = 3,
    MediaCore = 4,
    Live = 5,
    Talk = 6,
    Record = 7,
    Playback = 8,
    Cloud = 9,
    Upgrade = 10,
    ControlGateway = 11,
}

impl ServiceId {
    pub const COUNT: usize = 12;
}

/// Three-level degradation state for services.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceState {
    Normal = 0,
    Degraded = 1,
    Suspended = 2,
}

/// Health status returned by Service::health().
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthStatus {
    pub service: ServiceId,
    pub state: ServiceState,
    pub error_code: u8,
}

/// Communication topics for Pub/Sub and Request/Reply.
/// Data plane topics use shm RingBuffer; control/event topics use UDS on Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Topic {
    // ── Data plane (shm, high throughput) ──
    VideoMainStream = 0,
    VideoSubStream = 1,
    AudioCapture = 2,
    TalkDownlink = 3,
    TalkUplink = 4,
    PlaybackStream = 5,

    // ── Control plane (UDS, Request/Reply) ──
    CmdLive = 10,
    CmdTalk = 11,
    CmdRecord = 12,
    CmdPlayback = 13,
    CmdCloud = 14,
    CmdUpgrade = 15,
    CmdConfig = 16,
    CmdStorage = 17,
    CmdNetwork = 18,
    CmdTime = 19,
    CmdDevice = 20,
    CmdMediaCore = 21,
    CmdControl = 22,

    // ── Event plane (UDS, Pub/Sub broadcast) ──
    EvtConfigChanged = 30,
    EvtNetworkStatus = 31,
    EvtStorageStatus = 32,
    EvtTimeSync = 33,
    EvtAlarm = 34,
    EvtSessionStatus = 35,
    EvtUpgradeStatus = 36,
}

impl Topic {
    pub const DATA_PLANE_COUNT: usize = 6;
    pub const CTRL_PLANE_COUNT: usize = 13;
    pub const EVT_PLANE_COUNT: usize = 7;
    pub const TOTAL_COUNT: usize = 26;

    /// Returns true for high-throughput data plane topics (shm).
    pub const fn is_data_plane(&self) -> bool {
        (*self as u8) < 10
    }

    /// Returns true for control plane command topics (UDS).
    pub const fn is_control_plane(&self) -> bool {
        let v = *self as u8;
        v >= 10 && v < 30
    }

    /// Returns true for event broadcast topics (UDS).
    pub const fn is_event_plane(&self) -> bool {
        (*self as u8) >= 30
    }
}

/// Method IDs for Request/Reply dispatch (used in CtrlMsg and ControlGateway routing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum MethodId {
    // Live
    StartLive = 0x0100,
    StopLive = 0x0101,

    // Talk
    StartTalk = 0x0200,
    StopTalk = 0x0201,
    SetTalkMode = 0x0202,

    // Record
    StartRecord = 0x0300,
    StopRecord = 0x0301,

    // Playback
    StartPlayback = 0x0400,
    StopPlayback = 0x0401,
    QueryTimeline = 0x0402,
    SeekPlayback = 0x0403,

    // Cloud
    StartUpload = 0x0500,
    StopUpload = 0x0501,
    QueryUploadQueue = 0x0502,

    // Upgrade
    CheckUpdate = 0x0600,
    StartUpgrade = 0x0601,
    QueryUpgradeStatus = 0x0602,

    // Config
    GetConfig = 0x0700,
    SetConfig = 0x0701,

    // Storage
    QueryCapacity = 0x0800,
    FormatStorage = 0x0801,

    // Network
    ScanWifi = 0x0900,
    ConnectWifi = 0x0901,
    GetNetworkStatus = 0x0902,

    // Time
    SyncNow = 0x0A00,
    QueryTime = 0x0A01,

    // Device
    Reboot = 0x0B00,
    FactoryReset = 0x0B01,
    GetDeviceInfo = 0x0B02,

    // MediaCore
    SetBitrate = 0x0C00,
    RequestIdr = 0x0C01,
    SetResolution = 0x0C02,
}

/// Authentication levels for ControlGateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum AuthLevel {
    None = 0,
    Viewer = 1,
    Admin = 2,
}

/// Control plane message envelope. Binary, naturally aligned, for IPC.
/// Optional JSON payload follows this header in a separate buffer.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CtrlMsg {
    pub topic: u8,
    pub _pad: u8,
    pub method_id: u16,
    pub request_id: u16,
    pub source: u8,
    pub flags: u8,
    pub payload_len: u16,
    pub _reserved: u16,
    pub timestamp_ms: u32,
}

impl CtrlMsg {
    pub const HEADER_SIZE: usize = core::mem::size_of::<Self>();

    pub const fn new(topic: Topic, method_id: u16, request_id: u16) -> Self {
        Self {
            topic: topic as u8,
            _pad: 0,
            method_id,
            request_id,
            source: 0,
            flags: 0,
            payload_len: 0,
            _reserved: 0,
            timestamp_ms: 0,
        }
    }

    pub const fn with_source(mut self, source: ServiceId) -> Self {
        self.source = source as u8;
        self
    }

    pub const fn with_payload_len(mut self, len: u16) -> Self {
        self.payload_len = len;
        self
    }

    pub const fn with_timestamp(mut self, ts: u32) -> Self {
        self.timestamp_ms = ts;
        self
    }

    pub const FLAG_RESPONSE: u8 = 0x01;
    pub const FLAG_ERROR: u8 = 0x02;
    pub const FLAG_HAS_JSON: u8 = 0x04;

    pub const fn is_response(&self) -> bool {
        self.flags & Self::FLAG_RESPONSE != 0
    }

    pub const fn is_error(&self) -> bool {
        self.flags & Self::FLAG_ERROR != 0
    }

    /// Convert to bytes for IPC transmission.
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: CtrlMsg is repr(C), all fields are plain data, naturally aligned
        unsafe {
            core::slice::from_raw_parts(self as *const Self as *const u8, Self::HEADER_SIZE)
        }
    }

    /// Interpret a byte slice as CtrlMsg. Caller must ensure len >= HEADER_SIZE and alignment.
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

impl core::fmt::Debug for CtrlMsg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CtrlMsg")
            .field("topic", &self.topic)
            .field("method_id", &self.method_id)
            .field("request_id", &self.request_id)
            .field("source", &self.source)
            .field("flags", &self.flags)
            .field("payload_len", &self.payload_len)
            .field("timestamp_ms", &self.timestamp_ms)
            .finish()
    }
}
