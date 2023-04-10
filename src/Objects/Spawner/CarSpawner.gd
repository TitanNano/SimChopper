extends Node3D

const DefaultCar = preload("res://resources/Objects/Vehicles/car_station_wagon.tscn")
const Car := preload("res://src/Objects/Vehicles/Car.gd")

@export var road_network_path: NodePath

var timer: Timer

func spawn_car() -> void:
	var inst: Car = DefaultCar.instantiate()

	inst.road_network_path = self.road_network_path

	self.get_parent().add_child(inst)
	inst.owner = get_tree().current_scene
	inst.global_translate(self.global_transform.origin)
	inst.activate()


func start_auto_spawn():
	if not self.timer:
		self.timer = Timer.new()
		self.add_child(self.timer, true)
		self.timer.timeout.connect(self.spawn_car)

	self.spawn_car()
	self.timer.start(2.0)


func stop_auto_spawn():
	if not self.timer:
		return

	self.timer.stop()
