extends RigidBody3D

const Rotor := preload("res://src/Objects/Helicopters/Rotor.gd")
const Logger := preload("res://src/util/Logger.gd")

# pseudo constants
@export var CRUISE_SPEED := 159.0 # km/h
@export var RATE_OF_CLIMB := 3.8 # m/s
@export var RATE_OF_ROTATION := 90 # degrees / s

@export_group("Slots", "child_")

@export var child_engine_sound_tree: AnimationTree
@export var child_camera: Node3D
@export var child_dust_particles: GPUParticles3D
@export var child_rotor: Node3D
@export var child_main_camera: Node3D
@export var child_debug_camera: Node3D

const MAX_TILT := 45.0 # degrees
const ACCELERATION_TIME := 0.4 # amount of seconds to accelerate to top speed
var CRUISE_SPEED_MS := (CRUISE_SPEED * 1000) / 3600
var DIRECTIONAL_ACCELERATION := CRUISE_SPEED_MS / ACCELERATION_TIME
var ROTATIONAL_ACCELERATION := RATE_OF_ROTATION / ACCELERATION_TIME

var directional_velocity := Vector3.ZERO
var rotational_velocity := 0.0
var engine_speed := 0.0

@onready var camera: CameraInterpolation = self.child_camera
@onready var dust_particles: DustParticles = self.child_dust_particles
@onready var rotor: Rotor = self.child_rotor
@onready var main_camera: CameraInterpolation = self.child_main_camera
@onready var debug_camera: CameraInterpolation = self.child_debug_camera


# Called when the node enters the scene tree for the first time.
func _ready():
	self.rotor.power = 0

func _get_top_speed(delta: float) -> float:
	return CRUISE_SPEED_MS * delta


func _get_top_rotation(delta: float) -> float:
	return deg_to_rad(RATE_OF_ROTATION) * delta


func _update_directional_velocity(direction: Vector3, state: PhysicsDirectBodyState3D) -> Vector3:
	var acceleration := DIRECTIONAL_ACCELERATION
	var target_velocity := (direction * CRUISE_SPEED_MS)

	return state.linear_velocity.move_toward(target_velocity, acceleration)


func _update_rotational_velocity(direction: float, delta: float) -> float:
	var acceleration := deg_to_rad(ROTATIONAL_ACCELERATION) * delta

	if direction == 0:
		self.rotational_velocity = move_toward(self.rotational_velocity, 0, acceleration)
		return self.rotational_velocity

	var target_velocity := self._get_top_rotation(1) * direction
	self.rotational_velocity = move_toward(self.rotational_velocity, target_velocity, acceleration)

	return self.rotational_velocity


func _get_tilt(velocity: Vector3) -> Vector3:
	var top_speed := self._get_top_speed(1)
	var direction := velocity.rotated(Vector3.UP, -self.rotation.y) # revert rotation
	var tilt_z := MAX_TILT * (-direction.x / top_speed)
	var tilt_x := MAX_TILT * (direction.z / top_speed)
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
	var turn_strength := Input.get_axis("turn_right", "turn_left")
	var direction := Vector3(-turn_strength, 0, movement_strength).normalized()
	var sound_state_machine := anim_state_machine(self.child_engine_sound_tree)

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
	var target_climb_velocity := Vector3(0, climb, 0) + (state.total_gravity * -1)

	if direction.z == 0:
		direction = Vector3.ZERO

	direction = direction.rotated(Vector3.UP, self.rotation.y)

	@warning_ignore("shadowed_variable")
	var directional_velocity := self._update_directional_velocity(direction, state)

	# we calculate the velocity difference between what we target and what we actually got
	var required_force := ((directional_velocity + target_climb_velocity) - state.linear_velocity) * self.mass

	state.apply_central_force(required_force)

	# apply rotational velocity
	@warning_ignore("incompatible_ternary")
	var target_rotation_velocity: float = (self._update_rotational_velocity(turn_strength, delta) if direction == Vector3.ZERO else 0)
	var rotation_velocity := self.global_transform.basis.y * target_rotation_velocity

	# apply tilt due to linear velocity generated by the engine
	var target_tilt := self._get_tilt(directional_velocity)
	var tilt_offset := target_tilt - (self.rotation * (Vector3.RIGHT + Vector3.BACK))
	var tilt_velocity_x := (tilt_offset.x * self.global_transform.basis.x)
	var tilt_velocity_z := (tilt_offset.z * self.global_transform.basis.z)

	var torque_offset := tilt_velocity_x + tilt_velocity_z * 10 + rotation_velocity - state.angular_velocity
	var torque_impulse := self.get_inverse_inertia_tensor().inverse() * torque_offset

	state.apply_torque_impulse(torque_impulse)


func snap_camera():
	self.camera.snap = true

func anim_state_machine(tree: AnimationTree) -> AnimationNodeStateMachinePlayback:
	return tree.get("parameters/playback") as AnimationNodeStateMachinePlayback

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
