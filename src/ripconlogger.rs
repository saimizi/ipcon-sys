extern crate ipcon_sys;
use ipcon_sys::ipcon::{Ipcon, IpconFlag};
use ipcon_sys::ipcon_msg::{IpconKevent, IpconMsg, IpconMsgBody, IpconMsgType};
use ipcon_sys::{info, Result};
use std::collections::HashMap;

pub struct RIpconLogger {
    ih: Ipcon,
    lookup: Option<HashMap<String, bool>>,
}

impl RIpconLogger {
    pub fn new(
        name: &str,
        mut lookup: Option<HashMap<String, bool>>,
    ) -> Result<'static, RIpconLogger> {
        let ih = Ipcon::new(Some(name), Some(IpconFlag::IPFDisableKeventFilter))
            .expect("Failed to create Ipcon handler");

        ih.join_group(Ipcon::IPCON_KERNEL_NAME, Ipcon::IPCON_KERNEL_GROUP_NAME)
            .expect("failed to join ipcon kevent group");

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

        Ok(RIpconLogger {
            ih: ih,
            lookup: lookup,
        })
    }

    pub fn receive_msg(&self) -> Result<IpconMsg> {
        self.ih.receive_msg()
    }

    pub fn log_user_msg(&self, msg: IpconMsgBody) {
        let content =
            String::from_utf8(msg.buf.to_vec()).unwrap_or(String::from("Non text message"));
        let group = msg.group.unwrap_or(String::from("?"));

        match msg.msg_type {
            IpconMsgType::IpconMsgTypeNormal => {
                let lines = content.split("\n");

                for l in lines {
                    info!("{}: {}", msg.peer, l);
                }
            }
            IpconMsgType::IpconMsgTypeGroup => {
                let lines = content.split("\n");
                for l in lines {
                    info!("{}@{}: {}", group, msg.peer, l);
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
    }

    pub fn log_msg(&self, msg: IpconMsg) {
        match msg {
            IpconMsg::IpconMsgUser(m) => self.log_user_msg(m),
            IpconMsg::IpconMsgKevent(k) => self.log_kevent_msg(k),
            IpconMsg::IpconMsgInvalid => (),
        }
    }

    pub fn free(self) {
        self.ih.free();
    }
}
