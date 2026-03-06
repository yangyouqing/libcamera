#![no_std]
extern crate alloc;

use core_types::*;
use core_interfaces::{CommBus, Service};

const MAX_SESSIONS: usize = 8;

#[derive(Debug, Clone, Copy)]
struct CmdRoute {
    method_id: u16,
    target_topic: Topic,
    min_auth: AuthLevel,
}

const CMD_ROUTE_TABLE: &[CmdRoute] = &[
    CmdRoute { method_id: MethodId::StartLive as u16, target_topic: Topic::CmdLive, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::StopLive as u16, target_topic: Topic::CmdLive, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::StartTalk as u16, target_topic: Topic::CmdTalk, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::StopTalk as u16, target_topic: Topic::CmdTalk, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::SetTalkMode as u16, target_topic: Topic::CmdTalk, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::StartRecord as u16, target_topic: Topic::CmdRecord, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::StopRecord as u16, target_topic: Topic::CmdRecord, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::StartPlayback as u16, target_topic: Topic::CmdPlayback, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::StopPlayback as u16, target_topic: Topic::CmdPlayback, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::QueryTimeline as u16, target_topic: Topic::CmdPlayback, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::SeekPlayback as u16, target_topic: Topic::CmdPlayback, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::StartUpload as u16, target_topic: Topic::CmdCloud, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::StopUpload as u16, target_topic: Topic::CmdCloud, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::QueryUploadQueue as u16, target_topic: Topic::CmdCloud, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::CheckUpdate as u16, target_topic: Topic::CmdUpgrade, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::StartUpgrade as u16, target_topic: Topic::CmdUpgrade, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::QueryUpgradeStatus as u16, target_topic: Topic::CmdUpgrade, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::GetConfig as u16, target_topic: Topic::CmdConfig, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::SetConfig as u16, target_topic: Topic::CmdConfig, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::QueryCapacity as u16, target_topic: Topic::CmdStorage, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::FormatStorage as u16, target_topic: Topic::CmdStorage, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::ScanWifi as u16, target_topic: Topic::CmdNetwork, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::ConnectWifi as u16, target_topic: Topic::CmdNetwork, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::GetNetworkStatus as u16, target_topic: Topic::CmdNetwork, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::SyncNow as u16, target_topic: Topic::CmdTime, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::QueryTime as u16, target_topic: Topic::CmdTime, min_auth: AuthLevel::Viewer },
    CmdRoute { method_id: MethodId::SetBitrate as u16, target_topic: Topic::CmdMediaCore, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::RequestIdr as u16, target_topic: Topic::CmdMediaCore, min_auth: AuthLevel::Admin },
    CmdRoute { method_id: MethodId::SetResolution as u16, target_topic: Topic::CmdMediaCore, min_auth: AuthLevel::Admin },
];

const EVT_SUBSCRIBE_TABLE: &[Topic] = &[
    Topic::EvtConfigChanged,
    Topic::EvtNetworkStatus,
    Topic::EvtStorageStatus,
    Topic::EvtTimeSync,
    Topic::EvtAlarm,
    Topic::EvtUpgradeStatus,
];

#[derive(Debug, Clone, Copy)]
struct Session {
    session_id: u16,
    auth_level: AuthLevel,
    active: bool,
}

impl Session {
    const fn empty() -> Self {
        Self {
            session_id: 0,
            auth_level: AuthLevel::None,
            active: false,
        }
    }
}

pub struct ControlGateway {
    sessions: [Session; MAX_SESSIONS],
    session_count: u8,
    bus: Option<*const dyn CommBus>,
    service_state: ServiceState,
    network_connected: bool,
}

unsafe impl Send for ControlGateway {}
unsafe impl Sync for ControlGateway {}

impl ControlGateway {
    pub const fn new() -> Self {
        Self {
            sessions: [Session::empty(); MAX_SESSIONS],
            session_count: 0,
            bus: None,
            service_state: ServiceState::Normal,
            network_connected: true,
        }
    }

    fn bus(&self) -> &dyn CommBus {
        unsafe { &*self.bus.unwrap() }
    }

    pub fn session_count(&self) -> u8 {
        self.session_count
    }

    fn verify_password_stub(_hash: &[u8], _input: &[u8]) -> bool {
        // sha2 password hash verification stub
        true
    }

    pub fn create_session(&mut self, session_id: u16, auth_level: AuthLevel) -> CommResult<()> {
        for s in self.sessions.iter_mut() {
            if !s.active {
                *s = Session {
                    session_id,
                    auth_level,
                    active: true,
                };
                self.session_count += 1;
                self.publish_session_event(session_id, true);
                return Ok(());
            }
        }
        Err(CamError::ResourceExhausted)
    }

