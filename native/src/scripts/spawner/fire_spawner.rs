use std::ops::{Deref, DerefMut, Neg};

use godot::builtin::math::ApproxEq;
use godot::builtin::{Aabb, Array, GString, Transform3D, Vector3, Vector3Axis};
use godot::classes::{
    light_3d, AudioStreamPlayer3D, Engine, GpuParticles3D, Node, OmniLight3D,
    ParticleProcessMaterial, Texture2D,
};
use godot::obj::{Gd, Singleton};
use godot::prelude::Var;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor};
use num::ToPrimitive;

use crate::script_callable;
use crate::util::{logger, Uf32};

#[derive(Debug)]
struct OnReady<T>(Option<T>);

impl<T> OnReady<T> {
    fn init(&mut self, value: T) {
        self.0 = Some(value);
    }
}

impl<T> Default for OnReady<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> Deref for OnReady<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> DerefMut for OnReady<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

#[derive(GodotScript, Debug)]
#[script(base = Node, tool)]
pub struct FireSpawner {
    /// Density of the fire.
    ///
    /// The density will be multiplied by the number of emission points and the emission strength.
    #[export(range(min = 1.0, max = 200.0, step = 1.0))]
    #[prop(set = Self::set_density)]
    pub density: Uf32,

    /// Density of the fire.
    ///
    /// The density will be multiplied by the number of emission points and the emission strength.
    #[export(range(min = 1.0, max = 200.0, step = 1.0))]
    #[prop(set = Self::set_smoke_density)]
    pub smoke_density: Uf32,

    /// Fire will be emitted from these points. The points should be generated like the `Particle3DEditorPlugin` does.
    #[export]
    #[prop(set = Self::set_emission_points)]
    pub emission_points: OnEditor<Gd<Texture2D>>,

    /// Fire will be emitted with these normals. The normals should be generated like the `Particle3DEditorPlugin` does.
    #[export]
    #[prop(set = Self::set_emission_point_normals)]
    pub emission_point_normals: OnEditor<Gd<Texture2D>>,

    /// Number of points that are actually used to for emitting particles.
    #[export]
    #[prop(set = Self::set_emission_point_count)]
    pub emission_point_count: Uf32,

    /// Node path to the particle system for the flames.
    #[export]
    #[prop(set = Self::set_fire)]
    pub fire: OnEditor<Gd<GpuParticles3D>>,

    /// Node path to the particle system for the smoke.
    #[export]
    #[prop(set = Self::set_smoke)]
    pub smoke: OnEditor<Gd<GpuParticles3D>>,

    /// Node path to the omni-light source of the fire.
    #[export]
    #[prop(set = Self::set_light_source)]
    pub light_source: OnEditor<Gd<OmniLight3D>>,

    /// Strength of the fire.
    #[export(range(min = 0.0, max = 1.0, step = 0.01))]
    #[prop(set = Self::set_fire_strength)]
    pub strength: f32,

    /// Modulate the firelight source to create an animated variation in the brightness.
    #[export(range(min = 0.0, max = 2.0, step = 0.001))]
    #[prop(set = Self::set_light_modulator)]
    pub light_modulator: f32,

    #[export]
    #[prop(set = Self::set_fire_sound)]
    pub fire_sound: OnEditor<Gd<AudioStreamPlayer3D>>,

    fire_process_material: OnReady<Gd<ParticleProcessMaterial>>,

    smoke_process_material: OnReady<Gd<ParticleProcessMaterial>>,

    fire_lifetime: f64,

    base: Gd<<Self as GodotScript>::Base>,
}

