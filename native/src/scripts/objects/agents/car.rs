use std::ops::Neg;

use godot::builtin::math::ApproxEq;
use godot::builtin::{Transform3D, Vector2i, Vector3};
use godot::classes::{
    MeshInstance3D, PhysicsDirectBodyState3D, ProjectSettings, RayCast3D, RigidBody3D,
};
use godot::meta::ToGodot;
use godot::obj::{Gd, Singleton as _};
use godot_rust_script::{godot_script_impl, GodotScript, OnEditor, RsRef};

use crate::debug_3d;
use crate::project_settings::CustomProjectSettings;
use crate::road_navigation::{NavNodeRef, RoadNavigation, RoadNavigationConfig};
use crate::scripts::objects::debugger_3_d::Debugger3D;
use crate::util::{self, logger};
use crate::world::city_data::TileCoords;

#[derive(Default, Debug, Clone)]
enum Navigation {
    #[default]
    Uninitialized,
    Located(TileCoords),
    Targeted(TargetedNavigation),
    Moving(MovingNavigation),
}

impl Navigation {
    fn as_moving(&self) -> &'_ MovingNavigation {
        let Navigation::Moving(value_ref) = self else {
            panic!("Navigation is not Moving");
        };

        value_ref
    }
}

impl From<TargetedNavigation> for Navigation {
    fn from(value: TargetedNavigation) -> Self {
        Self::Targeted(value)
    }
}

impl From<MovingNavigation> for Navigation {
    fn from(value: MovingNavigation) -> Self {
        Self::Moving(value)
    }
}

#[derive(Debug, Clone)]
struct TargetedNavigation {
    current_node: TileCoords,
    target_node: TileCoords,
}

#[derive(Debug, Clone)]
struct MovingNavigation {
    current: TileCoords,
    target: TileCoords,
    next: TileCoords,
}

#[derive(GodotScript, Debug)]
#[script(base = RigidBody3D)]
struct Car {
    velocity: f32,
    safe_velocity: Vector3,
    target_angle: f32,
    on_ground: bool,
    ground_normal: Vector3,
    stuck: f32,
    last_transform: Transform3D,
    navigation: Navigation,

    display_vehicle_target: bool,

    #[export]
    pub debug_target: OnEditor<Gd<MeshInstance3D>>,

    #[export]
    pub ground_detector: OnEditor<Gd<RayCast3D>>,

    #[export]
    pub ground_normal_detector: OnEditor<Gd<RayCast3D>>,

    #[export]
    pub road_network: OnEditor<Gd<RoadNavigationConfig>>,

    /// Optional [`Debugger3D`] to inspect the internal state of the car.
    #[export]
    pub debugger: Option<RsRef<Debugger3D>>,

    base: Gd<<Self as GodotScript>::Base>,
}

#[godot_script_impl]
impl Car {
    pub fn _init(&mut self) {
        self.velocity = 30.0;
        self.ground_normal = Vector3::DOWN;
        self.display_vehicle_target = ProjectSettings::singleton()
            .get_setting(ProjectSettings::DEBUG_SHAPES_ROAD_NAVIGATION_DISPLAY_VEHICLE_TARGET)
            .to();
    }

    /// Original `GDScript` API
    pub fn activate(&mut self) {
        if self.display_vehicle_target {
            self.base.remove_child(&*self.debug_target);
            self.debug_target.set_visible(true);
            self.base.get_parent().map_or_else(
                || {
                    logger::warn!("Car has no parent! Can't move debug_target.");
                },
                |mut parent| {
                    parent.call_deferred("add_child", &[self.debug_target.to_variant()]);
                },
            );
        }

        self.choose_target();
    }

    /// Godot's physics callback called at the projects physics step.
    #[allow(clippy::used_underscore_items)]
    pub fn _physics_process(&mut self, delta: f32) {
        // unit vector that points in the direction of the agents heading.
        let agent_rot = Vector3::FORWARD.rotated(Vector3::UP, self.base.get_global_rotation().y);
        let agent_pos = self.base.get_global_transform().origin;

        let navigation = match &self.navigation {
            Navigation::Uninitialized | Navigation::Located(_) => return,
            Navigation::Targeted(targeted_navigation) => {
                let road_network = self.road_network.bind();
                let road_navigation = road_network.road_navigation();

                let current_node = road_navigation.node(targeted_navigation.current_node);
                let target_node = road_navigation.node(targeted_navigation.target_node);

                self.navigation = self
                    .get_next_node(&current_node, &target_node, agent_rot)
                    .into();
                self.navigation.as_moving()
            }
            Navigation::Moving(moving_navigation) => {
                let road_network = self.road_network.bind();
                let road_navigation = road_network.road_navigation();

                let current_node = road_navigation.node(moving_navigation.next);
                let target_node = road_navigation.node(moving_navigation.target);

                if current_node.has_arrived(agent_pos, agent_rot) {
                    self.navigation = self
                        .get_next_node(&current_node, &target_node, agent_rot)
                        .into();
                    self.navigation.as_moving()
                } else {
                    moving_navigation
                }
            }
        };

        self.stuck = if self.last_transform.origin.approx_eq(&agent_pos) {
            self.stuck + 1.0 * delta
        } else {
            0.0
        };

        if self.stuck >= 5.0 {
            logger::info!("despawning stuck car");
            self.base.queue_free();
            self.debug_target.queue_free();
        }

        self.last_transform = self.base.get_global_transform();

        if Self::is_target_reached(navigation) {
            logger::debug!("car navigation finished");
            self.set_velocity(Vector3::ZERO);
            self.choose_target();
            return;
        }

        self.on_ground = self.ground_detector.is_colliding();
        self.ground_normal = if self.ground_normal_detector.is_colliding() {
            self.ground_normal_detector.get_collision_normal()
        } else {
            Vector3::UP
        };

        let target = self.get_next_pos(navigation, agent_rot);

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
        // let angle_offset = angular_offset(self.base.get_rotation().y, target_angle);

        // // ban 180° turns
        // if angle_offset.abs() > 100.0f32.to_radians() {
        //     logger::debug!(
        //         "Car attempted a {}° turn. Finding new target...",
        //         angle_offset.to_degrees()
        //     );

        //     self.set_velocity(Vector3::ZERO);
        //     self.choose_target();
        //     return;
        // }

        self.target_angle = target_angle;
        self.set_velocity(current_velocity);

        #[cfg(debug_assertions)]
        let stuck = self.stuck;

        debug_3d!(self.debugger => stuck);
    }

