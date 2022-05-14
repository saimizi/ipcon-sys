use std::ffi::CStr;
use std::fmt;
use std::io::{Error, ErrorKind, Result};
use std::os::raw::c_char;

pub const IPCON_MAX_PAYLOAD_LEN: usize = 2048;
pub const IPCON_MAX_NAME_LEN: usize = 32;

pub type IpconKeventType = std::os::raw::c_int;
pub const IPCON_KEVENT_TYPE_PEER_ADD: IpconKeventType = 0;
pub const IPCON_KEVENT_TYPE_PEER_REMOVE: IpconKeventType = 1;
pub const IPCON_KEVENT_TYPE_GROUP_ADD: IpconKeventType = 2;
pub const IPCON_KEVENT_TYPE_GROUP_REMOVE: IpconKeventType = 3;

/// Group information of IpconKevent.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IpconKeventGroup {
    pub group_name: [std::os::raw::c_char; IPCON_MAX_NAME_LEN],
    pub peer_name: [std::os::raw::c_char; IPCON_MAX_NAME_LEN],
}

/// Peer information of IpconKevent.
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

/// IpconKevent is a group message deliveried from the IPCON_KERNEL_GROUP_NAME group of IPCON
/// kernel module peer named IPCON_KERNEL_NAME. It deliveries the following messages to peer:
/// * Peer added
/// * Peer exited
/// * Group of a peer added
/// * Group of a peer removed
///
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IpconKevent {
    pub ke_type: IpconKeventType,
    pub u: IpconKeventUnion,
}

