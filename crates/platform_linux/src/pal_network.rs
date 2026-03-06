use core_types::{CamError, CommResult};
use core_interfaces::NetworkHal;

/// Linux network HAL stub. Real implementation wraps NetworkManager D-Bus or wpa_supplicant.
pub struct LinuxNetworkHal {
    connected: bool,
}

impl LinuxNetworkHal {
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl NetworkHal for LinuxNetworkHal {
    fn is_connected(&self) -> bool {
        self.connected
    }

    fn signal_strength(&self) -> i8 {
        if self.connected { -50 } else { -127 }
    }

    fn connect(&mut self, _ssid: &str, _password: &str) -> CommResult<()> {
        // Stub: real impl would invoke wpa_supplicant
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) -> CommResult<()> {
        self.connected = false;
        Ok(())
    }

    fn scan_wifi(&self, _results: &mut [u8]) -> CommResult<usize> {
        Ok(0)
    }

    fn get_ip_address(&self, buf: &mut [u8]) -> CommResult<usize> {
        if !self.connected {
            return Err(CamError::NetworkError);
        }
        let ip = b"0.0.0.0";
        let len = ip.len().min(buf.len());
        buf[..len].copy_from_slice(&ip[..len]);
        Ok(len)
    }
}
