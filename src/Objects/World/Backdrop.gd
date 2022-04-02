extends Node

const horizon_depth := 40000
const merge_margin := 4 # this is related to the wave height of the main ocean material
export(ShaderMaterial) var material := preload("res://resources/Materials/ocean_backdrop.tres")

func _ready() -> void:
	pass

func create_mesh_instance(size_x: int, size_z: int, size_city: int) -> MeshInstance:
	# warning-ignore:integer_division
	# warning-ignore:integer_division
	var uv_aspect_ratio := Vector3((size_x / size_city), (size_z / size_city), 1)
	var instance: MeshInstance = MeshInstance.new()

	var inst_material := self.material.duplicate()
	inst_material.set_shader_param('uv_aspectratio', uv_aspect_ratio)

	instance.name = "backdrop"
	instance.mesh = PlaneMesh.new()
	instance.mesh.size = Vector2(size_x, size_z)
	instance.mesh.surface_set_material(0, inst_material)

	return instance


func create_quadrant(size_city: int, size_depth: int, sea_level: int, offset_x: int, offset_z: int) -> MeshInstance:
	var size_x := size_city if offset_x == 1 else size_depth
	var size_z := size_city if offset_z == 1 else size_depth
	var instance := self.create_mesh_instance(size_x, size_z, size_city)

	# warning-ignore:integer_division
	# warning-ignore:integer_division
	instance.translation.z = (size_city / 2 * offset_z) + (size_z / 2 * (offset_z - 1))

	# warning-ignore:integer_division
	# warning-ignore:integer_division
	instance.translation.x = (size_city / 2 * offset_x) + (size_x / 2 * (offset_x - 1))
	instance.translation.y = sea_level - 1

	if offset_z == 0 and offset_x == 1:
		instance.translation.z += merge_margin
	elif offset_z == 1 and offset_x == 0:
		instance.translation.x += merge_margin
	elif offset_z == 1 and offset_x == 2:
		instance.translation.x -= merge_margin
	elif offset_z == 2 and offset_x == 1:
		instance.translation.z -= merge_margin

	return instance


func build(city_size: int, tile_size: int, sea_level: int) -> void:
	var size_city := city_size * tile_size
	var size_depth := horizon_depth + (horizon_depth % size_city)

	for offset_z in range(0, 3):
		for offset_x in range(0, 3):
			if offset_x == 1 and offset_z == 1:
				continue

			var instance := self.create_quadrant(size_city, size_depth, sea_level, offset_x, offset_z)

			self.add_child(instance, true)
