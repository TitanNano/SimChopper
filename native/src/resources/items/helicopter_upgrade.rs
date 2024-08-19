use godot::{
    builtin::{GString, StringName},
    engine::{PackedScene, Resource},
    obj::{Base, Gd},
    prelude::GodotClass,
};

#[derive(GodotClass)]
#[class(base = Resource, init)]
struct HelicopterUpgrade {
    /// Name of the upgrade
    #[export]
    name: GString,

    /// The game object scene for this upgrade. It will be attached to the helicopter.
    #[export]
    object: Option<Gd<PackedScene>>,

    /// The input action which will enable or disable the game object of this upgrade.
    #[export]
    action: StringName,

    /// The price of this upgrade.
    #[export]
    price: u32,

    base: Base<Resource>,
}
