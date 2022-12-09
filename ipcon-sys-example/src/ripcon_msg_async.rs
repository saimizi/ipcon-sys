use clap::Parser;
#[allow(unused)]
use error_stack::{IntoReport, Report, Result, ResultExt};
use ipcon_sys::{ipcon, ipcon_async::AsyncIpcon, ipcon_error::IpconError};

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
const IPCON_SERVER: &str = "ipcon-str-server-async";

#[tokio::main]
async fn main() -> Result<(), IpconError> {
    JloggerBuilder::new()
        .max_level(log::LevelFilter::Trace)
        .log_runtime(true)
        .log_time(jlogger::LogTimeFormat::TimeStamp)
        .log_console(true)
        .build();

    let cli = Cli::parse();
    let mut handlers = Vec::new();

    for _i in 0..cli.count {
        let msg = cli.msg.clone();
        handlers.push(tokio::spawn(async move {
            let ipcon = AsyncIpcon::new(None, Some(ipcon::IPF_DEFAULT))
                .attach_printable("Failed to create Ipcon handler")
                .unwrap();

            jinfo!("send Msg");
            ipcon
                .send_unicast_msg(IPCON_SERVER, msg.as_bytes())
                .await
                .unwrap();
        }));
    }

    for handler in handlers {
        handler.await.unwrap();
    }

    Ok(())
}
