extends Spatial

const grass_material = preload("res://Materials/grass_material.tres")
const dirt_material = preload("res://Materials/dirt_material.tres")
const ocean_material = preload("res://Materials/ocean.tres")

const networks = {
	# Powerlines
	0x0E: preload("res://Objects/Networks/Powerline/left_right.tscn"),
	0x0F: preload("res://Objects/Networks/Powerline/top_bottom.tscn"),
	0x10: preload("res://Objects/Networks/Powerline/high_top_bottom.tscn"),
	0x11: preload("res://Objects/Networks/Powerline/left_high_right.tscn"),
	0x12: preload("res://Objects/Networks/Powerline/top_high_bottom.tscn"),
	0x13: preload("res://Objects/Networks/Powerline/high_left_right.tscn"),
	0x14: preload("res://Objects/Networks/Powerline/bottom_right.tscn"),
	0x15: preload("res://Objects/Networks/Powerline/bottom_left.tscn"),
	0x16: preload("res://Objects/Networks/Powerline/top_left.tscn"),
	0x17: preload("res://Objects/Networks/Powerline/top_right.tscn"),
	0x18: preload("res://Objects/Networks/Powerline/right_top_bottom.tscn"),
	0x19: preload("res://Objects/Networks/Powerline/left_bottom_right.tscn"),
	0x1A: preload("res://Objects/Networks/Powerline/top_left_bottom.tscn"),
	0x1B: preload("res://Objects/Networks/Powerline/left_top_right.tscn"),
	0x1C: preload("res://Objects/Networks/Powerline/left_top_bottom_right.tscn"),
	0x5C: preload("res://Objects/Networks/Powerline/bridge_top_bottom.tscn"),

	# Road
	0x1D: preload("res://Objects/Networks/Road/left_right.tscn"),
	0x1E: preload("res://Objects/Networks/Road/top_bottom.tscn"),
	0x1F: preload("res://Objects/Networks/Road/high_top_bottom.tscn"),
	0x20: preload("res://Objects/Networks/Road/left_high_right.tscn"),
	0x21: preload("res://Objects/Networks/Road/top_high_bottom.tscn"),
	0x22: preload("res://Objects/Networks/Road/high_left_right.tscn"),
	0x23: preload("res://Objects/Networks/Road/bottom_right.tscn"),
	0x24: preload("res://Objects/Networks/Road/bottom_left.tscn"),
	0x25: preload("res://Objects/Networks/Road/top_left.tscn"),
	0x26: preload("res://Objects/Networks/Road/top_right.tscn"),
	0x27: preload("res://Objects/Networks/Road/right_top_bottom.tscn"),
	0x28: preload("res://Objects/Networks/Road/left_bottom_right.tscn"),
	0x29: preload("res://Objects/Networks/Road/top_left_bottom.tscn"),
	0x2A: preload("res://Objects/Networks/Road/left_top_right.tscn"),
	0x2B: preload("res://Objects/Networks/Road/left_top_bottom_right.tscn"),
	0x43: preload("res://Objects/Networks/Road/left_right_power_top_bottom.tscn"),
	0x44: preload("res://Objects/Networks/Road/top_bottom_power_left_right.tscn"),

	# Suspension Bridge
	0x51: preload("res://Objects/Networks/Bridge/bridge_suspension_start_bottom.tscn"),
	0x52: preload("res://Objects/Networks/Bridge/bridge_suspension_middle_bottom.tscn"),
	0x53: preload("res://Objects/Networks/Bridge/bridge_suspension_center.tscn"),
	0x54: preload("res://Objects/Networks/Bridge/bridge_suspension_middle_top.tscn"),
	0x55: preload("res://Objects/Networks/Bridge/bridge_suspension_end_top.tscn"),

	# Pylon Bridge
	0x56: preload("res://Objects/Networks/Bridge/bridge_raising_tower_top_bottom.tscn"),
	0x57: preload("res://Objects/Networks/Bridge/bridge_top.tscn"),
	0x58: preload("res://Objects/Networks/Bridge/bridge_top.tscn"),
}

