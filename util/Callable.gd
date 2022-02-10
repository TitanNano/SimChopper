extends Reference

var object: Object
var method: String

# warning-ignore:shadowed_variable
# warning-ignore:shadowed_variable
func _init(object: Object, method: String) -> void:
	self.object = object
	self.method = method


func invoke(args: Array):
	return self.object.callv(self.method, args)
