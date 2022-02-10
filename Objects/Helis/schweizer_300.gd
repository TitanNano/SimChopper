extends RigidBody

const CRUISE_SPEED := 159.0 # km/h
const RATE_OF_CLIMB := 3.8 # m/s
const MAX_TILT := 45.0 # degrees
const RATE_OF_ROTATION = 90 # degrees / s
const CRUISE_SPEED_MS = (CRUISE_SPEED * 1000) / 3600
const ACCELERATION_TIME = 0.2 # amount of seconds to accelerate to top speed
const DIRECTIONAL_ACCELERATION = CRUISE_SPEED_MS / ACCELERATION_TIME
const ROTATIONAL_ACCELERATION = RATE_OF_ROTATION / ACCELERATION_TIME

var directional_velocity := Vector3.ZERO
var rotational_velocity := 0.0
var engine_speed := 0.0
var rotor_rotation := Vector3.ZERO;

onready var audio_fx := $AudioFx
onready var dust_particles := $Dust
onready var rotor := $rotor
onready var rotor_mesh := rotor.get_child(0)
onready var rotor_material: ShaderMaterial = rotor_mesh.get_surface_material(0)

# Called when the node enters the scene tree for the first time.
func _ready():
	$AnimationPlayer.play("Rotor");
	$AnimationPlayer.playback_speed = 0

func _get_top_speed(delta: float) -> float:
	return CRUISE_SPEED_MS * delta


func _get_top_rotation(delta: float) -> float:
	return deg2rad(RATE_OF_ROTATION) * delta


func _update_directional_velocity(direction: Vector3, state: PhysicsDirectBodyState) -> Vector3:
	var acceleration := DIRECTIONAL_ACCELERATION #* delta
	var y_velocity := Vector3.UP * state.linear_velocity

	var target_velocity = (direction * CRUISE_SPEED_MS) + y_velocity
	
#	print("target directional velocity: ", target_velocity)
	return state.linear_velocity.move_toward(target_velocity, acceleration)


func _update_rotational_velocity(direction: float, delta: float) -> float:
	var acceleration := deg2rad(ROTATIONAL_ACCELERATION) * delta
	
	if direction == 0:
		self.rotational_velocity = move_toward(self.rotational_velocity, 0, acceleration)
		return self.rotational_velocity
	
	var target_velocity := self._get_top_rotation(1) * direction
	self.rotational_velocity = move_toward(self.rotational_velocity, target_velocity, acceleration)
	
	return self.rotational_velocity


func _get_tilt(velocity: Vector3) -> Vector3:
	var top_speed := self._get_top_speed(1)
	var direction := velocity.rotated(Vector3.UP, -self.rotation.y) # revert rotation
	var tilt_z = MAX_TILT * (-direction.x / top_speed)
	var tilt_x = MAX_TILT * (direction.z / top_speed)
	var tilt := Vector3(deg2rad(tilt_x), 0, deg2rad(tilt_z))
	
	prints("direction:", velocity)
	prints("tilt direction:", direction)
	prints("z titl:", tilt_z)
	prints("z titl rad:", deg2rad(tilt_z))
	
	return tilt


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _physics_process_disabled(delta: float):	
#	var land_strength := Input.get_action_strength("land")
#	var climb_strength := Input.get_action_strength("rise") if land_strength == 0 else (land_strength * -1)
#	var climb := (RATE_OF_CLIMB * 0.7041980209) * climb_strength * delta
#
#	var movement_strength := (Input.get_action_strength("back") - Input.get_action_strength("forward"))
#	var turn_strength := Input.get_action_strength("turn_left") - Input.get_action_strength("turn_right")
#	var direction := Vector3(-turn_strength, 0, movement_strength).normalized()
#
#	if self.engine_speed < 1:
#		if self.engine_speed > 0 and not audio_fx.playing:
#			audio_fx.play_track(preload("res://Sounds/Helicopter/start.wav"))
#
#		self.engine_speed = min(self.engine_speed + climb * 0.1, 1)
#
#		$AnimationPlayer.playback_speed = self.engine_speed
#		self.dust_particles.strength = self.engine_speed
#		return
#
#	if audio_fx.track_name.ends_with("start.wav"):
#		var sfx = preload("res://Sounds/Helicopter/loop0.wav")
#
#		sfx.loop_mode = AudioStreamSample.LOOP_FORWARD
#		sfx.loop_end = sfx.data.size()
#
#		audio_fx.play_track(sfx)
#
#
#	# apply climb first	
#	self.transform.origin.y += climb
#
#
#	var rotation_velocity := self._update_rotational_velocity(turn_strength, delta)
#	self.rotate_y(deg2rad(rotation_velocity))
#
#	if direction.z == 0:
#		direction = Vector3.ZERO
#
#	var velocity := self._update_directional_velocity(direction, delta) * 0.7041980209
#	var top_speed := self._get_top_speed(delta)
#	var tilt = Vector3(deg2rad(MAX_TILT * (velocity.z / top_speed)), 0, deg2rad(MAX_TILT * (-velocity.x / top_speed)))
#
#	self.rotation = tilt + Vector3(0, self.rotation.y, 0)
#	self.transform.origin += velocity.rotated(Vector3(0, 1, 0), self.rotation.y)


