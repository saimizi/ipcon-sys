use std::{error::Error, fmt::Display};

use error_stack::Report;

#[derive(Debug, Clone, Copy)]
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

impl From<Report<IpconError>> for IpconError {
    fn from(report: Report<IpconError>) -> Self {
        report.downcast_ref::<IpconError>().unwrap().to_owned()
    }
}

impl From<IpconError> for std::io::Error {
    fn from(e: IpconError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}

impl From<std::io::Error> for IpconError {
    fn from(e: std::io::Error) -> Self {
        match e.to_string().as_str() {
            "Invalid name" => IpconError::InvalidName,
            "Invalid Kevent" => IpconError::InvalidKevent,
            "Invalid data" => IpconError::InvalidData,
            "Invalid libipcon message" => IpconError::InvalidLibIpconMsg,
            "Timeout system error" => IpconError::SysErrorTimeOut,
            "Invalid value system error" => IpconError::SysErrorInvalidValue,
            "Permission denied system error" => IpconError::SysErrorPermission,
            "Other system error" => IpconError::SystemErrorOther,
            _ => IpconError::Unexpected,
        }
    }
}

impl Error for IpconError {}