#[godot_script_impl]
impl FireSpawner {
    pub fn _ready(&mut self) {
        let is_editor = Engine::singleton().is_editor_hint();

        if !self.is_instance() && is_editor {
            return;
        }

        logger::debug!("Init Fire spawner...");

        if is_editor
            && (self.fire.get_property().is_none()
                || self.smoke.get_property().is_none()
                || self.emission_points.get_property().is_none()
                || self.emission_point_normals.get_property().is_none()
                || self.fire_sound.get_property().is_none())
        {
            self.base.update_configuration_warnings();
            return;
        }

        let Some(org_fire_process_material) = self.fire.get_process_material() else {
            logger::error!("Fire particles have no process material assigned!");
            return;
        };

        let Some(org_smoke_process_material) = self.smoke.get_process_material() else {
            logger::error!("Smoke particles have no process material assigned!");
            return;
        };

        let emission_point_count = self
            .emission_point_count
            .into_u32()
            .to_i32()
            .expect("emission point count should be in range");

        let mut fire_process_material = org_fire_process_material
            .duplicate()
            .expect("Duplicating shouldn't fail!")
            .cast::<ParticleProcessMaterial>();

        let mut smoke_process_material = org_smoke_process_material
            .duplicate()
            .expect("Duplicating should work!")
            .cast::<ParticleProcessMaterial>();

        fire_process_material.set_emission_point_count(emission_point_count);
        fire_process_material.set_emission_point_texture(&*self.emission_points);
        fire_process_material.set_emission_normal_texture(&*self.emission_point_normals);
        smoke_process_material.set_emission_point_count(emission_point_count);
        smoke_process_material.set_emission_point_texture(&*self.emission_points);
        smoke_process_material.set_emission_normal_texture(&*self.emission_point_normals);

        self.fire.set_process_material(&fire_process_material);
        self.fire_process_material.init(fire_process_material);

        self.smoke.set_process_material(&smoke_process_material);
        self.smoke_process_material.init(smoke_process_material);

        let particle_count =
            (self.density.into_f32() * self.emission_point_count.into_f32()).max(1.0);

        self.fire.set_amount(
            particle_count
                .to_i32()
                .expect("particle count should be in range"),
        );

        let smoke_particle_count =
            (self.smoke_density.into_f32() * self.emission_point_count.into_f32()).max(1.0);

        self.smoke.set_amount(
            smoke_particle_count
                .to_i32()
                .expect("particle count should be in range"),
        );

        // Rerun strength setter once we are ready.
        self.set_fire_strength(self.strength);

        if is_editor {
            self.base.update_configuration_warnings();
        }
    }

    /// Set the exact size of the burning object.
    ///
    /// A safety margin is added automatically and doesn't have to be calculated in.
    pub fn resize(&mut self, size: Vector3) {
        const BASE_SCALE: f32 = 1.0;
        const MARGIN_SCALE: f32 = 0.5;
        const SMOKE_Y_SCALE: f32 = 2.0;

        let light_size = size;

        let light_max_size = match light_size.max_axis().unwrap_or(Vector3Axis::X) {
            Vector3Axis::X => light_size.x,
            Vector3Axis::Y => light_size.y,
            Vector3Axis::Z => light_size.z,
        };

        self.light_source
            .set_param(light_3d::Param::RANGE, light_size.length() * 4.0);
        self.light_source
            .set_param(light_3d::Param::SIZE, light_max_size);
        self.light_source
            .set_transform(Transform3D::default().translated(Vector3::new(
                0.0,
                light_size.y / 2.0,
                0.0,
            )));

        let fire_aabb_size = size * (BASE_SCALE + MARGIN_SCALE);
        let fire_aabb_pos = {
            let mut base = size / 2.0;
            base.y = 0.0;
            base += (fire_aabb_size - size) / 2.0;
            base.neg()
        };

        let fire_aabb = Aabb::new(fire_aabb_pos, fire_aabb_size);
        self.fire.set_visibility_aabb(fire_aabb);

        let smoke_aabb_size = size
            * Vector3::new(
                BASE_SCALE + MARGIN_SCALE,
                SMOKE_Y_SCALE + MARGIN_SCALE,
                BASE_SCALE + MARGIN_SCALE,
            );

        let smoke_aabb_pos = {
            let size = size * Vector3::new(BASE_SCALE, SMOKE_Y_SCALE, BASE_SCALE);
            let mut base = size / 2.0;
            base.y = 0.0;
            base += (smoke_aabb_size - size) / 2.0;
            base.neg()
        };

        let smoke_aabb = Aabb::new(smoke_aabb_pos, smoke_aabb_size);
        self.smoke.set_visibility_aabb(smoke_aabb);
    }

    pub fn set_fire_strength(&mut self, strength: f32) {
        self.strength = strength;

        if !self.base.is_node_ready() {
            return;
        }

        self.update_light_energy();
        self.fire_sound.set_volume_linear(strength);

        if self.fire_lifetime == 0.0 {
            self.fire_lifetime = self.fire.get_lifetime();
        }

        // Disable emission if the strength is very low.
        if strength < 0.01 {
            self.fire.set_emitting(false);
            return;
        } else if !self.fire.is_emitting() {
            self.fire.set_emitting(true);
        }

        // adjust amount ratio to strength
        self.fire.set_amount_ratio(strength);
        self.smoke.set_amount_ratio(strength.max(0.2));

        // scale lifetime to strength
        let fire_strength = self.fire_lifetime * f64::from(strength);

        self.fire.set_lifetime(fire_strength);
    }