const buildings = {
	0x0D: preload("res://Objects/Buildings/park_small.tscn"),
	0x06: preload("res://Objects/Buildings/tree_single.tscn"),
	0x73: preload("res://Objects/Buildings/home_middle_class_1.tscn"),
	0x74: preload("res://Objects/Buildings/home_middle_class_2.tscn"),
	0x75: preload("res://Objects/Buildings/home_middle_class_3.tscn"),
	0x76: preload("res://Objects/Buildings/home_middle_class_4.tscn"),
	0x77: preload("res://Objects/Buildings/home_middle_class_5.tscn"),
	0xF7: preload("res://Objects/Buildings/church.tscn"),
	0x96: preload("res://Objects/Buildings/office_building_medium_1.tscn"),
	0x98: preload("res://Objects/Buildings/office_building_medium_2.tscn"),
	0x9A: preload("res://Objects/Buildings/office_building_medium_3.tscn"),
	0x9B: preload("res://Objects/Buildings/office_building_medium_4.tscn"),
	0x9C: preload("res://Objects/Buildings/office_building_medium_5.tscn"),
	0x9D: preload("res://Objects/Buildings/office_building_medium_6.tscn"),
	0x8A: preload("res://Objects/Buildings/abandoned_building_1.tscn"),
	0x8B: preload("res://Objects/Buildings/abandoned_building_2.tscn"),
	0xAA: preload("res://Objects/Buildings/abandoned_building_3.tscn"),
	0xAB: preload("res://Objects/Buildings/abandoned_building_4.tscn"),
	0xAC: preload("res://Objects/Buildings/abandoned_building_5.tscn"),
	0xAD: preload("res://Objects/Buildings/abandoned_building_6.tscn"),
	0x78: preload("res://Objects/Buildings/home_upper_class_1.tscn"),
	0x79: preload("res://Objects/Buildings/home_upper_class_2.tscn"),
	0x7A: preload("res://Objects/Buildings/home_upper_class_3.tscn"),
	0x7B: preload("res://Objects/Buildings/home_upper_class_4.tscn"),
	0xE6: preload("res://Objects/Ground/tarmac.tscn"),
	0xEA: preload("res://Objects/Buildings/tarmac_radar.tscn"),
	0xC2: preload("res://Objects/Buildings/construction_1-2.tscn"),
	0xC3: preload("res://Objects/Buildings/construction_1-2.tscn"),
	0xA6: preload("res://Objects/Buildings/construction_3.tscn"),
	0xA7: preload("res://Objects/Buildings/construction_4.tscn"),
	0xA8: preload("res://Objects/Buildings/construction_5.tscn"),
	0xA9: preload("res://Objects/Buildings/construction_6.tscn"),
	0x88: preload("res://Objects/Buildings/construction_7.tscn"),
	0x89: preload("res://Objects/Buildings/construction_8.tscn"),
	0xE3: preload("res://Objects/Buildings/airport_warehouse.tscn"),
	0xE4: preload("res://Objects/Buildings/airport_building_1.tscn"),
	0xE5: preload("res://Objects/Buildings/airport_building_2.tscn"),
	0xE8: preload("res://Objects/Buildings/airport_hangar_1.tscn"),
	0xDD: preload("res://Objects/Buildings/airport_runway.tscn"),
	0xDF: preload("res://Objects/Buildings/airport_runway_intersection.tscn"),
	0xF6: preload("res://Objects/Buildings/hangar_2.tscn"),
	0x91: preload("res://Objects/Buildings/condominiums_medium_1.tscn"),
	0x92: preload("res://Objects/Buildings/condominiums_medium_2.tscn"),
	0x93: preload("res://Objects/Buildings/condominiums_medium_3.tscn"),
	0xB0: preload("res://Objects/Buildings/condominiums_large_1.tscn"),
	0xB1: preload("res://Objects/Buildings/condominiums_large_2.tscn"),
	0xA0: preload("res://Objects/Buildings/factory_small_1.tscn"),
	0xA1: preload("res://Objects/Buildings/factory_small_2.tscn"),
	0xA2: preload("res://Objects/Buildings/factory_small_3.tscn"),
	0xA3: preload("res://Objects/Buildings/factory_small_4.tscn"),
	0xA4: preload("res://Objects/Buildings/factory_small_5.tscn"),
	0xA5: preload("res://Objects/Buildings/factory_small_6.tscn"),
	0xD2: preload("res://Objects/Buildings/station_police.tscn"),
	0x8F: preload("res://Objects/Buildings/apartments_medium_1.tscn"),
	0x90: preload("res://Objects/Buildings/apartments_medium_2.tscn"),
	0x83: preload("res://Objects/Buildings/toy_store.tscn"),
	0x87: preload("res://Objects/Buildings/industrial_substation.tscn"),
	0x80: preload("res://Objects/Buildings/offices_small_1.tscn"),
	0x81: preload("res://Objects/Buildings/offices_small_2.tscn"),
	0xBA: preload("res://Objects/Buildings/offices_historic.tscn"),
	0xDC: preload("res://Objects/Buildings/water_pump.tscn"),
	0xD1: preload("res://Objects/Buildings/station_hospital.tscn"),
	0x7E: preload("res://Objects/Buildings/convenience_store.tscn"),
	0x7C: preload("res://Objects/Buildings/station_gas_1.tscn"),
	0x7F: preload("res://Objects/Buildings/station_gas_2.tscn"),
	0x70: preload("res://Objects/Buildings/home_lower_class_1.tscn"),
	0x71: preload("res://Objects/Buildings/home_lower_class_2.tscn"),
	0x72: preload("res://Objects/Buildings/home_lower_class_3.tscn"),
	0x82: preload("res://Objects/Buildings/warehouse.tscn"),
	0xE1: preload("res://Objects/Buildings/airport_civilian_control_tower.tscn"),
	0xD3: preload("res://Objects/Buildings/station_fire.tscn"),
	0xCD: preload("res://Objects/Buildings/powerplant_microwave.tscn"),
	0x97: preload("res://Objects/Buildings/resort_hotel.tscn"),
	0xAE: preload("res://Objects/Buildings/apartments_large_1.tscn"),
	0xAF: preload("res://Objects/Buildings/apartments_large_2.tscn"),
	0x8C: preload("res://Objects/Buildings/apartments_small_1.tscn"),
	0x8D: preload("res://Objects/Buildings/apartments_small_2.tscn"),
	0x8E: preload("res://Objects/Buildings/apartments_small_3.tscn"),
	0x07: preload("res://Objects/Buildings/tree_couple.tscn"),
	0x85: preload("res://Objects/Buildings/chemical_storage.tscn"),
	0xBC: preload("res://Objects/Buildings/chemical_processing_1.tscn"),
	0x9F: preload("res://Objects/Buildings/chemical_processing_2.tscn"),
	0xD6: preload("res://Objects/Buildings/school.tscn"),
	0xF5: preload("res://Objects/Buildings/library.tscn"),
	0xF8: preload("res://Objects/Buildings/marina.tscn"),
	0xC0: preload("res://Objects/Buildings/warehouse_large_1.tscn"),
	0xC1: preload("res://Objects/Buildings/warehouse_large_2.tscn"),
	0x84: preload("res://Objects/Buildings/warehouse_small_1.tscn"),
	0x86: preload("res://Objects/Buildings/warehouse_small_2.tscn"),
	0x9E: preload("res://Objects/Buildings/warehouse_medium.tscn"),
	0x7D: preload("res://Objects/Buildings/bb_inn.tscn"),
	0xD9: preload("res://Objects/Buildings/college.tscn"),
	0xFB: preload("res://Objects/Buildings/arcology_plymouth.tscn"),
	0xFC: preload("res://Objects/Buildings/arcology_forest.tscn"),
	0xFD: preload("res://Objects/Buildings/arcology_darco.tscn"),
	0xFE: preload("res://Objects/Buildings/arcology_launch.tscn"),
	0xF3: preload("res://Objects/Buildings/mayors_house.tscn"),
	0xD4: preload("res://Objects/Buildings/museum.tscn"),
	0x99: preload("res://Objects/Buildings/office_retail.tscn"),
	0xB9: preload("res://Objects/Buildings/parking_lot.tscn"),
	0x94: preload("res://Objects/Buildings/shopping_centre.tscn"),
	0xB5: preload("res://Objects/Buildings/theatre.tscn"),
	0xF4: preload("res://Objects/Buildings/water_treatment.tscn"),
}

