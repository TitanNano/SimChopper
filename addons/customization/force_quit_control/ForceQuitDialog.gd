@tool
extends AcceptDialog

func _ready() -> void:
	self.exclusive = true
	self.visible = true
	self.confirmed.connect(self._on_invalid_engine_confirmed)


func _on_invalid_engine_confirmed():
	get_tree().quit(0)
