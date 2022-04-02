extends Reference

var start: int
var budget: int

func _init(budget: int):
	self.budget = budget
	self.start = OS.get_system_time_msecs()


func is_exceded() -> bool:
	return self.elapsed() > self.budget


func elapsed() -> int:
	return OS.get_system_time_msecs() - self.start


func restart():
	self.start = OS.get_system_time_msecs()