func _physics_process(_delta: float) -> void:
	var ray_cast: RayCast = $RayCast;
	var ground := ray_cast.get_collision_point()
	
	ray_cast.cast_to = ray_cast.global_transform.basis.xform_inv(Vector3.DOWN * 7)
	dust_particles.global_transform.origin = ground


func _process(_delta: float) -> void:
	#rotor velocity
	var rotation_offset: float = abs(rotor.rotation.y - self.rotor_rotation.y)
	var previous_transform := Transform.IDENTITY.rotated(Vector3.FORWARD, min(rotation_offset, 0.610865) * -1)
	
	self.rotor_rotation = Vector3(rotor.rotation)
	
	rotor_material.set_shader_param("previous_transform", previous_transform)



func _integrate_forces(state: PhysicsDirectBodyState) -> void:	
	var delta := state.step
	var land_strength := Input.get_action_strength("land")
	var climb_strength := Input.get_axis("land", "rise")
	var climb := (RATE_OF_CLIMB) * climb_strength if state.linear_velocity.y < RATE_OF_CLIMB else 0.0
	
	var movement_strength := Input.get_axis("forward", "back")
	var turn_strength := Input.get_axis("turn_right", "turn_left")
	var direction := Vector3(-turn_strength, 0, movement_strength).normalized()

	if self.engine_speed < 1:
		if self.engine_speed > 0 and not audio_fx.playing:
			audio_fx.play_track(preload("res://Sounds/Helicopter/start.wav"))

		self.engine_speed = min(self.engine_speed + climb * 0.001, 1)
		
		$AnimationPlayer.playback_speed = self.engine_speed
		self.dust_particles.strength = self.engine_speed
		return
	
	if audio_fx.track_name.ends_with("start.wav"):
		var sfx = preload("res://Sounds/Helicopter/loop0.wav")

		sfx.loop_mode = AudioStreamSample.LOOP_FORWARD
		sfx.loop_end = sfx.data.size()

		audio_fx.play_track(sfx)

	# apply climb first	
	var target_climb_velocity := Vector3(0, climb, 0) + (state.total_gravity * -1 * (1 - land_strength))

#	print("gravity: ", state.total_gravity)
#	print("land strength: ", land_strength)
#	print("current velocity: ", state.linear_velocity)
#	print("climb velocity: ", target_climb_velocity)

	if direction.z == 0:
		direction = Vector3.ZERO

	direction = direction.rotated(Vector3.UP, self.rotation.y)

	# warning-ignore:shadowed_variable
	var directional_velocity := self._update_directional_velocity(direction, state)
	
	# we calculate the velocity difference between what we target and what we actually got 
	var required_force := (directional_velocity + target_climb_velocity - state.linear_velocity) * self.mass

	state.add_central_force(required_force)

	# apply rotational velocity 
	var target_rotation_velocity: float = (self._update_rotational_velocity(turn_strength, delta) if direction == Vector3.ZERO else 0)
	var rotation_velocity := self.global_transform.basis.y * target_rotation_velocity
	# var rotation_torque_offset := rotation_velocity - state.angular_velocity
	# var rotation_torque_impulse := self.get_inverse_inertia_tensor().inverse().xform(rotation_torque_offset)

	# state.apply_torque_impulse(rotation_torque_impulse)
	
	# apply tilt due to linear velocity generated by the engine
	var target_tilt := self._get_tilt(directional_velocity)
	var tilt_offset := target_tilt - (self.rotation * (Vector3.RIGHT + Vector3.BACK))
	var tilt_velocity_x := (tilt_offset.x * self.global_transform.basis.x)
	var tilt_velocity_z := (tilt_offset.z * self.global_transform.basis.z)
	
	var torque_offset := tilt_velocity_x + tilt_velocity_z * 10 + rotation_velocity - state.angular_velocity
	var torque_impulse := self.get_inverse_inertia_tensor().inverse().xform(torque_offset)

#	print("top linear velocity: ", top_speed)
#	print("directional z velocity: ", directional_velocity.z)
	print("target tilt: ", target_tilt)
	print("tilt offset: ", tilt_offset)
	print("orientation offset: ", tilt_velocity_x)
	print("stabilizer impulse: ", torque_impulse)

	state.apply_torque_impulse(torque_impulse)

