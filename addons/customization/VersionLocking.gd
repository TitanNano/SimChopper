@tool
extends RefCounted

const CustomProjectSettings := preload("res://src/CustomProjectSettings.gd")

static func verify(editor: EditorInterface) -> void:
	var engine_version := Engine.get_version_info()
	var desired_version: String = ProjectSettings.get_setting(CustomProjectSettings.EDITOR_REQUIRED_VERSION)

	if engine_version.status != "stable":
		var dialog_scene: PackedScene = load("res://addons/customization/unstable_warning_control/unstable_warning_control.tscn")
		var dialog = dialog_scene.instantiate()

		editor.get_base_control().call_deferred("add_child", dialog)
		return

	var version_parts := desired_version.split(".")
	var major := int(version_parts[0])
	var minor := int(version_parts[1])
	var patch := int(version_parts[2] if version_parts.size() >= 3 else 0)

	if major != engine_version.major || minor != engine_version.minor || engine_version.patch != patch:
		var dialog_scene: PackedScene = load("res://addons/customization/force_quit_control/force_quit_control.tscn")
		var dialog = dialog_scene.instantiate()

		dialog.set_dialog_params([engine_version.major, engine_version.minor, engine_version.patch, major, minor, patch])
		editor.get_base_control().call_deferred("add_child", dialog)

		return
