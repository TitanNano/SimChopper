extends Spatial

export(float, 0, 1) var power := 1.0
export var rpm := 471

var previous_rotation := Vector3.ZERO

onready var mesh := self.get_child(0)
onready var material: ShaderMaterial = mesh.get_surface_material(0)

func _physics_process(delta: float) -> void:
	var speed = (self.rpm / 60 * delta) * 360 * self.power

	if speed == 0:
		return

	self.rotate_y(deg2rad(speed))

func _process(_delta: float) -> void:
	#rotor velocity
	var rotation_offset: float = abs(self.rotation.y - self.previous_rotation.y)
	var previous_transform := Transform.IDENTITY.rotated(Vector3.FORWARD, min(max(0.01, rotation_offset), 0.610865) * -1)

	self.previous_rotation = Vector3(self.rotation)
	self.material.set_shader_param("previous_transform", previous_transform)
