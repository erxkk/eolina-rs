use crossterm::style::Stylize;
use std::{
    lazy::{SyncLazy, SyncOnceCell},
    sync::Mutex,
};

///
/// Whether or not [`Stdin`](std::io::Stdin) is a tty.
///
pub static IS_IN_TTY: SyncLazy<bool> = SyncLazy::new(|| atty::is(atty::Stream::Stdin));

///
/// Whether or not [`Stdout`](std::io::Stdout) is a tty.
///
pub static IS_OUT_TTY: SyncLazy<bool> = SyncLazy::new(|| atty::is(atty::Stream::Stdout));

///
/// Whether or not [`Stderr`](std::io::Stderr) is a tty.
///
pub static IS_ERR_TTY: SyncLazy<bool> = SyncLazy::new(|| atty::is(atty::Stream::Stderr));

///
/// The global mutable log leve filter, used to set logging at runtime for repl.
///
/// **DO NOT** keep a lock around while logging, this value is accessed by the log filter.
///
pub static LOG_LEVEL_FILTER: SyncLazy<Mutex<log::LevelFilter>> =
    SyncLazy::new(|| Mutex::new(log::LevelFilter::Off));

///
/// Whether or not the log output should be colorful.
///
/// **DO NOT** keep a lock around while logging, this value is accessed by the log filter.
///
pub static IS_FANCY: SyncLazy<bool> = SyncLazy::new(|| *IS_FANCY_CELL.get_or_init(|| false));

///
/// A backing cell used to initialize [`IS_FANCY`] eliminating the need for get_or_init on
/// every get.
///
pub(super) static IS_FANCY_CELL: SyncOnceCell<bool> = SyncOnceCell::new();

///
/// Get the log prompt depending on the current state of [`IS_FANCY`]. If [`IS_FANCY`] is true,
/// the prompt will be styled, otherwise not.
///
pub fn get_prompt<'a>(level: log::Level) -> <&'a str as Stylize>::Styled {
    match level {
        log::Level::Error => {
            if *IS_FANCY {
                "err".red()
            } else {
                "err".stylize()
            }
        }
        log::Level::Warn => {
            if *IS_FANCY {
                "wrn".yellow()
            } else {
                "wrn".stylize()
            }
        }
        log::Level::Info => {
            if *IS_FANCY {
                "inf".green()
            } else {
                "inf".stylize()
            }
        }
        log::Level::Debug => {
            if *IS_FANCY {
                "dbg".cyan()
            } else {
                "dbg".stylize()
            }
        }
        log::Level::Trace => {
            if *IS_FANCY {
                "trc".grey()
            } else {
                "trc".stylize()
            }
        }
    }
}

///
/// Returns the before and after loglevel if it were adjusted with an increment/decrement.
///
/// ### Returns
/// * `(before, after)`
///
pub fn log_level_after_adjust(inc: bool) -> (log::LevelFilter, log::LevelFilter) {
    let lock = LOG_LEVEL_FILTER.lock().expect("mutext not acquired");
    let before = *lock;

    // incrementing log level means decreasing the filter
    let after = match before {
        log::LevelFilter::Off => {
            if inc {
                log::LevelFilter::Error
            } else {
                log::LevelFilter::Off
            }
        }
        log::LevelFilter::Error => {
            if inc {
                log::LevelFilter::Warn
            } else {
                log::LevelFilter::Off
            }
        }
        log::LevelFilter::Warn => {
            if inc {
                log::LevelFilter::Info
            } else {
                log::LevelFilter::Error
            }
        }
        log::LevelFilter::Info => {
            if inc {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Warn
            }
        }
        log::LevelFilter::Debug => {
            if inc {
                log::LevelFilter::Trace
            } else {
                log::LevelFilter::Info
            }
        }
        log::LevelFilter::Trace => {
            if inc {
                log::LevelFilter::Trace
            } else {
                log::LevelFilter::Debug
            }
        }
    };

    (before, after)
}
