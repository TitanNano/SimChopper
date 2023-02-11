extends Object

enum Level {
	ERROR = 0,
	WARNING = 1,
	INFO = 2,
	DEBUG = 3,
}

const LEVEL_FILTER := Level.INFO

const LEVEL_FILTER_MAP := {
#	"Car::_integrate_forces": Level.DEBUG,
}

static func level_to_string(level: int) -> String:
	match level:
		Level.ERROR:
			return "ERROR"
		Level.WARNING:
			return "WARNING"
		Level.INFO:
			return "INFO"
		Level.DEBUG:
			return "DEBUG"
		_: return "INVALID_LOG_LEVEL"


static func error(message) -> void:
	_log(message, Level.ERROR, 1)


static func warning(message) -> void:
	_log(message, Level.WARNING, 1)


static func info(message) -> void:
	_log(message, Level.INFO, 1)


static func debug(message) -> void:
	_log(message, Level.DEBUG, 1)


static func _log(message, level: int, skip_stack := 0) -> void:
	var stack := get_stack()

	for i in range(skip_stack + 1):
		stack.pop_front()

	var stack_frame: Dictionary = stack.pop_front()
	var file: String = stack_frame.get("source")
	var fun: String = stack_frame.get("function")
	var time := Time.get_ticks_msec()
	var minutes := time / 1000 / 60
	var seconds := time / 1000 % 60
	var milliseconds := time % 1000

	var path := file.split("/")
	var file_name := path[path.size() - 1].rsplit(".", false, 1)[0]
	var target := "{file}::{fun}".format({
		"file": file_name,
		"fun": fun,
	})
	var max_level := (LEVEL_FILTER_MAP[target] as int) if target in LEVEL_FILTER_MAP else LEVEL_FILTER

	if level > max_level:
		return

	if typeof(message) == TYPE_ARRAY:
		message = PoolStringArray(message).join(" ")

	var logline = "[{min}:{sec}:{msec}] [{level}] [{target}] {msg}".format({
		"target": target,
		"min": String(minutes).pad_zeros(3),
		"sec": String(seconds).pad_zeros(2),
		"msec": String(milliseconds).pad_zeros(3),
		"msg": message,
		"level": level_to_string(level),
	})

	match level:
		Level.ERROR:
			push_error(logline)
		Level.WARNING:
			push_warning(logline)
		Level.INFO, Level.DEBUG:
			print(logline)
