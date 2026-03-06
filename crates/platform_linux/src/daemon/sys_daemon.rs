use crate::shm_ring::ShmRingBuf;
use crate::uds_router::UdsTopicRouter;
use std::collections::HashMap;
use core_types::Topic;

const SHM_SLOT_SIZE: usize = 256 * 1024;
const SHM_SLOT_COUNT: usize = 16;

/// sys-daemon: minimal core process.
/// Responsibilities: shm lifecycle, UDS TopicRouter, watchdog, CmdDevice.
pub struct SysDaemon {
    shm_buffers: HashMap<u8, ShmRingBuf>, // topic_id -> shm ring
    topic_router: Option<UdsTopicRouter>,
    running: bool,
}

impl SysDaemon {
    pub fn new() -> Self {
        Self {
            shm_buffers: HashMap::new(),
            topic_router: None,
            running: false,
        }
    }

    /// Create shared memory rings for all data plane topics.
    pub fn init_shm(&mut self) -> std::io::Result<()> {
        let data_topics = [
            Topic::VideoMainStream,
            Topic::VideoSubStream,
            Topic::AudioCapture,
            Topic::TalkDownlink,
            Topic::TalkUplink,
            Topic::PlaybackStream,
        ];
        for topic in data_topics {
            let ring = ShmRingBuf::create(SHM_SLOT_SIZE, SHM_SLOT_COUNT)?;
            self.shm_buffers.insert(topic as u8, ring);
        }
        Ok(())
    }

    /// Initialize the UDS topic router.
    pub fn init_uds_router(&mut self, socket_path: &str) -> std::io::Result<()> {
        let router = UdsTopicRouter::new(socket_path)?;
        self.topic_router = Some(router);
        Ok(())
    }

    /// Get the fd for a data plane topic's shm buffer (for SCM_RIGHTS passing).
    pub fn get_shm_fd(&self, topic: Topic) -> Option<i32> {
        self.shm_buffers.get(&(topic as u8)).map(|r| r.fd())
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Main loop tick: accept UDS connections, route incoming messages.
    pub fn tick(&mut self) {
        if let Some(router) = &mut self.topic_router {
            router.accept_connections();
            let mut buf = [0u8; 4096];
            while let Some((_client_idx, topic, n)) = router.recv_from_any(&mut buf) {
                router.route(topic, &buf[..n]);
            }
        }
    }
}
