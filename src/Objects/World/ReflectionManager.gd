extends Node

export var update := false

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

	for probe in self.probes:
		probe.translation.y -= 1


func build_probes():
	if self.is_built:
		return

	if self.city_size < 1:
		return

	var probe_count: int = ProjectSettings.get_setting("rendering/quality/reflections/atlas_subdiv") * 2
	var probe_columns := sqrt(probe_count)
	var probe_size := (self.city_size * tile_size) / probe_columns

	self.is_built = true

	for x in range(0, probe_columns):
		for y in range(0, probe_columns):
			var probe := ReflectionProbe.new()
			var probe_height := self.tile_height * 4

			var translation = Vector3(
				probe_size * x + (probe_size / 2.0),
				self.sea_level + probe_height,
				probe_size * y + (probe_size / 2.0)
			)

			var extents = Vector3(probe_size / 2.0, probe_height, probe_size / 2.0)

			probe.extents = extents
			probe.translation = translation
			probe.enable_shadows = true
			probe.owner = get_tree().current_scene

			self.add_child(probe)

	yield(get_tree(), "idle_frame")
	yield(get_tree(), "idle_frame")

	for probe in self.get_children():
		probe.translation.y -= 10

