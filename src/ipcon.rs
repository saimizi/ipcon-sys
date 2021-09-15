extern crate libc;
use crate::error;
use crate::ipcon_msg::{IpconMsg, LibIpconMsg, IPCON_MAX_NAME_LEN, IPCON_MAX_PAYLOAD_LEN};
use bytes::Bytes;
use libc::{c_void, size_t};
use nix::errno::Errno;
use std::ffi::CString;
use std::io::{Error, ErrorKind, Result};
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
        srvname: *const c_char,
        grpname: *const c_char,
    ) -> i32;
    fn ipcon_leave_group(
        handler: *mut c_void,
        srvname: *const c_char,
        grpname: *const c_char,
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
}

pub struct Ipcon {
    handler: *mut c_void,
}

pub type IpconFlag = std::os::raw::c_ulong;
pub const IPF_DISABLE_KEVENT_FILTER: IpconFlag = 0x1 << 0;
pub const IPF_RCV_IF: IpconFlag = 0x1 << 1;
pub const IPF_SND_IF: IpconFlag = 0x1 << 2;
pub const IPF_DEFULT: IpconFlag = IPF_RCV_IF | IPF_SND_IF;

fn errno_to_error(i: i32) -> Error {
    let eno = Errno::from_i32(i.abs());
    match eno {
        Errno::ETIMEDOUT => Error::new(ErrorKind::TimedOut, eno.desc()),
        Errno::EINVAL => Error::new(ErrorKind::InvalidInput, eno.desc()),
        Errno::EPERM => Error::new(ErrorKind::PermissionDenied, eno.desc()),
        _ => Error::new(ErrorKind::Other, eno.desc()),
    }
}

impl Ipcon {
    pub const IPCON_KERNEL_NAME: &'static str = "ipcon";
    pub const IPCON_KERNEL_GROUP_NAME: &'static str = "ipcon_kevent";

    fn valid_name(peer_name: &str) -> bool {
        if peer_name.is_empty() {
            return false;
        }

        if peer_name.len() > IPCON_MAX_NAME_LEN {
            return false;
        }

        if peer_name.trim() != peer_name {
            return false;
        }

        true
    }
    pub fn new(peer_name: Option<&str>, flag: Option<IpconFlag>) -> Option<Ipcon> {
        let handler: *mut c_void;
        let mut flg = 0 as usize;

        if let Some(a) = flag {
            flg = a as usize;
        }

        let pname = match peer_name {
            Some(a) => {
                if !Ipcon::valid_name(a) {
                    error!("Ipcon::new() : Invalid peer name.");
                    return None;
                }

                match CString::new(a) {
                    Ok(a) => a.into_raw(),
                    Err(_) => return None,
                }
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
            None
        } else {
            Some(Ipcon { handler })
        }
    }

    pub fn free(self) {
        unsafe {
            ipcon_free_handler(self.handler);
        }
    }

    pub fn is_peer_present(&self, peer: &str) -> bool {
        let mut present = false;
        let p = match CString::new(peer) {
            Ok(a) => a,
            Err(_) => return false,
        };

        unsafe {
            let ptr = p.into_raw();
            let ret = is_peer_present(self.handler, ptr as *const c_char);
            if ret != 0 {
                present = true;
            }

            let _ = CString::from_raw(ptr);
        }

        present
    }

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
            let ret = is_group_present(self.handler, ptr as *const c_char, pgtr as *const c_char);
            let _ = CString::from_raw(ptr);
            let _ = CString::from_raw(pgtr);

            if ret != 0 {
                present = true;
            }
        }

