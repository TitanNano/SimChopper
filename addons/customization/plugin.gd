tool
extends EditorPlugin

const VersionLocking := preload("VersionLocking.gd")

func _ready() -> void:
	VersionLocking.verify(get_editor_interface())
