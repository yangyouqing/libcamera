#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, FileSystem, Service};

const MAX_CONFIG_ENTRIES: usize = 64;

#[derive(Clone)]
struct ConfigEntry {
    key: FixedString<32>,
    value: FixedString<128>,
}

#[allow(dead_code)]
enum ConfigLayer {
    Factory,
    User,
    Cloud,
}

pub struct ConfigService {
    factory: FixedVec<ConfigEntry, MAX_CONFIG_ENTRIES>,
    user: FixedVec<ConfigEntry, MAX_CONFIG_ENTRIES>,
    cloud: FixedVec<ConfigEntry, MAX_CONFIG_ENTRIES>,
    bus: Option<*const dyn CommBus>,
    fs: Option<*const dyn FileSystem>,
    state: ServiceState,
}

// SAFETY: bus and fs pointers are set once at init, used read-only afterward
unsafe impl Send for ConfigService {}
unsafe impl Sync for ConfigService {}

impl ConfigService {
    pub const fn new() -> Self {
        Self {
            factory: FixedVec::new(),
            user: FixedVec::new(),
            cloud: FixedVec::new(),
            bus: None,
            fs: None,
            state: ServiceState::Normal,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    /// Set the FileSystem for persistence. Call after new(), before init.
    pub fn set_filesystem(&mut self, fs: &dyn FileSystem) {
        // SAFETY: Caller must ensure fs outlives this ConfigService
        self.fs = Some(unsafe { core::mem::transmute(fs) });
    }

    /// Get a config value. Priority: cloud > user > factory.
    pub fn get(&self, key: &str) -> Option<&str> {
        for entry in self.cloud.as_slice() {
            if entry.key.as_str() == key {
                return Some(entry.value.as_str());
            }
        }
        for entry in self.user.as_slice() {
            if entry.key.as_str() == key {
                return Some(entry.value.as_str());
            }
        }
        for entry in self.factory.as_slice() {
            if entry.key.as_str() == key {
                return Some(entry.value.as_str());
            }
        }
        None
    }

    /// Set a config value in the user layer.
    pub fn set(&mut self, key: &str, value: &str) -> CommResult<()> {
        for entry in self.user.as_mut_slice() {
            if entry.key.as_str() == key {
                entry.value = FixedString::from_str(value);
                self.publish_change(key);
                return Ok(());
            }
        }
        let entry = ConfigEntry {
            key: FixedString::from_str(key),
            value: FixedString::from_str(value),
        };
        self.user.push(entry).map_err(|_| CamError::ResourceExhausted)?;
        self.publish_change(key);
        Ok(())
    }

    /// Set a cloud-override value. Priority: cloud > user > factory.
    pub fn set_cloud(&mut self, key: &str, value: &str) -> CommResult<()> {
        for entry in self.cloud.as_mut_slice() {
            if entry.key.as_str() == key {
                entry.value = FixedString::from_str(value);
                return Ok(());
            }
        }
        let entry = ConfigEntry {
            key: FixedString::from_str(key),
            value: FixedString::from_str(value),
        };
        self.cloud.push(entry).map_err(|_| CamError::ResourceExhausted)
    }

    /// Set a factory default.
    pub fn set_factory(&mut self, key: &str, value: &str) -> CommResult<()> {
        let entry = ConfigEntry {
            key: FixedString::from_str(key),
            value: FixedString::from_str(value),
        };
        self.factory
            .push(entry)
            .map_err(|_| CamError::ResourceExhausted)
    }

    fn publish_change(&self, _key: &str) {
        if let Some(bus_ptr) = self.bus {
            let bus = unsafe { &*bus_ptr };
            let msg = CtrlMsg::new(Topic::EvtConfigChanged, 0, 0)
                .with_source(ServiceId::Config);
            let _ = bus.publish_ctrl(Topic::EvtConfigChanged, &msg, &[]);
        }
    }

    /// Persist user config to filesystem (if available).
    fn persist_user_config(&self) {
        let fs = match self.fs {
            Some(ptr) => unsafe { &*ptr },
            None => return,
        };
        let mut buf = [0u8; 4096];
        let mut pos = 0;
        for entry in self.user.as_slice() {
            let k = entry.key.as_str().as_bytes();
            let v = entry.value.as_str().as_bytes();
            let line_len = k.len() + 1 + v.len() + 1; // "key=value\n"
            if pos + line_len > buf.len() {
                break;
            }
            buf[pos..pos + k.len()].copy_from_slice(k);
            pos += k.len();
            buf[pos] = b'=';
            pos += 1;
            buf[pos..pos + v.len()].copy_from_slice(v);
            pos += v.len();
            buf[pos] = b'\n';
            pos += 1;
        }
        let _ = fs.write_file("/etc/cam/user.conf", &buf[..pos]);
    }

    /// Load user config from filesystem (if available).
    fn load_user_config(&mut self) {
        let fs = match self.fs {
            Some(ptr) => unsafe { &*ptr },
            None => return,
        };
        let mut buf = [0u8; 4096];
        let n = match fs.read_file("/etc/cam/user.conf", &mut buf) {
            Ok(n) => n,
            Err(_) => return,
        };
        let data = &buf[..n];
        for line in data.split(|&b| b == b'\n') {
            if line.is_empty() {
                continue;
            }
            if let Some(eq_pos) = line.iter().position(|&b| b == b'=') {
                if let (Ok(k), Ok(v)) = (
                    core::str::from_utf8(&line[..eq_pos]),
                    core::str::from_utf8(&line[eq_pos + 1..]),
                ) {
                    let entry = ConfigEntry {
                        key: FixedString::from_str(k),
                        value: FixedString::from_str(v),
                    };
                    let _ = self.user.push(entry);
                }
            }
        }
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg, payload: &[u8]) {
        match msg.method_id {
            x if x == MethodId::GetConfig as u16 => {
                // payload contains key as UTF-8 bytes
                let key = core::str::from_utf8(payload).unwrap_or("");
                let value = self.get(key).unwrap_or("");
                let resp = CtrlMsg::new(Topic::CmdConfig, msg.method_id, msg.request_id)
                    .with_source(ServiceId::Config)
                    .with_payload_len(value.len() as u16);
                let _ = self.bus().reply(Topic::CmdConfig, msg.request_id, &resp, value.as_bytes());
            }
            x if x == MethodId::SetConfig as u16 => {
                // payload: "key=value"
                let data = core::str::from_utf8(payload).unwrap_or("");
                if let Some(eq_pos) = data.find('=') {
                    let k = &data[..eq_pos];
                    let v = &data[eq_pos + 1..];
                    let _ = self.set(k, v);
                    self.persist_user_config();
                }
                let resp = CtrlMsg::new(Topic::CmdConfig, msg.method_id, msg.request_id)
                    .with_source(ServiceId::Config);
                let _ = self.bus().reply(Topic::CmdConfig, msg.request_id, &resp, &[]);
            }
            _ => {}
        }
    }
}

impl Service for ConfigService {
    fn service_id(&self) -> ServiceId {
        ServiceId::Config
    }
    fn dependencies(&self) -> &'static [ServiceId] {
        &[]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        // SAFETY: Caller must ensure bus outlives this ConfigService
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdConfig)?;
        Ok(())
    }

    fn start(&mut self) -> CommResult<()> {
        self.load_user_config();
        self.state = ServiceState::Normal;
        Ok(())
    }

    fn stop(&mut self) -> CommResult<()> {
        self.state = ServiceState::Suspended;
        Ok(())
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            service: ServiceId::Config,
            state: self.state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((_topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            if msg.topic == Topic::CmdConfig as u8 && !msg.is_response() {
                let payload_len = msg.payload_len as usize;
                self.handle_cmd(&msg, &buf[..payload_len.min(buf.len())]);
                return Ok(true);
            }
        }
        Ok(false)
    }
}
