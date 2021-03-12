use crate::{error_str_result, Result};
use bytes::{Bytes, BytesMut};
use std::ffi::CStr;

pub const IPCON_MAX_PAYLOAD_LEN: usize = 2048;
pub const IPCON_MAX_NAME_LEN: usize = 32;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum IpconKeventType {
    IpconEventPeerAdd,
    IpconEventPeerRemove,
    IpconEventGroupAdd,
    IpconEventGroupRemove,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IpconKeventGroup {
    pub peer_name: [std::os::raw::c_char; IPCON_MAX_NAME_LEN],
    pub group_name: [std::os::raw::c_char; IPCON_MAX_NAME_LEN],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IpconKeventPeer {
    pub peer_name: [std::os::raw::c_char; IPCON_MAX_NAME_LEN],
    pub group_name: [std::os::raw::c_char; IPCON_MAX_NAME_LEN],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union IpconKeventUnion {
    pub peer: IpconKeventPeer,
    pub group: IpconKeventGroup,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct IpconKevent {
    pub ke_type: IpconKeventType,
    pub u: IpconKeventUnion,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum IpconMsgType {
    IpconNormalMsg,
    IpconGroupMsg,
    IpconKeventMsg,
    IpconInvalidMsg,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union IpconMsgUion {
    buf: [std::os::raw::c_uchar; IPCON_MAX_PAYLOAD_LEN],
    kevent: IpconKevent,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LibIpconMsg {
    msg_type: IpconMsgType,
    pub peer: [std::os::raw::c_char; IPCON_MAX_NAME_LEN],
    pub group: [std::os::raw::c_char; IPCON_MAX_NAME_LEN],
    len: u32,
    u: IpconMsgUion,
}

impl LibIpconMsg {
    pub fn new() -> LibIpconMsg {
        LibIpconMsg {
            msg_type: IpconMsgType::IpconInvalidMsg,
            peer: [0; IPCON_MAX_NAME_LEN],
            group: [0; IPCON_MAX_NAME_LEN],
            len: 0,
            u: IpconMsgUion {
                buf: [0; IPCON_MAX_PAYLOAD_LEN],
            },
        }
    }
}

pub struct IpconMsgBody {
    pub msg_type: IpconMsgType,
    pub peer: String,
    pub group: Option<String>,
    pub buf: Bytes,
}

pub enum IpconMsg {
    IpconMsgUser(IpconMsgBody),
    IpconMsgKevent(IpconKevent),
    IpconMsgInvalid,
}

impl IpconMsg {
    pub fn from_libipcon_msg(msg: LibIpconMsg) -> Result<'static, IpconMsg> {
        match msg.msg_type {
            IpconMsgType::IpconNormalMsg => {
                let peer_name: String;

                unsafe {
                    peer_name = match CStr::from_ptr(&msg.peer as *const i8).to_str() {
                        Ok(p) => String::from(p),
                        Err(_) => return error_str_result("Invalid message"),
                    }
                }

                let buf: BytesMut;
                unsafe {
                    buf = BytesMut::from(&msg.u.buf[..]);
                }

                let m = IpconMsgBody {
                    msg_type: msg.msg_type,
                    peer: peer_name,
                    group: None,
                    buf: Bytes::from(buf),
                };

                Ok(IpconMsg::IpconMsgUser(m))
            }

            IpconMsgType::IpconGroupMsg => {
                let peer_name: String;
                let group_name: String;

                unsafe {
                    peer_name = match CStr::from_ptr(&msg.peer as *const i8).to_str() {
                        Ok(p) => String::from(p),
                        Err(_) => return error_str_result("Invalid peer name"),
                    };

                    group_name = match CStr::from_ptr(&msg.group as *const i8).to_str() {
                        Ok(p) => String::from(p),
                        Err(_) => return error_str_result("Invalid group name"),
                    };
                }

                let buf: BytesMut;
                unsafe {
                    buf = BytesMut::from(&msg.u.buf[..]);
                }

                let m = IpconMsgBody {
                    msg_type: msg.msg_type,
                    peer: peer_name,
                    group: Some(group_name),
                    buf: Bytes::from(buf),
                };
                Ok(IpconMsg::IpconMsgUser(m))
            }

            IpconMsgType::IpconKeventMsg => unsafe {
                Ok(IpconMsg::IpconMsgKevent(msg.u.kevent.clone()))
            },

            IpconMsgType::IpconInvalidMsg => Ok(IpconMsg::IpconMsgInvalid),
        }
    }
}
