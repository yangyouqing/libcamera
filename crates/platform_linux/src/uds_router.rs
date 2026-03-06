use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::io::{AsRawFd, RawFd};
use std::collections::HashMap;
use core_types::{CtrlMsg, Topic};

const MAX_MSG_SIZE: usize = 4096;

/// UDS-based TopicRouter for Linux control/event plane.
/// sys-daemon runs this as the central router; other daemons connect as clients.
pub struct UdsTopicRouter {
    listener: UnixListener,
    clients: Vec<UdsClient>,
    subscriptions: HashMap<u8, Vec<usize>>, // topic_id -> [client_indices]
}

struct UdsClient {
    stream: UnixStream,
    id: usize,
}

impl UdsTopicRouter {
    /// Create a new UDS topic router bound to the given socket path.
    pub fn new(socket_path: &str) -> io::Result<Self> {
        let _ = std::fs::remove_file(socket_path);
        let listener = UnixListener::bind(socket_path)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            clients: Vec::new(),
            subscriptions: HashMap::new(),
        })
    }

    /// Accept pending connections (non-blocking).
    pub fn accept_connections(&mut self) -> usize {
        let mut count = 0;
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let _ = stream.set_nonblocking(true);
                    let id = self.clients.len();
                    self.clients.push(UdsClient { stream, id });
                    count += 1;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
        count
    }

    /// Register a client subscription to a topic.
    pub fn subscribe_client(&mut self, client_idx: usize, topic: Topic) {
        let topic_id = topic as u8;
        let subs = self.subscriptions.entry(topic_id).or_default();
        if !subs.contains(&client_idx) {
            subs.push(client_idx);
        }
    }

    /// Route a message to all subscribers of a topic.
    pub fn route(&mut self, topic: Topic, data: &[u8]) -> usize {
        let topic_id = topic as u8;
        let Some(subs) = self.subscriptions.get(&topic_id) else {
            return 0;
        };
        let mut count = 0;
        let sub_indices: Vec<usize> = subs.clone();
        for &idx in &sub_indices {
            if idx < self.clients.len() {
                // Write length-prefixed message
                let len = data.len() as u32;
                let len_bytes = len.to_le_bytes();
                if self.clients[idx].stream.write_all(&len_bytes).is_ok()
                    && self.clients[idx].stream.write_all(data).is_ok()
                {
                    count += 1;
                }
            }
        }
        count
    }

    /// Try to receive one message from any connected client (non-blocking).
    /// Returns (client_index, topic_id, bytes_read) or None if no data available.
    /// Protocol: client sends [1 byte topic_id][4 bytes len LE][len bytes data].
    pub fn recv_from_any(&mut self, buf: &mut [u8]) -> Option<(usize, Topic, usize)> {
        for client in self.clients.iter_mut() {
            let mut topic_byte = [0u8; 1];
            match client.stream.read_exact(&mut topic_byte) {
                Ok(()) => {}
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(_) => continue,
            }
            let mut len_buf = [0u8; 4];
            if client.stream.read_exact(&mut len_buf).is_err() {
                continue;
            }
            let len = u32::from_le_bytes(len_buf) as usize;
            let read_len = len.min(buf.len());
            if client.stream.read_exact(&mut buf[..read_len]).is_err() {
                continue;
            }
            let topic: Topic = unsafe { core::mem::transmute(topic_byte[0]) };
            return Some((client.id, topic, read_len));
        }
        None
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

/// UDS client for connecting to the TopicRouter.
pub struct UdsClient2 {
    stream: UnixStream,
}

impl UdsClient2 {
    pub fn connect(socket_path: &str) -> io::Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        Ok(Self { stream })
    }

    pub fn send(&mut self, data: &[u8]) -> io::Result<()> {
        let len = data.len() as u32;
        self.stream.write_all(&len.to_le_bytes())?;
        self.stream.write_all(data)?;
        Ok(())
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let read_len = len.min(buf.len());
        self.stream.read_exact(&mut buf[..read_len])?;
        Ok(read_len)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.stream.set_nonblocking(nonblocking)
    }

    pub fn as_raw_fd(&self) -> RawFd {
        self.stream.as_raw_fd()
    }
}

/// Send an fd over UDS using SCM_RIGHTS.
pub fn send_fd(stream: &UnixStream, fd: RawFd) -> io::Result<()> {
    use std::os::unix::io::FromRawFd;
    let buf = [0u8; 1];
    let iov = libc::iovec {
        iov_base: buf.as_ptr() as *mut libc::c_void,
        iov_len: 1,
    };

    let mut cmsg_buf = [0u8; 64];
    let cmsg_len = unsafe {
        libc::CMSG_SPACE(core::mem::size_of::<RawFd>() as u32) as usize
    };

    let mut msg: libc::msghdr = unsafe { core::mem::zeroed() };
    msg.msg_iov = &iov as *const _ as *mut _;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
    msg.msg_controllen = cmsg_len;

    unsafe {
        let cmsg = libc::CMSG_FIRSTHDR(&msg);
        (*cmsg).cmsg_level = libc::SOL_SOCKET;
        (*cmsg).cmsg_type = libc::SCM_RIGHTS;
        (*cmsg).cmsg_len = libc::CMSG_LEN(core::mem::size_of::<RawFd>() as u32) as usize;
        core::ptr::copy_nonoverlapping(
            &fd as *const RawFd as *const u8,
            libc::CMSG_DATA(cmsg),
            core::mem::size_of::<RawFd>(),
        );
    }

    let ret = unsafe { libc::sendmsg(stream.as_raw_fd(), &msg, 0) };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Receive an fd over UDS using SCM_RIGHTS.
pub fn recv_fd(stream: &UnixStream) -> io::Result<RawFd> {
    let mut buf = [0u8; 1];
    let mut iov = libc::iovec {
        iov_base: buf.as_mut_ptr() as *mut libc::c_void,
        iov_len: 1,
    };

    let mut cmsg_buf = [0u8; 64];
    let cmsg_len = unsafe {
        libc::CMSG_SPACE(core::mem::size_of::<RawFd>() as u32) as usize
    };

    let mut msg: libc::msghdr = unsafe { core::mem::zeroed() };
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
    msg.msg_controllen = cmsg_len;

    let ret = unsafe { libc::recvmsg(stream.as_raw_fd(), &mut msg, 0) };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    unsafe {
        let cmsg = libc::CMSG_FIRSTHDR(&msg);
        if cmsg.is_null() {
            return Err(io::Error::new(io::ErrorKind::Other, "no cmsg"));
        }
        if (*cmsg).cmsg_level != libc::SOL_SOCKET || (*cmsg).cmsg_type != libc::SCM_RIGHTS {
            return Err(io::Error::new(io::ErrorKind::Other, "wrong cmsg type"));
        }
        let mut fd: RawFd = 0;
        core::ptr::copy_nonoverlapping(
            libc::CMSG_DATA(cmsg),
            &mut fd as *mut RawFd as *mut u8,
            core::mem::size_of::<RawFd>(),
        );
        Ok(fd)
    }
}
