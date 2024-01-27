#[macro_export]
macro_rules! log {
    ($level:ident, $($param:expr),+) => {
        {
            let time = ::godot::engine::Time::singleton().get_ticks_msec();
            let minutes = time / 1000 / 60;
            let seconds = time / 1000 % 60;
            let milliseconds = time % 1000;

            let message = format!($($param),+);

            godot_print!("[{:03}:{:02}:{:03}] [{}] [{}:{}] {}", minutes, seconds, milliseconds, stringify!($level), file!(), line!(), message);
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($($param:expr),+) => {
        $crate::log!(DEBUG, $($param),+);
    };
}

#[macro_export]
macro_rules! info {
    ($($param:expr),+) => {
        $crate::log!(INFO, $($param),+);
    };
}

#[macro_export]
macro_rules! warn {
    ($($param:expr),+) => {
        $crate::log!(WARN, $($param),+);
    };
}

#[macro_export]
macro_rules! error {
    ($($param:expr),+) => {
        $crate::log!(ERROR, $($param),+);
    };
}