const TimeBudget := preload("util/TimeBudget.gd")
const TerrainRotation := preload("Terrain/TerrainRotation.gdns")
const TerrainBuilderFactory := preload("Terrain/TerrainBuilderFactory.gdns")
const MsgPack := preload("res://godot-msgpack/msgpack.gd")
const tile_size := 16
const tile_height := 8

signal loading_progress(count)

onready var world := $World
onready var terrain: MeshInstance = $World/Terrain
onready var powerline_network := $World/Networks/Powerlines
onready var road_network := $World/Networks/Road
onready var loading_screen := $LoadingScreen
onready var reflections := $World/Reflections
onready var buildings_node := $World/Buildings

var sea_level := 0

func _ready():
	var file := File.new()
	var result = file.open("res://Maps/career/city0.sc2.mpz", File.READ)

	if result != OK:
		printerr("failed to open file")

	var city_bytes := file.get_buffer(file.get_len()).decompress_dynamic(-1, File.COMPRESSION_GZIP)
	var city: Dictionary = MsgPack.decode(city_bytes).result

	self.loading_screen.total_jobs = city.buildings.size() + city.networks.size()
	self.sea_level = city.simulator_settings["GlobalSeaLevel"]

	# warning-ignore:return_value_discarded
	self.connect("loading_progress", self, "_on_progress")
	self._load_map_async(city)


