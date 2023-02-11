extends Spatial

const TimeBudget := preload("../../util/TimeBudget.gd")
const MsgPack := preload("../../godot-msgpack/msgpack.gd")
const SceneObjectRegistry := preload("res://src/SceneObjectRegistry.gd")
const Networks := preload("res://src/Objects/World/Networks.gd")
const CityCoordsFeature := preload("res://src/features/CityCoordsFeature.gd")
const Logger := preload("res://src/util/Logger.gd")

signal loading_progress(count)
signal loading_scale(count)

export var world_constants: Resource

onready var terrain := $Terrain
onready var networks: Networks = $Networks
onready var reflections := $Reflections
onready var buildings := $Buildings
onready var backdrop := $Backdrop

var sea_level: int
var city_coords_feature: CityCoordsFeature

func _ready():
	assert(world_constants is WorldConstants, "world_constants is not of type WorldConstants")

	self.networks.connect("loading_progress", self, "_on_child_progress")
	self.buildings.connect("spawn_point_encountered", self, "_on_spawn_point_encountered")
	self.buildings.connect("loading_progress", self, "_on_child_progress")

	call_deferred("_ready_deferred")


func _ready_deferred():
	var file := File.new()
	var result = file.open("res://resources/Maps/career/city0.sc2.mpz", File.READ)

	if result != OK:
		Logger.error("failed to open file")
		return

	var city_bytes := file.get_buffer(file.get_len()).decompress_dynamic(-1, File.COMPRESSION_DEFLATE)
	var city_result: Dictionary = MsgPack.decode(city_bytes)
	var city: Dictionary = city_result.result

	self.sea_level = city.simulator_settings["GlobalSeaLevel"]
	self.city_coords_feature = CityCoordsFeature.new(self.world_constants, self.sea_level)
	self.emit_signal("loading_scale", city.buildings.size() + city.networks.size() + 1)
	self._load_map_async(city)


func _on_child_progress(progress: int) -> void:
	self.emit_signal("loading_progress", progress)


func _on_spawn_point_encountered(tile_coords: Array, size: int, altitude: int) -> void:
	self._insert_spawn_point(tile_coords, size, altitude)


func _load_map_async(city: Dictionary):
	yield(self.terrain.build_async(city), "completed")
	yield(self.networks.build_async(city), "completed")
	yield(self.buildings.build_async(city), "completed")

	self._setup_probing(city.city_size)
	self.backdrop.build(
		city.city_size,
		self.world_constants.tile_size,
		self.sea_level * self.world_constants.tile_height
	)
	self._spawn_player()

#	self._create_snapshot()
	yield(self.get_tree(), "idle_frame")
	self.emit_signal("loading_progress", 1)


func _create_snapshot() -> void:
	var packed_scene = PackedScene.new()
	var file_name = "{year}-{month}-{day}-{hour}-{minute}-{second}.tscn".format(OS.get_datetime())
	packed_scene.pack(get_tree().get_current_scene())
	var result := ResourceSaver.save("res://snapshots/%s" % file_name, packed_scene)

	print("saved snapshot: ", result)


func _spawn_player() -> void:
	var spawns := get_tree().get_nodes_in_group("spawn")
	var players := get_tree().get_nodes_in_group("player")
	var player: Spatial = players[0]
	var spawn: Spatial = spawns[0]

	player.global_transform.origin = spawn.global_transform.origin
	player.force_update_transform()
	player.snap_camera()
	player.mode = RigidBody.MODE_RIGID


func _insert_spawn_point(building_coords: Array, building_size: int, altitude: int) -> void:
	print("SPAWN POINT AT {point}".format({ "point": building_coords }))
	var spawn_host_scene := preload("res://resources/Objects/spawn_host.tscn")
	var spawn_host := spawn_host_scene.instance()
	var location := self.city_coords_feature.get_building_coords(building_coords[0], building_coords[1], altitude, building_size)

	spawn_host.translate(location)
	self.add_child(spawn_host)


func _setup_probing(city_size: int) -> void:
	self.reflections.sea_level = self.sea_level * self.world_constants.tile_height
	self.reflections.tile_size = self.world_constants.tile_size
	self.reflections.tile_height = self.world_constants.tile_height
	self.reflections.city_size = city_size

	self.reflections.build_probes()
