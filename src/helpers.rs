/// Logging macros
#[macro_export]
macro_rules! log_info_cyan {
    ($($arg:tt)*) => {
        log::info!("{}", format_args!($($arg)*).to_string().cyan());
    };
}
