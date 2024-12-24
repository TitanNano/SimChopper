// Inspired by: https://developer.nvidia.com/gpugems/gpugems2/part-ii-shading-lighting-and-shadows/chapter-19-generic-refraction-simulation

shader_type spatial;

// Varyings
varying vec2 vary_world_uv;
varying vec2 vary_world_uv2;

uniform float noise_scale : hint_range(0, 1) = 0.01999999955297;
uniform float Wave_Dir : hint_range(0, 1);
uniform float speed : hint_range(0, 0.5, 0.05000000074506) = 0.10000000149012;
uniform sampler2D Noise : hint_default_black;
uniform float Wave2_Dir : hint_range(0, 1);
uniform float Wave_Ratio : hint_range(0, 1) = 0.5;
uniform float wave_height;
uniform sampler2D Normal_Map : hint_normal, repeat_enable;
uniform float Normal_Depth;
uniform sampler2D screen_texture : hint_screen_texture, repeat_disable, filter_linear_mipmap;
uniform sampler2D depth_texture : hint_depth_texture, repeat_disable;
uniform vec3 water_color: source_color = vec3(0, 0.15, 0.115);
uniform vec3 water_deep_color: source_color = vec3(0.0, 0.15, 0.115);

const float WAVE_CONST = 6.28300;

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

float fresnel(float n_dot_l, float fresnel_bias, float fresnel_pow) {
	float facing = (1.0 - n_dot_l);

	return max(fresnel_bias + (1.0 - fresnel_bias) * pow(facing, fresnel_pow), 0.0);
}

void vertex() {
	vec3 word_vertex = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	float scaled_time = speed * TIME;
	vec2 world_uv = vec2(word_vertex.x, word_vertex.z) * vec2(noise_scale);

	vec3 wave_uv_offset = vec3(vec2ComposeFunc(1.00000, WAVE_CONST * Wave_Dir), 0.0);
	vec2 wave_uv_offset_t = vec2(wave_uv_offset.xy) * vec2(scaled_time);

	float wave_noise = texture(Noise, world_uv + wave_uv_offset_t).r;

	vec3 wave2_uv_offset = vec3(vec2ComposeFunc(1.00000, WAVE_CONST * Wave2_Dir), 0.0);
	vec2 wave2_uv_offset_t = vec2(wave2_uv_offset.xy) * vec2(scaled_time);

	float wave2_noise = texture(Noise, world_uv + wave2_uv_offset_t).r;

	float mixed_waves = mix(wave_noise, wave2_noise, Wave_Ratio);
	float mixed_waves_inv = mixed_waves - 1.00000;

	float vertex_y_offset = mixed_waves_inv * wave_height;
	float vertex_y = VERTEX.y + vertex_y_offset;

	vec3 transformed_vertex = vec3(VERTEX.r, vertex_y, VERTEX.b);


// Output
	VERTEX = transformed_vertex;

	vary_world_uv = world_uv + wave_uv_offset_t;
	vary_world_uv2 = world_uv + wave2_uv_offset_t;
}

void fragment() {
	vec2 world_uv = mix(vary_world_uv, vary_world_uv2, Wave_Ratio);
	vec3 normal = texture(Normal_Map, world_uv).rgb;
	
	vec3 refraction_offset = (2.0 * normal.xyz - 1.0) * vec3(0.075, 0.075, 1.0);
	
	float n_dot_l = max(dot(VIEW, NORMAL), 0);
	float facing = (1.0 - n_dot_l);
	float fresnel = fresnel(n_dot_l, 0.2, 5.0);
	
	float depth_diff = FRAGCOORD.z - texture(depth_texture, SCREEN_UV + refraction_offset.rg).r;
	float depth_mask = ceil(clamp(depth_diff, 0.0, 1.0));
	
	vec4 refraction_a = texture(screen_texture, SCREEN_UV.xy + refraction_offset.xy);	
	vec4 refraction_b = texture(screen_texture, SCREEN_UV.xy);
	vec4 refraction_value = refraction_b * depth_mask + refraction_a * (1.0 - depth_mask);
	
	float dist_scale = clamp(10.0 / depth(SCREEN_UV, INV_PROJECTION_MATRIX), 0.0, 1.0);
	
	vec3 water_deep_albedo = (refraction_value.xyz * dist_scale + (1.0 - dist_scale) * water_deep_color);
  	
	// Lerp between water color and deep water color
  	vec3 water_albedo = (water_color * facing + water_deep_albedo * (1.0 - facing));

	ROUGHNESS = 1.0 - max(fresnel, 1.0);
	NORMAL_MAP = normal;
	NORMAL_MAP_DEPTH = Normal_Depth;
	
	// Refraction output
	ALBEDO = water_albedo;
}
