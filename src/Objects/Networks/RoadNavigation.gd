extends Node

const Building := preload("res://src/Objects/Map/Building.gd")
const Logger := preload("res://src/util/Logger.gd")
const CustomProjectSettings := preload("res://src/CustomProjectSettings.gd")

@export var world_constants: WorldConstants

var network := {}
var object_map := {}
var node_map := {}
var neighbor_cache := {}
var rng: RandomNumberGenerator

enum Corners {
	BOTTOM_RIGHT = 0x24,
	BOTTOM_LEFT = 0x25,
	TOP_LEFT = 0x26,
	TOP_RIGHT = 0x23,
}

const DIRECTION := {
	"FORWARD": 0,
	"BACK": 180,
	"LEFT": 90,
	"RIGHT": -90,
}

const ALL_CORNERS := [
	Corners.BOTTOM_LEFT,
	Corners.BOTTOM_RIGHT,
	Corners.TOP_LEFT,
	Corners.TOP_RIGHT
]

const CORNER_DIR = {
	Corners.TOP_LEFT: Vector3.FORWARD + Vector3.LEFT,
	Corners.TOP_RIGHT: Vector3.FORWARD + Vector3.RIGHT,
	Corners.BOTTOM_LEFT: Vector3.DOWN + Vector3.LEFT,
	Corners.BOTTOM_RIGHT: Vector3.DOWN + Vector3.RIGHT
}

func _ready() -> void:
	self.rng = RandomNumberGenerator.new()
	self.rng.randomize()


func insert_node(node: Building) -> void:
	self.network[node.tile_coords()] = node
	self.neighbor_cache.clear()


func get_neighbors(node: Building) -> Array[Building]:
	var tile_coords := node.tile_coords()
	assert(tile_coords.size() == 2)

	if self.neighbor_cache.has(tile_coords):
		return self.neighbor_cache[tile_coords]

	var neighbors: Array[Building] = []
	var x := tile_coords[0]
	var y := tile_coords[1]

	for nc in [PackedInt32Array([x, y-1]), PackedInt32Array([x-1, y]), PackedInt32Array([x+1, y]), PackedInt32Array([x, y+1])]:
		if not self.network.has(nc):
			continue

		neighbors.push_back(self.network[nc])

	self.neighbor_cache[tile_coords] = neighbors

	return neighbors


func associate_object(node: Building, object: Node3D) -> void:
	self.object_map[object] = node
	self.node_map[node] = object


func lookup_node(object: Node3D) -> Building:
	return self.object_map[object]


func lookup_object(node: Building) -> Node3D:
	return self.node_map[node]


static func diagonal_offset(width: float) -> float:
	return sqrt(pow(width, 2) + pow(width, 2)) / 8


func get_global_transform(node: Building, direction := Vector3.ZERO) -> Transform3D:
	var obj := self.lookup_object(node)
	var trans := obj.global_transform
	var node_building_id := node.building_id()

	var width: int = self.world_constants.tile_size
	
	@warning_ignore("unused_variable", "shadowed_variable")
	var diagonal_offset := self.diagonal_offset(width)
	var raw_angle := Vector3.FORWARD.signed_angle_to(direction, Vector3.UP)
	var angle := int(snapped(rad_to_deg(raw_angle), 90))
	var offset := (width / 4.0) * Vector3.RIGHT.rotated(Vector3.UP, angle)

	# set diagonal offset for road courbes
	if node_building_id in ALL_CORNERS:
		offset = CORNER_DIR[node_building_id]
		offset.x *= width / 4.0
		offset.z *= width / 4.0

	# if we have a direction we refine the diagonal offset
	if direction != Vector3.ZERO and node_building_id in ALL_CORNERS:
		offset = CORNER_DIR[node_building_id]
		var unit: float = width / 8.0

		match [angle, node_building_id]:
			[DIRECTION.FORWARD, Corners.BOTTOM_LEFT], \
			[DIRECTION.LEFT, Corners.BOTTOM_LEFT], \
			[DIRECTION.BACK, Corners.BOTTOM_RIGHT], \
			[DIRECTION.LEFT, Corners.BOTTOM_RIGHT], \
			[DIRECTION.FORWARD, Corners.TOP_LEFT], \
			[DIRECTION.RIGHT, Corners.TOP_LEFT], \
			[DIRECTION.BACK, Corners.TOP_RIGHT], \
			[DIRECTION.RIGHT, Corners.TOP_RIGHT]:
				pass

			[DIRECTION.BACK, Corners.BOTTOM_LEFT], \
			[DIRECTION.RIGHT, Corners.BOTTOM_LEFT], \
			[DIRECTION.FORWARD, Corners.BOTTOM_RIGHT], \
			[DIRECTION.RIGHT, Corners.BOTTOM_RIGHT], \
			[DIRECTION.BACK, Corners.TOP_LEFT], \
			[DIRECTION.LEFT, Corners.TOP_LEFT], \
			[DIRECTION.FORWARD, Corners.TOP_RIGHT], \
			[DIRECTION.LEFT, Corners.TOP_RIGHT]:
				unit *= 3

		offset.x *= unit
		offset.z *= unit

	return trans.translated(offset)


