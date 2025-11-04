use godot::builtin::{Color, NodePath};
use godot::classes::{Decal, Node};
use godot::meta::ToGodot;
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, Context, GodotScript};

use crate::util;
use crate::{engine_callable, script_callable, util::logger};

#[derive(GodotScript, Debug)]
#[script(base = Decal)]
struct WaterDecal {
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

            tween.tween_callback(&engine_callable!(&base, Node::queue_free));
        });
    }
}
