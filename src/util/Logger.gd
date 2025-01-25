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


static func error(message: Variant) -> void:
	_log(message, Level.ERROR, 1)


static func warning(message: Variant) -> void:
	_log(message, Level.WARNING, 1)


static func info(message: Variant) -> void:
	_log(message, Level.INFO, 1)


static func debug(message: Variant) -> void:
	_log(message, Level.DEBUG, 1)


static func _log(message: Variant, level: int, skip_stack := 0) -> void:
	var stack := get_stack()

	for i in range(skip_stack + 1):
		stack.pop_front()

	var target := "unknown"
	var maybe_stack_frame = stack.pop_front()
	
	if maybe_stack_frame != null:
		var stack_frame: Dictionary = maybe_stack_frame
		var file: String = stack_frame.get("source")
		var fun: String = stack_frame.get("function")

		var path := file.split("/")
		var file_name := path[path.size() - 1].rsplit(".", false, 1)[0]
		
		target = "{file}::{fun}".format({
			"file": file_name,
			"fun": fun,
		})
	
	var filter: int = LEVEL_FILTER_MAP.get(target) if LEVEL_FILTER_MAP.has(target) else LEVEL_FILTER
	
	if level > filter:
		return
		
	var time := Time.get_ticks_msec()
	@warning_ignore("integer_division")
	var minutes := time / 1000 / 60
	@warning_ignore("integer_division")
	var seconds := time / 1000 % 60
	var milliseconds := time % 1000

	if typeof(message) == TYPE_ARRAY:
		var message_parts: Array = message
		message = " ".join(PackedStringArray(message_parts))

	var logline = "[{min}:{sec}:{msec}] [{level}] [{target}] {msg}".format({
		"target": target,
		"min": String.num_int64(minutes).pad_zeros(3),
		"sec": String.num_int64(seconds).pad_zeros(2),
		"msec": String.num_int64(milliseconds).pad_zeros(3),
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
