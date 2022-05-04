tool
extends AcceptDialog

func _ready() -> void:
	self.call_deferred("show_modal", true)
	self.connect("confirmed", self, "_on_invalid_engine_confirmed")


func _on_invalid_engine_confirmed():
	get_parent().queue_free()
