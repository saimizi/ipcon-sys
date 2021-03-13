use crate::debug;
use crate::{error_str_result, Result};
use bytes::{Bytes, BytesMut};
use std::ffi::CStr;
use std::fmt;
use std::os::raw::c_char;

pub const IPCON_MAX_PAYLOAD_LEN: usize = 2048;
pub const IPCON_MAX_NAME_LEN: usize = 32;

pub type IpconKeventType = std::os::raw::c_int;
pub const IpconKeventTypePeerAdd: IpconMsgType = 0;
pub const IpconKeventTypePeerRemove: IpconMsgType = 1;
pub const IpconKeventTypeGroupAdd: IpconMsgType = 2;
pub const IpconKeventTypeGroupRemove: IpconMsgType = 3;

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

impl IpconKevent {
    pub fn get_string(&self) -> String {
        match self.ke_type {
            IpconKeventTypePeerAdd => unsafe {
                let peer_name = CStr::from_ptr(&self.u.peer.peer_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                format!("peer {} added", peer_name)
            },
            IpconKeventTypePeerRemove => unsafe {
                let peer_name = CStr::from_ptr(&self.u.peer.peer_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                format!("peer {} removed", peer_name)
            },
            IpconKeventTypeGroupAdd => unsafe {
                let peer_name = CStr::from_ptr(&self.u.group.peer_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                let group_name = CStr::from_ptr(&self.u.group.group_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                format!("group {}@{} added", group_name, peer_name)
            },

            IpconKeventTypeGroupRemove => unsafe {
                let peer_name = CStr::from_ptr(&self.u.group.peer_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                let group_name = CStr::from_ptr(&self.u.group.group_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                format!("group {}@{} removed", group_name, peer_name)
            },
            _ => format!("Invalid kevent type"),
        }
    }
}

impl fmt::Display for IpconKevent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_string())
    }
}

pub type IpconMsgType = std::os::raw::c_int;
pub const IpconMsgTypeNormal: IpconMsgType = 0;
pub const IpconMsgTypeGroup: IpconMsgType = 1;
pub const IpconMsgTypeKevent: IpconMsgType = 2;
pub const IpconMsgTypeInvalid: IpconMsgType = 3;

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
    pub group: [c_char; IPCON_MAX_NAME_LEN],
    pub peer: [c_char; IPCON_MAX_NAME_LEN],
    len: u32,
    u: IpconMsgUion,
}

impl LibIpconMsg {
    pub fn new() -> LibIpconMsg {
        LibIpconMsg {
            msg_type: IpconMsgTypeInvalid,
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
            IpconMsgTypeNormal => {
                let peer_name: String;

                unsafe {
                    peer_name = match CStr::from_ptr(&msg.peer as *const i8).to_str() {
                        Ok(p) => String::from(p),
                        Err(_) => return error_str_result("Invalid peer name"),
                    };
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

            IpconMsgTypeGroup => {
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

            IpconMsgTypeKevent => unsafe { Ok(IpconMsg::IpconMsgKevent(msg.u.kevent.clone())) },

            IpconMsgTypeInvalid => Ok(IpconMsg::IpconMsgInvalid),
            _ => Ok(IpconMsg::IpconMsgInvalid),
        }
    }
}