    /// Godot's physics body force callback.
    #[expect(clippy::needless_pass_by_value)]
    pub fn _integrate_forces(&mut self, state: Gd<PhysicsDirectBodyState3D>) {
        let Navigation::Moving(_) = self.navigation else {
            return;
        };

        #[cfg(debug_assertions)]
        let is_colliding = self.ground_detector.is_colliding();
        let needs_extra_gravity = self.ground_normal_detector.is_colliding() && !self.on_ground;

        // add additional gravity to keep car from jumping
        if needs_extra_gravity {
            let gravity = self.safe_velocity.length() * 20.0;
            debug_3d!(self.debugger => (float gravity));
            let gravity_force =
                self.ground_normal * gravity.neg() * state.get_step() * self.base.get_mass();

            self.base.apply_central_force(gravity_force);
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

        debug_3d!(self.debugger => is_colliding, (as_deg x_rot), needs_extra_gravity);
    }

    /// select a random navigation target on the road network.
    ///
    /// Selects a new target node and optionally locates the actor on the road network first.
    fn choose_target(&mut self) {
        let navigation = if matches!(self.navigation, Navigation::Uninitialized) {
            let road_network = self.road_network.bind();
            let node = road_network
                .road_navigation()
                .get_nearest_node(self.base.get_global_position());

            let Some(tile_coords) = node.map(|nav_ref| nav_ref.tile_coords()) else {
                logger::warn!("Unable to locate car on road network!");
                return;
            };

            Navigation::Located(tile_coords)
        } else {
            self.navigation.clone()
        };

        let road_network = self.road_network.bind();
        let road_navigation = road_network.road_navigation();

        let (current_node, next_target) = match navigation {
            Navigation::Uninitialized => unreachable!(),
            Navigation::Located(current_node) => (
                current_node,
                Self::get_random_street_location(road_navigation, None),
            ),

            Navigation::Targeted(TargetedNavigation {
                current_node: current,
                target_node: target,
            })
            | Navigation::Moving(MovingNavigation {
                current,
                target,
                next: _,
            }) => (
                current,
                Self::get_random_street_location(road_navigation, Some(target)),
            ),
        };

        logger::debug!(
            "car next target: {}",
            next_target.get_global_transform(Vector3::ZERO).origin
        );

        self.navigation = TargetedNavigation {
            current_node,
            target_node: next_target.tile_coords(),
        }
        .into();
    }

    /// Returns a random node of the street network.
    fn get_random_street_location(
        road_navigation: &RoadNavigation,
        target_node: Option<TileCoords>,
    ) -> NavNodeRef<'_> {
        let node = road_navigation.get_random_node();

        if let Some(ref current) = target_node {
            let next = node.tile_coords();

            let dist = Vector2i::new(
                next.0.try_into().expect("tile coords should fit into i32"),
                next.1.try_into().expect("tile coords should fit into i32"),
            ) - Vector2i::new(
                current
                    .0
                    .try_into()
                    .expect("tile coords should fit into i32"),
                current
                    .1
                    .try_into()
                    .expect("tile coords should fit into i32"),
            );

            if dist == Vector2i::ZERO {
                return Self::get_random_street_location(road_navigation, target_node);
            }
        }

        node
    }

    /// Check if the selected target has been reached.
    fn is_target_reached(navigation: &MovingNavigation) -> bool {
        // The target has been reached when all three node references are equal.
        navigation.current == navigation.next && navigation.current == navigation.target
    }

    /// Determines the next node on the path to the target node.
    fn get_next_node(
        &self,
        current_node: &NavNodeRef<'_>,
        target_node: &NavNodeRef<'_>,
        agent_rot: Vector3,
    ) -> MovingNavigation {
        let road_network = self.road_network.bind();

        let next_node =
            road_network
                .road_navigation()
                .get_next_node(current_node, target_node, agent_rot);

        MovingNavigation {
            current: current_node.tile_coords(),
            target: target_node.tile_coords(),
            next: next_node.tile_coords(),
        }
    }

    /// Get the world position of the next node on the path to the target.
    fn get_next_pos(&self, navigation: &MovingNavigation, agent_rot: Vector3) -> Vector3 {
        let road_network = self.road_network.bind();
        let next_node = road_network.road_navigation().node(navigation.next);

        next_node.get_global_transform(agent_rot).origin
    }

    fn set_velocity(&mut self, value: Vector3) {
        self.safe_velocity = value;
    }
}

#[inline]
fn angular_offset(from: f32, to: f32) -> f32 {
    let mut offset = to - from;

    // If offset is larger than 180 degrees we should rather rotate
    // in the other direction
    if offset.abs() > 180.0f32.to_radians() {
        offset = -((360.0f32.to_radians() - offset.abs()) * offset.signum());
    }

    offset
}
