extends Spatial

const TimeBudget := preload("../../util/TimeBudget.gd")
const MsgPack := preload("../../godot-msgpack/msgpack.gd")
const SceneObjectRegistry := preload("res://src/SceneObjectRegistry.gd")

signal loading_progress(count)
signal loading_scale(count)

onready var terrain: MeshInstance = $Terrain
onready var powerline_network := $Networks/Powerlines
onready var road_network := $Networks/Road
onready var reflections := $Reflections
onready var buildings_node := $Buildings
onready var backdrop := $Backdrop

func _ready():
	call_deferred("_ready_deferred")


func _ready_deferred():
	var file := File.new()
	var result = file.open("res://resources/Maps/career/city0.sc2.mpz", File.READ)

	if result != OK:
		printerr("failed to open file")
		return

	var city_bytes := file.get_buffer(file.get_len()).decompress_dynamic(-1, File.COMPRESSION_GZIP)
	var city: Dictionary = MsgPack.decode(city_bytes).result

	self.emit_signal("loading_scale", city.buildings.size() + city.networks.size() + 1)
	self._load_map_async(city)


func _load_map_async(city: Dictionary):
	yield(self.terrain.build_async(city), "completed")
	yield(self._insert_networks_async(city.networks, city.tilelist), "completed")
	yield(self._insert_buildings_async(city.buildings, city.tilelist), "completed")

	self._setup_probing(city.city_size)
	self.backdrop.build(
		city.city_size,
		self.terrain.tile_size,
		self.terrain.sea_level * self.terrain.tile_height
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


func _insert_buildings_async(buildings: Dictionary, tiles: Dictionary):
	var budget := TimeBudget.new(50)

	for key in buildings:
		var building: Dictionary = buildings[key]

		if building.building_id == 0x00:
			# ignoring empty building
			self.emit_signal("loading_progress", 1)
			print("skipping empty building")
			continue

		self._insert_building(building, tiles)
		self.emit_signal("loading_progress", 1)

		if budget.is_exceded():
			print("yielding after ", budget.elapsed(), "ms of work")
			budget.restart()
			yield(self.get_tree(), "idle_frame")

	print("finished buildings after ", budget.elapsed(), "ms of work")
	yield(self.get_tree(), "idle_frame")



func _is_spawn_point(building: Dictionary, tiles: Dictionary) -> bool:
	var x = building.tile_coords[0]
	var y = building.tile_coords[1]

	for index in range(x - 1, x + 3):
		var tile: Dictionary = tiles[[index, y]]

		if not tile.building:
			return false

		if tile.building.building_id == 0xE6:
			continue

		return false

	for index in range(y - 1, y + 3):
		var tile: Dictionary = tiles[[x, index]]

		if not tile.building:
			return false

		if tile.building.building_id == 0xE6:
			continue

		return false

	return true


func _insert_building(building: Dictionary, tiles: Dictionary) -> void:
	var budget := TimeBudget.new(0)
	var tile: Dictionary = tiles[Array(building.tile_coords)]
	var building_size: int = building.size
	var name: String =  building.name
	var object := SceneObjectRegistry.load_building(building.building_id)

	if not object:
		print("unknown building \"%s\"" % name)
		return

	if building.building_id == 0xE6 and self._is_spawn_point(building, tiles):
		self._insert_building({ "building_id": 0xF6, "tile_coords": building.tile_coords, "name": "Hangar", "size": 2 }, tiles)
		self._insert_spawn_point(building.tile_coords, 2, tile.altitude)

	budget.restart()
	var instance: Spatial = object.instance()
	var instance_time := budget.elapsed()

	var location := self._get_building_world_cords(building.tile_coords[0], building.tile_coords[1], tile.altitude, building_size)

	location.y += 0.1

	var sector_name := "{x}_{y}".format({
		"x": stepify(building.tile_coords[0], 10),
		"y": stepify(building.tile_coords[1], 10)
	})

	budget.restart()
	if buildings_node.get_node_or_null(sector_name) == null:
		var sector := Node.new()
		sector.name = sector_name
		buildings_node.add_child(sector)

	buildings_node \
		.get_node_or_null(sector_name) \
		.add_child(instance, true)
	var insert_time := budget.elapsed()

	instance.translate(location)

	if instance_time > 100:
		printerr("\"%s\" is very slow to instanciate" % building.name)

	if insert_time > 100:
		printerr("\"%s\" is very slow to insert" % building.name)


func _insert_spawn_point(building_coords: Array, building_size: int, altitude: int) -> void:
	print("SPAWN POINT AT {point}".format({ "point": building_coords }))
	var spawn_host_scene := preload("res://resources/Objects/spawn_host.tscn")
	var spawn_host := spawn_host_scene.instance()
	var location := self._get_building_world_cords(building_coords[0], building_coords[1], altitude, building_size)

	spawn_host.translate(location)
	self.add_child(spawn_host)


func _get_building_world_cords(x: int, y: int, z: int, size: int) -> Vector3:
	var offset: float = (size * self.terrain.tile_size / 2.0)

	# OpenCity2k gets the bottom left corner, we have to correct that.
	y -= (size - 1)

	return Vector3(
		(x * self.terrain.tile_size) + offset,
		max(z, self.terrain.sea_level - 1) * self.terrain.tile_height,
		(y * self.terrain.tile_size) + offset
	)

func _insert_networks_async(networks: Dictionary, tiles: Dictionary):
	var budget := TimeBudget.new(100)

	for key in networks:
		var network_section: Dictionary = networks[key]
		var object := SceneObjectRegistry.load_network(network_section.building_id)
		var name: String = network_section.name

		if not object:
			print("unknown network_section \"%s\"" % name)
			self.emit_signal("loading_progress", 1)
			continue

		var instance: Spatial = object.instance()
		var tile: Dictionary = tiles[key]
		var location := self._get_building_world_cords(network_section.tile_coords[0], network_section.tile_coords[1], tile.altitude, 1)

		# is a suspension / pylon bridge part or raised powerline
		if network_section.building_id in range(0x51, 0x5E):
			location.y += self.terrain.tile_height

		# buildings disapear under fully raised terrain
		if (tile.terrain & 0x0D) == 0x0D:
			location.y += self.terrain.tile_height

		if instance.has_method("set_orientation"):
			instance.set_orientation(
				tiles[[key[0], key[1] - 1]],
				tiles[[key[0] + 1, key[1]]],
				tiles[[key[0], key[1] + 1]],
				tiles[[key[0] - 1, key[1]]]
			)

		instance.transform.origin = location

		if network_section.building_id in range(0x0E, 0x1D) + range(0x5C, 0x5D):
			powerline_network.add_child(instance, true)
		elif network_section.building_id in (range(0x1D, 0x2C) + range(0x51, 0x5E) + range(0x43, 0x45)):
			road_network.add_child(instance, true)
		else:
			print("network secction doesn't belong to any network, ", network_section)

		self.emit_signal("loading_progress", 1)

		if budget.is_exceded():
			print("yielding after ", budget.elapsed(), "ms of work")
			budget.restart()
			yield(self.get_tree(), "idle_frame")

	# yield at least once at the end, to let the engine catch up
	yield(self.get_tree(), "idle_frame")


func _setup_probing(city_size: int) -> void:
	self.reflections.sea_level = self.terrain.sea_level * self.terrain.tile_height
	self.reflections.tile_size = self.terrain.tile_size
	self.reflections.tile_height = self.terrain.tile_height
	self.reflections.city_size = city_size

	self.reflections.build_probes()
