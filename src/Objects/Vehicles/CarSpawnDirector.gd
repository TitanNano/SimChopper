extends Node

const SceneObjectRegistry := preload("res://src/SceneObjectRegistry.gd")
const BuildingIds := SceneObjectRegistry.BuildingIds

@export_group("Related Nodes", "node_")
@export var node_camera: CameraInterpolation
@export var node_road_network: RoadNavigation

func _physics_process(delta):
	var range := (PI * 0.8) / 2
	
	var rand_pos := randf_range(range * -1, range) + -0.5
	var rand_vec := Vector3(cos(rand_pos), 0, sin(rand_pos))
	
	var ray_start := self.node_camera.global_position
	var ray_end := self.node_camera.global_transform * rand_vec
	var ray_m := (ray_end.z - ray_start.z) / (ray_end.x - ray_start.x)
	var ray_b := ray_start.z - ray_m
	# y = x * ray_m + ray_b
	# y - ray_b = x * ray_m
	# x = (y - ray_b) / ray_m
	var road_tiles := self.node_road_network.get_nodes_by_types([BuildingIds.ROAD_LEFT_RIGHT, BuildingIds.ROAD_TOP_BOTTOM])

	for road_tile in road_tiles:
		var position := self.node_road_network.get_global_transform(road_tile).origin
		var alignment := self.node_camera.global_position.direction_to(position).angle_to(ray_end)
		
		if alignment > 0.17:
			continue
		
		var distance_z := position.z - (position.x * ray_m + ray_b)
		var distance_x := position.x - ((position.z - ray_b) / ray_m)
	
