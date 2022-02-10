extends Reference

const TerrainRotation := preload("TerrainRotation.gd")
const ThreadPool := preload("res://util/ThreadPool.gd")
const Callable := preload("res://util/Callable.gd")

enum TileSurfaceType { Ground, Grass, Water }

signal progress(status)

class TileSurface:
	var type: int
	var corners := PoolVector3Array()
	var resolution := 2
	
	# warning-ignore:shadowed_variable
	func _init(type: int = TileSurfaceType.Ground) -> void:
		self.type = type


	func lerp_xyz(from: Vector3, to: Vector3, x, y, z) -> Vector3:
		var target_x: float = lerp(from.x, to.x, x)
		var target_y: float = lerp(from.y, to.y, y)
		var target_z: float = lerp(from.z, to.z, z)
		
		return Vector3(target_x, target_y, target_z)


	func generate_faces() -> Array:
		var faces := []
		var type := self.type 	# warning-ignore:shadowed_variable
		
		for ix in range(0, resolution):
			var weight_x_start := 1.0 / resolution * ix
			var weight_x_end := 1.0 / resolution * (ix+1)
			
			for iy in range(0, resolution):
				var weight_y_start := 1.0 / resolution * iy
				var weight_y_end := 1.0 / resolution * (iy+1)
				
				var x0 := lerp_xyz(self.corners[0], self.corners[3], weight_x_start, 0, weight_y_start)
				var x1 := lerp_xyz(self.corners[0], self.corners[3], weight_x_end, 0, weight_y_start)
				var y0 := lerp_xyz(self.corners[0], self.corners[3], weight_x_start, 0, weight_y_end)
				var y1 := lerp_xyz(self.corners[0], self.corners[3], weight_x_end, 0, weight_y_end) 
				
				faces.append_array([
					[Vertex.from_vector(type, x0), Vertex.from_vector(type, x1), Vertex.from_vector(type, y1)],
					[Vertex.from_vector(type, x0), Vertex.from_vector(type, y1), Vertex.from_vector(type, y0)],
				])

		return faces
	
	
	func duplicate(deep: bool) -> TileSurface:
		var new := TileSurface.new(self.type)
		new.resolution = self.resolution
		
		if not deep:
			new.corners = self.corners
			return new
			
		var new_corners := PoolVector3Array()
		new_corners.append_array(self.corners)
		
		new.corners = new_corners
		
		return new
		
	static func _vertex_from_vector(surface: int, vector: Vector3) -> Dictionary:
		return { "x": vector.x, "y": vector.y, "z": vector.z, "surface": surface }


	static func _vector_from_vertex(vertex: Dictionary) -> Vector3:
		return Vector3(vertex.get("x"), vertex.get("y"), vertex.get("z"))


class Vertex:
	var surface: int
	var x: float
	var y: float
	var z: float

	# warning-ignore:shadowed_variable
	# warning-ignore:shadowed_variable
	# warning-ignore:shadowed_variable
	# warning-ignore:shadowed_variable
	func _init(surface: int, x: float, y: float, z: float):
		self.surface = surface
		self.x = x
		self.y = y
		self.z = z


	func as_vector():
		return Vector3(self.x, self.y, self.z)


	# warning-ignore:shadowed_variable
	static func from_vector(surface: int, vector: Vector3) -> Vertex:
		return Vertex.new(surface, vector.x, vector.y, vector.z)


const materials := {
	TileSurfaceType.Ground: preload("res://Terrain/dirt_material.tres"),
	TileSurfaceType.Grass: preload("res://Terrain/grass_material.tres"),
	TileSurfaceType.Water: preload("res://Materials/ocean.tres")
}

var tile_size := 16
var city_size := 1
var tile_height := 8
var sea_level := 0
var rotation: TerrainRotation
var tilelist: Dictionary
var runner: SceneTree


# warning-ignore:shadowed_variable
# warning-ignore:shadowed_variable
# warning-ignore:shadowed_variable
func _init(tilelist: Dictionary, rotation: TerrainRotation, runner: SceneTree) -> void:
	self.tilelist = tilelist
	self.rotation = rotation
	self.runner = runner


# warning-ignore:shadowed_variable
func _apply_tile_slope(tile: TileSurface, slope: int, rotation: TerrainRotation) -> void:
	match slope:
		0x00:
			tile.type = TileSurfaceType.Grass

		0x01:
			tile.corners[rotation.nw()].y += tile_height
			tile.corners[rotation.ne()].y += tile_height

		0x02:
			tile.corners[rotation.ne()].y += tile_height
			tile.corners[rotation.se()].y += tile_height

		0x03:
			tile.corners[rotation.sw()].y += tile_height
			tile.corners[rotation.se()].y += tile_height

		0x04:
			tile.corners[rotation.nw()].y += tile_height
			tile.corners[rotation.sw()].y += tile_height

		0x05:
			tile.corners[rotation.nw()].y += tile_height
			tile.corners[rotation.ne()].y += tile_height
			tile.corners[rotation.se()].y += tile_height

		0x06:
			tile.corners[rotation.ne()].y += tile_height
			tile.corners[rotation.se()].y += tile_height
			tile.corners[rotation.sw()].y += tile_height

		0x07:
			tile.corners[rotation.se()].y += tile_height
			tile.corners[rotation.sw()].y += tile_height
			tile.corners[rotation.nw()].y += tile_height

		0x08:
			tile.corners[rotation.sw()].y += tile_height
			tile.corners[rotation.nw()].y += tile_height
			tile.corners[rotation.ne()].y += tile_height

		0x09:
			tile.corners[rotation.ne()].y += tile_height

		0x0A:
			tile.corners[rotation.se()].y += tile_height

		0x0B:
			tile.corners[rotation.sw()].y += tile_height

		0x0C:
			tile.corners[rotation.nw()].y += tile_height
			
		0x0D:
			tile.corners[rotation.nw()].y += tile_height
			tile.corners[rotation.ne()].y += tile_height
			tile.corners[rotation.sw()].y += tile_height
			tile.corners[rotation.se()].y += tile_height

		_:
			pass


