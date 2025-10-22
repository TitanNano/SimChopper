use std::ops::Deref;
use std::{cmp::Ordering, ops::DerefMut};

use anyhow::{bail, Result};
use godot::builtin::{Array, Callable, GString, Variant, Vector3};
use godot::classes::object::ConnectFlags;
use godot::classes::{
    Area3D, Decal, GpuParticles3D, Node, Node3D, Object, PhysicsRayQueryParameters3D,
    RenderingServer, ShapeCast3D,
};
use godot::global::{clampf, randf_range};
use godot::meta::ToGodot;
use godot::obj::bounds::Declarer;
use godot::obj::{Bounds, Gd, Inherits, InstanceId};
use godot::prelude::{GodotClass, NodePath};
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, RsRef};
use itertools::Itertools;

use crate::ext::node_3d::{Node3DExt, Vector3Ext};
use crate::scripts::objects::debugger_3_d::Debugger3D;
use crate::util::logger;
use crate::{debug_3d, util};

#[derive(GodotScript, Debug)]
#[script(base = GpuParticles3D)]
struct WaterJet {
    /// List of ShapeCast3D nodes to aproximate particle impact.
    #[export(node_path = ["ShapeCast3D"])]
    pub impact_cast_paths: Array<NodePath>,

    impact_casts: Vec<Gd<ShapeCast3D>>,

    #[export]
    pub impact_area: OnEditor<Gd<Area3D>>,

    #[export]
    pub decal: OnEditor<Gd<Decal>>,

    #[export]
    pub debugger: Option<RsRef<Debugger3D>>,

    /// Maximum number of decals that will be spawned at an impact point.
    #[export(range(min = 1.0, max = 255.0, step = 1.0))]
    pub max_decal_count: u8,

    /// Maximum decal spawn delay in seconds.
    #[export(range(min = 0.0, max = 20.0, step = 0.1))]
    pub max_delay: f64,

    base: Gd<GpuParticles3D>,
}

struct Intersection {
    position: Vector3,
    normal: Vector3,
}

#[godot_script_impl]
impl WaterJet {
    const MAX_DISTANCE: f64 = 60.0;

    pub fn _ready(&mut self) {
        self.impact_casts = self
            .impact_cast_paths
            .iter_shared()
            .map(|path| self.base.get_node_as(&path))
            .collect();
    }

    fn impact_area(&self) -> &Gd<Area3D> {
        self.impact_area.deref()
    }

    fn decal(&self) -> &Gd<Decal> {
        self.decal.deref()
    }

