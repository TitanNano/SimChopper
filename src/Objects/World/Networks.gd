extends Node

const TimeBudget := preload("res://src/util/TimeBudget.gd")
const CityCoordsFeature := preload("res://src/features/CityCoordsFeature.gd")
const WorldConstants := preload("res://src/Objects/Data/WorldConstants.gd")
const SceneObjectRegistry := preload("res://src/SceneObjectRegistry.gd")

signal loading_progress(value)

export var is_built := false
export var world_constants: Resource

var city_coords_feature: CityCoordsFeature

onready var powerline_network := $Powerlines
onready var road_network := $Road

func _ready() -> void:
	assert(world_constants is WorldConstants, "Networks.world_constants is not of type WorldConstants")


func build_async(city: Dictionary) -> void:
	var budget := TimeBudget.new(100)
	var networks: Dictionary = city.networks
	var tiles: Dictionary = city.tilelist
	var sea_level: int = city.simulator_settings["GlobalSeaLevel"]

	self.city_coords_feature = CityCoordsFeature.new(world_constants, sea_level)

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
		var location := self.city_coords_feature.get_building_coords(network_section.tile_coords[0], network_section.tile_coords[1], tile.altitude, 1)

		# is a suspension / pylon bridge part or raised powerline
		if network_section.building_id in range(0x51, 0x5E):
			location.y += self.world_constants.tile_height

		# buildings disapear under fully raised terrain
		if (tile.terrain & 0x0D) == 0x0D:
			location.y += self.world_constants.tile_height

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

	self.is_built = true
