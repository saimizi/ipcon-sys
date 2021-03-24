extern crate ipcon_sys;
use crate::message::Message;
use crate::{debug, error, info};
use ipcon_sys::ipcon::{Ipcon, IpconFlag};
use ipcon_sys::ipcon_msg::{IpconKevent, IpconMsg, IpconMsgBody, IpconMsgType};
use ipcon_sys::logger::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::net::{TcpListener, TcpStream};

use std::sync::{Arc, Mutex};
use std::thread;

fn handle_client(m: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>) -> i32 {
    let listener: TcpListener;
    listener = TcpListener::bind("0.0.0.0:7878").expect("Failed to bind address");
    info!("Listen on 0.0.0.0:7878...");

    for stream in listener.incoming() {
        if let Ok(s) = stream {
            if let Ok(a) = s.peer_addr() {
                debug!("Connected from {}", a);
                let mut h = m.lock().unwrap();
                h.insert(a, s);
            }
        }
    }

    0
}

pub struct RIpconLogger {
    ih: Ipcon,
    lookup: Option<HashMap<String, bool>>,
    thread: thread::JoinHandle<i32>,
    streams: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
}

impl RIpconLogger {
    pub fn new(
        name: &str,
        mut lookup: Option<HashMap<String, bool>>,
    ) -> Result<'static, RIpconLogger> {
        let ih = Ipcon::new(Some(name), Some(IpconFlag::IPFDisableKeventFilter))
            .expect("Failed to create Ipcon handler");

        ih.join_group(Ipcon::IPCON_KERNEL_NAME, Ipcon::IPCON_KERNEL_GROUP_NAME)
            .expect("Failed to join ipcon kevent group");

        if let Some(h) = &mut lookup {
            for pg in h.keys() {
                let mut group_peer = pg.split("@");
                let g = group_peer.next().expect(&format!("Invalid group {}", pg));
                let p = group_peer.next().expect(&format!("Invalid group {}", pg));
                if let Ok(_) = ih.join_group(&p, &g) {
                    info!("Joined group {}@{}", g, p);
                }
            }
        }

        let m = Arc::new(Mutex::new(HashMap::new()));
        let t = Arc::clone(&m);

        let server_handler = thread::spawn(move || handle_client(t));

        Ok(RIpconLogger {
            ih: ih,
            lookup: lookup,
            thread: server_handler,
            streams: m,
        })
    }

    pub fn receive_msg(&self) -> Result<IpconMsg> {
        self.ih.receive_msg()
    }

    pub fn send_remote(&self, msg: &str) -> Result<()> {
        let m = Message::MsgStrData(String::from(msg));

        let mut h = self.streams.lock().unwrap();
        for (addr, stream) in &mut *h {
            debug!("Send message to {}", addr);
            let _ = m.send_to(stream);
        }

        Ok(())
    }

    pub fn log_user_msg(&self, msg: IpconMsgBody) {
        let content =
            String::from_utf8(msg.buf.to_vec()).unwrap_or(String::from("Non text message"));
        let group = msg.group.unwrap_or(String::from("?"));

        match msg.msg_type {
            IpconMsgType::IpconMsgTypeNormal => {
                let lines = content.split("\n");
                let mut remote_msg = String::new();

                for l in lines {
                    info!("{}: {}", msg.peer, l);
                    remote_msg.push_str(&format!("{} : {}\n", msg.peer, l));
                }

                if let Err(e) = self.send_remote(&remote_msg) {
                    error!("{}", e);
                }
            }
            IpconMsgType::IpconMsgTypeGroup => {
                let lines = content.split("\n");
                let mut remote_msg = String::new();

                for l in lines {
                    info!("{}@{}: {}", group, msg.peer, l);
                    remote_msg.push_str(&format!("{}@{} : {}\n", group, msg.peer, l));
                }

                if let Err(e) = self.send_remote(&remote_msg) {
                    error!("{}", e);
                }
            }
            _ => (),
        }
    }

    pub fn log_kevent_msg(&self, msg: IpconKevent) {
        if let Some(l) = &self.lookup {
            if let Some((p, g)) = msg.group_added() {
                let k = format!("{}@{}", g, p);
                if l.contains_key(&k) {
                    if let Ok(_) = self.ih.join_group(&p, &g) {
                        info!("Logger joined group {}@{}", g, p);
                    }
                }
            }

            if let Some((p, g)) = msg.group_removed() {
                let k = format!("{}@{}", g, p);
                if l.contains_key(&k) {
                    info!("Logger left group {}@{}", g, p);
                }
            }
        }
        info!("{}", msg.to_string());

        if let Err(e) = self.send_remote(&msg.to_string()) {
            error!("{}", e);
        }
    }

    pub fn log_msg(&self, msg: IpconMsg) {
        match msg {
            IpconMsg::IpconMsgUser(m) => self.log_user_msg(m),
            IpconMsg::IpconMsgKevent(k) => self.log_kevent_msg(k),
            IpconMsg::IpconMsgInvalid => (),
        }
    }

    pub fn free(self) {
        self.thread.join().expect("Failed to join the thread");
        self.ih.free();
    }
}
