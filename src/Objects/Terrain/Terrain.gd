extends Node

const CityCoordsFeature := preload("res://src/features/CityCoordsFeature.gd")

signal build_progress(steps: int)

@export var is_built := false
@export var terrain_material: Material
@export var ocean_material: Material
@export var world_constants: WorldConstants

var city_coords_feature: CityCoordsFeature
var builder: TerrainBuilder

func _ready() -> void:
	assert(world_constants is WorldConstants, "Terrain.world_contstants is not of type WorldConstants")

func _forward_progress(count: int):
	self.build_progress.emit(count)

func init(city: Dictionary):
	var rotation := TerrainRotation.new()
	var builder_factory := TerrainBuilderFactory.new()
	var simulator_settings: Dictionary = city.get("simulator_settings")
	var compass: int = simulator_settings.get("Compass")
	var sea_level: int = simulator_settings.get("GlobalSeaLevel")
	var tilelist: Dictionary = city.get("tilelist")
	var city_size: int = city.get("city_size")
	
	self.city_coords_feature = CityCoordsFeature.new(self.world_constants, sea_level)

	rotation.set_rotation(compass)

	var materials := {
		"Ground": terrain_material,
		"Water": ocean_material
	}

	self.builder = builder_factory.create(tilelist, rotation, materials)

	self.builder.set_city_size(city_size)
	self.builder.set_tile_size(self.world_constants.tile_size)
	self.builder.set_tile_height(self.world_constants.tile_height)
	self.builder.set_sea_level(sea_level)
	self.builder.progress.connect(self._forward_progress)


func load_steps() -> int:
	var steps := self.builder.load_steps()
	prints("load steps:", steps)
	return steps


func build_async():
	var chunks: Array[TerrainChunk] = await self.builder.build_terain_async().completed
	
	for item in chunks:
		var chunk: TerrainChunk = item
		var tile_coords: Array[int] = chunk.tile_coords()
		var translation := self.city_coords_feature.get_world_coords(tile_coords[0], tile_coords[1], 0)
		
		translation.y = 0
		
		var mesh_instance := MeshInstance3D.new()

		mesh_instance.mesh = chunk.mesh()
		mesh_instance.cast_shadow = MeshInstance3D.SHADOW_CASTING_SETTING_DOUBLE_SIDED
		mesh_instance.create_trimesh_collision()

		self.add_child(mesh_instance, true)
		mesh_instance.owner = get_tree().current_scene
		mesh_instance.translate(translation)
	
	prints("generated terain:", self.get_child_count(), "nodes generated")
