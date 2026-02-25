@tool
extends Node3D
class_name CameraInterpolation

const ViewPort := preload("res://src/ViewPort.gd")

@export var local_transform: Transform3D
@export_range(0, 1) var interpolation_speed: float = 0.5
@export_range(0, 1) var transition_speed: float = 0.2
@export var snap := false
@export_group("Tracking", "track_")
@export var track_x_axis := false
@export var track_y_axis := false
@export var track_z_axis := false

var buffered_active := false
var in_transition := false

@export var active: bool:
	get:
		var viewport := self.get_viewport() as ViewPort
		
		if viewport == null:
			return self.buffered_active
		
		return viewport.current_camera_controller == self
	
	set(value):
		var viewport := self.get_viewport() as ViewPort
		
		if viewport == null:
			self.buffered_active = value
			return
		
		if value:
			viewport.current_camera_controller = self
			self.in_transition = true
		else:
			viewport.current_camera_controller = null
			
var speed: float:
	get:
		if self.in_transition:
			return self.transition_speed
		
		return self.interpolation_speed


@onready var camera := self.get_viewport().get_camera_3d()

func _ready():
	if self.buffered_active:
		self.active = true

	if Engine.is_editor_hint():
		return
 
	self.local_transform = self.transform
	
	if not self.active:
		return
		
	self.camera.global_transform = self.global_transform


func _physics_process(_delta: float):
	if Engine.is_editor_hint():
		if self.active:
			self.camera.global_transform = self.global_transform
		return
	
	if not self.active:
		return
	
	var target := self.camera
	var parent := self.get_parent() as Node3D
	var counter_transform := parent.global_transform
	@warning_ignore("shadowed_variable_base_class")
	var rotation_order := get_euler_order(parent.rotation_order)
	
	# first decompose parent rotation
	rotation_order.reverse()
	
	for axis in rotation_order:
		match axis:
			Vector3.AXIS_X:
				counter_transform = counter_transform.rotated_local(Vector3.RIGHT, -parent.rotation.x)
	
			Vector3.AXIS_Z:
				counter_transform = counter_transform.rotated_local(Vector3.BACK, -parent.rotation.z)
			
			Vector3.AXIS_Y:
				counter_transform = counter_transform.rotated_local(Vector3.UP, -parent.rotation.y)

	# reapply tracked asixes
	rotation_order.reverse()
	
	for axis in rotation_order:
		match axis:
			Vector3.AXIS_X:
				if self.track_x_axis:
					counter_transform = counter_transform.rotated_local(Vector3.RIGHT, parent.rotation.x)
	
			Vector3.AXIS_Z:
				if self.track_z_axis:
					counter_transform = counter_transform.rotated_local(Vector3.BACK, parent.rotation.z)

			Vector3.AXIS_Y:
				if self.track_y_axis:
					counter_transform = counter_transform.rotated_local(Vector3.UP, parent.rotation.y)


	self.global_transform = counter_transform * self.local_transform

	var current_transform := target.global_transform
	var destination_transform := self.global_transform

	target.transform = current_transform.interpolate_with(destination_transform, self.speed if not snap else 1.0)
	self.snap = false
	
	if target.transform.is_equal_approx(destination_transform):
		self.in_transition = false

func get_euler_order(order: EulerOrder) -> Array[int]:
	match order:
		EULER_ORDER_XYZ:
			return [Vector3.AXIS_X, Vector3.AXIS_Y, Vector3.AXIS_Z]
		EULER_ORDER_XZY:
			return [Vector3.AXIS_X, Vector3.AXIS_Z, Vector3.AXIS_Y]
		EULER_ORDER_YXZ:
			return [Vector3.AXIS_Y, Vector3.AXIS_X, Vector3.AXIS_Z]
		EULER_ORDER_YZX:
			return [Vector3.AXIS_Y, Vector3.AXIS_Z, Vector3.AXIS_X]
		EULER_ORDER_ZXY:
			return [Vector3.AXIS_Z, Vector3.AXIS_X, Vector3.AXIS_Y] 
		EULER_ORDER_ZYX:
			return [Vector3.AXIS_Z, Vector3.AXIS_Y, Vector3.AXIS_X]

	return []
	
