extends Reference


enum Fields {
	BYTE = 1,
	BYTE_2 = 2,
	BYTE_4 = 3,
	BYTE_8 = 8,
}

const fields = [Fields.BYTE, Fields.BYTE_4, Fields.BYTE_4, Fields.BYTE_4]

static func new() -> PoolByteArray:
	var instance := PoolByteArray()
	
	for field in fields:
		for _i in range(0, field):
			instance.append(0)
			
	return instance


static func _offset(field: int) -> int:
	var offset := 0
	
	for i in range(0, field - 1):
		offset += fields[i]

	return offset


static func _field_buffer(field: int) -> PoolByteArray:
	var buffer := PoolByteArray()
	var field_size: int = fields[field]
	
	buffer.resize(field_size)
	
	return buffer


static func _set_field(instance: PoolByteArray, field: int, value: PoolByteArray) -> void:
	var offset := _offset(field)
	
	for i in range(offset, offset + value.size()):
		instance[i] = value[i - offset]


static func x(instance: PoolByteArray, value: float):
	var field := 1
	var buffer := var2bytes(value)
	
	assert(buffer.size() == fields[field])
	
	_set_field(instance, field, buffer)


static func surface(instance: PoolByteArray, value: int):
	var field := 0
	var buffer := var2bytes(value)
	
	assert(buffer.size() == fields[field])
	
	_set_field(instance, field, buffer)


static func from_vector(surface: int, vector: Vector3) -> PoolByteArray:
	var inst = new()
	
	surface(inst, surface)
	x(inst, vector.x)
	
	return inst


func as_vector(id: int) -> Vector3:
	return Vector3(self.x[id], self.y[id], self.z[id])
