tool
extends Position3D


# Declare member variables here. Examples:
# var a: int = 2
# var b: String = "text"

var _vector := Vector3.UP

export(Color) var color setget _set_color, _get_color
export(Vector3) var vector setget _set_vector, _get_vector

onready var length = get_node("Length")
onready var head = get_node("Head")

var is_ready := false

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	self.is_ready = true
	self.head.material = self.head.material.duplicate()
	self.length.material = self.length.material.duplicate()


func _set_color(value: Color) -> void:
	if not self.is_ready:
		yield(self, "ready")

	self.head.material.albedo_color = value
	self.length.material.albedo_color = value


func _get_color() -> Color:
	return self.length.material.albedo_color


func _set_vector(v: Vector3) -> void:
	if not self.is_ready:
		yield(self, "ready")

	self._vector = v
	self.length.height = v.length()
	self.length.translation.y = v.length() / 2
	self.head.translation.y = v.length() + 0.4
	self.global_transform.basis = Basis.IDENTITY

	var counter_axis := self._vector.x if abs(self._vector.x) > abs(self.vector.z) else self._vector.z

	var direction_y := Vector3(counter_axis, self._vector.y,  0)
	var angle_y := Vector3.UP.angle_to(direction_y)

	self.global_rotate(Vector3.LEFT, angle_y)

	var direction_xz := self._vector * (Vector3.BACK + Vector3.RIGHT)
	var angle_xz := Vector3.FORWARD.angle_to(direction_xz)

	self.global_rotate(Vector3.UP, angle_xz * -1)


func _get_vector() -> Vector3:
	return self._vector
