extern crate libc;
use crate::ipcon_error::IpconError;
use crate::ipcon_msg::{IpconMsg, LibIpconMsg, IPCON_MAX_NAME_LEN, IPCON_MAX_PAYLOAD_LEN};
use error_stack::{IntoReport, Result, ResultExt};
#[allow(unused)]
use jlogger::{jdebug, jerror, jinfo, jwarn};
use libc::{c_void, size_t};
use nix::errno::Errno;
use std::ffi::CString;
use std::os::raw::{c_char, c_uchar};

#[link(name = "ipcon")]
extern "C" {
    fn ipcon_create_handler(peer_name: *const c_char, flags: usize) -> *mut c_void;
    fn ipcon_free_handler(handler: *mut c_void);
    fn is_peer_present(handler: *mut c_void, peer: *const c_char) -> i32;
    fn is_group_present(handler: *mut c_void, peer: *const c_char, group: *const c_char) -> i32;
    fn ipcon_rcv(handler: *mut c_void, msg: &LibIpconMsg) -> i32;
    fn ipcon_send_unicast(
        handler: *mut c_void,
        peer: *const c_char,
        buf: *const c_uchar,
        size: size_t,
    ) -> i32;
    fn ipcon_register_group(handler: *mut c_void, name: *const c_char) -> i32;
    fn ipcon_unregister_group(handler: *mut c_void, name: *const c_char) -> i32;
    fn ipcon_join_group(
        handler: *mut c_void,
        srv_name: *const c_char,
        grp_name: *const c_char,
    ) -> i32;
    fn ipcon_leave_group(
        handler: *mut c_void,
        srv_name: *const c_char,
        grp_name: *const c_char,
    ) -> i32;
    fn ipcon_send_multicast(
        handler: *mut c_void,
        name: *const c_char,
        buf: *const c_uchar,
        size: size_t,
        sync: i32,
    ) -> i32;
    fn ipcon_rcv_timeout(
        handler: *mut c_void,
        im: &LibIpconMsg,
        timeout: *const libc::timeval,
    ) -> i32;
    fn ipcon_get_read_fd(handler: *mut c_void) -> i32;
    fn ipcon_get_write_fd(handler: *mut c_void) -> i32;
    fn ipcon_get_ctrl_fd(handler: *mut c_void) -> i32;
}

/// IPCON peer.
pub struct Ipcon {
    handler: usize,
}

pub type IpconFlag = std::os::raw::c_ulong;
pub const IPF_DISABLE_KEVENT_FILTER: IpconFlag = 0x1 << 0;
pub const IPF_RCV_IF: IpconFlag = 0x1 << 1;
pub const IPF_SND_IF: IpconFlag = 0x1 << 2;
pub const IPF_DEFAULT: IpconFlag = IPF_RCV_IF | IPF_SND_IF;
pub const IPCON_KERNEL_NAME: &str = "ipcon";
pub const IPCON_KERNEL_GROUP_NAME: &str = "ipcon_kevent";

fn errno_to_error(i: i32) -> IpconError {
    let eno = Errno::from_i32(i.abs());
    match eno {
        Errno::ETIMEDOUT => IpconError::SysErrorTimeOut,
        Errno::EINVAL => IpconError::SysErrorInvalidValue,
        Errno::EPERM => IpconError::SysErrorPermission,
        _ => IpconError::SystemErrorOther,
    }
}

pub fn valid_name(name: &str) -> Result<(), IpconError> {
    let mut error_str = None;

    if name.is_empty() {
        error_str = Some("Name is null".to_owned());
    }

    if name.len() > IPCON_MAX_NAME_LEN {
        error_str = Some(format!(
            "Name is too long {} > {}",
            name.len(),
            IPCON_MAX_NAME_LEN
        ));
    }

    if name.trim() != name {
        error_str = Some("Name has blank character".to_owned());
    }

    if let Some(err_str) = error_str {
        Err(IpconError::InvalidName)
            .into_report()
            .attach_printable(err_str)
    } else {
        Ok(())
    }
}

impl Drop for Ipcon {
    fn drop(&mut self) {
        unsafe {
            ipcon_free_handler(Ipcon::to_handler(self.handler));
        }
    }
}

