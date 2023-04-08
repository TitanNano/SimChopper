extends Control

@onready var progress = $ProgressBar

var total_jobs: int : get = _get_total_jobs, set = _set_total_jobs
var completed_jobs: int : get = _get_completed_jobs, set = _set_completed_jobs

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

