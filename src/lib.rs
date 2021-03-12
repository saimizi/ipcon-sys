pub mod ipcon;
pub mod ipcon_msg;
pub mod logger;

use std::fmt;
pub type Result<'a, T> = std::result::Result<T, Error>;

pub fn error_result<T>(err_code: i32, err_str: Option<String>) -> Result<'static, T> {
    Err(Error {
        err_code: err_code,
        err_str: err_str,
    })
}

pub fn error_str_result<T>(err_str: &str) -> Result<T> {
    Err(Error {
        err_code: Error::COMMON_ERROR,
        err_str: Some(String::from(err_str)),
    })
}

pub fn error_code_result<T>(err_code: i32) -> Result<'static, T> {
    Err(Error {
        err_code: err_code,
        err_str: None,
    })
}

pub struct Error {
    err_code: i32,
    err_str: Option<String>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.err_str {
            Some(s) => write!(f, "{}({})", s, self.err_code),
            None => write!(f, "{}", self.err_code()),
        }?;

        Ok(())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.err_str {
            Some(s) => write!(f, "{}({})", s, self.err_code),
            None => write!(f, "{}", self.err_code()),
        }?;

        Ok(())
    }
}

impl Error {
    pub const COMMON_ERROR: i32 = -1;
    pub fn new(code: i32, err_str: Option<String>) -> Error {
        Error {
            err_code: code,
            err_str: err_str,
        }
    }

    pub fn new_str_err(err_str: &str) -> Error {
        Error {
            err_code: Error::COMMON_ERROR,
            err_str: Some(err_str.to_string()),
        }
    }

    pub fn new_code_err(code: i32) -> Error {
        Error {
            err_code: code,
            err_str: None,
        }
    }

    pub fn err_code(&self) -> i32 {
        self.err_code
    }

    pub fn err_str(&self) -> Option<String> {
        match &self.err_str {
            Some(a) => Some(a.clone()),
            None => None,
        }
    }
}
