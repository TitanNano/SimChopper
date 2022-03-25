extends Control

var game_ready := false

onready var loading_screen := $LoadingScreen
onready var viewport := $ViewportContainer
onready var world := $ViewportContainer/Viewport/World

func _ready():
	world.connect("loading_scale", self, "_on_loading_scale")
	world.connect("loading_progress", self, "_on_loading_progress")


func _process(delta: float) -> void:
	loading_screen.visible = !game_ready
	viewport.visible = game_ready


func _on_loading_scale(total: int):
	loading_screen.total_jobs = total


func _on_loading_progress(new_progress: int):
	loading_screen.completed_jobs += new_progress

	if loading_screen.completed_jobs == loading_screen.total_jobs:
		self.game_ready = true
