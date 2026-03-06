//! RTOS Platform Abstraction Layer (stub).
//! Placeholder implementations that return errors. Will be filled in when
//! targeting real RTOS hardware.

#![no_std]

extern crate alloc;

use core_types::{CamError, CommResult};
use core_interfaces::{
    BootManager, FileSystem, HttpClient, NetworkHal, PtzHal, StorageHal, SystemClock,
    SystemControl, Timer, UdpSocket,
};

// ── Stub FileSystem ──
pub struct RtosFileSystem;

impl RtosFileSystem {
    pub const fn new() -> Self {
        Self
    }
}

impl FileSystem for RtosFileSystem {
    fn read_file(&self, _path: &str, _buf: &mut [u8]) -> CommResult<usize> {
        Err(CamError::Unsupported)
    }
    fn write_file(&self, _path: &str, _data: &[u8]) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn file_exists(&self, _path: &str) -> bool {
        false
    }
    fn remove_file(&self, _path: &str) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn file_size(&self, _path: &str) -> CommResult<u64> {
        Err(CamError::Unsupported)
    }
    fn list_dir(&self, _path: &str, _entries: &mut [u8], _max_entries: usize) -> CommResult<usize> {
        Err(CamError::Unsupported)
    }
    fn create_dir(&self, _path: &str) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn free_space(&self, _path: &str) -> CommResult<u64> {
        Err(CamError::Unsupported)
    }
    fn total_space(&self, _path: &str) -> CommResult<u64> {
        Err(CamError::Unsupported)
    }
}

// ── Stub Timer ──
pub struct RtosTimer;

impl RtosTimer {
    pub const fn new() -> Self {
        Self
    }
}

impl Timer for RtosTimer {
    fn monotonic_ms(&self) -> u64 {
        0
    }
    fn sleep_ms(&self, _ms: u32) {
        // No-op stub
    }
}

// ── Stub NetworkHal ──
pub struct RtosNetworkHal;

impl RtosNetworkHal {
    pub const fn new() -> Self {
        Self
    }
}

impl NetworkHal for RtosNetworkHal {
    fn is_connected(&self) -> bool {
        false
    }
    fn signal_strength(&self) -> i8 {
        0
    }
    fn connect(&mut self, _ssid: &str, _password: &str) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn disconnect(&mut self) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn scan_wifi(&self, _results: &mut [u8]) -> CommResult<usize> {
        Err(CamError::Unsupported)
    }
    fn get_ip_address(&self, _buf: &mut [u8]) -> CommResult<usize> {
        Err(CamError::Unsupported)
    }
}

// ── Stub StorageHal ──
pub struct RtosStorageHal;

impl RtosStorageHal {
    pub const fn new() -> Self {
        Self
    }
}

impl StorageHal for RtosStorageHal {
    fn is_card_inserted(&self) -> bool {
        false
    }
    fn mount(&mut self) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn unmount(&mut self) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn format(&mut self) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn capacity_bytes(&self) -> CommResult<u64> {
        Err(CamError::Unsupported)
    }
    fn used_bytes(&self) -> CommResult<u64> {
        Err(CamError::Unsupported)
    }
}

// ── Stub SystemClock ──
pub struct RtosSystemClock;

impl RtosSystemClock {
    pub const fn new() -> Self {
        Self
    }
}

impl SystemClock for RtosSystemClock {
    fn now_ms(&self) -> u64 {
        0
    }
    fn set_time_ms(&mut self, _epoch_ms: u64) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn monotonic_ms(&self) -> u64 {
        0
    }
}

// ── Stub UdpSocket ──
pub struct RtosUdpSocket;

impl RtosUdpSocket {
    pub const fn new() -> Self {
        Self
    }
}

impl UdpSocket for RtosUdpSocket {
    fn send_to(&mut self, _addr: &str, _port: u16, _data: &[u8]) -> CommResult<usize> {
        Err(CamError::Unsupported)
    }
    fn recv_from(&mut self, _buf: &mut [u8], _timeout_ms: u32) -> CommResult<usize> {
        Err(CamError::Unsupported)
    }
    fn bind(&mut self, _port: u16) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
}

// ── Stub HttpClient ──
pub struct RtosHttpClient;

impl RtosHttpClient {
    pub const fn new() -> Self {
        Self
    }
}

impl HttpClient for RtosHttpClient {
    fn get(
        &mut self,
        _url: &str,
        _headers: &[(&str, &str)],
        _response_buf: &mut [u8],
    ) -> CommResult<usize> {
        Err(CamError::Unsupported)
    }
    fn put(
        &mut self,
        _url: &str,
        _headers: &[(&str, &str)],
        _body: &[u8],
        _response_buf: &mut [u8],
    ) -> CommResult<usize> {
        Err(CamError::Unsupported)
    }
    fn post(
        &mut self,
        _url: &str,
        _headers: &[(&str, &str)],
        _body: &[u8],
        _response_buf: &mut [u8],
    ) -> CommResult<usize> {
        Err(CamError::Unsupported)
    }
    fn status_code(&self) -> u16 {
        0
    }
}

// ── Stub BootManager ──
pub struct RtosBootManager;

impl RtosBootManager {
    pub const fn new() -> Self {
        Self
    }
}

impl BootManager for RtosBootManager {
    fn current_slot(&self) -> u8 {
        0
    }
    fn set_next_boot_slot(&mut self, _slot: u8) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn mark_boot_successful(&mut self) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn rollback(&mut self) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
}

// ── Stub SystemControl ──
pub struct RtosSystemControl;

impl RtosSystemControl {
    pub const fn new() -> Self {
        Self
    }
}

impl SystemControl for RtosSystemControl {
    fn reboot(&mut self) -> ! {
        loop {}
    }
    fn factory_reset(&mut self) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn get_device_info(&self, _buf: &mut [u8]) -> CommResult<usize> {
        Err(CamError::Unsupported)
    }
}

// ── Stub PtzHal ──
pub struct RtosPtzHal;

impl RtosPtzHal {
    pub const fn new() -> Self {
        Self
    }
}

impl PtzHal for RtosPtzHal {
    fn move_to(&mut self, _pan: i16, _tilt: i16, _speed: u8) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn zoom(&mut self, _level: u8) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
    fn get_position(&self) -> CommResult<(i16, i16)> {
        Err(CamError::Unsupported)
    }
    fn stop(&mut self) -> CommResult<()> {
        Err(CamError::Unsupported)
    }
}
