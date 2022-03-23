extends Spatial

export var remote_path: NodePath
export var local_transform: Transform
export var full_transform := false
export(float, 0, 1) var speed: float = 0.5

func _ready():
	var target = get_node(self.remote_path) as Spatial

	self.local_transform = self.transform

	if not target:
		return

	target.global_transform = self.global_transform


func _physics_process(_delta: float):
	if not self.remote_path:
		push_warning("CameraInterpolation has no remote target!")
		return

	var target := get_node(self.remote_path) as Camera

	if not self.full_transform:
		var parent := self.get_parent() as Spatial

		var counter_transform = Transform() \
			.rotated(Vector3.RIGHT, -parent.rotation.x) \
			.rotated(Vector3.BACK, -parent.rotation.z)

		self.global_transform = parent.global_transform * counter_transform * self.local_transform


#	var current_rotation := target.rotation
	var current_transform := target.global_transform
	var destination_transform := self.global_transform
#	var is_moving := !self.last_destination.is_equal_approx(destination_transform)
#	var acceleration = self.acceleration * delta
#
#	self.speed += acceleration if is_moving else -acceleration
#	self.speed = min(1.0, max(self.speed, 0.1))

	target.transform = current_transform.interpolate_with(destination_transform, self.speed)

	# if not full_transform:
	#	target.rotation = Vector3(current_rotation.x, target.rotation.y, current_rotation.z)
