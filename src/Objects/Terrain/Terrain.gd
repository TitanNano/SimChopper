extends Node

const build_progress_steps := 0

signal build_progress(steps)

@export var is_built := false
@export var terrain_material: Material
@export var ocean_material: Material
@export var world_constants: WorldConstants

func _ready() -> void:
	assert(world_constants is WorldConstants, "Terrain.world_contstants is not of type WorldConstants")

func build_async(city: Dictionary):
	var rotation := TerrainRotation.new()
	var builder_factory := TerrainBuilderFactory.new()
	var simulator_settings: Dictionary = city.get("simulator_settings")
	var compass: int = simulator_settings.get("Compass")
	var sea_level: int = simulator_settings.get("GlobalSeaLevel")
	var tilelist: Dictionary = city.get("tilelist")
	var city_size: int = city.get("city_size")

	rotation.set_rotation(compass)

	# make function async
	await get_tree().process_frame

	var materials := {
		"Ground": terrain_material,
		"Water": ocean_material
	}

	var builder = builder_factory.create(tilelist, rotation, materials)

	builder.set_city_size(city_size)
	builder.set_tile_size(self.world_constants.tile_size)
	builder.set_tile_height(self.world_constants.tile_height)
	builder.set_sea_level(sea_level)

	for mesh in builder.build_terain_async():
		var mesh_instance := MeshInstance3D.new()

		mesh_instance.mesh = mesh
		mesh_instance.cast_shadow = MeshInstance3D.SHADOW_CASTING_SETTING_DOUBLE_SIDED
		mesh_instance.create_trimesh_collision()

		self.add_child(mesh_instance, true)
		mesh_instance.owner = get_tree().current_scene
	
	prints("generated terain:", self.get_child_count(), "nodes generated")
