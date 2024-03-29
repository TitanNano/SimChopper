@tool
extends VisualShaderNodeCustom
class_name VisualShaderToolsRandomFloat

func _init():
	pass

func _get_name() -> String:
	return "RandomFloat"

func _get_category() -> String:
	return "Tools"

func _get_subcategory() -> String:
	return "Random"

func _get_description() -> String:
	return "Returns random float based checked input value. UV is default input value."

func _get_return_icon_type() -> VisualShaderNode.PortType:
	return VisualShaderNode.PORT_TYPE_SCALAR

func _get_input_port_count() -> int:
	return 1

func _get_input_port_name(port: int) -> String:
	return "in"

func _get_input_port_type(port: int) -> VisualShaderNode.PortType:
	return VisualShaderNode.PORT_TYPE_VECTOR_3D

func _get_output_port_count() -> int:
	return 1

func _get_output_port_name(port: int) -> String:
	return "rand"

func _get_output_port_type(port: int) -> VisualShaderNode.PortType:
	return VisualShaderNode.PORT_TYPE_SCALAR

func _get_code(input_vars: Array[String], output_vars: Array[String], mode: Shader.Mode, type: VisualShader.Type) -> String:
	var uv = "UV"
	
	if input_vars[0]:
		uv = input_vars[0]
	
	return output_vars[0] + " = fract(sin(dot(%s.xy, vec2(12.9898, 78.233))) * 43758.5453123);" % uv
