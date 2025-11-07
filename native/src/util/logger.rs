#[allow(unused)]
pub use crate::{debug, error, info, warn};

#[macro_export(local_inner_macros)]
macro_rules! log {
    ($level:ident, $out:ident, $($param:expr),+) => {
        {
            let time = <::godot::classes::Time as ::godot::obj::Singleton>::singleton().get_ticks_msec();
            let minutes = time / 1000 / 60;
            let seconds = time / 1000 % 60;
            let milliseconds = time % 1000;

            let message = ::std::format!($($param),+);

            log!(print:$out, "[{:03}:{:02}:{:03}] [{}] [{}:{}] {}", minutes, seconds, milliseconds, ::std::stringify!($level), ::std::file!(), ::std::line!(), message);
        }
    };

    (print:default, $string_lit:literal, $($arg:expr),*) => {
        ::godot::global::godot_print!($string_lit, $($arg),*);
    };

    (print:warn, $string_lit:literal, $($arg:expr),*) => {
        ::godot::global::godot_warn!($string_lit, $($arg),*);
    };

    (print:error, $string_lit:literal, $($arg:expr),*) => {
        ::godot::global::godot_error!($string_lit, $($arg),*);
    }
}

#[macro_export(local_inner_macros)]
macro_rules! debug {
    ($($param:expr),+) => {
        log!(DEBUG, default, $($param),+)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! info {
    ($($param:expr),+) => {
        log!(INFO, default, $($param),+)
    };
}

#[macro_export]
macro_rules! warn {
    ($($param:expr),+) => {
        $crate::log!(WARN, warn, $($param),+)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! error {
    ($($param:expr),+) => {
        log!(ERROR, error, $($param),+)
    };
}
