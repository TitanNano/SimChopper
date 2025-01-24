extends RigidBody3D

const Rotor := preload("res://src/Objects/Helicopters/Rotor.gd")
const Logger := preload("res://src/util/Logger.gd")

# pseudo constants
@export var CRUISE_SPEED := 159.0 # km/h
@export var RATE_OF_CLIMB := 3.8 # m/s
@export var RATE_OF_ROTATION := 1 # degrees / s

@export_group("Slots", "child_")

@export var child_engine_sound_tree: AnimationTree
@export var child_dust_particles: GPUParticles3D
@export var child_upgrade_mount: Node3D
@export var child_body_mesh: MeshInstance3D
@export var child_rotor: Node3D
@export var child_camera: Node3D
@export var child_main_camera: Node3D
@export var child_debug_camera: Node3D

@export_group("Upgrades", "upgrades_")

@export var upgrades_available: Array[HelicopterUpgrade]
@export var upgrades_owned: Array[HelicopterUpgrade]

const AIR_DENSITY := 1.2
const MAX_TILT := 10.0 # degrees
const ACCELERATION_TIME := 0.4 # amount of seconds to accelerate to top speed
const TILT_ACCELERATION_TIME := 0.4
const THRUST_INCREASE := 1.0 / ACCELERATION_TIME
var CRUISE_SPEED_MS := (CRUISE_SPEED * 1000) / 3600 # cruise velocity in m/s
var THRUST_ACCELERATION := CRUISE_SPEED_MS / ACCELERATION_TIME
var DIRECTIONAL_ACCELERATION := CRUISE_SPEED_MS / ACCELERATION_TIME
var ROTATIONAL_ACCELERATION := RATE_OF_ROTATION / ACCELERATION_TIME
@onready var DRAG_CONSTANT := self.get_drag_constant()

# rotational velocity in local space
var rotational_velocity := 0.0

var engine_speed := 0.0
var engine_thrust := Vector3.ZERO
var is_on_ground := true
var upgrade_action_dispatch: Dictionary = {}

@onready var camera: CameraInterpolation = self.child_camera
@onready var dust_particles: DustParticles = self.child_dust_particles
@onready var rotor: Rotor = self.child_rotor
@onready var main_camera: CameraInterpolation = self.child_main_camera
@onready var debug_camera: CameraInterpolation = self.child_debug_camera


# Called when the node enters the scene tree for the first time.
func _ready():
	self.rotor.power = 0
	self.mount_upgrades()


func _get_top_speed(delta: float) -> float:
	return CRUISE_SPEED_MS * delta


func _get_top_rotation(delta: float) -> float:
	return deg_to_rad(RATE_OF_ROTATION) * delta


func _get_acceleration(direction: Vector3, state: PhysicsDirectBodyState3D) -> Vector3:
	var velocity_target := direction * CRUISE_SPEED_MS
	var velocity_delta := velocity_target - state.linear_velocity
	var normal := velocity_delta.normalized()
	var signs := normal.sign()
	var acceleration := normal * DIRECTIONAL_ACCELERATION

	acceleration = _min_vector3(acceleration.abs(), velocity_delta.abs()) * signs
	
	return acceleration


func _min_vector3(a: Vector3, b: Vector3) -> Vector3:
	return Vector3(
		minf(a.x, b.x),
		minf(a.y, b.y),
		minf(a.z, b.z)
	)


func _update_rotational_velocity(direction: float, delta: float) -> float:
	var acceleration := deg_to_rad(ROTATIONAL_ACCELERATION)

	if direction == 0:
		var sign := signf(self.rotational_velocity)
		self.rotational_velocity = min(acceleration, abs(self.rotational_velocity)) * sign * -1.0
		return self.rotational_velocity

	var target_velocity := RATE_OF_ROTATION * direction
	var velocity_delta := target_velocity - self.rotational_velocity
	var sign := signf(velocity_delta)

	self.rotational_velocity += minf(absf(acceleration), absf(velocity_delta)) * sign

	return self.rotational_velocity


