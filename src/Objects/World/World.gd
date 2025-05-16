extends Node3D

const TimeBudget := preload("../../util/TimeBudget.gd")
const MsgPack := preload("../../godot-msgpack/msgpack.gd")
const SceneObjectRegistry := preload("res://src/SceneObjectRegistry.gd")
const Networks := preload("res://src/Objects/World/Networks.gd")
const CityCoordsFeature := preload("res://src/features/CityCoordsFeature.gd")
const Logger := preload("res://src/util/Logger.gd")
const Terrain := preload("res://src/Objects/Terrain/Terrain.gd")
const Backdrop := preload("res://src/Objects/World/Backdrop.gd")
const Helicopter := preload("res://src/Objects/Helicopters/Helicopter.gd")

signal loading_progress(count)
signal loading_scale(count)

@export var world_constants: WorldConstants

@onready var terrain: Terrain = $Terrain
@onready var networks: Networks = $Networks
@onready var buildings: Buildings = $Buildings
@onready var backdrop: Backdrop = $Backdrop

var sea_level: int
var city_coords_feature: CityCoordsFeature

func _ready():
	assert(world_constants is WorldConstants, "world_constants is not of type WorldConstants")

	self.networks.loading_progress.connect(self._on_child_progress)
	self.buildings.spawn_point_encountered.connect(self._on_spawn_point_encountered)
	self.buildings.loading_progress.connect(self._on_child_progress)
	self.terrain.build_progress.connect(self._on_child_progress)

	self._ready_deferred.call_deferred()


func _ready_deferred():
	var file := FileAccess.open("res://resources/Maps/career/city0.sc2.mpz", FileAccess.READ)

	var city_bytes := file.get_buffer(file.get_length()).decompress_dynamic(-1, FileAccess.COMPRESSION_DEFLATE)
	var city_result: Dictionary = MsgPack.decode(city_bytes)
	var city: Dictionary = city_result.result
	var buildings: Dictionary = city.buildings
	var networks: Dictionary = city.networks

	self.sea_level = city.simulator_settings["GlobalSeaLevel"]
	self.city_coords_feature = CityCoordsFeature.new(self.world_constants, self.sea_level)
	self.terrain.init(city)
	
	self.loading_scale.emit(buildings.size() + networks.size() + self.terrain.load_steps() + 1)
	self._load_map_async(city)


func _on_child_progress(progress: int) -> void:
	self.loading_progress.emit(progress)


func _on_spawn_point_encountered(tile_coords: Array[int], size: int, altitude: int) -> void:
	self._insert_spawn_point(tile_coords, size, altitude)


func _load_map_async(city: Dictionary):
	await self.terrain.build_async()
	await self.networks.build_async(city)
	await self.buildings.build_async(city).completed
	
	var city_size: int = city.get("city_size")

	self.backdrop.build(
		city_size,
		self.world_constants.tile_size,
		self.sea_level * self.world_constants.tile_height
	)
	self._spawn_player()

#	self._create_snapshot()
	await self.get_tree().process_frame
	self.loading_progress.emit(1)


func _create_snapshot() -> void:
	var packed_scene := PackedScene.new()
	var file_name := "{year}-{month}-{day}-{hour}-{minute}-{second}.tscn".format(Time.get_datetime_dict_from_system())
	packed_scene.pack(get_tree().get_current_scene())
	var result := ResourceSaver.save(packed_scene, "res://snapshots/%s" % file_name)

	print("saved snapshot: ", result)


func _spawn_player() -> void:
	var spawns := get_tree().get_nodes_in_group("spawn")
	var players := get_tree().get_nodes_in_group("player")
	var player := players[0] as Helicopter
	var spawn := spawns[0] as Node3D

	player.global_transform.origin = spawn.global_transform.origin + Vector3(0, -0.1, 0)
	player.snap_camera()


func _insert_spawn_point(building_coords: Array[int], building_size: int, altitude: int) -> void:
	print("SPAWN POINT AT {point}".format({ "point": building_coords }))
	var spawn_host_scene := preload("res://resources/Objects/spawn_host.tscn")
	var spawn_host: Node3D = spawn_host_scene.instantiate()
	var location := self.city_coords_feature.get_building_coords(building_coords[0], building_coords[1], altitude, building_size)

	spawn_host.translate(location)
	self.add_child(spawn_host)
