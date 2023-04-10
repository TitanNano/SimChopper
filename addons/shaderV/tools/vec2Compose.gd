@tool
extends VisualShaderNodeCustom
class_name VisualShaderToolsvec2Compose

func _init():
	set_input_port_default_value(0, 1.0)
	set_input_port_default_value(1, 0.0)

func _get_name() -> String:
	return "vec2Compose"

func _get_category() -> String:
	return "Tools"

#func _get_subcategory():
#	return ""

func _get_description() -> String:
	return "Makes 2d vector from length and angle (in radians)"

func _get_return_icon_type() -> VisualShaderNode.PortType:
	return VisualShaderNode.PORT_TYPE_VECTOR_3D

func _get_input_port_count() -> int:
	return 2

func _get_input_port_name(port: int):
	match port:
		0:
			return "length"
		1:
			return "radians"

func _get_input_port_type(port: int):
	match port:
		0:
			return VisualShaderNode.PORT_TYPE_SCALAR
		1:
			return VisualShaderNode.PORT_TYPE_SCALAR
	

func _get_output_port_count() -> int:
	return 1

func _get_output_port_name(port ) -> String:
	return "vec2"

func _get_output_port_type(port) -> VisualShaderNode.PortType:
	return VisualShaderNode.PORT_TYPE_VECTOR_3D

func _get_global_code(mode: Shader.Mode) -> String:
	return """
vec2 vec2ComposeFunc(float _vec2_length, float _vec2_angl_rad){
	return vec2(cos(_vec2_angl_rad), sin(_vec2_angl_rad)) * _vec2_length;
}
"""

func _get_code(input_vars: Array[String], output_vars: Array[String], mode: Shader.Mode, type: VisualShader.Type) -> String:
	return "%s.xy = vec2ComposeFunc(%s, %s);" % [output_vars[0], input_vars[0], input_vars[1]]




