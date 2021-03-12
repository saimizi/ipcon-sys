extern crate libc;
use crate::ipcon_msg::{IpconMsg, LibIpconMsg, IPCON_MAX_NAME_LEN, IPCON_MAX_PAYLOAD_LEN};
use crate::{error_str_result, Result};
use bytes::Bytes;
use libc::c_void;
use std::ffi::CString;
use std::os::raw::{c_char, c_uchar};

#[link(name = "ipcon")]
extern "C" {
    fn ipcon_create_handler(peer_name: *const c_char, flags: usize) -> *mut c_void;
    fn ipcon_free_handler(handler: *mut c_void);
    fn is_peer_present(handler: *mut c_void, peer: *const c_char) -> i32;
    fn is_group_present(handler: *mut c_void, peer: *const c_char, group: *const c_char) -> i32;
    fn ipcon_rcv(handler: *mut c_void, msg: &LibIpconMsg) -> i32;
    fn ipcon_send_unicast(handler: *mut c_void, peer: *const c_char, buf: *const c_uchar) -> i32;
}

pub struct Ipcon {
    handler: *mut c_void,
}

pub enum IpconFlag {
    IPFDisableKeventFilter = 1,
}

impl Ipcon {
    pub fn new(peer_name: &str, flag: Option<IpconFlag>) -> Option<Ipcon> {
        let pname = match CString::new(peer_name) {
            Ok(a) => a,
            Err(_) => return None,
        };

        let handler: *mut c_void;
        let mut flg = 0 as usize;

        if let Some(a) = flag {
            flg = a as usize;
        }

        unsafe {
            handler = ipcon_create_handler(pname.as_ptr(), flg as usize);
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
            let ret = is_peer_present(self.handler, p.as_ptr());
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
            let ret = is_group_present(self.handler, p.as_ptr(), g.as_ptr());
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
                return error_str_result(&format!("system error :{}", ret));
            }
        }

        IpconMsg::from_libipcon_msg(lmsg)
    }

    pub fn send_unicast_msg(&self, peer: &str, buf: Bytes) -> Result<()> {
        if peer.len() > IPCON_MAX_NAME_LEN {
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
            let ret = ipcon_send_unicast(self.handler, pname.into_raw(), buf.as_ptr());
            if ret < 0 {
                return error_str_result(&format!("system error :{}", ret));
            }
        }

        Ok(())
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
