@tool
extends ColorRect

@onready var dialog: AcceptDialog = $AcceptDialog

var params: Array

func _ready() -> void:
	self.dialog.dialog_text = self.dialog.dialog_text % self.params
	self.dialog.popup_centered()


func set_dialog_params(params: Array) -> void:
	self.params = params
