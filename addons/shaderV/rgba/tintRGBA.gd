@tool
extends VisualShaderNodeCustom
class_name VisualShaderNodeRGBAtintRGBA

func _init():
	set_input_port_default_value(1, 1.0)
	set_input_port_default_value(2, Vector3(1.0, 1.0, 1.0))
	set_input_port_default_value(3, 1)

func _get_name() -> String:
	return "TintRGBA"

func _get_category() -> String:
	return "RGBA"

#func _get_subcategory():
#	return ""

func _get_description() -> String:
	return "Tints RGBA with tint color (same as modulate property in editor)"

func _get_return_icon_type() -> VisualShaderNode.PortType:
	return VisualShaderNode.PORT_TYPE_VECTOR_3D

func _get_input_port_count() -> int:
	return 4

func _get_input_port_name(port: int):
	match port:
		0:
			return "col"
		1:
			return "alpha"
		2:
			return "colorTint"
		3:
			return "alphaTint"

func _get_input_port_type(port: int):
	match port:
		0:
			return VisualShaderNode.PORT_TYPE_VECTOR_3D
		1:
			return VisualShaderNode.PORT_TYPE_SCALAR
		2:
			return VisualShaderNode.PORT_TYPE_VECTOR_3D
		3:
			return VisualShaderNode.PORT_TYPE_SCALAR

func _get_output_port_count() -> int:
	return 2

func _get_output_port_name(port: int):
	match port:
		0:
			return "col"
		1:
			return "alpha"

func _get_output_port_type(port: int):
	match port:
		0:
			return VisualShaderNode.PORT_TYPE_VECTOR_3D
		1:
			return VisualShaderNode.PORT_TYPE_SCALAR

func _get_code(input_vars: Array[String], output_vars: Array[String], mode: Shader.Mode, type: VisualShader.Type) -> String:
	return """%s = %s * %s;
%s = %s * %s;""" % [output_vars[0], input_vars[0], input_vars[2],
					output_vars[1], input_vars[1], input_vars[3]]
