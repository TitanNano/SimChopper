extends MeshInstance

const TerrainRotation := preload("TerrainRotation.gdns")
const TerrainBuilderFactory := preload("TerrainBuilderFactory.gdns")
const WorldConstants := preload("res://src/Objects/Data/WorldConstants.gd")

const build_progress_steps := 0

signal build_progress(steps)

export var is_built := false
export var terrain_material: Material
export var ocean_material: Material
export var world_constants: Resource

func _ready() -> void:
	assert(world_constants is WorldConstants, "Terrain.world_contstants is not of type WorldConstants")

func build_async(city: Dictionary):
	var rotation := TerrainRotation.new()
	var builder_factory := TerrainBuilderFactory.new()
	var sea_level = city.simulator_settings["GlobalSeaLevel"]

	rotation.set_rotation(city.simulator_settings['Compass'])

	# make function async
	yield(get_tree(), "idle_frame")

	var materials := {
		"Ground": terrain_material,
		"Water": ocean_material
	}

	var builder = builder_factory.create(city.tilelist, rotation, materials)

	builder.set_city_size(city.city_size)
	builder.set_tile_size (self.world_constants.tile_size)
	builder.set_tile_height(self.world_constants.tile_height)
	builder.set_sea_level(sea_level)

	var mesh: ArrayMesh = builder.build_terain_async()

	self.mesh = mesh
	self.create_trimesh_collision()
