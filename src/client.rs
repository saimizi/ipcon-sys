use crate::logger::{error_str_result, Error, Result};

use std::net::TcpStream;
use std::str;

#[derive(Debug)]
pub struct Client {
    stream: Option<TcpStream>,
}

impl Client {
    pub fn new<'a>() -> Result<'a, Client> {
        Ok(Client { stream: None })
    }

    pub fn stream_ref(&mut self) -> Result<&mut TcpStream> {
        let s = match &mut self.stream {
            Some(a) => a,
            None => return error_str_result("not connected."),
        };

        Ok(s)
    }

    pub fn connect(&mut self, ip_str: &str) -> Result<()> {
        if ip_str.is_empty() {
            return Err(Error::new_str_err("IP address is empty."));
        }

        let s: TcpStream;
        match TcpStream::connect(ip_str) {
            Ok(a) => s = a,
            Err(e) => return error_str_result(&format!("{}", e)),
        }
        self.stream = Some(s);
        Ok(())
    }
}