impl Ipcon {
    pub fn to_handler(u: usize) -> *mut c_void {
        u as *mut c_void
    }

    ///# Safety
    pub unsafe fn from_handler(h: *mut c_void) -> usize {
        h as usize
    }

    /// Create an IPCON peer.
    /// If the name is omitted, an anonymous will be created.
    /// Following flags can be specified with bitwise OR (|).
    /// * IPF_DISABLE_KEVENT_FILTER  
    ///   By default, IPCON kernel module will only delivery the add/remove notification of
    ///   peers and groups which are considered to be interested by the peer. If this flag is
    ///   enabled, all notification will be delivered by IPCON kernel module.
    /// * IPF_SND_IF  
    ///   Use message sending interface.
    /// * IPF_RCV_IF  
    ///   Use message receiving interface.
    /// * IPF_DEFAULT  
    ///   This is same to IPF_RCV_IF | IPF_SND_IF.
    ///
    ///   
    pub fn new(peer_name: Option<&str>, flag: Option<IpconFlag>) -> Result<Ipcon, IpconError> {
        let handler: *mut c_void;
        let mut flg = 0_usize;

        if let Some(a) = flag {
            flg = a as usize;
        }

        let pname = match peer_name {
            Some(a) => {
                valid_name(a)?;
                CString::new(a)
                    .into_report()
                    .change_context(IpconError::InvalidName)?
                    .into_raw()
            }
            None => std::ptr::null(),
        };

        unsafe {
            handler = ipcon_create_handler(pname as *const c_char, flg as usize);

            if !pname.is_null() {
                /* deallocate the pname */
                let _ = CString::from_raw(pname as *mut c_char);
            }
        }
        if handler.is_null() {
            Err(IpconError::SystemErrorOther)
                .into_report()
                .attach_printable("Failed to create ipcon handler")
        } else {
            Ok(Ipcon {
                handler: unsafe { Ipcon::from_handler(handler) },
            })
        }
    }

    /// Retrieve netlink socket file descriptor of message receiving interface.
    pub fn get_read_fd(&self) -> Result<i32, IpconError> {
        unsafe {
            let fd = ipcon_get_read_fd(Ipcon::to_handler(self.handler));
            if fd < 0 {
                Err(errno_to_error(fd))
                    .into_report()
                    .attach_printable(format!("ipcon_get_read_fd() error: {}", fd))
            } else {
                Ok(fd)
            }
        }
    }

    /// Retrieve netlink socket file descriptor of message sending interface.
    pub fn get_write_fd(&self) -> Result<i32, IpconError> {
        unsafe {
            let fd = ipcon_get_write_fd(Ipcon::to_handler(self.handler));
            if fd < 0 {
                Err(errno_to_error(fd))
                    .into_report()
                    .attach_printable(format!("ipcon_get_write_fd() error: {}", fd))
            } else {
                Ok(fd)
            }
        }
    }

    /// Retrieve netlink socket file descriptor of control interface.
    pub fn get_ctrl_fd(&self) -> Result<i32, IpconError> {
        unsafe {
            let fd = ipcon_get_ctrl_fd(Ipcon::to_handler(self.handler));
            if fd < 0 {
                Err(errno_to_error(fd))
                    .into_report()
                    .attach_printable(format!("ipcon_get_write_fd() error: {}", fd))
            } else {
                Ok(fd)
            }
        }
    }

    /// Inquiry whether a peer is present.
    pub fn is_peer_present(&self, peer: &str) -> bool {
        let mut present = false;
        let p = match CString::new(peer) {
            Ok(a) => a,
            Err(_) => return false,
        };

        unsafe {
            let ptr = p.into_raw();
            let ret = is_peer_present(Ipcon::to_handler(self.handler), ptr as *const c_char);
            if ret != 0 {
                present = true;
            }

            let _ = CString::from_raw(ptr);
        }

        present
    }

