@tool
extends VisualShaderNodeCustom
class_name VisualShaderToolsPolarToCartesian

func _init():
	set_input_port_default_value(0, Vector3(1.0, 1.0, 0.0))

func _get_name() -> String:
	return "PolarToCartesian"

func _get_category() -> String:
	return "Tools"

func _get_subcategory():
	return "TransformCoordinates"

func _get_description() -> String:
	return "Polar (r, theta) -> Cartesian (x, y)"

func _get_return_icon_type() -> VisualShaderNode.PortType:
	return VisualShaderNode.PORT_TYPE_VECTOR_3D

func _get_input_port_count() -> int:
	return 1

func _get_input_port_name(port: int) -> String:
	return "polar"

func _get_input_port_type(port: int) -> VisualShaderNode.PortType:
	return VisualShaderNode.PORT_TYPE_VECTOR_3D
	

func _get_output_port_count() -> int:
	return 1

func _get_output_port_name(port ) -> String:
	return "cartesian"

func _get_output_port_type(port) -> VisualShaderNode.PortType:
	return VisualShaderNode.PORT_TYPE_VECTOR_3D

func _get_global_code(mode: Shader.Mode) -> String:
	return """
vec2 polarToCartesianFunc(vec2 _polar_vec2){
//	(r, theta) -> (x, y)
	return vec2(_polar_vec2.x * cos(_polar_vec2.y),
				_polar_vec2.x * sin(_polar_vec2.y));
}
"""

func _get_code(input_vars: Array[String], output_vars: Array[String], mode: Shader.Mode, type: VisualShader.Type) -> String:
	return "%s.xy = polarToCartesianFunc(%s.xy);" % [output_vars[0], input_vars[0]]

