extends Spatial

export var connected_ranges := PoolIntArray([0x1D, 0x2C, 0x51, 0x57])

func _ready() -> void:
	pass
	
func set_orientation(north: Dictionary, east: Dictionary, south: Dictionary, west: Dictionary):
	var connected_range := [] 
	
	assert(self.connected_ranges.size() % 2 == 0)
	
	for i in range(0, self.connected_ranges.size(), 2):
		var start := self.connected_ranges[i]
		var end := self.connected_ranges[i + 1]
		
		connected_range += range(start, end)
	
	var north_weight := 1 if (north.building and north.building.building_id in connected_range) else 0  
	var south_weight := 1 if (south.building and south.building.building_id in connected_range) else 0  
	var west_weight := 1 if (west.building and west.building.building_id in connected_range) else 0  
	var east_weight := 1 if (east.building and east.building.building_id in connected_range) else 0  

	var east_west := east_weight + west_weight
	var north_south := north_weight + south_weight
	
	var rotation := 90 if east_west > north_south else 0
	
	self.rotation.y = deg2rad(rotation)	
