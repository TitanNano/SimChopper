use gdnative::prelude::*;

use backtrace::Backtrace;

pub fn init_panic_hook() {
    let old_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let loc_string = match panic_info.location() {
            Some(location) => format!("file '{}' at line {}", location.file(), location.line()),
            None => "unknown location".to_owned(),
        };

        let error_message;
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            error_message = format!("[RUST] {}: panic occurred: {:?}", loc_string, s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            error_message = format!("[RUST] {}: panic occurred: {:?}", loc_string, s);
        } else {
            error_message = format!("[RUST] {}: unknown panic occurred", loc_string);
        }

        godot_error!("{}", error_message);
        godot_error!("Backtrace:\n{:?}", Backtrace::new());

        (*(old_hook.as_ref()))(panic_info);
    }));
}
