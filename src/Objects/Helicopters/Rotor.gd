extends Node3D

@export_range(0, 1) var power := 1.0
@export var rpm := 471

var previous_rotation := Vector3.ZERO

@onready var mesh: MeshInstance3D = self.get_child(0)
@onready var material: ShaderMaterial = mesh.get_surface_override_material(0)

func _physics_process(delta: float) -> void:
	@warning_ignore("integer_division")
	var speed = (self.rpm / 60 * delta) * 360 * self.power

	if speed == 0:
		return

	self.rotate_y(deg_to_rad(speed))

func _process(_delta: float) -> void:
	#rotor velocity
	var rotation_offset: float = abs(self.rotation.y - self.previous_rotation.y)
	var previous_transform := Transform3D.IDENTITY.rotated(Vector3.FORWARD, min(max(0.01, rotation_offset), 0.610865) * -1)

	self.previous_rotation = Vector3(self.rotation)
	self.material.set_shader_parameter("previous_transform", previous_transform)