    fn get_decal_count_at(
        &self,
        point: Vector3,
        extent: Vector3,
        max: u8,
    ) -> Result<Box<[Vector3]>> {
        let Some(physics_world) = self.base.get_world_3d() else {
            bail!("Failed to obtain physics world!");
        };

        let results = RenderingServer::singleton()
            .instances_cull_aabb_ex(godot::builtin::Aabb {
                position: point - extent / 2.0,
                size: extent,
            })
            .scenario(physics_world.get_scenario())
            .done();

        Ok(results
            .as_slice()
            .iter()
            .filter_map(|object_id| {
                let obj = Gd::<Object>::from_instance_id(InstanceId::from_i64(*object_id));

                obj.try_cast::<Decal>().ok()
            })
            .map(|decal| decal.get_global_position())
            .take(max as usize)
            .collect())
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
        #[cfg(debug_assertions)]
        let active = self.base.is_emitting();
        #[cfg(debug_assertions)]
        let area = self.impact_area().get_overlapping_bodies().len() as u32;

        debug_3d!(self.debugger => active, area);

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

        for shape_cast in self.impact_casts.iter() {
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

                let point = Intersection {
                    position: shape_cast.get_collision_point(index),
                    normal: shape_cast.get_collision_normal(index),
                };

                let impact_distance = self.base.get_global_position().distance_to(point.position);
                let target_decal_count = self.max_decal_count;
                let extent = clampf(
                    7.0 / Self::MAX_DISTANCE * (impact_distance as f64),
                    1.0,
                    7.0,
                );
                let decal_scale = (extent * 2.0 / 3.0) as f32;

                let decals_at_point = match self.get_decal_count_at(
                    point.position,
                    Vector3::splat(extent as f32 * 2.0),
                    target_decal_count,
                ) {
                    Ok(count) => count,
                    Err(err) => {
                        logger::error!("{:?}", err.context("Failed to get decals at point!"));
                        return;
                    }
                };

                let decal_inst = (decals_at_point.len() < target_decal_count as usize).then(|| {
                    self.spawn_decal(
                        &decal,
                        &decals_at_point,
                        point,
                        extent,
                        decal_scale,
                        &mut target_node,
                    )
                });

                let impact_delay = (self.max_delay / Self::MAX_DISTANCE * (impact_distance as f64))
                    .min(self.max_delay);

                let mut timer = util::timer(&mut self.base.get_tree().unwrap(), impact_delay);
                let id = decal_inst.as_ref().map(Gd::instance_id);
                let target_id = target_node.instance_id();

                timer
                    .connect_ex(
                        "timeout",
                        &Callable::from_local_fn("timeout", move |_| {
                            let target: Gd<Node3D> =
                                Gd::try_from_instance_id(target_id).map_err(|err| {
                                    logger::error!("Failed to obtain target node ref: {}", err);
                                })?;

                            if let Some(id) = id {
                                let decal: Gd<Decal> =
                                    Gd::try_from_instance_id(id).map_err(|err| {
                                        logger::error!("Failed to obtain decal node ref: {}", err);
                                    })?;

                                Self::show_decal(decal);
                            }

                            Self::propagate_impact(target, delta);

                            Ok(Variant::nil())
                        }),
                    )
                    .flags(ConnectFlags::ONE_SHOT.to_godot() as u32)
                    .done();

                #[cfg(debug_assertions)]
                {
                    decal_spawned = decal_inst.is_some();
                }
                break;
            }
        }