static func _add_to_surface(surfaces: Dictionary, vertex: Vertex):
	var surface: int = vertex.surface
	
	if !surfaces.has(surface):
		surfaces[surface] = []
	
	surfaces.get(surface).append(vertex)


static func _add_face_to_surface(surfaces: Dictionary, face: Array):
	for vertex in face:
		_add_to_surface(surfaces, vertex)


static func _add_to_ybuffer(ybuffer: Dictionary, vertex: Vertex):
	var xz: Array = [vertex.x, vertex.z]
	
	if !ybuffer.has(xz):
		ybuffer[xz] = []
	
	ybuffer.get(xz).append(vertex)


# warning-ignore:shadowed_variable
# warning-ignore:shadowed_variable
static func _tile_vertex_to_city_uv(vertex: Vertex, tile_size: float, city_size: float) -> Vector2:
	var uv_x: float = vertex.x / (city_size * tile_size)
	var uv_y: float = vertex.z / (city_size * tile_size)
	
	return Vector2(uv_x, uv_y)


func _process_tile(tileData: Dictionary):
	var tile_type: int = (tileData.terrain & 0xF0) >> 4
	var tile_slope: int = tileData.terrain & 0x0F

#		assert((tile_type > 0 && tileData.is_water) || (tile_type == 0 && not tileData.is_water))

	var tile_x: int = tileData.coordinates[0] * tile_size
	var tile_y: int = tileData.coordinates[1] * tile_size
	var tile_z: int = tileData.altitude * tile_height

	var tile := TileSurface.new()

	tile.corners = PoolVector3Array([
#			0											1
		Vector3(tile_x, tile_z, tile_y), 				Vector3(tile_x + tile_size, tile_z, tile_y),
#			2											3
		Vector3(tile_x, tile_z, tile_y + tile_size),	Vector3(tile_x + tile_size, tile_z, tile_y + tile_size),
	])
	
	var water: TileSurface = null
	
	# tile is covered by water
	if tile_type > 0 && self.sea_level >= tileData.altitude:
		var water_altitude: float = tile_height * self.sea_level
		
		water = tile.duplicate(true)
		water.type = TileSurfaceType.Water
		water.resolution = 3
		
		water.corners[0].y = water_altitude
		water.corners[1].y = water_altitude
		water.corners[2].y = water_altitude
		water.corners[3].y = water_altitude

	# tile is surface water
	if tile_type == 3:
		water = tile.duplicate(true)
		water.type = TileSurfaceType.Water
		water.resolution = 3

		tile.corners[rotation.nw()].y -= tile_height
		tile.corners[rotation.ne()].y -= tile_height
		tile.corners[rotation.sw()].y -= tile_height
		tile.corners[rotation.se()].y -= tile_height

	self._apply_tile_slope(tile, tile_slope, rotation)
	
	return [tile.generate_faces(), water.generate_faces() if water != null else []]


func _thread_pool_progress(status: Dictionary) -> void:
	self.emit_signal("progress", status)


func build_terain_async() -> Mesh:
	var ybuffer := {}
	var surfaces := {}
	var thread_pool = ThreadPool.new(self.runner, Callable.new(self, "_process_tile"))
	
	thread_pool.connect("progress", self, '_thread_pool_progress')
	thread_pool.fill(self.tilelist.values())

	var terain_faces: Array = yield(thread_pool, "completed")
	thread_pool = null # free thread_pool references

	for result in terain_faces:
		var tile_faces: Array = result[0]
		var water_faces: Array = result[1]
		
		for face in tile_faces:
			for vertex in face:
				_add_to_ybuffer(ybuffer, vertex)
				_add_to_surface(surfaces, vertex)		
		
		if water_faces.size() > 0:
			for face in water_faces:
				_add_face_to_surface(surfaces, face)

	for key in ybuffer:
		var vertex_group: Array = ybuffer[key]
		var total_y: float = 0
		var count: int = vertex_group.size()
		
		for vertex in vertex_group:
			total_y += vertex.y
		
		var average_y = total_y / count
		
		for vertex in vertex_group:
			vertex.y = average_y

	var generator := SurfaceTool.new()
	var mesh := ArrayMesh.new()
	var vertex_count := 0
	
	# all vertecies are added to the y buffer and their surfaces
	# we now have to merge the y positions in the y buffer
	# and afterwards we can generate all the surfaces
	for surface_type in surfaces.keys():
		var surface: Array = surfaces.get(surface_type)
		
		generator.clear()
		generator.begin(Mesh.PRIMITIVE_TRIANGLES)
		
		for vertex in surface:
			generator.add_uv(self._tile_vertex_to_city_uv(vertex, tile_size, city_size))
			generator.add_vertex(vertex.as_vector())
			
			vertex_count += 1
		
		generator.index()
		generator.generate_normals()
		generator.generate_tangents()
		
		var surface_arrays := generator.commit_to_arrays()
		var new_index := mesh.get_surface_count()

		mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, surface_arrays)
		mesh.surface_set_material(new_index, self.materials[surface_type])

	prints("generated", vertex_count, "vertecies for terain")
	prints("done generating surfaces", OS.get_system_time_msecs())
	return mesh