    pub fn remove_session(&mut self, session_id: u16) {
        for s in self.sessions.iter_mut() {
            if s.active && s.session_id == session_id {
                s.active = false;
                self.session_count = self.session_count.saturating_sub(1);
                self.publish_session_event(session_id, false);
                return;
            }
        }
    }

    fn get_session_auth(&self, session_id: u16) -> Option<AuthLevel> {
        for s in self.sessions.iter() {
            if s.active && s.session_id == session_id {
                return Some(s.auth_level);
            }
        }
        None
    }

    fn publish_session_event(&self, session_id: u16, connected: bool) {
        if let Some(bus_ptr) = self.bus {
            let bus = unsafe { &*bus_ptr };
            let msg = CtrlMsg::new(
                Topic::EvtSessionStatus,
                session_id,
                if connected { 1 } else { 0 },
            )
            .with_source(ServiceId::ControlGateway);
            let _ = bus.publish_ctrl(Topic::EvtSessionStatus, &msg, &[]);
        }
    }

    fn find_route(method_id: u16) -> Option<&'static CmdRoute> {
        for route in CMD_ROUTE_TABLE {
            if route.method_id == method_id {
                return Some(route);
            }
        }
        None
    }

    fn handle_cmd(&mut self, msg: &CtrlMsg, payload: &[u8]) {
        let session_auth = self.get_session_auth(msg.request_id);

        if let Some(route) = Self::find_route(msg.method_id) {
            let auth = session_auth.unwrap_or(AuthLevel::None);
            if auth < route.min_auth {
                let resp = CtrlMsg::new(Topic::CmdControl, msg.method_id, msg.request_id)
                    .with_source(ServiceId::ControlGateway);
                let mut err_resp = resp;
                err_resp.flags |= CtrlMsg::FLAG_ERROR;
                let _ = self.bus().reply(Topic::CmdControl, msg.request_id, &err_resp, &[]);
                return;
            }

            let fwd = CtrlMsg::new(route.target_topic, msg.method_id, msg.request_id)
                .with_source(ServiceId::ControlGateway)
                .with_payload_len(msg.payload_len);
            let _ = self.bus().publish_ctrl(route.target_topic, &fwd, payload);
        } else {
            let resp = CtrlMsg::new(Topic::CmdControl, msg.method_id, msg.request_id)
                .with_source(ServiceId::ControlGateway);
            let _ = self.bus().reply(Topic::CmdControl, msg.request_id, &resp, &[]);
        }
    }
}

impl Service for ControlGateway {
    fn service_id(&self) -> ServiceId {
        ServiceId::ControlGateway
    }

    fn dependencies(&self) -> &'static [ServiceId] {
        &[
            ServiceId::Config,
            ServiceId::Storage,
            ServiceId::Network,
            ServiceId::TimeSync,
            ServiceId::MediaCore,
        ]
    }

    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()> {
        self.bus = Some(unsafe { core::mem::transmute(bus) });
        bus.subscribe(Topic::CmdControl)?;
        for &evt_topic in EVT_SUBSCRIBE_TABLE {
            bus.subscribe(evt_topic)?;
        }
        Ok(())
    }

    fn start(&mut self) -> CommResult<()> {
        self.service_state = ServiceState::Normal;
        Ok(())
    }

    fn stop(&mut self) -> CommResult<()> {
        self.session_count = 0;
        for s in self.sessions.iter_mut() {
            s.active = false;
        }
        self.service_state = ServiceState::Suspended;
        Ok(())
    }

    fn health(&self) -> HealthStatus {
        HealthStatus {
            service: ServiceId::ControlGateway,
            state: self.service_state,
            error_code: 0,
        }
    }

    fn poll(&mut self) -> CommResult<bool> {
        let mut buf = [0u8; 256];
        if let Some((topic, msg)) = self.bus().poll_ctrl(&mut buf)? {
            if topic == Topic::CmdControl && !msg.is_response() {
                let payload_len = msg.payload_len as usize;
                self.handle_cmd(&msg, &buf[..payload_len.min(buf.len())]);
                return Ok(true);
            }
            if topic == Topic::EvtNetworkStatus {
                self.network_connected = msg.method_id != 0;
                if !self.network_connected {
                    self.service_state = ServiceState::Degraded;
                } else if self.service_state == ServiceState::Degraded {
                    self.service_state = ServiceState::Normal;
                }
                return Ok(true);
            }
            // Forward subscribed events to connected sessions (push to App)
            for &evt_topic in EVT_SUBSCRIBE_TABLE {
                if topic == evt_topic {
                    // TODO: serialize event and push to each active session's transport
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