impl IpconKevent {
    /// Get a string of the events like following:
    /// ```
    /// "peer <peer name> added"
    /// "peer <peer name> removed"
    /// "group <group name>@<peer name> added"
    /// "group <group name>@<peer name> removed"
    /// ```
    pub fn get_string(&self) -> String {
        match self.ke_type {
            IPCON_KEVENT_TYPE_PEER_ADD => unsafe {
                let peer_name = CStr::from_ptr(&self.u.peer.peer_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                format!("peer {} added", peer_name)
            },
            IPCON_KEVENT_TYPE_PEER_REMOVE => unsafe {
                let peer_name = CStr::from_ptr(&self.u.peer.peer_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                format!("peer {} removed", peer_name)
            },
            IPCON_KEVENT_TYPE_GROUP_ADD => unsafe {
                let peer_name = CStr::from_ptr(&self.u.group.peer_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                let group_name = CStr::from_ptr(&self.u.group.group_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                format!("group {}@{} added", group_name, peer_name)
            },

            IPCON_KEVENT_TYPE_GROUP_REMOVE => unsafe {
                let peer_name = CStr::from_ptr(&self.u.group.peer_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                let group_name = CStr::from_ptr(&self.u.group.group_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid");
                format!("group {}@{} removed", group_name, peer_name)
            },
            _ => "Invalid kevent type".to_string(),
        }
    }

    /// Get the name of peer newly added.
    /// IPCON kernel module will not delivery this event of an anonymous peer.
    pub fn peer_added(&self) -> Option<String> {
        match self.ke_type {
            IPCON_KEVENT_TYPE_PEER_ADD => unsafe {
                Some(
                    CStr::from_ptr(&self.u.peer.peer_name as *const i8)
                        .to_str()
                        .unwrap_or("invalid")
                        .to_string(),
                )
            },
            _ => None,
        }
    }

    /// Get the name of peer removed.
    /// IPCON kernel module will not delivery this event of an anonymous peer.
    pub fn peer_removed(&self) -> Option<String> {
        match self.ke_type {
            IPCON_KEVENT_TYPE_PEER_REMOVE => unsafe {
                Some(
                    CStr::from_ptr(&self.u.peer.peer_name as *const i8)
                        .to_str()
                        .unwrap_or("invalid")
                        .to_string(),
                )
            },
            _ => None,
        }
    }

    /// Get the newly added group information.
    /// The first element of the tuple stores the name of peer who owns the group, and the second
    /// element stores the group name.
    pub fn group_added(&self) -> Option<(String, String)> {
        match self.ke_type {
            IPCON_KEVENT_TYPE_GROUP_ADD => unsafe {
                let peer_name = CStr::from_ptr(&self.u.group.peer_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid")
                    .to_string();
                let group_name = CStr::from_ptr(&self.u.group.group_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid")
                    .to_string();
                Some((peer_name, group_name))
            },
            _ => None,
        }
    }

    /// Get the newly removed group information.
    /// The first element of the tuple stores the name of peer who owns the group, and the second
    /// element stores the group name.
    pub fn group_removed(&self) -> Option<(String, String)> {
        match self.ke_type {
            IPCON_KEVENT_TYPE_GROUP_REMOVE => unsafe {
                let peer_name = CStr::from_ptr(&self.u.group.peer_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid")
                    .to_string();
                let group_name = CStr::from_ptr(&self.u.group.group_name as *const i8)
                    .to_str()
                    .unwrap_or("invalid")
                    .to_string();
                Some((peer_name, group_name))
            },
            _ => None,
        }
    }
}

impl fmt::Display for IpconKevent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_string())
    }
}

pub type LibIpconMsgType = std::os::raw::c_int;
pub const LIBIPCON_MSG_TYPE_NORMAL: LibIpconMsgType = 0;
pub const LIBIPCON_MSG_TYPE_GROUP: LibIpconMsgType = 1;
pub const LIBIPCON_MSG_TYPE_KEVENT: LibIpconMsgType = 2;
pub const LIBIPCON_MSG_TYPE_INVALID: LibIpconMsgType = 3;

#[repr(C)]
#[derive(Clone, Copy)]
pub union IpconMsgUion {
    buf: [std::os::raw::c_uchar; IPCON_MAX_PAYLOAD_LEN],
    kevent: IpconKevent,
}

/// Message interface to libipcon.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LibIpconMsg {
    msg_type: LibIpconMsgType,
    pub group: [c_char; IPCON_MAX_NAME_LEN],
    pub peer: [c_char; IPCON_MAX_NAME_LEN],
    len: u32,
    u: IpconMsgUion,
}

impl LibIpconMsg {
    pub fn new() -> LibIpconMsg {
        LibIpconMsg {
            msg_type: LIBIPCON_MSG_TYPE_INVALID,
            peer: [0; IPCON_MAX_NAME_LEN],
            group: [0; IPCON_MAX_NAME_LEN],
            len: 0,
            u: IpconMsgUion {
                buf: [0; IPCON_MAX_PAYLOAD_LEN],
            },
        }
    }
}

impl Default for LibIpconMsg {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Clone, Copy, Debug)]
pub enum IpconMsgType {
    IpconMsgTypeNormal,
    IpconMsgTypeGroup,
    IpconMsgTypeKevent,
    IpconMsgTypeInvalid,
}

/// The body of a IPCON message.
///
/// * msg_type  
///   * IpconMsgTypeNormal : a normal message.
///   * IpconMsgTypeGroup  : a multicast group message.
///   * IpconMsgTypeKevent : a IPCON kernel module message
///   * IpconMsgTypeInvalid: an invalid IPCON message
/// * peer  
///   The name of peer who sent this message.
/// * group  
///   The group of this message. It will be None if the message is not a multicast group message.
/// * buf  
///   Message content.
///
pub struct IpconMsgBody {
    pub msg_type: IpconMsgType,
    pub peer: String,
    pub group: Option<String>,
    pub buf: Vec<u8>,
}

/// IPCON message.
pub enum IpconMsg {
    IpconMsgUser(IpconMsgBody),
    IpconMsgKevent(IpconKevent),
    IpconMsgInvalid,
}

impl IpconMsg {
    pub fn from_libipcon_msg(msg: LibIpconMsg) -> Result<IpconMsg> {
        match msg.msg_type {
            LIBIPCON_MSG_TYPE_NORMAL => {
                let peer_name: String;

                unsafe {
                    peer_name = match CStr::from_ptr(&msg.peer as *const i8).to_str() {
                        Ok(p) => String::from(p),
                        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.to_string())),
                    };
                }

                let mut buf = Vec::<u8>::with_capacity(msg.len as usize);

                unsafe {
                    buf.set_len(msg.len as usize);
                    buf.copy_from_slice(&msg.u.buf[..msg.len as usize]);
                }

                let m = IpconMsgBody {
                    msg_type: IpconMsgType::IpconMsgTypeNormal,
                    peer: peer_name,
                    group: None,
                    buf,
                };

                Ok(IpconMsg::IpconMsgUser(m))
            }

            LIBIPCON_MSG_TYPE_GROUP => {
                let peer_name: String;
                let group_name: String;

                unsafe {
                    peer_name = match CStr::from_ptr(&msg.peer as *const i8).to_str() {
                        Ok(p) => String::from(p),
                        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.to_string())),
                    };

                    group_name = match CStr::from_ptr(&msg.group as *const i8).to_str() {
                        Ok(p) => String::from(p),
                        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.to_string())),
                    };
                }

                let mut  buf = Vec::<u8>::with_capacity(msg.len as usize);

                unsafe {
                    buf.set_len(msg.len as usize);
                    buf.copy_from_slice(&msg.u.buf[..msg.len as usize]);
                }

                let m = IpconMsgBody {
                    msg_type: IpconMsgType::IpconMsgTypeGroup,
                    peer: peer_name,
                    group: Some(group_name),
                    buf,
                };
                Ok(IpconMsg::IpconMsgUser(m))
            }

            LIBIPCON_MSG_TYPE_KEVENT => unsafe {
                Ok(IpconMsg::IpconMsgKevent(msg.u.kevent))
            },

            LIBIPCON_MSG_TYPE_INVALID => Ok(IpconMsg::IpconMsgInvalid),
            _ => Ok(IpconMsg::IpconMsgInvalid),
        }
    }
}
