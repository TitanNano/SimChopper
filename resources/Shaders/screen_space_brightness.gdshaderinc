float YtoLstar(float y) {
    	if (y <= (216.0/24389.0)) {    
            return y * (24389.0/27.0);
        } else {
            return pow(y,(1.0/3.0)) * 116.0 - 16.0;
        }
}

vec3 perceived_brightness(sampler2D screen_tex, vec2 screen_uv) {
	vec3 bg_color = textureLod(screen_tex, screen_uv, 7.0).rgb;
	float luminance = (0.299 * bg_color.r) + (0.587 * bg_color.g) + (0.114 * bg_color.b);
	
	return vec3(YtoLstar(luminance) / 100.0);
}