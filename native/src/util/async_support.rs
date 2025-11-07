use godot::builtin::Variant;
use godot::classes::RefCounted;
use godot::meta::ToGodot;
use godot::obj::{Base, Gd, NewGd};
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass)]
#[class(base = RefCounted, init)]
pub struct GodotFuture {
    base: Base<RefCounted>,
}

#[godot_api]
impl GodotFuture {
    /// Is emitted as soon as the async operation of the function has been completed.
    #[signal]
    fn completed(result: Variant);
}

/// Creates a new [`GodotFuture`] that can be returned from a function which performs an async operation. This works similar to `GdFunctionState`.
///
/// Example:
/// ```rs
/// fn async_do_task() -> Gd<GodotFuture> {
///     let (resolve, future) = godot_future();
///
///     godot_task(async move {
///         // do async operations
///         resolve(true);
///     });
///
///     future
/// }
/// ```
pub fn godot_future<R: ToGodot>() -> (impl Fn(R), Gd<GodotFuture>) {
    let future = GodotFuture::new_gd();
    let sender = future.clone();

    (
        move |value: R| {
            sender.signals().completed().emit(&value.to_variant());
        },
        future,
    )
}
