// Inspired by: https://developer.nvidia.com/gpugems/gpugems2/part-ii-shading-lighting-and-shadows/chapter-19-generic-refraction-simulation

shader_type spatial;
render_mode cull_back, specular_schlick_ggx;

// Varyings
varying vec2 vary_world_uv;
varying vec2 vary_world_uv2;

global uniform float ocean_noise_scale : hint_range(0, 1);
global uniform float ocean_wave_dir : hint_range(0, 1);
global uniform float ocean_speed : hint_range(0, 0.5, 0.001);
global uniform sampler2D ocean_noise : hint_default_black, repeat_enable, filter_linear_mipmap;
global uniform float ocean_wave2_dir : hint_range(0, 1);
global uniform float ocean_wave_ratio : hint_range(0, 1);
global uniform sampler2D ocean_normal_map : hint_normal, repeat_enable, filter_linear_mipmap;
global uniform vec4 ocean_water_color: source_color;
global uniform vec4 ocean_deep_water_color: source_color;
uniform float Normal_Depth;
uniform sampler2D screen_texture : hint_screen_texture, repeat_disable, filter_linear_mipmap;
uniform sampler2D depth_texture : hint_depth_texture, repeat_disable, filter_linear_mipmap;
uniform float wave_height;

const float WAVE_CONST = 6.28300;

// Perform color correction for Spatial shader:
// https://github.com/godotengine/godot/issues/70927#issuecomment-1371464030
vec4 correct_color(vec4 color) {
	return pow(color, vec4(2.2));
}

vec2 vec2ComposeFunc(float _vec2_length, float _vec2_angl_rad){
	return vec2(cos(_vec2_angl_rad), sin(_vec2_angl_rad)) * _vec2_length;
}

float depth(vec2 screen_uv, mat4 inv_projection_matrix) {
	float depth = texture(depth_texture, screen_uv).x;
	vec3 ndc = vec3(screen_uv * 2.0 - 1.0, depth);

	vec4 view = inv_projection_matrix * vec4(ndc, 1.0);
	view.xyz /= view.w;

	return -view.z;
}

vec3 sample_wave_noise(sampler2D map) {
	vec3 wave_normal = texture(map, vary_world_uv).rgb;
	vec3 wave2_normal = texture(map, vary_world_uv2).rgb;

	vec3 mixed_waves = mix(wave_normal, wave2_normal, ocean_wave_ratio);

	return mixed_waves;
}

void vertex() {
	vec3 word_vertex = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	float scaled_time = ocean_speed * 0.5 * TIME;
	vec2 world_uv = vec2(word_vertex.x, word_vertex.z) * vec2(ocean_noise_scale);

	vec3 wave_uv_offset = vec3(vec2ComposeFunc(1.00000, WAVE_CONST * ocean_wave_dir), 0.0);
	vec2 wave_uv_offset_t = vec2(wave_uv_offset.xy) * vec2(scaled_time);

	vary_world_uv = world_uv + wave_uv_offset_t;

	vec3 wave2_uv_offset = vec3(vec2ComposeFunc(1.00000, WAVE_CONST * ocean_wave2_dir), 0.0);
	vec2 wave2_uv_offset_t = vec2(wave2_uv_offset.xy) * vec2(scaled_time);

	vary_world_uv2 = world_uv + wave2_uv_offset_t;

	float mixed_waves = sample_wave_noise(ocean_noise).r;
	float mixed_waves_inv = mixed_waves - 1.00000;

	float vertex_y_offset = mixed_waves_inv * wave_height;
	float vertex_y = VERTEX.y + vertex_y_offset;

	vec3 transformed_vertex = vec3(VERTEX.r, vertex_y, VERTEX.b);


// Output
	VERTEX = transformed_vertex;
}

void fragment() {
	vec3 normal = sample_wave_noise(ocean_normal_map);

	NORMAL_MAP = normal;
	NORMAL_MAP_DEPTH = Normal_Depth;
	ROUGHNESS = 0.2;

	float n_dot_l = max(dot(VIEW, NORMAL), 0);
	float facing = (1.0 - n_dot_l);

	if (facing > 0.999) {
		ALBEDO = correct_color(ocean_water_color).rgb;
	} else {
		float dist_scale = clamp(6.0 / depth(SCREEN_UV, INV_PROJECTION_MATRIX), 0.0, 1.0);
		vec4 water_deep_albedo = vec4(0, 0, 0, 0);

		if (dist_scale > 0.0) {
			vec3 refraction_offset = (2.0 * normal.xyz - 1.0) * vec3(0.075, 0.075, 1.0);

			float depth_diff = FRAGCOORD.z - texture(depth_texture, SCREEN_UV + refraction_offset.rg).r;
			float depth_mask = ceil(clamp(depth_diff, 0.0, 1.0));

			vec4 refraction_a = texture(screen_texture, SCREEN_UV.xy + refraction_offset.xy);
			vec4 refraction_b = texture(screen_texture, SCREEN_UV.xy);
			vec4 refraction_value = refraction_b * depth_mask + refraction_a * (1.0 - depth_mask);

			water_deep_albedo = (refraction_value * dist_scale + (1.0 - dist_scale) * correct_color(ocean_deep_water_color));
		} else {
			water_deep_albedo = correct_color(ocean_deep_water_color);
		}

		// Lerp between water color and deep water color
	  	vec4 water_albedo = (correct_color(ocean_water_color) * facing + water_deep_albedo * (1.0 - facing));

		// Refraction output
		ALBEDO = water_albedo.rgb;
	}
}
