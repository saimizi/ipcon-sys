extern crate ipcon_sys;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

pub mod message;
pub mod ripconlogger;

use libc::{getrlimit, rlimit, setrlimit, RLIMIT_RTPRIO};
use libc::{sched_get_priority_min, sched_param, sched_setscheduler, SCHED_FIFO};
use std::collections::HashMap;
use std::env;
use std::process::exit;

use getopts::Options;
use ipcon_sys::logger::env_log_init;
use ipcon_sys::{debug, error, warn};

use ripconlogger::RIpconLogger;

fn main() {
    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();

    env_log_init();

    unsafe {
        let rl = rlimit {
            rlim_cur: 99,
            rlim_max: 99,
        };

        let ret = setrlimit(RLIMIT_RTPRIO, &rl);
        if ret < 0 {
            warn!(
                "setrlimit() failed: {} {:?}",
                ret,
                std::io::Error::last_os_error()
            );
        }

        let mut rl = rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };

        let ret = getrlimit(RLIMIT_RTPRIO, &mut rl);
        if ret < 0 {
            warn!(
                "getrlimit() failed: {} {:?}",
                ret,
                std::io::Error::last_os_error()
            );
        }

        debug!("RLIMIT_RTPRIO: cur: {} max: {}", rl.rlim_cur, rl.rlim_max);

        let priority = sched_get_priority_min(SCHED_FIFO);

        debug!("Min SCHED_FIFO priority: {}", priority);

        let param = sched_param {
            sched_priority: priority,
        };
        let ret = sched_setscheduler(0, SCHED_FIFO, &param as *const sched_param);
        if ret != 0 {
            warn!(
                "Faied to set scheduling policy {} {:?}",
                ret,
                std::io::Error::last_os_error()
            );
        }
    }

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
        rlogger = RIpconLogger::new("ripcon_logger", None).expect("Failed to create Ipcon handler");
    } else {
        rlogger = RIpconLogger::new("ripcon_logger", Some(lookup))
            .expect("Failed to create Ipcon handler");
    }

    loop {
        let msg = match rlogger.receive_msg() {
            Ok(m) => m,
            Err(e) => {
                error!("{}", e);
                break;
            }
        };

        rlogger.log_msg(msg);
    }

    rlogger.free();
}
