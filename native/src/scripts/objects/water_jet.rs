use std::boxed::Box;

use anyhow::{bail, Result};
use godot::builtin::{Aabb, Array, Callable, GString, Variant, Vector3};
use godot::classes::object::ConnectFlags;
use godot::classes::{
    Area3D, Decal, GpuParticles3D, Node, Node3D, PhysicsRayQueryParameters3D, ShapeCast3D,
};
use godot::meta::ToGodot;
use godot::obj::{Gd, Inherits};
use godot::prelude::{GodotClass, NodePath};
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, RsRef};

use crate::ext::node_3d::Node3DExt;
use crate::resources::WaterDecalTracker;
use crate::scripts::objects::debugger_3_d::Debugger3D;
use crate::util::logger;
use crate::{debug_3d, util};

#[derive(GodotScript, Debug)]
#[script(base = GpuParticles3D)]
struct WaterJet {
    /// List of [`ShapeCast3D`] nodes to approximate particle impact.
    #[export(node_path = ["ShapeCast3D"])]
    pub impact_cast_paths: Array<NodePath>,

    #[export]
    pub impact_area: OnEditor<Gd<Area3D>>,

    #[export]
    pub decal: OnEditor<Gd<Decal>>,

    #[export]
    pub debugger: Option<RsRef<Debugger3D>>,

    #[export]
    pub decal_tracker: OnEditor<Gd<WaterDecalTracker>>,

    /// Maximum number of decals that will be spawned at an impact point.
    #[export(range(min = 1.0, max = 255.0, step = 1.0))]
    pub max_decal_count: u8,

    /// Maximum decal spawn delay in seconds.
    #[export(range(min = 0.0, max = 20.0, step = 0.1))]
    pub max_delay: f32,

    impact_casts: Vec<Gd<ShapeCast3D>>,

    base: Gd<GpuParticles3D>,
}

#[derive(Clone, Copy)]
struct Intersection {
    position: Vector3,
    normal: Vector3,
}

#[godot_script_impl]
impl WaterJet {
    const MAX_DISTANCE: f32 = 60.0;

    pub fn _ready(&mut self) {
        self.impact_casts = self
            .impact_cast_paths
            .iter_shared()
            .map(|path| self.base.get_node_as(&path))
            .collect();
    }

    fn impact_area(&self) -> &Gd<Area3D> {
        &self.impact_area
    }

    fn decal(&self) -> &Gd<Decal> {
        &self.decal
    }

    fn get_decal_count_at(
        &self,
        point: Vector3,
        size: Vector3,
        max: u8,
    ) -> Box<[godot::prelude::Vector3]> {
        let aabb = Aabb {
            position: point - (size / 2.0),
            size,
        };

        self.decal_tracker
            .bind()
            .get_decals_at_point(aabb)
            .iter()
            .take(max as usize)
            .map(|decal| decal.get_global_position())
            .collect()
    }

