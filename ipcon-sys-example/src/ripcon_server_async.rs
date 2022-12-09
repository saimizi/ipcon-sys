#[allow(unused)]
use error_stack::{IntoReport, Report, Result, ResultExt};
use ipcon_sys::{
    ipcon,
    ipcon_async::AsyncIpcon,
    ipcon_error::IpconError,
    ipcon_msg::{IpconMsg, IpconMsgType},
};

#[allow(unused)]
use jlogger::{jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder};

#[tokio::main]
async fn main() -> Result<(), IpconError> {
    JloggerBuilder::new()
        .max_level(log::LevelFilter::Trace)
        .log_time(jlogger::LogTimeFormat::TimeStamp)
        .log_console(true)
        .build();

    let ipcon = AsyncIpcon::new(Some("ipcon-str-server-async"), Some(ipcon::IPF_DEFAULT))
        .attach_printable("Failed to create Ipcon handler")?;

    log::info!("Start to waiting for message.");
    loop {
        match ipcon.receive_msg().await? {
            IpconMsg::IpconMsgUser(msg) if (msg.msg_type == IpconMsgType::IpconMsgTypeNormal) => {
                let body = String::from_utf8(msg.buf).unwrap();
                log::info!("Msg from {} : {}", msg.peer, body);
            }
            _ => {}
        }
    }
}
