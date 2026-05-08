use log::{Level, LevelFilter, error};

/// Convert `log::LevelFilter` to a u32 representation for host function calls.
pub fn from_log_level_filter(level: LevelFilter) -> u32 {
    match level {
        LevelFilter::Off => 0,
        LevelFilter::Error => 1,
        LevelFilter::Warn => 2,
        LevelFilter::Info => 3,
        LevelFilter::Debug => 4,
        LevelFilter::Trace => 5,
    }
}

/// Convert `log::Level` to a u32 representation for host function calls.
pub fn from_log_level(level: Level) -> u32 {
    match level {
        Level::Error => 1,
        Level::Warn => 2,
        Level::Info => 3,
        Level::Debug => 4,
        Level::Trace => 5,
    }
}

/// Convert a u32 log level from host function calls to `log::LevelFilter`.
pub fn to_log_level_filter(level: u32) -> LevelFilter {
    match level {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => {
            error!("Invalid log level: {level}");
            LevelFilter::Off
        }
    }
}

/// Convert a u32 log level from host function calls to `log::Level`.
pub fn to_log_level(level: u32) -> Level {
    match level {
        1 => Level::Error,
        2 => Level::Warn,
        3 => Level::Info,
        4 => Level::Debug,
        5 => Level::Trace,
        _ => {
            error!("Invalid log level: {level}");
            Level::Error
        }
    }
}

#[cfg(feature = "impl_host_logger")]
mod host_fn {
    use super::*;
    use log::{log, log_enabled};

    pub fn host_logger_max_level() -> u32 {
        let level = log::max_level();
        from_log_level_filter(level)
    }

    pub fn host_logger_enabled(level: u32) -> u32 {
        let log_level = to_log_level(level);
        log_enabled!(log_level) as u32
    }

    pub fn host_logger_log(level: u32, target: &[u8], msg: &[u8]) {
        let log_level = to_log_level(level);

        if log_enabled!(log_level) {
            if let Ok(target_str) = std::str::from_utf8(target) {
                log!(target: target_str, log_level, "{}", String::from_utf8_lossy(msg));
            } else {
                log!(log_level, "{}", String::from_utf8_lossy(msg));
            }
        }
    }
}

#[cfg(not(feature = "impl_host_logger"))]
mod host_fn {
    use super::*;
    use log::{Metadata, Record, SetLoggerError};

    #[cfg_attr(feature = "polkavm", polkavm_derive::polkavm_import)]
    unsafe extern "C" {
        fn host_logger_max_level() -> u32;
        fn host_logger_enabled(level: u32) -> u32;
        fn host_logger_log(level: u32, target_fat_ptr: u64, msg_fat_ptr: u64);
    }

    struct HostLogger;

    impl log::Log for HostLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            let level = from_log_level(metadata.level());
            unsafe { host_logger_enabled(level) != 0 }
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                let level = from_log_level(record.level());
                let target = record.target().as_bytes();
                let msg = ark_std::format!("{}", record.args());
                unsafe {
                    let target_fat_ptr =
                        crate::pack_fat_pointer(target.as_ptr() as u32, target.len() as u32) as u64;
                    let msg_fat_ptr =
                        crate::pack_fat_pointer(msg.as_ptr() as u32, msg.len() as u32) as u64;
                    host_logger_log(level, target_fat_ptr, msg_fat_ptr);
                }
            }
        }

        fn flush(&self) {}
    }

    pub fn init() -> Result<(), SetLoggerError> {
        use ark_std::boxed::Box;
        let logger = Box::new(HostLogger);
        let level = to_log_level_filter(unsafe { host_logger_max_level() });
        log::set_logger(Box::leak(logger)).map(|()| log::set_max_level(level))
    }
}

pub use host_fn::*;
