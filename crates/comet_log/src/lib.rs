#[macro_export]
macro_rules! info {
    ($fmt:expr $(, $args:expr)*) => {
        eprintln!(
            "{} [{}::{}] [{}] : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            module_path!(),
            "\x1b[32m\x1b[1mINFO\x1b[0m",
            format!($fmt $(, $args)*)
        );
    };
}

#[macro_export]
macro_rules! debug {
    ($fmt:expr $(, $args:expr)*) => {
        eprintln!(
            "{} [{}::{}:{}] [{}] : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            module_path!(),
            line!(),
            "\x1b[34m\x1b[1mDEBUG\x1b[0m",
            format!($fmt $(, $args)*)
        );
    };
}

#[macro_export]
macro_rules! warn {
    ($fmt:expr $(, $args:expr)*) => {
        eprintln!(
            "{} [{}::{}] [{}] : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            module_path!(),
            "\x1b[33m\x1b[1mWARNING\x1b[0m",
            format!($fmt $(, $args)*)
        );
    };
}

#[macro_export]
macro_rules! error {
    ($fmt:expr $(, $args:expr)*) => {
        eprintln!(
            "{} [{}::{}] [{}] : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            module_path!(),
            "\x1b[31m\x1b[1mERROR\x1b[0m",
            format!($fmt $(, $args)*)
        );
    };
}

#[macro_export]
macro_rules! fatal {
    ($fmt:expr $(, $args:expr)*) => {
        eprintln!(
            "{} [{}::{}] [{}] : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            module_path!(),
            "\x1b[41mFATAL\x1b[0m",
            format!($fmt $(, $args)*)
        );
    };
}

#[macro_export]
macro_rules! trace {
    ($fmt:expr $(, $args:expr)*) => {
        eprintln!(
            "{} [{}::{}] [{}] : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            std::env::var("CARGO_PKG_NAME").unwrap(),
            module_path!(),
            "\x1b[35m\x1b[1mTRACE\x1b[0m",
            format!($fmt $(, $args)*)
        );
    };
}