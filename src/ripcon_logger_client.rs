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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    env_log_init();

    let mut opts = Options::new();
    opts.optopt(
        "c",
        "",
        "Specify remote ip address and port. Ex: \"127.0.0.0:8888\"",
        "",
    );

    opts.optflag("h", "help", "Print help");

    let matches = opts.parse(&args[1..]).unwrap_or_else(|e| {
        eprintln!("opts.parse(): {}", e);
        exit(1)
    });

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
