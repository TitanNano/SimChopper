extends Reference

const WorldConstants := preload("res://src/Objects/Data/WorldConstants.gd")

var world_constants: WorldConstants
var sea_level: int


func _init(world_constants: WorldConstants, sea_level: int) -> void:
	self.world_constants = world_constants
	self.sea_level = sea_level


func get_world_coords(x: int, y: int, z: int) -> Vector3:
	return Vector3(
		(x * self.world_constants.tile_size),
		max(z, self.sea_level - 1) * self.world_constants.tile_height,
		(y * self.world_constants.tile_size)
	)


func get_building_coords(x: int, y: int, z: int, size: int) -> Vector3:
	var offset: float = (size * self.world_constants.tile_size / 2.0)

	# OpenCity2k gets the bottom left corner, we have to correct that.
	y -= (size - 1)

	var location := self.get_world_coords(x, y, z)

	location.x += offset
	location.z += offset

	return location
