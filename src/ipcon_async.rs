use crate::ipcon::{Ipcon, IpconFlag};
use crate::ipcon_msg::IpconMsg;
use bytes::Bytes;
use std::io::Result;
use tokio::io::unix::AsyncFd;

#[link(name = "ipcon")]
extern "C" {}

pub struct AsyncIpcon {
    ih: Ipcon,
}

impl AsyncIpcon {
    pub fn new(peer_name: Option<&str>, flag: Option<IpconFlag>) -> Option<AsyncIpcon> {
        if let Some(ih) = Ipcon::new(peer_name, flag) {
            Some(AsyncIpcon { ih })
        } else {
            None
        }
    }

    pub async fn is_peer_present(&self, peer: &str) -> bool {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        let mut guide = async_ctrl.writable().await.unwrap();
        loop {
            match guide.try_io(|_inner| Ok(self.ih.is_peer_present(peer))) {
                Ok(Ok(ret)) => {
                    return ret;
                }
                Ok(Err(e)) => {
                    panic!("Unexpected error: {}", e);
                }
                Err(_would_block) => {}
            }
        }
    }

    pub async fn is_group_present(&self, peer: &str, group: &str) -> bool {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        let mut guide = async_ctrl.writable().await.unwrap();
        loop {
            match guide.try_io(|_inner| Ok(self.ih.is_group_present(peer, group))) {
                Ok(Ok(ret)) => {
                    return ret;
                }
                Ok(Err(e)) => {
                    panic!("Unexpected error: {}", e);
                }
                Err(_would_block) => {}
            }
        }
    }

    pub async fn receive_msg(&self) -> Result<IpconMsg> {
        let async_ctrl = AsyncFd::new(self.ih.get_read_fd().unwrap()).unwrap();

        let mut guide = async_ctrl.readable().await.unwrap();
        loop {
            match guide.try_io(|_inner| self.ih.receive_msg()) {
                Ok(ret) => {
                    return ret;
                }
                Err(_would_block) => {}
            }
        }
    }

    pub async fn send_unicast_msg(&self, peer: &str, buf: Bytes) -> Result<()> {
        let async_ctrl = AsyncFd::new(self.ih.get_write_fd().unwrap()).unwrap();

        let mut guide = async_ctrl.writable().await.unwrap();
        loop {
            match guide.try_io(|_inner| self.ih.send_unicast_msg_by_ref(peer, &buf)) {
                Ok(ret) => {
                    return ret;
                }
                Err(_would_block) => {}
            }
        }
    }

    pub async fn register_group(&self, group: &str) -> Result<()> {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        let mut guide = async_ctrl.writable().await.unwrap();
        loop {
            match guide.try_io(|_inner| self.ih.register_group(group)) {
                Ok(ret) => {
                    return ret;
                }
                Err(_would_block) => {}
            }
        }
    }

    pub async fn unregister_group(&self, group: &str) -> Result<()> {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        let mut guide = async_ctrl.writable().await.unwrap();
        loop {
            match guide.try_io(|_inner| self.ih.unregister_group(group)) {
                Ok(ret) => {
                    return ret;
                }
                Err(_would_block) => {}
            }
        }
    }

    pub async fn join_group(&self, peer: &str, group: &str) -> Result<()> {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        let mut guide = async_ctrl.writable().await.unwrap();
        loop {
            match guide.try_io(|_inner| self.ih.join_group(peer, group)) {
                Ok(ret) => {
                    return ret;
                }
                Err(_would_block) => {}
            }
        }
    }

    pub async fn leave_group(&self, peer: &str, group: &str) -> Result<()> {
        let async_ctrl = AsyncFd::new(self.ih.get_ctrl_fd().unwrap()).unwrap();

        let mut guide = async_ctrl.writable().await.unwrap();
        loop {
            match guide.try_io(|_inner| self.ih.leave_group(peer, group)) {
                Ok(ret) => {
                    return ret;
                }
                Err(_would_block) => {}
            }
        }
    }

    pub async fn send_multicast(&self, group: &str, buf: Bytes, sync: bool) -> Result<()> {
        let async_ctrl = AsyncFd::new(self.ih.get_write_fd().unwrap()).unwrap();

        let mut guide = async_ctrl.writable().await.unwrap();
        loop {
            match guide.try_io(|_inner| self.ih.send_multicast_by_ref(group, &buf, sync)) {
                Ok(ret) => {
                    return ret;
                }
                Err(_would_block) => {}
            }
        }
    }

    pub fn receive_msg_timeout(&self, tv_sec: u32, tv_usec: u32) -> Result<IpconMsg> {
        self.ih.receive_msg_timeout(tv_sec, tv_usec)
    }

    pub fn receive_msg_nonblock(&self) -> Result<IpconMsg> {
        self.ih.receive_msg_nonblock()
    }
}