func get_nearest_node(global_translation: Vector3) -> Building:
	var distance := -1.0
	var nearest: Node3D = null

	for object in self.node_map.values():
		var new := global_translation.distance_squared_to(object.global_transform.origin)

		Logger.debug(["node distance to actor:", new])

		if distance >= 0 && new > distance:
			continue

		distance = new
		nearest = object

	return self.lookup_node(nearest)


func has_arrived(location: Vector3, direction: Vector3, node: Building) -> bool:
	var target := self.get_global_transform(node, direction).origin

	target = target - Vector3(0, target.y, 0)
	location = location - Vector3(0, location.y, 0)

	var distance := location.distance_squared_to(target)

	Logger.debug(["distance from target:", distance, pow(4, 2)])

	return distance <= pow(4, 2)


func get_next_node(current: Building, target: Building, actor_orientation: Vector3) -> Building:
	var next := current
	var closest := 10.0
	var current_location := self.get_global_transform(current).origin
	var dir_target := current_location.direction_to(self.get_global_transform(target).origin)

	for neighbor in self.get_neighbors(current):
		var neighbor_location := self.get_global_transform(neighbor).origin
		var dir := current_location.direction_to(neighbor_location)
		var angle_actor_orientation := dir.angle_to(actor_orientation)
		var angle := dir.angle_to(dir_target)

		# multiplying the angle betten the target and the neighbor with the
		# angle between the current actor orientation and the required actor
		# orientation, adds so bias towards a neighbor that is in the direction
		# of the actors current orientation.
		var weight := angle * (angle_actor_orientation / 2)

		if closest < weight:
			continue

		closest = weight
		next = neighbor

	assert(next != current)

	return next


func get_random_node() -> Building:
	var keys := self.network.keys()
	var r := self.rng.randi()
	var index := r % keys.size()

	return self.network.get(keys[index])


func update_debug():
	if not ProjectSettings.get(CustomProjectSettings.DEBUG_SHAPES_ROAD_NAVIGATION_DISPLAY_NETWORK):
		return

	for child in self.get_children():
		if not child.is_in_group("debug"):
			continue

		self.remove_child(child)

	for tile_coords in self.network:
		var node: Building = self.network[tile_coords]
		var node_debug = CSGSphere3D.new()

		node_debug.add_to_group("debug")
		node_debug.global_transform = self.get_global_transform(node)
		node_debug.translate(Vector3(0, 4, 0))

		self.add_child(node_debug)

		for con in self.get_neighbors(node):
			var con_debug := CSGCylinder3D.new()
			var node_pos := 	self.get_global_transform(node).origin + Vector3(0, 4, 0)
			var con_pos := 	self.get_global_transform(con).origin + Vector3(0, 4, 0)
			var dis := node_pos.distance_to(con_pos)
			var dir := node_pos.direction_to(con_pos)
			var angle := Vector3.UP.angle_to(dir) * -1
			var rotation_axis = dir.cross(Vector3.UP).normalized()

			con_debug.radius = 0.3
			con_debug.height = dis / 2.0

			self.add_child(con_debug)

			con_debug.global_transform.origin = node_pos
			con_debug.translate(dir * (dis / 4.0))
			con_debug.rotate(rotation_axis, angle)