    /// Inquiry whether the group of a peer is present.
    pub fn is_group_present(&self, peer: &str, group: &str) -> bool {
        let mut present = false;
        let p = match CString::new(peer) {
            Ok(a) => a,
            Err(_) => return false,
        };

        let g = match CString::new(group) {
            Ok(a) => a,
            Err(_) => return false,
        };

        unsafe {
            let ptr = p.into_raw();
            let pgtr = g.into_raw();
            let ret = is_group_present(
                Ipcon::to_handler(self.handler),
                ptr as *const c_char,
                pgtr as *const c_char,
            );
            let _ = CString::from_raw(ptr);
            let _ = CString::from_raw(pgtr);

            if ret != 0 {
                present = true;
            }
        }

        present
    }

    /// Receive IPCON message.
    /// This function will fail if the peer doesn't enable IPF_RCV_IF.
    pub fn receive_msg(&self) -> Result<IpconMsg, IpconError> {
        let lmsg = LibIpconMsg::new();

        unsafe {
            let ret = ipcon_rcv(Ipcon::to_handler(self.handler), &lmsg);
            if ret < 0 {
                return Err(errno_to_error(ret))
                    .into_report()
                    .attach_printable(format!("ipcon_rcv() error: {}", ret));
            }
        }

        IpconMsg::from_libipcon_msg(lmsg)
    }

    /// Send an unicast IPCON message to a specific peer.
    /// This function will fail if the peer doesn't enable IPF_SND_IF.
    pub fn send_unicast_msg(&self, peer: &str, buf: &[u8]) -> Result<(), IpconError> {
        self.send_unicast_msg_by_ref(peer, buf)
    }

    /// Send an unicast IPCON message to a specific peer.
    /// This function will fail if the peer doesn't enable IPF_SND_IF.
    pub fn send_unicast_msg_by_ref(&self, peer: &str, buf: &[u8]) -> Result<(), IpconError> {
        valid_name(peer)?;

        if buf.len() > IPCON_MAX_PAYLOAD_LEN {
            return Err(IpconError::InvalidData)
                .into_report()
                .attach_printable(format!(
                    "Buffer length is to large {} > {}",
                    buf.len(),
                    IPCON_MAX_PAYLOAD_LEN
                ));
        }

        let pname = CString::new(peer)
            .into_report()
            .change_context(IpconError::InvalidData)?;

        unsafe {
            let ptr = pname.into_raw();
            let ret = ipcon_send_unicast(
                Ipcon::to_handler(self.handler),
                ptr as *const c_char,
                buf.as_ptr(),
                buf.len() as size_t,
            );

            let _ = CString::from_raw(ptr);

            if ret < 0 {
                return Err(errno_to_error(ret))
                    .into_report()
                    .attach_printable(format!("ipcon_send_unicast() error: {}", ret));
            }
        }

        Ok(())
    }

    /// Register a multicast group.
    pub fn register_group(&self, group: &str) -> Result<(), IpconError> {
        valid_name(group).attach_printable("register_group error: invalid group name")?;

        let g = CString::new(group)
            .into_report()
            .change_context(IpconError::InvalidName)?;

        unsafe {
            let ptr = g.into_raw();
            let ret = ipcon_register_group(Ipcon::to_handler(self.handler), ptr as *const c_char);
            let _ = CString::from_raw(ptr);
            if ret < 0 {
                return Err(errno_to_error(ret))
                    .into_report()
                    .attach_printable(format!("ipcon_get_read_fd() error: {}", ret));
            }
        }

        Ok(())
    }

    /// Unregister a multicast group.
    pub fn unregister_group(&self, group: &str) -> Result<(), IpconError> {
        valid_name(group)?;

        let g = CString::new(group)
            .into_report()
            .change_context(IpconError::InvalidName)?;

        unsafe {
            let ptr = g.into_raw();
            let ret = ipcon_unregister_group(Ipcon::to_handler(self.handler), ptr as *const c_char);
            let _ = CString::from_raw(ptr);
            if ret < 0 {
                return Err(errno_to_error(ret))
                    .into_report()
                    .attach_printable(format!("ipcon_unregister_group() error: {}", ret));
            }
        }

        Ok(())
    }