func _get_tilt(rotation_velocity: float, delta: float) -> Vector3:
	var virt_thrust := self.engine_thrust.rotated(Vector3.UP, rotation_velocity * (1 + delta))
	
	var tilt_z := MAX_TILT * -virt_thrust.x
	var tilt_x := MAX_TILT * virt_thrust.z
	var tilt := Vector3(deg_to_rad(tilt_x), 0, deg_to_rad(tilt_z))

	return tilt

func _process(_delta):
	if Input.is_action_just_pressed("debug_cam"):
		self.switch_debug_camera()


func _physics_process(_delta: float) -> void:
	var ray_cast: RayCast3D = $RayCast3D;
	var ground := ray_cast.get_collision_point()
	var colliding := ray_cast.is_colliding()

	ray_cast.target_position = Vector3.DOWN * 7 * ray_cast.global_transform.basis
	
	var distance := ray_cast.global_transform.origin - ground
	var dust_strength := 1 - (distance.length() / 7)

	self.is_on_ground = dust_strength > 0.99
	self.rotor.power = self.engine_speed
	@warning_ignore("unsafe_property_access")
	self.dust_particles.strength = self.engine_speed * dust_strength if colliding else 0.0

	if colliding:
		self.dust_particles.global_transform.origin = ground


func _integrate_forces(state: PhysicsDirectBodyState3D) -> void:
	var delta := state.step
	var land_strength := Input.get_action_strength("land")
	var climb_strength := Input.get_axis("land", "rise")
	var climb := (RATE_OF_CLIMB) * (climb_strength if climb_strength > land_strength else land_strength * -1)

	var movement_strength := Input.get_axis("forward", "back")
	var strafe_strength := Input.get_axis("strafe_right", "strafe_left")
	var turn_strength := Input.get_axis("turn_right", "turn_left") if strafe_strength == 0.0 else 0.0
	var direction := Vector3(-strafe_strength, 0, movement_strength).normalized()
	var sound_state_machine := anim_state_machine(self.child_engine_sound_tree)

	if climb < 0 && self.is_on_ground && self.engine_speed == 1:
		self.engine_speed -= 0.001

	if self.engine_speed < 1:
		if climb > 0:
			self.engine_speed = min(self.engine_speed + 0.36 * delta, 1)
			Logger.info(["state: ", sound_state_machine.get_current_node(), "Engine: ", self.engine_speed])
		elif self.engine_speed > 0:
			self.engine_speed = max(self.engine_speed - 0.36 * delta, 0)
		
		self.bind_states(climb_strength)
		return

	self.bind_states(climb_strength)

	# apply climb first
	var target_climb_velocity := climb + (state.total_gravity.y * -1)

	# apply rotational velocity
	var rotation_velocity := self._update_rotational_velocity(turn_strength, delta)
	var global_rotation_velocity := self.global_transform.basis.y * rotation_velocity

	# calculate thrust
	var thrust_increase := direction * THRUST_INCREASE * delta

	if direction.is_zero_approx():
		var old_thrust := self.engine_thrust
		self.engine_thrust -= self.engine_thrust.normalized() * THRUST_INCREASE * delta
		self.engine_thrust = self.engine_thrust.clamp(Vector3.ZERO, old_thrust)
	elif self.engine_thrust.length() < 1.0:
		self.engine_thrust += thrust_increase

		if self.engine_thrust.length() > 1.0:
			self.engine_thrust = self.engine_thrust.normalized()

	var current_linear_velocity := state.linear_velocity * (Vector3.RIGHT + Vector3.BACK)
	var drag_force := self.drag_force(current_linear_velocity, self.get_front_size())
	var thrust_force := self.engine_thrust * (THRUST_ACCELERATION * self.mass)
	var rotated_thrust_force := thrust_force.rotated(Vector3.UP, self.rotation.y)
	var climb_force := Vector3(0, target_climb_velocity - state.linear_velocity.y, 0) * self.mass

	state.apply_central_force(rotated_thrust_force + (drag_force * -1) + climb_force)
	
	# apply tilt due to linear velocity generated by the engine
	var target_tilt := self._get_tilt(rotation_velocity, delta)
	var tilt_offset := target_tilt - (self.rotation * (Vector3.RIGHT + Vector3.BACK))
	var tilt_transform := self.global_transform.rotated(Vector3.UP, rotation_velocity * 1.5)
	var tilt_velocity_x := (tilt_offset.x * tilt_transform.basis.x)
	var tilt_velocity_z := (tilt_offset.z * tilt_transform.basis.z)
	var tilt_velocity := tilt_velocity_x + tilt_velocity_z
	
	var target_torque_velocity := tilt_velocity + global_rotation_velocity
	var torque_offset := target_torque_velocity - state.angular_velocity
	
	var torque := self.get_inverse_inertia_tensor().inverse() * torque_offset

	state.apply_torque(torque / delta)


