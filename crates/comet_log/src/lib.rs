#[macro_export]
macro_rules! info {
    ($fmt:expr $(, $args:expr)*) => {{
        use std::io::Write;
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        writeln!(
            handle,
            "{} [{}] [{}::{}] : {}",
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            "\x1b[32m\x1b[1mINFO\x1b[0m",
            env!("CARGO_PKG_NAME"),
            module_path!(),
            format_args!($fmt $(, $args)*)
        ).unwrap();
    }};
}
#[macro_export]
macro_rules! debug {
    ($fmt:expr $(, $args:expr)*) => {{
        use std::io::Write;
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        writeln!(
            handle,
            "{} [{}] [{}::{}:{}] : {}",
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            "\x1b[34m\x1b[1mDEBUG\x1b[0m",
            env!("CARGO_PKG_NAME"),
            module_path!(),
            line!(),
            format_args!($fmt $(, $args)*)
        ).unwrap();
    }};
}
#[macro_export]
macro_rules! warn {
    ($fmt:expr $(, $args:expr)*) => {{
        use std::io::Write;
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        writeln!(
            handle,
            "{} [{}] [{}::{}] : {}",
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            "\x1b[33m\x1b[1mWARNING\x1b[0m",
            env!("CARGO_PKG_NAME"),
            module_path!(),
            format_args!($fmt $(, $args)*)
        ).unwrap();
    }};
}
#[macro_export]
macro_rules! error {
    ($fmt:expr $(, $args:expr)*) => {{
        use std::io::Write;
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        writeln!(
            handle,
            "{} [{}] [{}::{}] : {}",
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            "\x1b[31m\x1b[1mERROR\x1b[0m",
            env!("CARGO_PKG_NAME"),
            module_path!(),
            format_args!($fmt $(, $args)*)
        ).unwrap();
    }};
}
#[macro_export]
macro_rules! fatal {
    ($fmt:expr $(, $args:expr)*) => {{
        use std::io::Write;
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        writeln!(
            handle,
            "{} [{}] [{}::{}] : {}",
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            "\x1b[41mFATAL\x1b[0m",
            env!("CARGO_PKG_NAME"),
            module_path!(),
            format_args!($fmt $(, $args)*)
        ).unwrap();
        std::process::exit(1)
    }};
}
#[macro_export]
macro_rules! trace {
    ($fmt:expr $(, $args:expr)*) => {{
        use std::io::Write;
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        writeln!(
            handle,
            "{} [{}] [{}::{}] : {}",
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            "\x1b[35m\x1b[1mTRACE\x1b[0m",
            env!("CARGO_PKG_NAME"),
            module_path!(),
            format_args!($fmt $(, $args)*)
        ).unwrap();
    }};
}
