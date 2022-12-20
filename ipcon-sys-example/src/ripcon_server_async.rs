#[allow(unused)]
use error_stack::{IntoReport, Report, Result, ResultExt};
use ipcon_sys::{
    ipcon,
    ipcon_async::AsyncIpcon,
    ipcon_error::IpconError,
    ipcon_msg::{IpconMsg, IpconMsgType},
};

#[allow(unused)]
use jlogger_tracing::{
    jdebug, jerror, jinfo, jtrace, jwarn, JloggerBuilder, LevelFilter, LogTimeFormat,
};

#[tokio::main]
async fn main() -> Result<(), IpconError> {
    JloggerBuilder::new()
        .log_time(LogTimeFormat::TimeStamp)
        .log_console(true)
        .build();

    let ipcon = AsyncIpcon::new(Some("ipcon-str-server"), Some(ipcon::IPF_DEFAULT))
        .attach_printable("Failed to create Ipcon handler")?;

    jinfo!("Start to waiting for message.");
    loop {
        match ipcon.receive_msg().await? {
            IpconMsg::IpconMsgUser(msg) if (msg.msg_type == IpconMsgType::IpconMsgTypeNormal) => {
                let body = String::from_utf8(msg.buf).unwrap();
                jinfo!(sender = msg.peer, msg = body);
            }
            _ => {}
        }
    }
}