func snap_camera():
	self.camera.snap = true

func anim_state_machine(tree: AnimationTree) -> AnimationNodeStateMachinePlayback:
	var playback: AnimationNodeStateMachinePlayback = tree.get("parameters/playback")
	
	return playback


func bind_states(climb_strength: float):
	self.child_engine_sound_tree.set("parameters/conditions/engine_off", self.engine_speed == 0 and climb_strength == 0)
	self.child_engine_sound_tree.set("parameters/conditions/lift_off", self.engine_speed == 1)
	self.child_engine_sound_tree.set("parameters/conditions/spin_down", self.engine_speed < 1 and climb_strength == 0)
	self.child_engine_sound_tree.set("parameters/conditions/spin_up", self.engine_speed < 1 and climb_strength > 0)


func switch_debug_camera():
	if self.main_camera.active:
		self.debug_camera.active = true
	else:
		self.main_camera.active = true


func _unhandled_key_input(event: InputEvent):
	for action: StringName in upgrade_action_dispatch:
		var target: Node3D = upgrade_action_dispatch[action]

		if event.is_action_pressed(action):
			target.call("action_start", action)
			continue
		
		if event.is_action_released(action):
			target.call("action_end", action)
			continue
			
func mount_upgrades():
	for upgrade in self.upgrades_owned:
		var scene = upgrade.object
		var object := scene.instantiate()
		var duplicate := false

		object.set_meta("scene_instance_id", scene.get_instance_id())

		for child in self.child_upgrade_mount.get_children():
			if child.get_meta("scene_instance_id") == scene.get_instance_id():
				duplicate = true
				break

		if duplicate:
			return

		self.child_upgrade_mount.add_child(object, true)
		object.owner = self.child_upgrade_mount
		self.upgrade_action_dispatch[upgrade.action] = object


func drag_force(velocity: Vector3, area: float) -> Vector3:
	var velocity_magnitude := velocity.length()
	var direction := velocity.normalized()

	var force = 0.5 * AIR_DENSITY * (velocity_magnitude ** 2) * area * DRAG_CONSTANT
	
	return direction * force


func get_model_size() -> Vector3:      
	var mesh_aabb := self.child_body_mesh.get_aabb()
	var mesh_basis := self.child_body_mesh.basis
	
	return (Transform3D(mesh_basis, Vector3.ZERO) * mesh_aabb.size).abs()


func get_front_size() -> float:
	var size := self.get_model_size()
	
	return size.x * size.y
	

func get_drag_constant() -> float:
	# 0.5 * AIR_DENSITY * velocity ^ 2 * area * dc = acceleartion * mass
	return (CRUISE_SPEED_MS / ACCELERATION_TIME) * self.mass / 0.5 / AIR_DENSITY / (CRUISE_SPEED_MS ** 2) / self.get_front_size()
