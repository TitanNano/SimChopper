use std::ops::Neg;

use godot::builtin::math::ApproxEq;
use godot::builtin::{Transform3D, Vector2i, Vector3};
use godot::classes::{
    MeshInstance3D, PhysicsDirectBodyState3D, ProjectSettings, RayCast3D, RigidBody3D,
};
use godot::meta::ToGodot;
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, RsRef};

use crate::debug_3d;
use crate::project_settings::CustomProjectSettings;
use crate::road_navigation::RoadNavigationConfig;
use crate::scripts::objects::debugger_3_d::Debugger3D;
use crate::util::{self, logger};
use crate::world::city_data::Building;

#[derive(GodotScript, Debug)]
#[script(base = RigidBody3D)]
struct Car {
    velocity: f32,
    safe_velocity: Vector3,
    target_angle: f32,
    on_ground: bool,
    ground_normal: Vector3,
    stuck: f32,
    new: bool,
    last_transform: Transform3D,
    target_nav_node: Option<Building>,
    current_nav_node: Option<Building>,

    display_vehicle_target: bool,

    #[export]
    pub debug_target: OnEditor<Gd<MeshInstance3D>>,

    #[export]
    pub ground_detector: OnEditor<Gd<RayCast3D>>,

    #[export]
    pub ground_normal_detector: OnEditor<Gd<RayCast3D>>,

    #[export]
    pub road_network: OnEditor<Gd<RoadNavigationConfig>>,

    /// Optional Debugger3D to inspect the internal state of the car.
    #[export]
    pub debugger: Option<RsRef<Debugger3D>>,

    base: Gd<<Self as GodotScript>::Base>,
}

#[godot_script_impl]
impl Car {
    pub fn _init(&mut self) {
        self.velocity = 30.0;
        self.ground_normal = Vector3::DOWN;
        self.new = true;
        self.display_vehicle_target = ProjectSettings::singleton()
            .get_setting(ProjectSettings::DEBUG_SHAPES_ROAD_NAVIGATION_DISPLAY_VEHICLE_TARGET)
            .to();
    }

    /// Original GDScript API
    pub fn activate(&mut self) {
        if self.display_vehicle_target {
            self.base.remove_child(&*self.debug_target);
            self.debug_target.set_visible(true);
            self.base
                .get_parent()
                .map(|mut parent| {
                    parent.call_deferred("add_child", &[self.debug_target.to_variant()]);
                })
                .unwrap_or_else(|| {
                    logger::warn!("Car has no parent! Can't move debug_target.");
                });
        }

        self.on_choose_target();
    }

    pub fn _physics_process(&mut self, delta: f32) {
        if self.target_nav_node.is_none() {
            return;
        }

        if self
            .last_transform
            .origin
            .approx_eq(&self.base.get_global_transform().origin)
        {
            self.stuck += 1.0 * delta;
        } else {
            self.stuck = 0.0;
        }

        if self.stuck >= 5.0 {
            logger::info!("despawning stuck car");
            self.base.queue_free();
            self.debug_target.queue_free();
        }

        self.last_transform = self.base.get_global_transform();

        if self.is_target_reached() {
            logger::debug!("car navigation finished");
            self.safe_velocity = Vector3::ZERO;
            self.new = false;
            self.on_choose_target();
            return;
        }

        self.on_ground = self.ground_detector.is_colliding();
        self.ground_normal = if self.ground_normal_detector.is_colliding() {
            self.ground_normal_detector.get_collision_normal()
        } else {
            Vector3::UP
        };

        let target = self.get_next_location();

        // target debugger
        if self.display_vehicle_target && self.debug_target.is_inside_tree() {
            self.debug_target.set_global_position(target);
        }

        // directional velocity
        let basis = util::basis_from_normal(self.ground_normal);
        let mut direction = self.base.get_global_transform().origin.direction_to(target);

        direction.y = 0.0;
        direction = (basis * direction).normalized();

        let current_velocity = direction * self.velocity;

        // rotation
        let angle_dir = (self.base.get_global_transform().origin * util::vector3::XZ_PLANE)
            .direction_to(target * util::vector3::XZ_PLANE);
        let target_angle = Vector3::FORWARD.signed_angle_to(angle_dir, Vector3::UP);
        let angle_offset = angular_offset(self.base.get_rotation().y, target_angle);

        // ban 180° turns
        if angle_offset.abs() > 100.0f32.to_radians() {
            logger::debug!(
                "Car attempted a {}° turn. Finding new target...",
                angle_offset.to_degrees()
            );

            self.set_velocity(Vector3::ZERO);
            self.current_nav_node = None;
            self.on_choose_target();
            return;
        }

        self.target_angle = target_angle;
        self.set_velocity(current_velocity);

        let stuck = self.stuck;

        debug_3d!(self.debugger => stuck);
    }