    /// Indicates if the fire is extinguished.
    pub fn is_dead(&self) -> bool {
        self.strength.approx_eq(&0.0)
    }

    /// Indicates whether the current node is instanced in an other scene or not.
    fn is_instance(&self) -> bool {
        let root_scene_path = self
            .base
            .get_tree()
            .and_then(|tree| {
                if Engine::singleton().is_editor_hint() {
                    tree.get_edited_scene_root()
                } else {
                    tree.get_current_scene()
                }
            })
            .map(|node| node.get_scene_file_path())
            .unwrap_or_default();

        let self_scene_path = self.base.get_scene_file_path();

        self_scene_path.is_empty() || self_scene_path != root_scene_path
    }

    pub fn set_emission_point_count(&mut self, value: Uf32) {
        self.emission_point_count = value;

        if Engine::singleton().is_editor_hint() && self.base.is_node_ready() {
            script_callable!(self, Self::_ready).call_deferred(&[]);
        }
    }

    pub fn set_emission_point_normals(&mut self, value: Option<Gd<Texture2D>>) {
        self.emission_point_normals.set_property(value);

        if Engine::singleton().is_editor_hint() && self.base.is_node_ready() {
            script_callable!(self, Self::_ready).call_deferred(&[]);
        }
    }

    pub fn set_emission_points(&mut self, value: Option<Gd<Texture2D>>) {
        self.emission_points.set_property(value);

        if Engine::singleton().is_editor_hint() && self.base.is_node_ready() {
            script_callable!(self, Self::_ready).call_deferred(&[]);
        }
    }

    fn set_density(&mut self, value: Uf32) {
        self.density = value;

        if Engine::singleton().is_editor_hint() && self.base.is_node_ready() {
            script_callable!(self, Self::_ready).call_deferred(&[]);
        }
    }

    fn set_fire(&mut self, value: Option<Gd<GpuParticles3D>>) {
        self.fire.set_property(value);

        if Engine::singleton().is_editor_hint() && self.base.is_node_ready() {
            script_callable!(self, Self::_ready).call_deferred(&[]);
        }
    }

    fn set_smoke(&mut self, value: Option<Gd<GpuParticles3D>>) {
        self.smoke.set_property(value);

        if Engine::singleton().is_editor_hint() && self.base.is_node_ready() {
            script_callable!(self, Self::_ready).call_deferred(&[]);
        }
    }

    fn set_smoke_density(&mut self, value: Uf32) {
        self.smoke_density = value;

        if Engine::singleton().is_editor_hint() && self.base.is_node_ready() {
            script_callable!(self, Self::_ready).call_deferred(&[]);
        }
    }

    fn set_light_source(&mut self, value: Option<Gd<OmniLight3D>>) {
        self.light_source.set_property(value);

        if Engine::singleton().is_editor_hint() && self.base.is_node_ready() {
            script_callable!(self, Self::_ready).call_deferred(&[]);
        }
    }

    fn set_fire_sound(&mut self, value: Option<Gd<AudioStreamPlayer3D>>) {
        self.fire_sound.set_property(value);

        if Engine::singleton().is_editor_hint() && self.base.is_node_ready() {
            script_callable!(self, Self::_ready).call_deferred(&[]);
        }
    }

    fn set_light_modulator(&mut self, value: f32) {
        self.light_modulator = value;
        self.update_light_energy();
    }

    fn update_light_energy(&mut self) {
        if !self.base.is_node_ready() {
            return;
        }

        self.light_source.set_param(
            light_3d::Param::ENERGY,
            self.strength * self.light_modulator,
        );
    }

    pub fn _get_configuration_warnings(&self) -> Array<GString> {
        let mut warnings = Array::new();

        if self.fire.get_property().is_none() {
            warnings.push("Requires a GpuParticles3D node for the fire vfx");
        }

        if self.smoke.get_property().is_none() {
            warnings.push("Requires a GpuParticles3D node for the smoke vfx");
        }

        if self.light_source.get_property().is_none() {
            warnings.push("A light source must be assigned");
        }

        if self.emission_points.get_property().is_none() {
            warnings.push("Emission points texture must be assinged");
        }

        if self.emission_point_normals.get_property().is_none() {
            warnings.push("Emission point normals texture must be assinged");
        }

        if self.fire_sound.get_property().is_none() {
            warnings.push("Fire audio source must be assigned");
        }

        warnings
    }
}
