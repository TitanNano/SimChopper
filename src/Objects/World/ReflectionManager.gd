extends Node

@export var update := false

var sea_level := 0
var city_size := 0
var tile_size := 0
var tile_height := 0
var is_built := false

func _process(_delta):
	if not self.is_built:
		return

	if not self.update:
		return

	self.update = false


func build_probes():
	if self.is_built:
		return

	if self.city_size < 1:
		return

	var probe_count: int = ProjectSettings.get_setting("rendering/reflections/reflection_atlas/reflection_count")
	var probe_columns := sqrt(probe_count)
	var probe_size := (self.city_size * tile_size) / probe_columns

	self.is_built = true

	for x in range(0, probe_columns):
		for y in range(0, probe_columns):
			var probe := ReflectionProbe.new()
			var probe_height := self.tile_height * 8

			var position = Vector3(
				probe_size * x + (probe_size / 2.0),
				self.sea_level + (probe_height / 2.0) - 4,
				probe_size * y + (probe_size / 2.0)
			)

			var extents = Vector3(probe_size, probe_height, probe_size)

			probe.size = extents
			probe.position = position
			probe.enable_shadows = true

			self.add_child(probe)
			probe.owner = get_tree().current_scene

	await get_tree().process_frame
	await get_tree().process_frame

	for probe in self.get_children():
		(probe as ReflectionProbe).position.y -= 10

