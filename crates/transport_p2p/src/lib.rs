#![no_std]
extern crate alloc;

use core_types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum P2pState {
    Disconnected = 0,
    Connecting = 1,
    Connected = 2,
}

pub trait P2pChannel {
    fn connect(&mut self, peer_id: &[u8]) -> CommResult<()>;
    fn send(&self, data: &[u8]) -> CommResult<usize>;
    fn recv(&self, buf: &mut [u8]) -> CommResult<usize>;
    fn disconnect(&mut self) -> CommResult<()>;
    fn state(&self) -> P2pState;
}

pub struct P2pChannelStub {
    state: P2pState,
}

impl P2pChannelStub {
    pub const fn new() -> Self {
        Self {
            state: P2pState::Disconnected,
        }
    }
}

impl P2pChannel for P2pChannelStub {
    fn connect(&mut self, _peer_id: &[u8]) -> CommResult<()> {
        self.state = P2pState::Connected;
        Ok(())
    }

    fn send(&self, data: &[u8]) -> CommResult<usize> {
        if self.state != P2pState::Connected {
            return Err(CamError::NotReady);
        }
        Ok(data.len())
    }

    fn recv(&self, _buf: &mut [u8]) -> CommResult<usize> {
        if self.state != P2pState::Connected {
            return Err(CamError::NotReady);
        }
        Ok(0)
    }

    fn disconnect(&mut self) -> CommResult<()> {
        self.state = P2pState::Disconnected;
        Ok(())
    }

    fn state(&self) -> P2pState {
        self.state
    }
}
