// NOTE: Shader automatically converted from Godot Engine 3.4.4.stable's SpatialMaterial.

shader_type spatial;
render_mode blend_mix,depth_draw_opaque,cull_back,diffuse_burley,specular_schlick_ggx;
uniform vec4 albedo_grass : source_color;
uniform vec4 albedo: source_color;
uniform float specular;
uniform float metallic;
uniform float roughness : hint_range(0,1);
uniform vec3 uv1_scale;
uniform vec3 uv1_offset;
varying vec3 world_normal;


void vertex() {
	UV=UV*uv1_scale.xy+uv1_offset.xy;
	world_normal = (vec4(NORMAL, 1.0) * MODEL_MATRIX).xyz;
}




void fragment() {
	float grass_contribution = clamp(((world_normal.y - 0.95) / 0.05), 0.0, 1.0);
	float dirt_contribution = clamp(((1.0 - world_normal.y) / 0.05), 0.0, 1.0);

	if (COLOR.rgb != vec3(0, 0, 0)) {
		ALBEDO = COLOR.rgb;
	} else {
		ALBEDO = (dirt_contribution  * albedo.rgb) + (albedo_grass.rgb * grass_contribution);
		METALLIC = metallic;
		ROUGHNESS = roughness;
		SPECULAR = specular;
	}
}
