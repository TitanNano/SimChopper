extends Control

const World := preload("res://src/Objects/World/World.gd")
const LoadingScreen := preload("res://src/HUD/LoadingScreen.gd")

@onready var loading_screen: LoadingScreen = $LoadingScreen
@onready var viewport: SubViewportContainer = $SubViewportContainer
@onready var world: World = $SubViewportContainer/SubViewport/World

func _ready():
	world.loading_scale.connect(self._on_loading_scale)
	world.loading_progress.connect(self._on_loading_progress)
	
	var scale := DisplayServer.screen_get_scale(DisplayServer.SCREEN_OF_MAIN_WINDOW)
	var window := self.get_window()
	
	window.size *= scale
	window.position -= Vector2i(self.get_window().size / scale / 2)


func game_ready() -> void:
	loading_screen.visible = false
	viewport.visible = true
	world.process_mode = Node.PROCESS_MODE_PAUSABLE

func _on_loading_scale(total: int):
	loading_screen.total_jobs = total


func _on_loading_progress(new_progress: int):
	loading_screen.completed_jobs += new_progress

	if loading_screen.completed_jobs == loading_screen.total_jobs:
		self.game_ready()
