use clap::Parser;
#[allow(unused)]
use error_stack::{IntoReport, Report, Result, ResultExt};
use ipcon_sys::{
    ipcon::{self, Ipcon},
    ipcon_error::IpconError,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    msg: String,

    #[arg(short, long, default_value_t = 3)]
    count: usize,
}

#[allow(unused)]
use jlogger::{jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder};
const IPCON_SERVER: &'static str = "ipcon-str-server";

fn main() -> Result<(), IpconError> {
    JloggerBuilder::new()
        .max_level(log::LevelFilter::Trace)
        .log_runtime(true)
        .log_time(jlogger::LogTimeFormat::TimeStamp)
        .log_console(true)
        .build();

    let cli = Cli::parse();
    let mut handlers = Vec::new();

    for i in 0..cli.count {
        let msg = cli.msg.clone();
        handlers.push(
            std::thread::Builder::new()
                .name(format!("Worker-{}", i + 1))
                .spawn(move || {
                    let ipcon = Ipcon::new(None, Some(ipcon::IPF_DEFAULT))
                        .attach_printable("Failed to create Ipcon handler")
                        .unwrap();

                    jinfo!("send Msg");
                    ipcon
                        .send_unicast_msg(IPCON_SERVER, msg.as_bytes())
                        .unwrap();
                })
                .unwrap(),
        );
    }

    for handler in handlers {
        handler.join().unwrap();
    }

    Ok(())
}
