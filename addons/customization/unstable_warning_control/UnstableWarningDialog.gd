@tool
extends AcceptDialog

func _ready() -> void:
	self.confirmed.connect(self._on_invalid_engine_confirmed)
	self.popup_centered()


func _on_invalid_engine_confirmed():
	get_parent().queue_free()
