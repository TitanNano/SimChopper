extends GPUParticles3D

@export_range(0, 1) var strength: float : get = _get_strength, set = _set_strength

func _ready() -> void:
	self._set_strength(0)

func _get_strength() -> float:
	if not self.process_material is ParticleProcessMaterial:
		return 0.0

	var material: ParticleProcessMaterial = self.process_material

	return material.initial_velocity_min / 15.0

func _set_strength(value: float):
	@warning_ignore("shadowed_variable_base_class")
	var is_emitting := value > 0

	if self.emitting != is_emitting:
		self.emitting = is_emitting

	if not self.emitting:
		return
		
	var mesh: PrimitiveMesh = self.draw_pass_1
	var material: StandardMaterial3D = mesh.material

	material.proximity_fade_distance = max(100 * (1 - value), 2)