func _load_map_async(city: Dictionary):
	self.world.visible = false
	self._generate_terain_with_native_builder(city)
	yield(self._insert_networks_async(city.networks, city.tilelist), "completed")
	yield(self._insert_buildings_async(city.buildings, city.tilelist), "completed")

	self._setup_probing(city.city_size)
	self._spawn_player()

#	self._create_snapshot()
	self.world.visible = true


func _on_progress(count: int) -> void:
	self.loading_screen.completed_jobs += count


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
	player.mode = RigidBody.MODE_RIGID


func _generate_terain_with_native_builder(city: Dictionary):
	var rotation := TerrainRotation.new()
	rotation.set_rotation(city.simulator_settings['Compass'])

	var materials := {
		"Ground": dirt_material,
		"Grass": grass_material,
		"Water": ocean_material
	}

	var builder_factory := TerrainBuilderFactory.new()
	var builder = builder_factory.create(city.tilelist, rotation, materials)

	builder.set_city_size(city.city_size)
	builder.set_tile_size (self.tile_size)
	builder.set_tile_height(self.tile_height)
	builder.set_sea_level(self.sea_level)

	var mesh: ArrayMesh = builder.build_terain_async()

	terrain.mesh = mesh
	terrain.create_trimesh_collision()


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
	var object: PackedScene = self.buildings.get(building.building_id)

	if not object:
		print("unknown building \"%s\"" % name)
		return

	if building.building_id == 0xE6 and self._is_spawn_point(building, tiles):
		print("SPAWN POINT AT {point}".format({ "point": building.tile_coords }))
		self._insert_building({ "building_id": 0xF6, "tile_coords": building.tile_coords, "name": "Hangar", "size": 1 }, tiles)

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


func _get_building_world_cords(x: int, y: int, z: int, size: int) -> Vector3:
	var offset := (size * tile_size / 2.0)

	# OpenCity2k gets the bottom left corner, we have to correct that.
	y -= (size - 1)

	return Vector3(
		(x * self.tile_size) + offset,
		max(z, self.sea_level - 1) * self.tile_height,
		(y * self.tile_size) + offset
	)

func _insert_networks_async(networks: Dictionary, tiles: Dictionary):
	var budget := TimeBudget.new(100)

	for key in networks:
		var network_section: Dictionary = networks[key]
		var object: PackedScene = self.networks.get(network_section.building_id)
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
			location.y += tile_height

		# buildings disapear under fully raised terrain
		if (tile.terrain & 0x0D) == 0x0D:
			location.y += tile_height

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
	self.reflections.sea_level = self.sea_level * tile_height
	self.reflections.tile_size = self.tile_size
	self.reflections.tile_height = self.tile_height
	self.reflections.city_size = city_size

	self.reflections.build_probes()
