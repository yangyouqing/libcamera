/// Lightweight logging macro. No allocation, no formatting beyond core::fmt.
/// On release builds, only Warn and Error are compiled in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

/// Global log sink function pointer. Set by platform layer at startup.
static mut LOG_SINK: Option<fn(LogLevel, &str, &str)> = None;

/// Install a log sink. Call once at startup from platform layer.
///
/// # Safety
/// Must be called before any logging occurs, from a single thread.
pub unsafe fn set_log_sink(sink: fn(LogLevel, &str, &str)) {
    LOG_SINK = Some(sink);
}

#[doc(hidden)]
pub fn _log(level: LogLevel, module: &str, msg: &str) {
    // SAFETY: LOG_SINK is only written once at init before any logging
    if let Some(sink) = unsafe { LOG_SINK } {
        sink(level, module, msg);
    }
}

#[macro_export]
macro_rules! cam_log {
    (error, $module:expr, $($arg:tt)*) => {
        $crate::logging::_log($crate::logging::LogLevel::Error, $module, {
            // Use a stack buffer to format the message
            &{
                use core::fmt::Write;
                let mut buf = $crate::FixedString::<256>::new();
                let _ = write!(buf, $($arg)*);
                buf
            }
        }.as_str())
    };
    (warn, $module:expr, $($arg:tt)*) => {
        $crate::logging::_log($crate::logging::LogLevel::Warn, $module, {
            use core::fmt::Write;
            let mut buf = $crate::FixedString::<256>::new();
            let _ = write!(buf, $($arg)*);
            &buf
        }.as_str())
    };
    (info, $module:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::logging::_log($crate::logging::LogLevel::Info, $module, {
            use core::fmt::Write;
            let mut buf = $crate::FixedString::<256>::new();
            let _ = write!(buf, $($arg)*);
            &buf
        }.as_str())
    };
    (debug, $module:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::logging::_log($crate::logging::LogLevel::Debug, $module, {
            use core::fmt::Write;
            let mut buf = $crate::FixedString::<256>::new();
            let _ = write!(buf, $($arg)*);
            &buf
        }.as_str())
    };
    (trace, $module:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::logging::_log($crate::logging::LogLevel::Trace, $module, {
            use core::fmt::Write;
            let mut buf = $crate::FixedString::<256>::new();
            let _ = write!(buf, $($arg)*);
            &buf
        }.as_str())
    };
}
