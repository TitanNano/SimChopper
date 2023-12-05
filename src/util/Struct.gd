extends RefCounted


enum Fields {
	BYTE = 1,
	BYTE_2 = 2,
	BYTE_4 = 3,
	BYTE_8 = 8,
}

const fields = [Fields.BYTE, Fields.BYTE_4, Fields.BYTE_4, Fields.BYTE_4]

static func _new() -> PackedByteArray:
	var instance := PackedByteArray()
	
	for field in fields:
		for _i in range(0, field):
			instance.append(0)
			
	return instance


static func _offset(field: int) -> int:
	var offset := 0
	
	for i in range(0, field - 1):
		offset += fields[i]

	return offset


static func _field_buffer(field: int) -> PackedByteArray:
	var buffer := PackedByteArray()
	var field_size: int = fields[field]
	
	buffer.resize(field_size)
	
	return buffer


static func _set_field(instance: PackedByteArray, field: int, value: PackedByteArray) -> void:
	var offset := _offset(field)
	
	for i in range(offset, offset + value.size()):
		instance[i] = value[i - offset]


static func x(instance: PackedByteArray, value: float):
	var field := 1
	var buffer := var_to_bytes(value)
	
	assert(buffer.size() == fields[field])
	
	_set_field(instance, field, buffer)


static func surface(instance: PackedByteArray, value: int):
	var field := 0
	var buffer := var_to_bytes(value)
	
	assert(buffer.size() == fields[field])
	
	_set_field(instance, field, buffer)


static func from_vector(surface: int, vector: Vector3) -> PackedByteArray:
	var inst := _new()
	
	surface(inst, surface)
	x(inst, vector.x)
	
	return inst


# func as_vector(id: int) -> Vector3:
#	return Vector3(self.x[id], self.y[id], self.z[id])
