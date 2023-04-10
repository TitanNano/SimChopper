shader_type spatial;
render_mode blend_mix;

uniform sampler2D lifetime_fade: hint_default_white;
uniform sampler2D texture1;
uniform sampler2D texture2;
uniform sampler2D texture3;
uniform sampler2D texture4;
uniform sampler2D texture5;
uniform sampler2D texture6;
uniform float distance_fade_start = 10.0;
uniform float distance_fade_length = 10.0;

varying float age;
varying float fade_distance;

void vertex() {
	age = INSTANCE_CUSTOM.y;

	// billboard mode
	mat4 mat_world = mat4(normalize(INV_VIEW_MATRIX[0])*length(MODEL_MATRIX[0]),normalize(INV_VIEW_MATRIX[1])*length(MODEL_MATRIX[0]),normalize(INV_VIEW_MATRIX[2])*length(MODEL_MATRIX[2]),MODEL_MATRIX[3]);
	mat_world = mat_world * mat4( vec4(cos(INSTANCE_CUSTOM.x),-sin(INSTANCE_CUSTOM.x), 0.0, 0.0), vec4(sin(INSTANCE_CUSTOM.x), cos(INSTANCE_CUSTOM.x), 0.0, 0.0),vec4(0.0, 0.0, 1.0, 0.0),vec4(0.0, 0.0, 0.0, 1.0));
	MODELVIEW_MATRIX = VIEW_MATRIX * mat_world;
	
	// face distance
	float world_y = (MODEL_MATRIX * vec4(VERTEX, 1.0)).y;
	float camera_world_y = (INV_VIEW_MATRIX * vec4(0.0, 0.0, 0.0, 1.0)).y;
	
	fade_distance = abs(camera_world_y - world_y);
}

void fragment() {
	vec4 sample1 = texture(texture1, UV);
	vec4 sample2 = texture(texture2, UV);
	vec4 sample3 = texture(texture3, UV);
	vec4 sample4 = texture(texture4, UV);
	vec4 sample5 = texture(texture5, UV);
	vec4 sample6 = texture(texture6, UV);
	vec4 alpha_multiply = texture(lifetime_fade, vec2(age, age));
	
	vec4 options[] = { sample1, sample2, sample3, sample4, sample5, sample6 };
	
	int index = int(COLOR.r * 6.0);
	float fade_strength = 1.0 - (clamp(fade_distance, distance_fade_start, distance_fade_start + distance_fade_length) - distance_fade_start) / distance_fade_length;
	
	
	
	ALBEDO = options[index].rgb;
	ALPHA = options[index].a * alpha_multiply.x * fade_strength;
}