extern crate ipcon_sys;
use scheduler;
use std::collections::HashMap;
use std::env;
use std::process::exit;

use getopts::Options;
use ipcon_sys::ipcon::{Ipcon, IpconFlag};
use ipcon_sys::ipcon_msg::{IpconKevent, IpconMsg, IpconMsgBody, IpconMsgType};
use ipcon_sys::logger::env_log_init;
use ipcon_sys::{error, info, Result};

struct RIpconLogger {
    ih: Ipcon,
    lookup: Option<HashMap<String, bool>>,
}

impl RIpconLogger {
    fn new(mut lookup: Option<HashMap<String, bool>>) -> Result<'static, RIpconLogger> {
        let ih = Ipcon::new(
            Some("ripcon_logger"),
            Some(IpconFlag::IPFDisableKeventFilter),
        )
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

    fn receive_msg(&self) -> Result<IpconMsg> {
        self.ih.receive_msg()
    }

    fn log_user_msg(&self, msg: IpconMsgBody) {
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

    fn log_kevent_msg(&self, msg: IpconKevent) {
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

    fn free(self) {
        self.ih.free();
    }
}

fn main() {
    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();

    env_log_init();

    scheduler::set_self_policy(scheduler::Policy::Fifo, 1)
        .expect("failed to set scheduling policy");

    opts.optopt("j", "join-group", "Join a string group.", "");

    let matches = opts.parse(&args[1..]).unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(1)
    });

    let mut lookup = HashMap::new();

    /*-j group@peer1,group@peer2,,,*/
    match matches.opt_str("j") {
        Some(a) => {
            let groups = a.split(",");
            for t in groups {
                lookup.insert(t.to_string(), false);
            }
        }
        None => (),
    }

    let rlogger: RIpconLogger;

    if lookup.is_empty() {
        rlogger = RIpconLogger::new(None).expect("Failed to create Ipcon handler");
    } else {
        rlogger = RIpconLogger::new(Some(lookup)).expect("Failed to create Ipcon handler");
    }

    loop {
        let msg = match rlogger.receive_msg() {
            Ok(m) => m,
            Err(e) => {
                error!("{}", e);
                break;
            }
        };

        match msg {
            IpconMsg::IpconMsgUser(m) => {
                rlogger.log_user_msg(m);
            }
            IpconMsg::IpconMsgKevent(k) => {
                rlogger.log_kevent_msg(k);
            }
            IpconMsg::IpconMsgInvalid => (),
        }
    }

    rlogger.free();
}
