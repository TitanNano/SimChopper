extends RefCounted

var start: int
var budget: int

func _init(budget: int):
	self.budget = budget
	self.start = Time.get_ticks_msec()


func is_exceded() -> bool:
	return self.elapsed() > self.budget


func elapsed() -> int:
	return Time.get_ticks_msec() - self.start


func restart():
	self.start = Time.get_ticks_msec()
