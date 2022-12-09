use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum IpconError {
    InvalidName,
    InvalidKevent,
    InvalidLibIpconMsg,
    InvalidData,
    SysErrorTimeOut,
    SysErrorInvalidValue,
    SysErrorPermission,
    SystemErrorOther,
    Unexpected,
}

impl Display for IpconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_str = match self {
            IpconError::InvalidName => "Invalid name",
            IpconError::InvalidKevent => "Invalid Kevent",
            IpconError::InvalidData => "Invalid data",
            IpconError::InvalidLibIpconMsg => "Invalid libipcon message",
            IpconError::SysErrorTimeOut => "Timeout system error",
            IpconError::SysErrorInvalidValue => "Invalid value system error",
            IpconError::SysErrorPermission => "Permission denied system error",
            IpconError::SystemErrorOther => "Other system error",
            _ => "Unexpected error",
        };

        write!(f, "{}", err_str)
    }
}

impl Error for IpconError {}
