use std::io::Write;

#[macro_export]
macro_rules! error{
    () => {
        log::error!(
            "{}@{}-{} : arrived.",
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap_or("undef")
                .split('/')
                .last()
                .unwrap_or("undef")
                .to_string(),
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        log::error!(
            "{}@{}-{} : {}",
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap_or("undef")
                .split('/')
                .last()
                .unwrap_or("undef")
                .to_string(),
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        log::error!(
            "{}@{}-{} : {}",
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap_or("undef")
                .split('/')
                .last()
                .unwrap_or("undef")
                .to_string(),
            file!(),
            line!(),
            format!($fmt, $($val),*)
        );
    }};
}

#[macro_export]
macro_rules! warn{
    () => {
        log::warn!(
            "{}@{}-{} : arrived.",
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap_or("undef")
                .split('/')
                .last()
                .unwrap_or("undef")
                .to_string(),
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        log::warn!(
            "{}@{}-{} : {}",
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap_or("undef")
                .split('/')
                .last()
                .unwrap_or("undef")
                .to_string(),
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        log::warn!(
            "{}@{}-{} : {}",
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap_or("undef")
                .split('/')
                .last()
                .unwrap_or("undef")
                .to_string(),
            file!(),
            line!(),
            format!($fmt, $($val),*)
        );
    }};
}

#[macro_export]
macro_rules! debug {
    () => {
        log::debug!(
            "{}@{}-{} : arrived.",
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap_or("undef")
                .split('/')
                .last()
                .unwrap_or("undef")
                .to_string(),
            file!(),
            line!(),
        );
    };
    ($val:tt) => {
        log::debug!(
            "{}@{}-{} : {}",
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap_or("undef")
                .split('/')
                .last()
                .unwrap_or("undef")
                .to_string(),
            file!(),
            line!(),
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        log::debug!(
            "{}@{}-{} : {}",
            std::env::current_exe()
                .unwrap()
                .to_str()
                .unwrap_or("undef")
                .split('/')
                .last()
                .unwrap_or("undef")
                .to_string(),
            file!(),
            line!(),
            format!($fmt, $($val),*)
        );
    }};
}

#[macro_export]
macro_rules! info{
    ($val:tt) => {
        log::info!(
            "{}",
            $val
        );
    };
    ($fmt:expr,$($val:expr),*) => {{
        log::info!(
            "{}",
            format!($fmt, $($val),*)
        );
    }};
}

pub fn env_log_init() {
    env_logger::builder()
        .format(|buf, record| {
            use env_logger::fmt::Color;
            use std::time::SystemTime;

            let mut level_style = buf.style();

            if record.level().to_string().eq("ERROR") {
                level_style.set_color(Color::Red).set_bold(true);
            }

            if record.level().to_string().eq("WARN") {
                level_style.set_color(Color::Cyan).set_bold(false);
            }

            if record.level().to_string().eq("INFO") {
                level_style.set_color(Color::Blue).set_bold(false);
            }

            if record.level().to_string().eq("DEBUG") {
                level_style.set_color(Color::Green).set_bold(false);
            }

            let nanos = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Wrong Time")
                .as_nanos();

            writeln!(
                buf,
                "{}.{:0>9} [{:5}] {}",
                nanos / 1000000000,
                nanos % 1000000000,
                level_style.value(record.level()),
                record.args()
            )
        })
        .init();
}

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

#[macro_export]
macro_rules! test_debug {
    ($fmt:tt, $val:expr) => {
        println!("fmt: {}, val: ${}", stringify!($fmt), stringify!($val));
    };
}

#[test]
fn test_debug_macro() {
    debug!("test: {}", String::from("hello"));
    debug!("hello world");
    debug!();
}
