tool
extends DirectionalLight

func _process(_delta):
	var parent_env = self.get_parent()

	if not parent_env is WorldEnvironment:
		return
	
	var sky: ProceduralSky = parent_env.environment.background_sky;
	
	if not sky is ProceduralSky:
		return
	
	var sun_x = sky.sun_latitude * -1.0
	var sun_y = 180.0 - sky.sun_longitude 
	
	self.rotation = Vector3(deg2rad(sun_x), deg2rad(sun_y), 0)
