extends Node

const TimeBudget := preload("res://src/util/TimeBudget.gd")
const CityCoordsFeature := preload("res://src/features/CityCoordsFeature.gd")
const SceneObjectRegistry := preload("res://src/SceneObjectRegistry.gd")
const Building := preload("res://src/Objects/Map/Building.gd")
const RoadNavigation := preload("res://src/Objects/Networks/RoadNavigation.gd")

signal loading_progress(value)

@export var is_built := false
@export var world_constants: WorldConstants

var city_coords_feature: CityCoordsFeature

@onready var powerline_network := $Powerlines
@onready var road_network: RoadNavigation = $Road

func _ready() -> void:
	assert(world_constants is WorldConstants, "Networks.world_constants is not of type WorldConstants")


func build_async(city: Dictionary):
	var budget := TimeBudget.new(100)
	var networks: Dictionary = city.networks
	var tiles: Dictionary = city.tilelist
	var sea_level: int = city.simulator_settings["GlobalSeaLevel"]

	self.city_coords_feature = CityCoordsFeature.new(world_constants, sea_level)

	for key in networks:
		var network_section: Building = Building.new(networks.get(key) as Dictionary)
		var object := SceneObjectRegistry.load_network(network_section.building_id())
		@warning_ignore("shadowed_variable_base_class")
		var name: String = network_section.name()

		if not object:
			print("unknown network_section \"%s\"" % name)
			self.emit_signal("loading_progress", 1)
			continue

		var instance: Node3D = object.instantiate()
		var tile: Dictionary = tiles[key]
		var altitude: int = tile.get("altitude")
		var location := self.city_coords_feature.get_building_coords(network_section.tile_coords()[0], network_section.tile_coords()[1], altitude, 1)

		# is a suspension / pylon bridge part or raised powerline
		if network_section.building_id() in range(0x51, 0x5E):
			location.y += self.world_constants.tile_height

		# buildings disapear under fully raised terrain
		if (tile.terrain & 0x0D) == 0x0D:
			location.y += self.world_constants.tile_height

		if instance.has_method("set_orientation"):
			instance.call("set_orientation",
				tiles[[key[0], key[1] - 1]],
				tiles[[key[0] + 1, key[1]]],
				tiles[[key[0], key[1] + 1]],
				tiles[[key[0] - 1, key[1]]]
			)

		instance.transform.origin = location

		if network_section.building_id() in range(0x0E, 0x1D) + range(0x5C, 0x5D):
			powerline_network.add_child(instance, true)
		elif network_section.building_id() in (range(0x1D, 0x2C) + range(0x51, 0x5E) + range(0x43, 0x45)):
			road_network.add_child(instance, true)
			road_network.insert_node(network_section)
			road_network.associate_object(network_section, instance)
		else:
			print("network secction doesn't belong to any network, ", network_section)

		self.emit_signal("loading_progress", 1)

		if budget.is_exceded():
			print("yielding after ", budget.elapsed(), "ms of work")
			budget.restart()
			await self.get_tree().process_frame

	road_network.update_debug()

	for _i in range(3):
		var car_spawner: CarSpawner = (load("res://resources/Objects/Spawner/CarSpawner.tscn") as PackedScene).instantiate()
		var random_child: Node3D = road_network.get_child(randi() % road_network.get_child_count()) 
		var transform := random_child.global_transform.origin
		car_spawner.road_network_path = road_network.get_path()
		car_spawner.translate(transform)
		car_spawner.translate(Vector3.UP * 2)
		self.get_parent().add_child(car_spawner)
		car_spawner.start_auto_spawn()


	# yield at least once at the end, to let the engine catch up
	await self.get_tree().process_frame
	self.is_built = true