    fn raycast_to_surface(&self, origin: Vector3, target: Vector3) -> Result<Option<Intersection>> {
        let Some(mut physics_world) = self.base.get_world_3d() else {
            bail!("Failed to acquire physics world!");
        };

        let Some(mut space) = physics_world.get_direct_space_state() else {
            bail!("Failed to acquire space!");
        };

        let Some(mut query) = PhysicsRayQueryParameters3D::create(origin, target) else {
            bail!("Failed to create raycast query!");
        };

        query.set_hit_back_faces(false);

        let result = space.intersect_ray(&query);

        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(Intersection {
            position: result
                .get("position")
                .expect("result struct musst have a position key")
                .to(),
            normal: result
                .get("normal")
                .expect("result struct musst have a normal key")
                .to(),
        }))
    }

    pub fn _physics_process(&mut self, delta: f64) {
        let active = self.base.is_emitting();
        debug_3d!(self.debugger => active);

        if !active {
            return;
        }

        #[cfg(debug_assertions)]
        let area = self
            .impact_area()
            .get_overlapping_bodies()
            .len()
            .try_into()
            .unwrap_or(u32::MAX);
        debug_3d!(self.debugger => area);

        if !self.impact_area().has_overlapping_bodies() {
            return;
        }

        let decal = self.decal().to_owned();

        #[cfg(debug_assertions)]
        let mut shape_cast_name = "N/A".to_owned();
        #[cfg(debug_assertions)]
        let mut impacting = Array::new();
        #[cfg(debug_assertions)]
        let mut decal_spawned = false;

        for shape_cast in &self.impact_casts {
            shape_cast.clone().force_shapecast_update();

            let count = shape_cast.get_collision_count();

            if count == 0 {
                continue;
            }

            let mut impact_targets = Array::<GString>::new();

            #[cfg(debug_assertions)]
            {
                shape_cast_name = shape_cast.get_name().to_string();
                impacting = impact_targets.clone();
            }

            for index in 0..count {
                let Some(target_node) = shape_cast.get_collider(index) else {
                    logger::error!("Failed to get coliding node for index {}!", index);
                    continue;
                };

                let mut target_node: Gd<Node3D> = match target_node.try_cast() {
                    Ok(node) => node,
                    Err(original_node) => {
                        logger::error!("Failed to cast coliding node: {}", original_node);
                        continue;
                    }
                };

                impact_targets.push(target_node.get_name().arg());

                let point = self.align_decal_normal(Intersection {
                    position: shape_cast.get_collision_point(index),
                    normal: shape_cast.get_collision_normal(index),
                });

                let impact_distance = self.base.get_global_position().distance_to(point.position);
                let target_decal_count = self.max_decal_count;
                let extent = Self::distance_scale(impact_distance, 1.0, 8.0);

                let decal_scale = extent * 2.0;

                let decals_at_point = self.get_decal_count_at(
                    point.position,
                    Vector3::splat(decal_scale),
                    target_decal_count,
                );

                logger::debug!("impact node: {}", target_node.get_name());
                logger::debug!("impact distance: {impact_distance}");
                logger::debug!("existing decals near point: {}", decals_at_point.len());

                let mut decal_inst = self
                    .can_spawn_decal(&point, extent, &decals_at_point)
                    .then(|| self.spawn_decal(&decal, &point, decal_scale, &mut target_node));

                if let Some(inst) = &decal_inst {
                    self.decal_tracker.bind_mut().insert(inst);
                }

                let impact_delay = Self::distance_scale(impact_distance, 0.0, self.max_delay);

                let mut timer =
                    util::timer(&mut self.base.get_tree().unwrap(), impact_delay.into());

                #[cfg(debug_assertions)]
                {
                    decal_spawned = decal_inst.is_some();
                }

                timer.connect_flags(
                    "timeout",
                    &Callable::from_fn("timeout", move |_| {
                        if let Some(decal) = &mut decal_inst {
                            Self::show_decal(decal);
                        }

                        Self::propagate_impact(&mut target_node, delta);

                        Variant::nil()
                    }),
                    ConnectFlags::ONE_SHOT,
                );
            }

            break;
        }

        debug_3d!(self.debugger => shape_cast_name, impacting, decal_spawned);
    }

    fn distance_scale(distance: f32, min: f32, max: f32) -> f32 {
        (max / Self::MAX_DISTANCE * distance).clamp(min, max)
    }

    fn can_spawn_decal(
        &self,
        point: &Intersection,
        extent: f32,
        decals_at_point: &[Vector3],
    ) -> bool {
        if decals_at_point.len() > self.max_decal_count as usize {
            return false;
        }

        let can_spawn = decals_at_point
            .iter()
            .all(|decal| decal.distance_to(point.position) > extent);

        can_spawn
    }

    fn align_decal_normal(&self, point: Intersection) -> Intersection {
        let surface_intersect =
            self.raycast_to_surface(point.position, point.position + point.normal);

        let surface_intersect = match surface_intersect {
            Ok(value) => value,
            Err(err) => {
                logger::error!("{:?}", err.context("Failed to get surface intersection!"));
                return point;
            }
        };

        surface_intersect.unwrap_or(point)
    }

    fn spawn_decal<T>(
        &self,
        template: &Gd<Decal>,
        point: &Intersection,
        decal_scale: f32,
        target_node: &mut Gd<T>,
    ) -> Gd<Decal>
    where
        T: GodotClass + Inherits<Node>,
    {
        #[cfg(debug_assertions)]
        let mut debugger = self.debugger.clone();
        let mut decal_inst: Gd<Decal> = template
            .duplicate()
            .expect("Failed to duplicate decal node!")
            .cast();

        let decal_size = Vector3::new(decal_scale, decal_inst.get_size().y, decal_scale);
        logger::debug!("setting decal size: {decal_size}");
        let decal_normal = point.normal;

        debug_3d!(debugger => decal_normal);

        decal_inst.set_size(decal_size);
        decal_inst.set("is_active", &true.to_variant());

        target_node.upcast_mut().add_child(&decal_inst);

        decal_inst.set_global_position(point.position);
        decal_inst.align_up(decal_normal);
        decal_inst.global_rotate(
            decal_normal,
            rand::random_range::<f32, _>(-90.0..90.0).to_radians(),
        );

        if let Some(scene_root) = target_node
            .upcast_ref()
            .get_tree()
            .and_then(|tree| tree.get_current_scene())
        {
            decal_inst.set_owner(&scene_root);
        } else {
            logger::error!("Failed to access scene root! Unable to set decal owner.");
        }

        decal_inst
    }

    fn show_decal(decal_inst: &mut Gd<Decal>) {
        decal_inst.set_visible(true);
    }

    fn propagate_impact(target: &mut Gd<Node3D>, delta: f64) {
        if !target.has_method("impact_water") {
            return;
        }

        target.call("impact_water", &[delta.to_variant()]);
    }
}
