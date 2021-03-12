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
    () => {
        log::info!(
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
        log::info!(
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
        log::info!(
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

pub fn env_log_init() {
    env_logger::builder()
        .format(|buf, record| {
            use env_logger::fmt::Color;
            let mut level_style = buf.style();

            if record.level().to_string().eq("ERROR") {
                level_style.set_color(Color::Red).set_bold(true);
            }

            if record.level().to_string().eq("INFO") {
                level_style.set_color(Color::Blue).set_bold(false);
            }

            if record.level().to_string().eq("DEBUG") {
                level_style.set_color(Color::Green).set_bold(false);
            }

            writeln!(
                buf,
                "[{:5}] {}",
                level_style.value(record.level()),
                record.args()
            )
        })
        .init();
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