        debug_3d!(self.debugger => shape_cast_name, impacting, decal_spawned);
    }

    fn vector_local_component(vector: Vector3, axis: Vector3, normal: Vector3) -> Vector3 {
        axis.align_up(normal) * vector
    }

    fn shift_new_point(
        origin: Vector3,
        bound: f32,
        radius: f32,
        normal: Vector3,
        existing_points: &[Vector3],
        new_point: Vector3,
    ) -> Vector3 {
        let lower_bound_x = {
            let low_point = origin + Vector3::new(-bound, 0.0, 0.0).align_up(normal);
            let vector =
                Self::vector_local_component(new_point, Vector3::RIGHT, normal) - low_point;

            vector.normalized() * (radius / vector.length_squared())
        };

        let lower_bound_z = {
            let low_point = origin + Vector3::new(0.0, 0.0, -bound).align_up(normal);
            let vector = Self::vector_local_component(new_point, Vector3::BACK, normal) - low_point;

            vector.normalized() * (radius / vector.length_squared())
        };

        let upper_bound_x = {
            let high_point = origin + Vector3::new(bound, 0.0, 0.0).align_up(normal);
            let vector =
                Self::vector_local_component(new_point, Vector3::RIGHT, normal) - high_point;

            vector.normalized() * (radius / vector.length_squared())
        };

        let upper_bound_z = {
            let high_point = origin + Vector3::new(0.0, 0.0, bound).align_up(normal);
            let vector =
                Self::vector_local_component(new_point, Vector3::RIGHT, normal) - high_point;

            vector.normalized() * (radius / vector.length_squared())
        };

        let (positive_push_vectors, negative_push_vectors): (Vec<Vector3>, Vec<Vector3>) =
            existing_points
                .iter()
                .map(|point| {
                    let vector = new_point - *point;
                    let unit_vector = vector.normalized();
                    let length = radius / vector.length_squared();
                    unit_vector * length
                })
                .chain([lower_bound_x, lower_bound_z, upper_bound_x, upper_bound_z])
                .map(|vector| {
                    (
                        Vector3::new(vector.x.max(0.0), vector.y.max(0.0), vector.z.max(0.0)),
                        Vector3::new(vector.x.min(0.0), vector.y.min(0.0), vector.z.min(0.0)),
                    )
                })
                .unzip();

        let (negative_x, negative_y, negative_z): (Vec<_>, Vec<_>, Vec<_>) = negative_push_vectors
            .into_iter()
            .map(|vector| (vector.x, vector.y, vector.z))
            .multiunzip();

        let negative_push_vector = Vector3::new(
            negative_x
                .into_iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater))
                .unwrap_or_default(),
            negative_y
                .into_iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater))
                .unwrap_or_default(),
            negative_z
                .into_iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater))
                .unwrap_or_default(),
        );

        let (positive_x, positive_y, positive_z): (Vec<_>, Vec<_>, Vec<_>) = positive_push_vectors
            .into_iter()
            .map(|vector| (vector.x, vector.y, vector.z))
            .multiunzip();

        let positive_push_vector = Vector3::new(
            positive_x
                .into_iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater))
                .unwrap_or_default(),
            positive_y
                .into_iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater))
                .unwrap_or_default(),
            positive_z
                .into_iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater))
                .unwrap_or_default(),
        );

        new_point + positive_push_vector + negative_push_vector
    }

    fn spawn_decal<T>(
        &self,
        template: &Gd<Decal>,
        decals_at_point: &[Vector3],
        point: Intersection,
        extent: f64,
        decal_scale: f32,
        target_node: &mut Gd<T>,
    ) -> Gd<Decal>
    where
        T: GodotClass + DerefMut<Target = Node> + Inherits<Node>,
        <T as Bounds>::Declarer: Declarer<DerefTarget<T> = T>,
    {
        #[cfg(debug_assertions)]
        let mut debugger = self.debugger.clone();
        let mut decal_inst: Gd<Decal> = template
            .duplicate()
            .expect("Failed to duplicate decal node!")
            .cast();

        let offset = if decals_at_point.is_empty() {
            let offset = Vector3::new(
                randf_range(-extent, extent) as f32,
                0.0,
                randf_range(-extent, extent) as f32,
            )
            .align_up(point.normal);

            let new_point = point.position + offset;

            let shifted = Self::shift_new_point(
                point.position,
                decal_scale,
                extent as f32,
                point.normal,
                decals_at_point,
                new_point,
            );

            shifted - point.position
        } else {
            Vector3::ZERO
        };

        let decal_size = decal_inst.get_size() * Vector3::new(decal_scale, 1.0, decal_scale);
        let decal_normal = point.normal;

        debug_3d!(debugger => decal_normal);

        decal_inst.set_size(decal_size);
        decal_inst.set("is_active", &true.to_variant());

        target_node.add_child(&decal_inst);

        decal_inst.set_global_position(decal_normal);
        decal_inst.align_up(decal_normal);
        decal_inst.global_rotate(decal_normal, randf_range(-90.0, 90.0).to_radians() as f32);
        decal_inst.translate(offset);

        if let Some(scene_root) = target_node
            .get_tree()
            .and_then(|tree| tree.get_current_scene())
        {
            decal_inst.set_owner(&scene_root);
        } else {
            logger::error!("Failed to access scene root! Unable to set decal owner.");
        }

        let surface_intersect =
            self.raycast_to_surface(point.position, point.position + point.normal);

        let surface_intersect = match surface_intersect {
            Ok(value) => value,
            Err(err) => {
                logger::error!("{:?}", err.context("Failed to get surface intersection!"));
                return decal_inst;
            }
        };

        if let Some(surface_intersect) = surface_intersect {
            let decal_normal = surface_intersect.normal;
            decal_inst.set_global_position(surface_intersect.position);
            decal_inst.align_up(decal_normal);
            debug_3d!(debugger => decal_normal);
        }

        decal_inst
    }

    fn show_decal(mut decal_inst: Gd<Decal>) {
        decal_inst.set_visible(true);
    }

    fn propagate_impact(mut target: Gd<Node3D>, delta: f64) {
        if !target.has_method("impact_water") {
            return;
        }

        target.call("impact_water", &[delta.to_variant()]);
    }
}