    pub fn _integrate_forces(&mut self, state: Gd<PhysicsDirectBodyState3D>) {
        if self.target_nav_node.is_none() {
            return;
        }

        let is_colliding = self.ground_detector.is_colliding();

        // add additional gravity to keep car from jumping
        if self.ground_normal_detector.is_colliding() && !self.on_ground {
            logger::debug!("applying extra gravity");
            let gravity = self.safe_velocity.length() * 20.0;
            debug_3d!(self.debugger => (float gravity));
            let gravity_force =
                self.ground_normal * gravity.neg() * state.get_step() * self.base.get_mass();

            self.base.apply_central_force(gravity_force)
        }

        let applied_velocity = self.safe_velocity - state.get_linear_velocity();
        let applied_force = applied_velocity * self.base.get_mass();

        self.base.apply_central_force(applied_force);

        // add x rotation for ground alignment
        let x_rot = Vector3::UP.signed_angle_to(
            self.ground_normal,
            Vector3::RIGHT.rotated(Vector3::UP, self.base.get_rotation().y),
        );

        let x_offset = x_rot - self.base.get_rotation().x;
        let x_angular_velocity = self.base.get_global_transform().basis * Vector3::RIGHT;
        let x_angular = x_angular_velocity * x_offset * 6.0;

        // add y rotation for orientation
        let target_angle = self.target_angle;
        let offset = angular_offset(self.base.get_rotation().y, target_angle);

        let angular_vector = self.base.get_global_transform().basis * Vector3::UP;
        let angular = angular_vector * offset * 12.0;

        let torque_impulse = self.base.get_inverse_inertia_tensor().inverse()
            * ((angular + x_angular) - state.get_angular_velocity());

        self.base.apply_torque_impulse(torque_impulse);

        debug_3d!(self.debugger => is_colliding, (as_deg x_rot));
    }

    fn on_choose_target(&mut self) {
        let target_translation = self.get_random_street_location();

        if self.current_nav_node.is_none() {
            let road_network = self.road_network.bind();
            let node = road_network
                .road_navigation()
                .get_nearest_node(self.base.get_global_position());

            self.current_nav_node = node.map(|nav_ref| nav_ref.building().clone());
        }

        logger::debug!("car next target: {target_translation}");
    }

    fn get_random_street_location(&mut self) -> Vector3 {
        let road_network = self.road_network.bind();
        let node = road_network.road_navigation().get_random_node();

        if let Some(ref current_target) = self.target_nav_node {
            let current = current_target.tile_coords;
            let next = node.building().tile_coords;

            let dist = Vector2i::new(next.0 as i32, next.1 as i32)
                - Vector2i::new(current.0 as i32, current.1 as i32);

            if dist == Vector2i::ZERO {
                drop(road_network);
                return self.get_random_street_location();
            }
        }

        self.target_nav_node = Some(node.building().clone());

        node.get_global_transform(Vector3::ZERO).origin
    }

    fn is_target_reached(&self) -> bool {
        let rotation = self.base.get_global_rotation();
        let orientation = Vector3::FORWARD.rotated(Vector3::UP, rotation.y);

        let road_network = self.road_network.bind();

        road_network
            .road_navigation()
            .get_node(self.target_nav_node.as_ref().unwrap().tile_coords)
            .unwrap()
            .has_arrived(self.base.get_global_position(), orientation)
    }

    fn get_next_location(&mut self) -> Vector3 {
        let agent_pos = self.base.get_global_transform().origin;
        let agent_rot = Vector3::FORWARD.rotated(Vector3::UP, self.base.get_global_rotation().y);

        // logger::debug!("agent rotation: {}", self.base.get_global_rotation().y);

        let road_network = self.road_network.bind();

        if road_network
            .road_navigation()
            .get_node(self.current_nav_node.as_ref().unwrap().tile_coords)
            .unwrap()
            .has_arrived(agent_pos, agent_rot)
        {
            let current = road_network
                .road_navigation()
                .get_node(self.current_nav_node.as_ref().unwrap().tile_coords)
                .unwrap();
            let target = road_network
                .road_navigation()
                .get_node(self.target_nav_node.as_ref().unwrap().tile_coords)
                .unwrap();

            let next_node = road_network
                .road_navigation()
                .get_next_node(&current, &target, agent_rot);

            self.current_nav_node = Some(next_node.building().clone());
        }

        road_network
            .road_navigation()
            .get_node(self.current_nav_node.as_ref().unwrap().tile_coords)
            .unwrap()
            .get_global_transform(agent_rot)
            .origin
    }

    fn set_velocity(&mut self, value: Vector3) {
        self.safe_velocity = value;
    }
}

#[inline]
fn angular_offset(from: f32, to: f32) -> f32 {
    let mut offset = to - from;

    // if offset is larger than 180 degrees we should rather rotate
    // in the other direction
    if offset.abs() > 180.0f32.to_radians() {
        offset = (360.0f32.to_radians() - offset.abs()) * offset.signum() * -1.0
    }

    offset
}
