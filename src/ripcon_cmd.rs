extern crate ipcon_sys;
use bytes::Bytes;
use std::env;
use std::process::exit;

use getopts::Options;
use ipcon_sys::error;
use ipcon_sys::ipcon::{Ipcon, IPF_SND_IF};
use ipcon_sys::logger::env_log_init;

fn show_help() {
    eprintln!("Usage: ripcon_cmd -p <peer> -m <message> [-h | --help]");
}

fn main() {
    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();

    env_log_init();

    opts.optopt("p", "peer", "Peer name to which message is to be sent.", "");
    opts.optopt("m", "message", "Message to be sent.", "");
    opts.optflag("h", "help", "Show help information.");

    let matches = opts.parse(&args[1..]).unwrap_or_else(|e| {
        eprintln!("{}", e);
        eprintln!("");
        show_help();
        exit(1)
    });

    let s = [String::from("h")];
    if matches.opts_present(&s) {
        show_help();
        return;
    }

    let pname = match matches.opt_str("p") {
        Some(p) => p,
        None => {
            error!("No target peer specified");
            exit(1)
        }
    };

    let msg = match matches.opt_str("m") {
        Some(p) => p,
        None => {
            error!("No message specified");
            exit(1)
        }
    };

    let ih = Ipcon::new(None, Some(IPF_SND_IF)).expect("failed to create ipcon handler");
    if let Err(e) = ih.send_unicast_msg(&pname, Bytes::from(msg)) {
        error!(e);
    }

    ih.free();
}
