###
# Copyright (c) SimChopper; Jovan Gerodetti and contributors.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
###

extends Control

const World := preload("res://src/Objects/World/World.gd")
const LoadingScreen := preload("res://src/HUD/LoadingScreen.gd")

@onready var loading_screen: LoadingScreen = $LoadingScreen
@onready var viewport: SubViewportContainer = $SubViewportContainer
@onready var world: World = $SubViewportContainer/SubViewport/World

func _ready():
	world.loading_scale.connect(self._on_loading_scale)
	world.loading_progress.connect(self._on_loading_progress)


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
