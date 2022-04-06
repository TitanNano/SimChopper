shader_type spatial;
//render_mode unshaded;

uniform mat4 previous_transform;
uniform mat4 current_transform;
uniform vec4 color : hint_color = vec4(0);

varying float angle;
varying float falloff; 

void vertex() {
	vec4 current_vertex = vec4(VERTEX, 1.0);
	vec4 previous_vertex = vec4(VERTEX, 1.0) * previous_transform;
	
	vec3 velocity = current_vertex.xyz - previous_vertex.xyz;
	vec3 velocity_direction = normalize(velocity);

	angle = dot(NORMAL, velocity_direction);
	
	if (angle < 0.0) {
		angle = -1.0;
	} else {
		angle = 0.0;
	}
	
	vec3 scale = velocity * 2.0;
	VERTEX += velocity * angle;
	falloff = (length(VERTEX - previous_vertex.xyz) / length(current_vertex.xyz - previous_vertex.xyz));
}

void fragment() {
	ALBEDO = color.rgb;
	ALPHA = min(1.0 - abs(falloff * 2.0 - 1.0), 1.0);
}