use godot_rust_script::godot::classes::{GpuParticles3D, PrimitiveMesh, StandardMaterial3D};
use godot_rust_script::{
    godot::prelude::{godot_error, Gd},
    godot_script_impl, GodotScript,
};

/// Dust Particle behavior for a particle system.
/// This is used for the rotor effects
#[derive(GodotScript, Debug)]
#[script(base = GpuParticles3D)]
struct DustParticles {
    /// The strength of the emitted dust.
    #[prop(get = Self::strength, set = Self::set_strength)]
    #[export(range(min = 0.1, max = 1.5, step = 0.05))]
    strength: f64,

    base: Gd<GpuParticles3D>,
}

#[godot_script_impl]
impl DustParticles {
    pub fn _ready(&mut self) {
        self.set_strength(0.0);
    }

    /// get effect strength
    fn strength(&self) -> f64 {
        self.strength
    }

    pub fn set_strength(&mut self, value: f64) {
        self.strength = value;

        let is_emitting = value > 0.0;

        if self.base.is_emitting() != is_emitting {
            self.base.set_emitting(is_emitting);
        }

        if !self.base.is_emitting() {
            return;
        }

        let Some(mesh) = self.base.get_draw_pass_mesh(0) else {
            godot_error!("Draw pass 1 does not exist!");
            return;
        };

        let mesh: Gd<PrimitiveMesh> = mesh.cast();

        let Some(material) = mesh.get_material() else {
            godot_error!("mesh has no material!");
            return;
        };

        let mut material: Gd<StandardMaterial3D> = material.cast();

        let distance = (100.0 * (1.0 - value)).max(2.0);

        material.set_proximity_fade_distance(distance as f32);
    }
}