        present
    }

    pub fn receive_msg(&self) -> Result<IpconMsg> {
        let lmsg = LibIpconMsg::new();

        unsafe {
            let ret = ipcon_rcv(self.handler, &lmsg);
            if ret < 0 {
                return Err(errno_to_error(ret));
            }
        }

        IpconMsg::from_libipcon_msg(lmsg)
    }

    pub fn send_unicast_msg(&self, peer: &str, buf: Bytes) -> Result<()> {
        if !Ipcon::valid_name(peer) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Name is too long > {}", IPCON_MAX_NAME_LEN),
            ));
        }

        if buf.len() > IPCON_MAX_PAYLOAD_LEN {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Data is too long > {}", IPCON_MAX_PAYLOAD_LEN),
            ));
        }

        let pname = match CString::new(peer) {
            Ok(s) => s,
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.to_string())),
        };

        unsafe {
            let ptr = pname.into_raw();
            let ret = ipcon_send_unicast(
                self.handler,
                ptr as *const c_char,
                buf.as_ptr(),
                buf.len() as size_t,
            );

            let _ = CString::from_raw(ptr);

            if ret < 0 {
                return Err(errno_to_error(ret));
            }
        }

        Ok(())
    }

    pub fn register_group(&self, group: &str) -> Result<()> {
        if !Ipcon::valid_name(group) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid group name".to_string(),
            ));
        }

        let g = match CString::new(group) {
            Ok(a) => a,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        unsafe {
            let ptr = g.into_raw();
            let ret = ipcon_register_group(self.handler, ptr as *const c_char);
            let _ = CString::from_raw(ptr);
            if ret < 0 {
                return Err(errno_to_error(ret));
            }
        }

        Ok(())
    }

    pub fn unregister_group(&self, group: &str) -> Result<()> {
        if !Ipcon::valid_name(group) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid group name".to_string(),
            ));
        }

        let g = match CString::new(group) {
            Ok(a) => a,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        unsafe {
            let ptr = g.into_raw();
            let ret = ipcon_unregister_group(self.handler, ptr as *const c_char);
            let _ = CString::from_raw(ptr);
            if ret < 0 {
                return Err(errno_to_error(ret));
            }
        }

        Ok(())
    }

    pub fn join_group(&self, peer: &str, group: &str) -> Result<()> {
        if !Ipcon::valid_name(peer) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid peer name".to_string(),
            ));
        }

        if !Ipcon::valid_name(group) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid group name".to_string(),
            ));
        }

        let p = match CString::new(peer) {
            Ok(a) => a,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        let g = match CString::new(group) {
            Ok(a) => a,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        unsafe {
            let ptr = p.into_raw();
            let pgtr = g.into_raw();
            let ret = ipcon_join_group(self.handler, ptr as *const c_char, pgtr as *const c_char);
            let _ = CString::from_raw(ptr);
            let _ = CString::from_raw(pgtr);
            if ret < 0 {
                return Err(errno_to_error(ret));
            }
        }

        Ok(())
    }

    pub fn leave_group(&self, peer: &str, group: &str) -> Result<()> {
        if !Ipcon::valid_name(peer) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid peer name".to_string(),
            ));
        }

        if !Ipcon::valid_name(group) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Invalid group name".to_string(),
            ));
        }

        let p = match CString::new(peer) {
            Ok(a) => a,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        let g = match CString::new(group) {
            Ok(a) => a,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        unsafe {
            let ptr = p.into_raw();
            let pgtr = g.into_raw();
            let ret = ipcon_leave_group(self.handler, ptr as *const c_char, pgtr as *const c_char);
            let _ = CString::from_raw(ptr);
            let _ = CString::from_raw(pgtr);

            if ret < 0 {
                return Err(errno_to_error(ret));
            }
        }

        Ok(())
    }

    pub fn send_multicast(&self, group: &str, buf: Bytes, sync: bool) -> Result<()> {
        if !Ipcon::valid_name(group) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Name is too long > {}", IPCON_MAX_NAME_LEN),
            ));
        }

        if buf.len() > IPCON_MAX_PAYLOAD_LEN {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Data is too long > {}", IPCON_MAX_PAYLOAD_LEN),
            ));
        }

        let g = match CString::new(group) {
            Ok(s) => s,
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        let mut s: i32 = 0;
        if sync {
            s = 1;
        }

        unsafe {
            let pgtr = g.into_raw();
            let ret = ipcon_send_multicast(
                self.handler,
                pgtr as *const c_char,
                buf.as_ptr(),
                buf.len() as size_t,
                s,
            );
            let _ = CString::from_raw(pgtr);

            if ret < 0 {
                return Err(errno_to_error(ret));
            }
        }

        Ok(())
    }

    pub fn receive_msg_timeout(&self, tv_sec: u32, tv_usec: u32) -> Result<IpconMsg> {
        let lmsg = LibIpconMsg::new();
        let t = libc::timeval {
            tv_sec: tv_sec as libc::time_t,
            tv_usec: tv_usec as libc::suseconds_t,
        };

        unsafe {
            let ret = ipcon_rcv_timeout(self.handler, &lmsg, &t);
            if ret < 0 {
                return Err(errno_to_error(ret));
            }
        }

        IpconMsg::from_libipcon_msg(lmsg)
    }

    pub fn receive_msg_nonblock(&self) -> Result<IpconMsg> {
        self.receive_msg_timeout(0, 0)
    }
}
