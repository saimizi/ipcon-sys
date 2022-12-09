//! # ipcon-sys Module
//! IPCON (IPC Over Netlink) is a packet type IPC mechanism by using Netlink. A simple (but a
//! little old) explanation can be found at:
//!
//! <https://github.com/saimizi/libipcon/blob/master/Document/Tutorial.txt>.
//!
//! ipcon-sys provides the rust binding for it. Following are required to use this module:
//!
//! * libipcon library
//! * ipcon kernel module

pub mod ipcon;

#[cfg(feature = "async")]
pub mod ipcon_async;

pub mod ipcon_msg;

pub mod ipcon_error;
