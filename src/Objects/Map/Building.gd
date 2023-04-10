var data: Dictionary

func _init(data: Dictionary):
	self.data = data


func building_id() -> int:
	return self.data.get("building_id")


func size() -> int:
	return self.data.get("size")


func name() -> String:
	return self.data.get("name")


func tile_coords() -> PackedInt32Array:
	return self.data.get("tile_coords")
