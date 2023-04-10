extends Node3D

@export var remote_path: NodePath
@export var local_transform: Transform3D
@export var full_transform := false
@export_range(0, 1) var speed: float = 0.5
@export var snap := false

func _ready():
	var target = get_node(self.remote_path) as Node3D

	self.local_transform = self.transform

	if not target:
		return

	target.global_transform = self.global_transform


func _physics_process(_delta: float):
	if self.remote_path == null:
		push_warning("CameraInterpolation has no remote target!")
		return

	var target := get_node(self.remote_path) as Camera3D

	if not self.full_transform:
		var parent := self.get_parent() as Node3D

		var counter_transform = Transform3D() \
			.rotated(Vector3.RIGHT, -parent.rotation.x) \
			.rotated(Vector3.BACK, -parent.rotation.z)

		self.global_transform = parent.global_transform * counter_transform * self.local_transform

	var current_transform := target.global_transform
	var destination_transform := self.global_transform

	target.transform = current_transform.interpolate_with(destination_transform, self.speed if not snap else 1.0)
	self.snap = false
