use godot::builtin::{Callable, Color, NodePath};
use godot::classes::Decal;
use godot::meta::ToGodot;
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, CastToScript, Context, GodotScript, OnEditor, RsRef};

use crate::resources::WaterDecalTracker;
use crate::util;
use crate::{script_callable, util::logger};

#[derive(GodotScript, Debug)]
#[script(base = Decal)]
struct WaterDecal {
    #[export]
    pub decal_tracker: OnEditor<Gd<WaterDecalTracker>>,

    pub is_active: bool,
    base: Gd<Decal>,
}

#[godot_script_impl]
impl WaterDecal {
    const LIFETIME: f64 = 6.0;

    pub fn _ready(&mut self) {
        if !self.is_active {
            return;
        }

        util::timer(&mut self.base.get_tree().unwrap(), Self::LIFETIME)
            .connect("timeout", &script_callable!(self, Self::on_timeout));
    }

    pub fn on_timeout(&mut self, mut ctx: Context<Self>) {
        let Some(mut tween) = self.base.create_tween() else {
            logger::error!("Failed to create tween!");
            return;
        };

        ctx.reentrant_scope(self, |base: Gd<Decal>| {
            tween.tween_property(
                &base,
                &NodePath::from("modulate"),
                &Color::TRANSPARENT_WHITE.to_variant(),
                1.0,
            );

            let mut script: RsRef<Self> = base.clone().into_script();

            tween.tween_callback(&Callable::from_fn("water_decal_tween_out", move |_| {
                script.clean_up();
            }));
        });
    }

    pub fn clean_up(&mut self) {
        self.decal_tracker.bind_mut().free(&self.base);
        self.base.queue_free();
    }
}
