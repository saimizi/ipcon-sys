use crate::ipcon::{Ipcon, IpconFlag};
use crate::ipcon_error::IpconError;
use crate::ipcon_msg::IpconMsg;
use error_stack::{Context, IntoReport, Result, ResultExt};
use tokio::io::unix::AsyncFd;

#[link(name = "ipcon")]
extern "C" {}

/// Async version of IPCON peer.
pub struct AsyncIpcon {
    ih: Ipcon,
}

impl AsyncIpcon {
    /// Create an async IPCON peer.
    /// If the name is omitted, an anonymous will be created.
    /// Following flags can be specified with bitwise OR (|).
    /// * IPF_DISABLE_KEVENT_FILTER  
    ///   By default, IPCON kernel module will only delivery the add/remove notification of
    ///   peers and groups which are considered to be interested by the peer. If this flag is
    ///   enabled, all notification will be delivered by IPCON kernel module.
    /// * IPF_SND_IF  
    ///   Use message sending interface.
    /// * IPF_RCV_IF  
    ///   Use message receiving interface.
    /// * IPF_DEFAULT  
    ///   This is same to IPF_RCV_IF | IPF_SND_IF.
    ///
    ///   
    pub fn new(peer_name: Option<&str>, flag: Option<IpconFlag>) -> Result<AsyncIpcon, IpconError> {
        let ih = Ipcon::new(peer_name, flag)?;
        Ok(AsyncIpcon { ih })
    }

    /// Inquiry whether a peer is present.
    pub async fn is_peer_present(&self, peer: &str) -> bool {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        loop {
            let mut guide = async_ctrl.writable().await.unwrap();
            match guide.try_io(|_inner| Ok(self.ih.is_peer_present(peer))) {
                Ok(ret) => {
                    return ret.unwrap();
                }
                Err(_would_block) => {}
            }
        }
    }

    /// Inquiry whether the group of a peer is present.
    pub async fn is_group_present(&self, peer: &str, group: &str) -> bool {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        loop {
            let mut guide = async_ctrl.writable().await.unwrap();
            match guide.try_io(|_inner| Ok(self.ih.is_group_present(peer, group))) {
                Ok(ret) => return ret.unwrap(),
                Err(_would_block) => {}
            }
        }
    }

    /// Receive IPCON message.
    /// This function will fail if the peer doesn't enable IPF_RCV_IF.
    pub async fn receive_msg(&self) -> Result<IpconMsg, IpconError> {
        let async_ctrl = AsyncFd::new(self.ih.get_read_fd().unwrap()).unwrap();

        loop {
            let mut guide = async_ctrl.readable().await.unwrap();
            match guide.try_io(|_inner| Ok(self.ih.receive_msg())) {
                Ok(ret) => return ret.unwrap().attach_printable("Async receive_msg() failed."),
                Err(_would_block) => {}
            }
        }
    }

    /// Send an unicast IPCON message to a specific peer.
    /// This function will fail if the peer doesn't enable IPF_SND_IF.
    pub async fn send_unicast_msg(&self, peer: &str, buf: &[u8]) -> Result<(), IpconError> {
        let async_ctrl = AsyncFd::new(self.ih.get_write_fd().unwrap()).unwrap();

        loop {
            let mut guide = async_ctrl.writable().await.unwrap();
            match guide.try_io(|_inner| Ok(self.ih.send_unicast_msg_by_ref(peer, &buf))) {
                Ok(ret) => {
                    return ret
                        .unwrap()
                        .attach_printable("Async send_unicast_msg() failed.")
                }
                Err(_would_block) => {}
            }
        }
    }

    /// Register a multicast group.
    pub async fn register_group(&self, group: &str) -> Result<(), IpconError> {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        loop {
            let mut guide = async_ctrl.writable().await.unwrap();
            match guide.try_io(|_inner| Ok(self.ih.register_group(group))) {
                Ok(ret) => {
                    return ret
                        .unwrap()
                        .attach_printable("Async register_group() failed.")
                }
                Err(_would_block) => {}
            }
        }
    }

    /// Unregister a multicast group.
    pub async fn unregister_group(&self, group: &str) -> Result<(), IpconError> {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        loop {
            let mut guide = async_ctrl.writable().await.unwrap();
            match guide.try_io(|_inner| Ok(self.ih.unregister_group(group))) {
                Ok(ret) => {
                    return ret
                        .unwrap()
                        .attach_printable("Async unregister_group() failed.")
                }
                Err(_would_block) => {}
            }
        }
    }

    /// Subscribe a multicast group of a peer.
    pub async fn join_group(&self, peer: &str, group: &str) -> Result<(), IpconError> {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        loop {
            let mut guide = async_ctrl.writable().await.unwrap();
            match guide.try_io(|_inner| Ok(self.ih.join_group(peer, group))) {
                Ok(ret) => return ret.unwrap().attach_printable("Async join_group() failed."),
                Err(_would_block) => {}
            }
        }
    }

    /// Unsubscribe a multicast group of a peer.
    pub async fn leave_group(&self, peer: &str, group: &str) -> Result<(), IpconError> {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        loop {
            let mut guide = async_ctrl.writable().await.unwrap();
            match guide.try_io(|_inner| Ok(self.ih.leave_group(peer, group))) {
                Ok(ret) => return ret.unwrap().attach_printable("Async leave_group() failed."),
                Err(_would_block) => {}
            }
        }
    }

    /// Send multicast messages to an owned group.
    pub async fn send_multicast(
        &self,
        group: &str,
        buf: &[u8],
        sync: bool,
    ) -> Result<(), IpconError> {
        let async_ctrl = AsyncFd::new(self.ih.get_write_fd().unwrap()).unwrap();

        loop {
            let mut guide = async_ctrl.writable().await.unwrap();
            match guide.try_io(|_inner| Ok(self.ih.send_multicast_by_ref(group, &buf, sync))) {
                Ok(ret) => {
                    return ret
                        .unwrap()
                        .attach_printable("Async send_multicast() failed.")
                }
                Err(_would_block) => {}
            }
        }
    }

    /// Receiving message with timeout.
    /// receive_msg() will block until a message come. receive_msg_timeout() adds a timeout to
    /// it.The timeout is specified with seconds and microseconds.
    pub fn receive_msg_timeout(&self, tv_sec: u32, tv_usec: u32) -> Result<IpconMsg, IpconError> {
        self.ih.receive_msg_timeout(tv_sec, tv_usec)
    }

    /// Receiving message without block.
    /// This is same to receive_msg_timeout(0, 0);
    pub fn receive_msg_nonblock(&self) -> Result<IpconMsg, IpconError> {
        self.ih.receive_msg_nonblock()
    }
}
