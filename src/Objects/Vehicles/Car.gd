extends RigidBody3D

const V3Util := preload("res://src/util/V3Util.gd")
const RoadNavigation := preload("res://src/Objects/Networks/RoadNavigation.gd")
const Building := preload("res://src/Objects/Map/Building.gd")
const Logger := preload("res://src/util/Logger.gd")
const CustomProjectSettings := preload("res://src/CustomProjectSettings.gd")

@export var road_network_path: NodePath

var velocity := 30
var safe_velocity := Vector3.ZERO
var target_angle := 0.0
var rng: RandomNumberGenerator

var ground_normal := Vector3.DOWN

var stuck := 0
var new := true
var last_transform := Transform3D.IDENTITY

var target_nav_node: Building
var current_nav_node: Building

@onready var navigation: NavigationAgent3D = $NavigationAgent3D
@onready var debug_target: MeshInstance3D = $DebugTarget
@onready var ground_detector: RayCast3D = $GroundDetector
@onready var road_network: RoadNavigation = get_node(road_network_path)

func _ready() -> void:
	self.rng = RandomNumberGenerator.new()
	self.rng.randomize()


func activate() -> void:
	if ProjectSettings.get_setting(CustomProjectSettings.DEBUG_SHAPES_ROAD_NAVIGATION_DISPLAY_VEHICLE_TARGET):
		self.remove_child(debug_target)
		self.debug_target.visible = true
		self.get_parent().call_deferred("add_child", debug_target)

	self._on_choose_target()


func _physics_process(_delta: float) -> void:
	if not self.target_nav_node:
		return

	if self.last_transform.origin.is_equal_approx(self.global_transform.origin):
		self.stuck += 1
	else:
		self.stuck = 0

	if self.stuck >= 5:
		Logger.info("despawning stuck car")
		self.queue_free()

	self.last_transform = self.global_transform


	if self.is_target_reached():
		Logger.debug("car navigation finished")
		self.safe_velocity = Vector3.ZERO
		self.new = false
		self._on_choose_target()
		return

	self.ground_normal = self.ground_detector.get_collision_normal() \
		if self.ground_detector.is_colliding() else Vector3.UP

	var target := self.get_next_location()

	# directional velocity
	@warning_ignore("shadowed_variable_base_class")
	var basis := V3Util.basis_from_normal(self.ground_normal)
	var direction := self.global_transform.origin.direction_to(target)
	direction.y = 0

	direction = basis * direction

	var current_velocity := direction * self.velocity

	# rotation
	var angle_dir = (self.global_transform.origin * V3Util.XZ_PLANE).direction_to(target * V3Util.XZ_PLANE)
	@warning_ignore("shadowed_variable")
	var target_angle = Vector3.FORWARD.signed_angle_to(angle_dir, Vector3.UP)
	var angle_offset := self.angular_offset(self.rotation.y, target_angle)

	# ban 180 deg turns
	if not self.new and abs(angle_offset) > deg_to_rad(100):
		Logger.debug(["attempted", rad_to_deg(angle_offset), "deg turn, blocked"])
		self.safe_velocity = Vector3.ZERO
		self._on_choose_target()
		return

	if abs(angle_offset) > deg_to_rad(5):
		current_velocity *= 1

	self.target_angle = target_angle
	self.set_velocity(current_velocity)

	if ProjectSettings.get_setting(CustomProjectSettings.DEBUG_SHAPES_ROAD_NAVIGATION_DISPLAY_VEHICLE_TARGET) \
		and debug_target.is_inside_tree():
			debug_target.global_transform.origin = target


func _on_velocity_computed(new_velocity: Vector3) -> void:
	self.safe_velocity = new_velocity


func _integrate_forces(state: PhysicsDirectBodyState3D) -> void:
	if not self.target_nav_node:
		return

	# add gravity
	if not self.ground_detector.is_colliding():
		Logger.debug("applying gravity")
		self.apply_central_force(state.total_gravity * 300 * state.step * self.mass)

	var applied_velocity := (self.safe_velocity - state.linear_velocity)

	applied_velocity *= min(applied_velocity.length_squared(),  self.safe_velocity.length_squared()) / max(applied_velocity.length_squared(), 1)

	self.apply_central_force(applied_velocity * self.mass)

	# add x rotation for ground alignemnt
	var x_rot := Vector3.UP.signed_angle_to(self.ground_normal, Vector3.RIGHT.rotated(Vector3.UP, self.rotation.y))
	var x_offset := x_rot - self.rotation.x
	var x_angular_vector := self.global_transform.basis * Vector3.RIGHT
	var x_angular := x_angular_vector * x_offset * 6

	# add y rotation for orientation
	@warning_ignore("shadowed_variable")
	var target_angle := self.target_angle
	var offset := self.angular_offset(self.rotation.y, target_angle)

	Logger.debug(["angular offset", offset])

	var angular_vector := self.global_transform.basis * Vector3.UP
	var angular := angular_vector * offset * 12

	var torque_impulse := self.get_inverse_inertia_tensor().inverse() * ((angular + x_angular) - state.angular_velocity)

	self.apply_torque_impulse(torque_impulse)


func _on_choose_target() -> void:
	var target_translation := self.get_random_street_location()

	if not self.current_nav_node:
		var node := self.road_network.get_nearest_node(self.global_transform.origin)

		self.current_nav_node = node

	Logger.debug(["car next target:", target_translation])


func get_random_street_location() -> Vector3:
	var node := self.road_network.get_random_node()

	if self.target_nav_node != null:
		var current := self.current_nav_node.tile_coords()
		var next := node.tile_coords()
		var dist := Vector2(next[0], next[1]) - Vector2(current[0], current[1])

		if dist == Vector2.ZERO:
			return self.get_random_street_location()

		if node == self.current_nav_node:
			Logger.warning("new target is equal to current node")

	self.target_nav_node = node

	return self.road_network.get_global_transform(node).origin


func is_target_reached() -> bool:
	@warning_ignore("shadowed_variable_base_class")
	var rotation := self.global_rotation
	var orientation := Vector3.FORWARD.rotated(Vector3.UP, rotation.y)
	return self.road_network.has_arrived(self.global_transform.origin, orientation, self.target_nav_node)


func get_next_location() -> Vector3:
	var agent_pos := self.global_transform.origin
	var agent_rot := Vector3.FORWARD.rotated(Vector3.UP, self.global_rotation.y)

	Logger.debug(["agent rotation:", self.global_rotation.y])

	if not self.road_network.has_arrived(agent_pos, agent_rot, self.current_nav_node):
		return self.road_network.get_global_transform(self.current_nav_node, agent_rot).origin

	self.current_nav_node = self.road_network.get_next_node(self.current_nav_node, self.target_nav_node, agent_rot)
	return self.get_next_location()


func set_velocity(value: Vector3) -> void:
	self.safe_velocity = value


func is_float_equal(a: float, b: float, precision := 0.001) -> bool:
	return a > (b - precision) && a < (b + precision)


func angular_offset(from: float, to: float) -> float:
	var offset := to - from

	# if offset is larger than 180 degrees we should rather rotate
	# in the other direction
	if abs(offset) > deg_to_rad(180):
		offset = (deg_to_rad(360) - abs(offset)) * sign(offset) * -1

	return offset
