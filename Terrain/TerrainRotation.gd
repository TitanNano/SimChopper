extends Reference


const corners := [0, 1, 3, 2]
var offset := 0

func _init(rotation):
	assert(rotation >= 0)
	assert(rotation <= 3)
	
	self.offset = rotation
	
func _get_corner(index: int) -> int:
	var shifted_index := (index + offset) % 4
	var target_value: int = self.corners[shifted_index]
	
	return target_value
	
func nw() -> int:
	return self._get_corner(0)

func ne() -> int:
	return self._get_corner(1)

func se() -> int:
	return self._get_corner(2)

func sw() -> int:
	return self._get_corner(3)
