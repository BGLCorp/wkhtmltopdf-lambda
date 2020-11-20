#[macro_export]
macro_rules! error {
    ($($args:tt)+) => {
        #[cfg(not(test))] slog::log!(crate::LOGGER.get().unwrap(), slog::Level::Error, "", $($args)+);
        #[cfg(test)] eprintln!("{}: {}", slog::Level::Error, format!($($args)+));
    };
}
#[macro_export]
macro_rules! warn {
    ($($args:tt)+) => {
        #[cfg(not(test))] slog::log!(crate::LOGGER.get().unwrap(), slog::Level::Warning, "", $($args)+);
        #[cfg(test)] eprintln!("{}: {}", slog::Level::Warning, format!($($args)+));
    };
}
#[macro_export]
macro_rules! info {
    ($($args:tt)+) => {
        #[cfg(not(test))] slog::log!(crate::LOGGER.get().unwrap(), slog::Level::Info, "", $($args)+);
        #[cfg(test)] eprintln!("{}: {}", slog::Level::Info, format!($($args)+));
    };
}
#[macro_export]
macro_rules! debug {
    ($($args:tt)+) => {
        #[cfg(not(test))] slog::log!(crate::LOGGER.get().unwrap(), slog::Level::Debug, "", $($args)+);
        #[cfg(test)] eprintln!("{}: {}", slog::Level::Debug, format!($($args)+));
    };
}
