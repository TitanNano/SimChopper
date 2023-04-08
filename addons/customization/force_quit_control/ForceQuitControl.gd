@tool
extends ColorRect

@onready var dialog := $AcceptDialog

var params: Array

func _ready() -> void:
	self.dialog.dialog_text = self.dialog.dialog_text % self.params


func set_dialog_params(params: Array) -> void:
	self.params = params
