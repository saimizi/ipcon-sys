use crate::ipcon_error::IpconError;
use error_stack::{IntoReport, Result, ResultExt};
use std::ffi::CStr;
use std::fmt;
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

fn c_str_name(name: &[std::os::raw::c_char; IPCON_MAX_NAME_LEN]) -> Result<&str, IpconError> {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        CStr::from_ptr(name as *const i8)
            .to_str()
            .into_report()
            .change_context(IpconError::InvalidName)
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        CStr::from_ptr(name as *const u8)
            .to_str()
            .into_report()
            .change_context(IpconError::InvalidName)
    }
}

/// IpconKevent is a group message delivered from the IPCON_KERNEL_GROUP_NAME group of IPCON
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
    pub fn get_string(&self) -> Result<String, IpconError> {
        let result = match self.ke_type {
            IPCON_KEVENT_TYPE_PEER_ADD => unsafe {
                format!("peer {} added", c_str_name(&self.u.peer.peer_name)?)
            },

            IPCON_KEVENT_TYPE_PEER_REMOVE => unsafe {
                format!("peer {} removed", c_str_name(&self.u.peer.peer_name)?)
            },
            IPCON_KEVENT_TYPE_GROUP_ADD => unsafe {
                format!(
                    "group {}@{} added",
                    c_str_name(&self.u.group.group_name)?,
                    c_str_name(&self.u.group.peer_name)?
                )
            },

            IPCON_KEVENT_TYPE_GROUP_REMOVE => unsafe {
                format!(
                    "group {}@{} removed",
                    c_str_name(&self.u.group.group_name)?,
                    c_str_name(&self.u.group.peer_name)?
                )
            },
            _ => {
                return Err(IpconError::InvalidKevent)
                    .into_report()
                    .attach_printable(format!("Invalid kevent type {}", self.ke_type))
            }
        };

        Ok(result)
    }

    /// Get the name of peer newly added.
    /// IPCON kernel module will not delivery this event of an anonymous peer.
    pub fn peer_added(&self) -> Option<String> {
        match self.ke_type {
            IPCON_KEVENT_TYPE_PEER_ADD => unsafe {
                c_str_name(&self.u.peer.peer_name).map_or(None, |name| Some(name.to_owned()))
            },
            _ => None,
        }
    }

    /// Get the name of peer removed.
    /// IPCON kernel module will not delivery this event of an anonymous peer.
    pub fn peer_removed(&self) -> Option<String> {
        match self.ke_type {
            IPCON_KEVENT_TYPE_PEER_REMOVE => unsafe {
                c_str_name(&self.u.peer.peer_name).map_or(None, |name| Some(name.to_owned()))
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
                if let (Ok(peer_name), Ok(group_name)) = (
                    c_str_name(&self.u.group.peer_name),
                    c_str_name(&self.u.group.group_name),
                ) {
                    Some((peer_name.to_owned(), group_name.to_owned()))
                } else {
                    None
                }
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
                if let (Ok(peer_name), Ok(group_name)) = (
                    c_str_name(&self.u.group.peer_name),
                    c_str_name(&self.u.group.group_name),
                ) {
                    Some((peer_name.to_owned(), group_name.to_owned()))
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}

impl fmt::Display for IpconKevent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_string().unwrap_or_else(|e| e.to_string()))
    }
}

pub type LibIpconMsgType = std::os::raw::c_int;
pub const LIBIPCON_MSG_TYPE_NORMAL: LibIpconMsgType = 0;
pub const LIBIPCON_MSG_TYPE_GROUP: LibIpconMsgType = 1;
pub const LIBIPCON_MSG_TYPE_KEVENT: LibIpconMsgType = 2;
pub const LIBIPCON_MSG_TYPE_INVALID: LibIpconMsgType = 3;

#[repr(C)]
#[derive(Clone, Copy)]
pub union IpconMsgUnion {
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
    u: IpconMsgUnion,
}

impl LibIpconMsg {
    pub fn new() -> LibIpconMsg {
        LibIpconMsg {
            msg_type: LIBIPCON_MSG_TYPE_INVALID,
            peer: [0; IPCON_MAX_NAME_LEN],
            group: [0; IPCON_MAX_NAME_LEN],
            len: 0,
            u: IpconMsgUnion {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl From<LibIpconMsg> for Result<IpconMsg, IpconError> {
    fn from(msg: LibIpconMsg) -> Self {
        match msg.msg_type {
            LIBIPCON_MSG_TYPE_NORMAL => {
                let m = IpconMsgBody {
                    msg_type: IpconMsgType::IpconMsgTypeNormal,
                    peer: c_str_name(&msg.peer)?.to_owned(),
                    group: None,
                    buf: unsafe { msg.u.buf[..msg.len as usize].to_vec() },
                };

                Ok(IpconMsg::IpconMsgUser(m))
            }

            LIBIPCON_MSG_TYPE_GROUP => {
                let m = IpconMsgBody {
                    msg_type: IpconMsgType::IpconMsgTypeGroup,
                    peer: c_str_name(&msg.peer)?.to_owned(),
                    group: Some(c_str_name(&msg.group)?.to_owned()),
                    buf: unsafe { msg.u.buf[..msg.len as usize].to_vec() },
                };
                Ok(IpconMsg::IpconMsgUser(m))
            }

            LIBIPCON_MSG_TYPE_KEVENT => unsafe { Ok(IpconMsg::IpconMsgKevent(msg.u.kevent)) },
            LIBIPCON_MSG_TYPE_INVALID => Ok(IpconMsg::IpconMsgInvalid),
            _ => Ok(IpconMsg::IpconMsgInvalid),
        }
    }
}
