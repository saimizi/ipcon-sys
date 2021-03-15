extern crate ipcon_sys;
use bytes::Bytes;
use scheduler;
use std::collections::HashMap;
use std::env;
use std::process::exit;

use getopts::Options;
use ipcon_sys::ipcon::{Ipcon, IpconFlag};
use ipcon_sys::ipcon_msg::{IpconKevent, IpconMsg, IpconMsgBody, IpconMsgType};
use ipcon_sys::logger::env_log_init;

fn main() {
    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();

    env_log_init();

    scheduler::set_self_policy(scheduler::Policy::Fifo, 1);

    opts.optopt("p", "wait-peer", "Wait a peer.", "");
    opts.optopt("g", "wait-group", "Wait a group.", "");

    let matches = opts.parse(&args[1..]).unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(1)
    });

    let ih = Ipcon::new(None, Some(IpconFlag::IPFDisableKeventFilter))
        .expect("failed to create ipcon handler");

    ih.join_group(Ipcon::IpconKernelName, Ipcon::IpconKernelGroupName)
        .expect("failed to join ipcon kevent group");

    let mut lookup = HashMap::new();

    /*-g group@peer1,group@peer2,,,*/
    match matches.opt_str("g") {
        Some(a) => {
            let groups = a.split(",");
            for t in groups {
                let mut group_peer = t.split("@");
                let g = group_peer.next().expect(&format!("Invalid group {}", t));
                let p = group_peer.next().expect(&format!("Invalid group {}", t));

                if !ih.is_group_present(&p, &g) {
                    ipcon_sys::debug! {"Wait {}@{}", g, p};
                    lookup.insert(t.to_string(), false);
                } else {
                    ipcon_sys::debug! {"{}@{} has been presented", g, p};
                }
            }
        }
        None => (),
    }

    /*-p peer1,peer2,,,*/
    match matches.opt_str("p") {
        Some(a) => {
            let peers = a.split(",");
            for t in peers {
                if !ih.is_peer_present(&t) {
                    ipcon_sys::debug! {"Wait {}", t};
                    lookup.insert(t.to_string(), false);
                } else {
                    ipcon_sys::debug! {"{} has been presented", t};
                }
            }
        }
        None => (),
    }

    if lookup.is_empty() {
        ipcon_sys::debug! {"finish waitting"};
        return;
    }

    loop {
        let msg = match ih.receive_msg() {
            Ok(m) => m,
            Err(e) => {
                ipcon_sys::error!("{}", e);
                break;
            }
        };

        match msg {
            IpconMsg::IpconMsgKevent(k) => {
                if let Some(p) = k.peer_added() {
                    if lookup.contains_key(&p) {
                        ipcon_sys::debug! {"found {}", p};
                        let _ = lookup.remove(&p);
                        if lookup.is_empty() {
                            ipcon_sys::debug! {"finish waitting"};
                            return;
                        }
                    }
                }

                if let Some((p, g)) = k.group_added() {
                    let k = format!("{}@{}", g, p);
                    if lookup.contains_key(&k) {
                        ipcon_sys::debug! {"found {}", k};
                        let _ = lookup.remove(&k);
                        if lookup.is_empty() {
                            ipcon_sys::debug! {"finish waitting"};
                            return;
                        }
                    }
                }
            }

            _ => (),
        }
    }

    ih.free();
}
