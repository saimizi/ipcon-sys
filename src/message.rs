use bincode;
use bytes::{Bytes, BytesMut};
use ipcon_sys::{debug, error_str_result, info, Error, Result};
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    MsgFormat(Vec<u8>),
    MsgData(Vec<u8>),
    MsgReboot,
    MsgErr(String),
    MsgOk,
}

pub const MESSAGE_HEADER_TAG: u32 = 0x10101010;

#[derive(Serialize, Deserialize, Debug)]
#[repr(align(4))]
pub struct MessageHeader {
    tag: u32,
    len: u32,
}

impl MessageHeader {
    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn to_mut_bytes(&self) -> Result<BytesMut> {
        match bincode::serialize(self) {
            Ok(code) => {
                let mut b = BytesMut::new();
                b.extend_from_slice(&code);
                Ok(b)
            }
            Err(e) => error_str_result(&format!("{}", e)),
        }
    }

    pub fn deserialize(data: &[u8]) -> Result<MessageHeader> {
        if data.is_empty() {
            return error_str_result("data is empry");
        }

        match bincode::deserialize(data) {
            Ok(m) => Ok(m),
            Err(e) => error_str_result(&format!("{}", e)),
        }
    }
}

impl Message {
    pub const READ_EOF_ERROR: i32 = -2;
    pub fn serialize(msg: &Message) -> Result<Bytes> {
        match bincode::serialize(msg) {
            Ok(code) => Ok(Bytes::from(code)),
            Err(e) => error_str_result(&format!("{}", e)),
        }
    }

    pub fn deserialize(data: &[u8]) -> Result<Message> {
        if data.is_empty() {
            return error_str_result("data is empry");
        }

        match bincode::deserialize(data) {
            Ok(m) => Ok(m),
            Err(e) => error_str_result(&format!("{}", e)),
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        match bincode::serialize(self) {
            Ok(code) => Ok(Bytes::from(code)),
            Err(e) => error_str_result(&format!("{}", e)),
        }
    }

    pub fn to_bytes_with_header(&self) -> Result<Bytes> {
        let body = self.to_bytes()?;
        let mut h = BytesMut::from(
            MessageHeader {
                tag: MESSAGE_HEADER_TAG,
                len: body.len() as u32,
            }
            .to_mut_bytes()?,
        );

        h.extend_from_slice(&body);

        Ok(Bytes::from(h))
    }

    pub fn serialize_to_json<'a>(msg: &Message) -> Result<'a, String> {
        match serde_json::to_string(msg) {
            Ok(a) => Ok(a),
            Err(_) => error_str_result("failed to serialized"),
        }
    }

    pub fn deserialize_from_json<'a>(json: &str) -> Result<'a, Message> {
        match serde_json::from_str(json) {
            Ok(a) => Ok(a),
            Err(e) => error_str_result(&format!("{}", e)),
        }
    }

    pub fn send_to(&self, s: &mut TcpStream) -> Result<()> {
        let m = self.to_bytes_with_header()?;

        match s.write(&m) {
            Ok(_) => Ok(()),
            Err(e) => error_str_result(&format!("{}", e)),
        }
    }

    pub fn send_msg(s: &mut TcpStream, msg: Message) -> Result<()> {
        msg.send_to(s)
    }

    pub fn receive_msg(s: &mut TcpStream) -> Result<Message> {
        let mut header_buf = [0; std::mem::size_of::<MessageHeader>()];

        match s.read(&mut header_buf) {
            Ok(bytes_read) => {
                debug!("bytes_read: {}", bytes_read);
                if bytes_read == 0 {
                    return Err(Error::new_code_err(Message::READ_EOF_ERROR));
                }
                let mhr = MessageHeader::deserialize(&header_buf)?;
                debug!("bodylen: {}", mhr.len());
                let mut buf = BytesMut::with_capacity(mhr.len() as usize);
                unsafe {
                    buf.set_len(mhr.len() as usize);
                }
                debug!("buf len: {}", buf.len());
                match s.read(&mut buf) {
                    Ok(bytes_read) => {
                        debug!("read body len: {}", bytes_read);
                        if bytes_read == 0 {
                            Err(Error::new_code_err(Message::READ_EOF_ERROR))
                        } else {
                            Message::deserialize(&buf)
                        }
                    }
                    Err(e) => {
                        debug!("error {}", e);
                        error_str_result(&format!("read_str(): {}", e))
                    }
                }
            }
            Err(e) => error_str_result(&format!("read_str(): {}", e)),
        }
    }
}
