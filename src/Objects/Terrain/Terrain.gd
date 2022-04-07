extends MeshInstance

const TerrainRotation := preload("TerrainRotation.gdns")
const TerrainBuilderFactory := preload("TerrainBuilderFactory.gdns")

const build_progress_steps := 0
const tile_size := 16
const tile_height := 8

signal build_progress(steps)

export var is_built := false
export(Material) var terrain_material = null
export(Material) var ocean_material = null

var sea_level := 0

func _ready() -> void:
	pass

func build_async(city: Dictionary):
	var rotation := TerrainRotation.new()
	var builder_factory := TerrainBuilderFactory.new()

	self.sea_level = city.simulator_settings["GlobalSeaLevel"]
	rotation.set_rotation(city.simulator_settings['Compass'])

	# make function async
	yield(get_tree(), "idle_frame")

	var materials := {
		"Ground": terrain_material,
		"Water": ocean_material
	}

	var builder = builder_factory.create(city.tilelist, rotation, materials)

	builder.set_city_size(city.city_size)
	builder.set_tile_size (self.tile_size)
	builder.set_tile_height(self.tile_height)
	builder.set_sea_level(self.sea_level)

	var mesh: ArrayMesh = builder.build_terain_async()

	self.mesh = mesh
	self.create_trimesh_collision()
