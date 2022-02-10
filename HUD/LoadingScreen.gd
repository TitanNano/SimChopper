extends Control

onready var progress = $ProgressBar

var _total_jobs := 0

var total_jobs: int setget _set_total_jobs, _get_total_jobs
var completed_jobs: int setget _set_completed_jobs, _get_completed_jobs

func _set_total_jobs(value: int):
	self.progress.step = 1
	self.progress.max_value = value


func _get_total_jobs() -> int:
	return self.progress.max_value


func _set_completed_jobs(value: int):
	self.progress.value = value

	if self.progress.value == self.progress.max_value:
		self.visible = false


func _get_completed_jobs() -> int:
	return self.progress.value

