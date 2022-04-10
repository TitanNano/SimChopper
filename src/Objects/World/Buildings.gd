extends Node

const TimeBudget := preload("res://src/util/TimeBudget.gd")
const SceneObjectRegistry := preload("res://src/SceneObjectRegistry.gd")
const CityCoordsFeature := preload("res://src/features/CityCoordsFeature.gd")

signal loading_progress(value)
signal spawn_point_encountered(tile_coords, size, altitude)

export var world_constants: Resource

var city_coords_feature: CityCoordsFeature

func _ready() -> void:
	assert(self.world_constants is WorldConstants, "Buildings.world_constants is not of type WorldConstants")


func build_async(city) -> void:
	var sea_level: int = city.simulator_settings["GlobalSeaLevel"]
	var budget := TimeBudget.new(50)
	var buildings: Dictionary = city.buildings
	var tiles: Dictionary = city.tilelist

	self.city_coords_feature = CityCoordsFeature.new(self.world_constants, sea_level)

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
		self.emit_signal("spawn_point_encountered", building.tile_coords, 2, tile.altitude)

	budget.restart()
	var instance: Spatial = object.instance()
	var instance_time := budget.elapsed()

	var location := self.city_coords_feature.get_building_coords(building.tile_coords[0], building.tile_coords[1], tile.altitude, building_size)

	location.y += 0.1

	var sector_name := "{x}_{y}".format({
		"x": stepify(building.tile_coords[0], 10),
		"y": stepify(building.tile_coords[1], 10)
	})

	budget.restart()
	if self.get_node_or_null(sector_name) == null:
		var sector := Node.new()
		sector.name = sector_name
		self.add_child(sector)

	self \
		.get_node_or_null(sector_name) \
		.add_child(instance, true)
	var insert_time := budget.elapsed()

	instance.translate(location)

	if instance_time > 100:
		printerr("\"%s\" is very slow to instanciate" % building.name)

	if insert_time > 100:
		printerr("\"%s\" is very slow to insert" % building.name)
