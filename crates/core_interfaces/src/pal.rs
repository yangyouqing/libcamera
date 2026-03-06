use core_types::CommResult;

/// Platform Abstraction Layer: File system operations.
pub trait FileSystem {
    fn read_file(&self, path: &str, buf: &mut [u8]) -> CommResult<usize>;
    fn write_file(&self, path: &str, data: &[u8]) -> CommResult<()>;
    fn file_exists(&self, path: &str) -> bool;
    fn remove_file(&self, path: &str) -> CommResult<()>;
    fn file_size(&self, path: &str) -> CommResult<u64>;
    fn list_dir(&self, path: &str, entries: &mut [u8], max_entries: usize) -> CommResult<usize>;
    fn create_dir(&self, path: &str) -> CommResult<()>;
    fn free_space(&self, path: &str) -> CommResult<u64>;
    fn total_space(&self, path: &str) -> CommResult<u64>;
}

/// Platform Abstraction Layer: Network HAL.
pub trait NetworkHal {
    fn is_connected(&self) -> bool;
    fn signal_strength(&self) -> i8;
    fn connect(&mut self, ssid: &str, password: &str) -> CommResult<()>;
    fn disconnect(&mut self) -> CommResult<()>;
    fn scan_wifi(&self, results: &mut [u8]) -> CommResult<usize>;
    fn get_ip_address(&self, buf: &mut [u8]) -> CommResult<usize>;
}

/// Platform Abstraction Layer: Storage detection HAL.
pub trait StorageHal {
    fn is_card_inserted(&self) -> bool;
    fn mount(&mut self) -> CommResult<()>;
    fn unmount(&mut self) -> CommResult<()>;
    fn format(&mut self) -> CommResult<()>;
    fn capacity_bytes(&self) -> CommResult<u64>;
    fn used_bytes(&self) -> CommResult<u64>;
}

/// Platform Abstraction Layer: System clock.
pub trait SystemClock {
    fn now_ms(&self) -> u64;
    fn set_time_ms(&mut self, epoch_ms: u64) -> CommResult<()>;
    fn monotonic_ms(&self) -> u64;
}

/// Platform Abstraction Layer: UDP socket for NTP.
pub trait UdpSocket {
    fn send_to(&mut self, addr: &str, port: u16, data: &[u8]) -> CommResult<usize>;
    fn recv_from(&mut self, buf: &mut [u8], timeout_ms: u32) -> CommResult<usize>;
    fn bind(&mut self, port: u16) -> CommResult<()>;
}

/// Platform Abstraction Layer: HTTP client for cloud/upgrade.
pub trait HttpClient {
    fn get(&mut self, url: &str, headers: &[(&str, &str)], response_buf: &mut [u8])
        -> CommResult<usize>;
    fn put(&mut self, url: &str, headers: &[(&str, &str)], body: &[u8], response_buf: &mut [u8])
        -> CommResult<usize>;
    fn post(&mut self, url: &str, headers: &[(&str, &str)], body: &[u8], response_buf: &mut [u8])
        -> CommResult<usize>;
    fn status_code(&self) -> u16;
}

/// Platform Abstraction Layer: Timer.
pub trait Timer {
    fn monotonic_ms(&self) -> u64;
    fn sleep_ms(&self, ms: u32);
}

/// Platform Abstraction Layer: Boot manager for A/B upgrade.
pub trait BootManager {
    fn current_slot(&self) -> u8;
    fn set_next_boot_slot(&mut self, slot: u8) -> CommResult<()>;
    fn mark_boot_successful(&mut self) -> CommResult<()>;
    fn rollback(&mut self) -> CommResult<()>;
}

/// Platform Abstraction Layer: System control (reboot, factory reset).
pub trait SystemControl {
    fn reboot(&mut self) -> !;
    fn factory_reset(&mut self) -> CommResult<()>;
    fn get_device_info(&self, buf: &mut [u8]) -> CommResult<usize>;
}

/// Platform Abstraction Layer: PTZ (pan/tilt/zoom) control.
pub trait PtzHal {
    fn move_to(&mut self, pan: i16, tilt: i16, speed: u8) -> CommResult<()>;
    fn zoom(&mut self, level: u8) -> CommResult<()>;
    fn get_position(&self) -> CommResult<(i16, i16)>;
    fn stop(&mut self) -> CommResult<()>;
}
