extern crate libc;
use crate::ipcon_msg::{IpconMsg, LibIpconMsg, IPCON_MAX_NAME_LEN, IPCON_MAX_PAYLOAD_LEN};
use crate::{debug, error};
use crate::{error_code_result, error_result, error_str_result, Result};
use bytes::Bytes;
use libc::{c_void, size_t};
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

pub enum IpconFlag {
    IPFDisableKeventFilter = 1,
}

impl Ipcon {
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

        let pname: *const c_char;
        pname = match peer_name {
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
            handler = ipcon_create_handler(pname, flg as usize);
        }
        if handler.is_null() {
            None
        } else {
            Some(Ipcon { handler: handler })
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
            let ret = is_peer_present(self.handler, p.into_raw());
            if ret != 0 {
                present = true;
            }
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
            let ret = is_group_present(self.handler, p.into_raw(), g.into_raw());
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
                return error_result(ret, Some(String::from("System error.")));
            }
        }

        IpconMsg::from_libipcon_msg(lmsg)
    }

    pub fn send_unicast_msg(&self, peer: &str, buf: Bytes) -> Result<()> {
        if !Ipcon::valid_name(peer) {
            return error_str_result(&format!("Name is too long > {}", IPCON_MAX_NAME_LEN));
        }

        if buf.len() > IPCON_MAX_PAYLOAD_LEN {
            return error_str_result(&format!("Data is too long > {}", IPCON_MAX_PAYLOAD_LEN));
        }

        let pname = match CString::new(peer) {
            Ok(s) => s,
            Err(_) => return error_str_result("Invalid peer"),
        };

        unsafe {
            let ret = ipcon_send_unicast(
                self.handler,
                pname.into_raw(),
                buf.as_ptr(),
                buf.len() as size_t,
            );

            if ret < 0 {
                return error_result(ret, Some(String::from("System error.")));
            }
        }

        Ok(())
    }

    pub fn register_group(&self, group: &str) -> Result<()> {
        if !Ipcon::valid_name(group) {
            return error_str_result("Invalid group name");
        }

        let g = match CString::new(group) {
            Ok(a) => a,
            Err(e) => return error_str_result(&format!("{}", e)),
        };

        unsafe {
            let ret = ipcon_register_group(self.handler, g.into_raw());
            if ret < 0 {
                return error_result(ret, Some(String::from("System error.")));
            }
        }

        Ok(())
    }

    pub fn unregister_group(&self, group: &str) -> Result<()> {
        if !Ipcon::valid_name(group) {
            return error_str_result("Invalid group name");
        }

        let g = match CString::new(group) {
            Ok(a) => a,
            Err(e) => return error_str_result(&format!("{}", e)),
        };

        unsafe {
            let ret = ipcon_unregister_group(self.handler, g.into_raw());
            if ret < 0 {
                return error_str_result(&format!("system error :{}", ret));
            }
        }

        Ok(())
    }

    pub fn join_group(&self, peer: &str, group: &str) -> Result<()> {
        if !Ipcon::valid_name(peer) {
            return error_str_result("Invalid peer name");
        }

        if !Ipcon::valid_name(group) {
            return error_str_result("Invalid group name");
        }

        let p = match CString::new(peer) {
            Ok(a) => a,
            Err(e) => return error_str_result(&format!("{}", e)),
        };

        let g = match CString::new(group) {
            Ok(a) => a,
            Err(e) => return error_str_result(&format!("{}", e)),
        };

        unsafe {
            let ret = ipcon_join_group(self.handler, p.into_raw(), g.into_raw());
            if ret < 0 {
                return error_result(ret, Some(String::from("System error.")));
            }
        }

        Ok(())
    }

    pub fn leave_group(&self, peer: &str, group: &str) -> Result<()> {
        if !Ipcon::valid_name(peer) {
            return error_str_result("Invalid peer name");
        }

        if !Ipcon::valid_name(group) {
            return error_str_result("Invalid group name");
        }

        let p = match CString::new(peer) {
            Ok(a) => a,
            Err(e) => return error_str_result(&format!("{}", e)),
        };

        let g = match CString::new(group) {
            Ok(a) => a,
            Err(e) => return error_str_result(&format!("{}", e)),
        };

        unsafe {
            let ret = ipcon_leave_group(self.handler, p.into_raw(), g.into_raw());
            if ret < 0 {
                return error_result(ret, Some(String::from("System error.")));
            }
        }

        Ok(())
    }

    pub fn send_multicast(&self, group: &str, buf: Bytes, sync: bool) -> Result<()> {
        if !Ipcon::valid_name(group) {
            return error_str_result(&format!("Name is too long > {}", IPCON_MAX_NAME_LEN));
        }

        if buf.len() > IPCON_MAX_PAYLOAD_LEN {
            return error_str_result(&format!("Data is too long > {}", IPCON_MAX_PAYLOAD_LEN));
        }

        let g = match CString::new(group) {
            Ok(s) => s,
            Err(_) => return error_str_result("Invalid group"),
        };

        let mut s: i32 = 0;
        if sync {
            s = 1;
        }

        unsafe {
            let ret = ipcon_send_multicast(
                self.handler,
                g.into_raw(),
                buf.as_ptr(),
                buf.len() as size_t,
                s,
            );

            if ret < 0 {
                return error_result(ret, Some(String::from("System error.")));
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
                if ret == -libc::ETIMEDOUT {
                    return error_result(ret, Some(String::from("Receive message timetout.")));
                }
                return error_result(ret, Some(String::from("System error.")));
            }
        }

        IpconMsg::from_libipcon_msg(lmsg)
    }

    pub fn receive_msg_nonblock(&self) -> Result<IpconMsg> {
        self.receive_msg_timeout(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::ipcon::{Ipcon, IpconFlag};
    #[test]
    fn it_works() {
        let ipcon = Ipcon::new("test", Some(IpconFlag::IPFDisableKeventFilter))
            .expect("failed to create ipcon handler");
        ipcon.free();
    }
}