    /// Subscribe a multicast group of a peer.
    pub fn join_group(&self, peer: &str, group: &str) -> Result<(), IpconError> {
        valid_name(peer)?;
        valid_name(group)?;

        let p = CString::new(peer)
            .into_report()
            .change_context(IpconError::InvalidName)?;

        let g = CString::new(group)
            .into_report()
            .change_context(IpconError::InvalidName)?;

        unsafe {
            let ptr = p.into_raw();
            let pgtr = g.into_raw();
            let ret = ipcon_join_group(
                Ipcon::to_handler(self.handler),
                ptr as *const c_char,
                pgtr as *const c_char,
            );
            let _ = CString::from_raw(ptr);
            let _ = CString::from_raw(pgtr);
            if ret < 0 {
                return Err(errno_to_error(ret))
                    .into_report()
                    .attach_printable(format!("ipcon_join_group() error: {}", ret));
            }
        }

        Ok(())
    }

    /// Unsubscribe a multicast group of a peer.
    pub fn leave_group(&self, peer: &str, group: &str) -> Result<(), IpconError> {
        valid_name(peer)?;
        valid_name(group)?;

        let p = CString::new(peer)
            .into_report()
            .change_context(IpconError::InvalidName)?;

        let g = CString::new(group)
            .into_report()
            .change_context(IpconError::InvalidName)?;

        unsafe {
            let ptr = p.into_raw();
            let pgtr = g.into_raw();
            let ret = ipcon_leave_group(
                Ipcon::to_handler(self.handler),
                ptr as *const c_char,
                pgtr as *const c_char,
            );
            let _ = CString::from_raw(ptr);
            let _ = CString::from_raw(pgtr);

            if ret < 0 {
                return Err(errno_to_error(ret))
                    .into_report()
                    .attach_printable(format!("ipcon_leave_group() error: {}", ret));
            }
        }

        Ok(())
    }

    /// Send multicast messages to an owned group.
    pub fn send_multicast(&self, group: &str, buf: &[u8], sync: bool) -> Result<(), IpconError> {
        self.send_multicast_by_ref(group, buf, sync)
    }

    /// Send multicast messages to an owned group.
    pub fn send_multicast_by_ref(
        &self,
        group: &str,
        buf: &[u8],
        sync: bool,
    ) -> Result<(), IpconError> {
        valid_name(group)?;

        if buf.len() > IPCON_MAX_PAYLOAD_LEN {
            return Err(IpconError::InvalidData)
                .into_report()
                .attach_printable(format!(
                    "Buffer length is too large {} > {}",
                    buf.len(),
                    IPCON_MAX_PAYLOAD_LEN,
                ));
        }

        let g = CString::new(group)
            .into_report()
            .change_context(IpconError::InvalidName)?;

        let mut s: i32 = 0;
        if sync {
            s = 1;
        }

        unsafe {
            let pgtr = g.into_raw();
            let ret = ipcon_send_multicast(
                Ipcon::to_handler(self.handler),
                pgtr as *const c_char,
                buf.as_ptr(),
                buf.len() as size_t,
                s,
            );
            let _ = CString::from_raw(pgtr);

            if ret < 0 {
                return Err(errno_to_error(ret))
                    .into_report()
                    .attach_printable(format!("ipcon_send_multicast() error: {}", ret));
            }
        }

        Ok(())
    }

    /// Receiving message with timeout.
    /// receive_msg() will block until a message come. receive_msg_timeout() adds a timeout to
    /// it.The timeout is specified with seconds and microseconds.
    pub fn receive_msg_timeout(&self, tv_sec: u32, tv_usec: u32) -> Result<IpconMsg, IpconError> {
        let lmsg = LibIpconMsg::new();
        let t = libc::timeval {
            tv_sec: tv_sec as libc::time_t,
            tv_usec: tv_usec as libc::suseconds_t,
        };

        unsafe {
            let ret = ipcon_rcv_timeout(Ipcon::to_handler(self.handler), &lmsg, &t);
            if ret < 0 {
                return Err(errno_to_error(ret))
                    .into_report()
                    .attach_printable(format!("ipcon_rcv_timeout() error: {}", ret));
            }
        }

        IpconMsg::from_libipcon_msg(lmsg)
    }

    /// Receiving message without block.
    /// This is same to receive_msg_timeout(0, 0);
    pub fn receive_msg_nonblock(&self) -> Result<IpconMsg, IpconError> {
        self.receive_msg_timeout(0, 0)
    }
}
