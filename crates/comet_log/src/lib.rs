#[macro_export]
macro_rules! info {
    ($msg:expr) => {
        println!(
            "{} {} : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            "\x1b[32m\x1b[1mINFO\x1b[0m",
            $msg
        );
    };
}

#[macro_export]
macro_rules! debug {
    ($msg:expr) => {
        println!(
            "{} {} : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            "\x1b[34m\x1b[1mDEBUG\x1b[0m",
            $msg
        );
    };
}

#[macro_export]
macro_rules! warn {
    ($msg:expr) => {
        println!(
            "{} {} : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            "\x1b[33m\x1b[1mWARNING\x1b[0m",
            $msg
        );
    };
}

#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        println!(
            "{} {} : {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            "\x1b[31m\x1b[1mERROR\x1b[0m",
            $msg
        );
    };
}