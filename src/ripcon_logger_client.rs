#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

pub mod client;
pub mod logger;
pub mod message;

use client::Client;
use logger::env_log_init;
use message::Message;
use std::process::exit;

use getopts::Options;

fn show_help() {
    eprintln!("Usage:RUST_LOG=info ripcon_logger_client");
    eprintln!("\t[-c  <Server ip>:<port> (ex: \"127.0.0.1:7878\")");
    eprintln!("\t[-h | --help]");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    env_log_init();

    let mut opts = Options::new();
    opts.optopt(
        "c",
        "",
        "Specify remote ip address and port. Ex: \"127.0.0.1:8888\"",
        "",
    );

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

    let mut client = Client::new().unwrap_or_else(|e| {
        eprintln!("Client::new():{}", e);
        exit(1)
    });

    match matches.opt_str("c") {
        Some(ip_str) => {
            client.connect(&ip_str).unwrap_or_else(|e| {
                eprintln!("client.connect(): {}", e);
                exit(1)
            });
            info!("Log server {} connected", ip_str);
        }
        None => {
            error!("No server specified");
            exit(1);
        }
    }

    let stream = client.stream_ref().unwrap();

    loop {
        match Message::receive_msg(stream) {
            Ok(msg) => match msg {
                Message::MsgStrData(b) => {
                    let lines = b.split("\n");
                    for l in lines {
                        info!("{}", l);
                    }
                }
                _ => (),
            },
            Err(e) => {
                if e.err_code() == Message::READ_EOF_ERROR {
                    error!("Remote disconnected.");
                    break;
                }
            }
        }
    }
}
