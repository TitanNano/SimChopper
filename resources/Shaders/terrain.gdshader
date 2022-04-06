// NOTE: Shader automatically converted from Godot Engine 3.4.4.stable's SpatialMaterial.

shader_type spatial;
render_mode blend_mix,depth_draw_opaque,cull_back,diffuse_burley,specular_schlick_ggx;
uniform vec4 albedo_grass : hint_color;
uniform vec4 albedo: hint_color;
uniform float specular;
uniform float metallic;
uniform float roughness : hint_range(0,1);
uniform float point_size : hint_range(0,128);
uniform sampler2D texture_ambient_occlusion : hint_white;
uniform vec4 ao_texture_channel;
uniform float ao_light_affect;
uniform vec4 transmission : hint_color;
uniform sampler2D texture_transmission : hint_black;
uniform vec3 uv1_scale;
uniform vec3 uv1_offset;
uniform vec3 uv2_scale;
uniform vec3 uv2_offset;
varying vec3 world_normal;


void vertex() {
	UV=UV*uv1_scale.xy+uv1_offset.xy;
	world_normal = (vec4(NORMAL, 1.0) * WORLD_MATRIX).xyz;
}




void fragment() {
	vec2 base_uv = UV;
	float grass_contribution = clamp(((world_normal.y - 0.95) / 0.05), 0.0, 1.0);
	float dirt_contribution = clamp(((1.0 - world_normal.y) / 0.05), 0.0, 1.0);
	ALBEDO = (dirt_contribution  * albedo.rgb) + (albedo_grass.rgb * grass_contribution);
	METALLIC = metallic;
	ROUGHNESS = roughness;
	SPECULAR = specular;
	AO = dot(texture(texture_ambient_occlusion,base_uv),ao_texture_channel);
	AO_LIGHT_AFFECT = ao_light_affect;
	vec3 transmission_tex = texture(texture_transmission,base_uv).rgb;
	TRANSMISSION = (transmission.rgb+transmission_tex);
}
