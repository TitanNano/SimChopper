extends Particles

export(float, 0, 1) var strength: float setget _set_strength, _get_strength

func _ready() -> void:
	self._set_strength(0)

func _get_strength() -> float:
	if not self.process_material is ParticlesMaterial:
		return 0.0
	
	var material: ParticlesMaterial = self.process_material
	
	return material.initial_velocity / 15.0
	
func _set_strength(value: float):
	var emitting := value > 0
	
	if self.emitting != emitting:
		self.emitting = emitting

	if not self.process_material is ParticlesMaterial:
		return
	
	var material: ParticlesMaterial = self.process_material
	
	material.initial_velocity = value * 15
	material.scale = value * 4
