extern crate ipcon_sys;
use bytes::Bytes;
use std::env;
use std::process::exit;

use getopts::Options;
use ipcon_sys::ipcon::{Ipcon, IpconFlag};
use ipcon_sys::ipcon_msg::{IpconKevent, IpconMsg, IpconMsgBody, IpconMsgType};
use ipcon_sys::logger::env_log_init;
use ipcon_sys::{debug, error, info, warn};

fn do_user_msg(msg: IpconMsgBody) {
    let content = String::from_utf8(msg.buf.to_vec()).unwrap_or(String::from("Non text message"));
    let group = msg.group.unwrap_or(String::from("?"));

    debug!("do_user_msg: msg.msg_type: {:?}", msg.msg_type);
    match msg.msg_type {
        IpconMsgType::IpconMsgTypeNormal => {
            info!("{}: {}", msg.peer, content);
        }
        IpconMsgType::IpconMsgTypeGroup => {
            info!("{}@{}: {}", group, msg.peer, content);
        }
        _ => (),
    }
}

fn do_kevent_msg(msg: IpconKevent) {
    info!("{}", msg.to_string());
}

fn main() {
    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();

    env_log_init();

    opts.optopt("j", "join-group", "Join a string group.", "");

    let matches = opts.parse(&args[1..]).unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(1)
    });

    let ih = Ipcon::new(
        Some("ripcon_logger"),
        Some(IpconFlag::IPFDisableKeventFilter),
    )
    .expect("failed to create ipcon handler");

    ih.join_group(Ipcon::IpconKernelName, Ipcon::IpconKernelGroupName)
        .expect("failed to join ipcon kevent group");

    /*-j group@peer1,group@peer2,,,*/
    match matches.opt_str("j") {
        Some(a) => {
            let groups = a.split(",");
            for t in groups {
                let mut group_peer = t.split("@");
                let g = group_peer.next().expect(&format!("Invalid group {}", t));
                let p = group_peer.next().expect(&format!("Invalid group {}", t));
                info!("Join {}@{}", g, p);
                if let Err(e) = ih.join_group(&p, &g) {
                    warn!("failed to join {}@{} : {}", g, p, e);
                }
            }
        }
        None => (),
    }

    loop {
        let msg = match ih.receive_msg() {
            Ok(m) => m,
            Err(e) => {
                error!("{}", e);
                break;
            }
        };

        match msg {
            IpconMsg::IpconMsgUser(m) => {
                do_user_msg(m);
            }
            IpconMsg::IpconMsgKevent(k) => {
                do_kevent_msg(k);
            }
            IpconMsg::IpconMsgInvalid => (),
        }
    }

    ih.free();
}
