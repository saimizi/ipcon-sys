use std::fmt;
//use crate::{debug, error, info};
use bytes::Bytes;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Data {
    i: usize,
    d: usize,
}

impl Data {
    fn is_zero(&self) -> bool {
        self.i == 0 && self.d == 0
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "{}", format!("{}.{}", self.i, self.d)) }
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct CpuReport {
    usr: Data,
    sys: Data,
    nic: Data,
    idle: Data,
    io: Data,
    irq: Data,
    sirq: Data,
}

impl fmt::Display for CpuReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s: Vec<String> = Vec::new();

        if !self.usr.is_zero() {
            s.push(format!("{} us,", self.usr.to_string()));
        }

        if !self.sys.is_zero() {
            s.push(format!("{} sy,", self.sys.to_string()));
        }

        if !self.nic.is_zero() {
            s.push(format!("{} ni,", self.nic.to_string()));
        }

        if !self.idle.is_zero() {
            s.push(format!("{} id,", self.idle.to_string()));
        }

        if !self.io.is_zero() {
            s.push(format!("{} io,", self.io.to_string()));
        }

        if !self.irq.is_zero() {
            s.push(format!("{} irq,", self.irq.to_string()));
        }

        if !self.sirq.is_zero() {
            s.push(format!("{} sirq,", self.sirq.to_string()));
        }

        write!(f, "{}", format!("CPU(%): {}", s.join(" ")))
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct MemoryReport {
    free: Data,
    anon: Data,
    file: Data,
    avail: Data,
    buffer: Data,
    cached: Data,
    unevictable: Data,
    cmatotal: Data,
    cmafree: Data,
    slab: Data,
    slab_rec: Data,
    slab_unrec: Data,
    shmem: Data,
}

impl fmt::Display for MemoryReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s: Vec<String> = Vec::new();

        if !self.free.is_zero() {
            s.push(format!("{} free,", self.free.to_string()));
        }

        if !self.avail.is_zero() {
            s.push(format!("{} avail,", self.avail.to_string()));
        }

        if !self.anon.is_zero() {
            s.push(format!("{} anon,", self.anon.to_string()));
        }

        if !self.file.is_zero() {
            s.push(format!("{} file,", self.file.to_string()));
        }

        if !self.buffer.is_zero() {
            s.push(format!("{} buffer,", self.buffer.to_string()));
        }

        if !self.cached.is_zero() {
            s.push(format!("{} cached,", self.cached.to_string()));
        }

        if !self.unevictable.is_zero() {
            s.push(format!("{} unevict,", self.unevictable.to_string()));
        }

        if !self.cmatotal.is_zero() {
            s.push(format!("{} cma,", self.cmatotal.to_string()));
        }

        if !self.cmafree.is_zero() {
            s.push(format!("{} cmafree,", self.cmafree.to_string()));
        }

        if !self.slab.is_zero() {
            s.push(format!("{} slab,", self.slab.to_string()));
        }

        if !self.slab_unrec.is_zero() {
            s.push(format!("{} slab_unrec,", self.slab_unrec.to_string()));
        }

        if !self.shmem.is_zero() {
            s.push(format!("{} shmem,", self.shmem.to_string()));
        }

        write!(f, "{}", format!("Memory(%): {}", s.join(" ")))
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct InfomgrReport {
    cpu: CpuReport,
    memory: MemoryReport,
}

impl InfomgrReport {
    pub const PEER_NAME: &'static str = "infomgr";
    pub const GROUP_NAME: &'static str = "infomgr_group";
    pub fn new(buf: &Bytes) -> InfomgrReport {
        let v = buf.to_vec();
        let (_head, body, _tail) = unsafe { v.align_to::<InfomgrReport>() };
        body[0]
    }
}

impl fmt::Display for InfomgrReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{}", self.cpu.to_string(), self.memory.to_string())
    }
}
