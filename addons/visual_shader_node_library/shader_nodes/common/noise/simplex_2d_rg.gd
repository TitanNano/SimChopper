@tool
extends VisualShaderNodeCustom
class_name VisualShaderNodeSimplexNoise2DWithRotatingGradient

enum Inputs {
	OFFSET,
	PERIOD,
	SCALE,
	GRADIENT_ROTATION,

	I_COUNT
}

const INPUT_NAMES = ["offset", "period", "scale", "gradient_rotation"];
const INPUT_TYPES = [
	VisualShaderNode.PORT_TYPE_VECTOR_2D,
	VisualShaderNode.PORT_TYPE_VECTOR_2D,
	VisualShaderNode.PORT_TYPE_VECTOR_2D,
	VisualShaderNode.PORT_TYPE_SCALAR
]

enum Outputs {
	NOISE,
	GRADIENT,

	O_COUNT
}

const OUTPUT_NAMES = ["noise", "gradient"]
const OUTPUT_TYPES = [VisualShaderNode.PORT_TYPE_SCALAR, VisualShaderNode.PORT_TYPE_VECTOR_2D]

func _get_name():
	return "SimplexNoise2DRG"

func _get_category():
	return "Common"

func _get_subcategory():
	return "Noise"

func _get_description():
	return "Textureless simplex noise with rotating gradients and analytical gradient"

func _get_return_icon_type():
	return VisualShaderNode.PORT_TYPE_SCALAR

func _get_input_port_count():
	return Inputs.I_COUNT

func _get_input_port_name(port):
	return INPUT_NAMES[port]

func _get_input_port_type(port):
	return INPUT_TYPES[port]

func _get_output_port_count():
	return Outputs.O_COUNT

func _get_output_port_name(port):
	return OUTPUT_NAMES[port]

func _get_output_port_type(port):
	return OUTPUT_TYPES[port]

func _get_global_code(mode):
	var code = preload("simplex_2d_rg.gdshader").code
	code = code.replace("shader_type spatial;", "")
	code = code.replace("HELPER_", "HELPER_%s_" % [self._get_name()])
	return code

func _get_code(input_vars, output_vars, mode, type):
	var offset = input_vars[Inputs.OFFSET];

	if offset == null:
		offset = "vec2(0.0, 0.0)"

	offset = "%s * %s" % [offset, input_vars[Inputs.SCALE]]

	var gradient_rotation = input_vars[Inputs.GRADIENT_ROTATION]

	var result_assignment = ""

	if input_vars[Inputs.PERIOD]:
		# periodic noise
		result_assignment = "vec3 result = simplex_noise_2d_rg_p(%s, %s, %s);" % [
			offset,
			input_vars[Inputs.PERIOD],
			gradient_rotation
		]
	else:
		result_assignment = " vec3 result = simplex_noise_2d_rg_np(%s, %s);" % [
			offset,
			gradient_rotation
		]

	var noise_assignment = "%s = result.x;" % [output_vars[Outputs.NOISE]];

	var gradient_assignment = "%s = result.xy;" % [output_vars[Outputs.GRADIENT]]

	return """
	%s
	%s
	%s
	""" % [result_assignment, noise_assignment, gradient_assignment]

func _init():
	if not get_input_port_default_value(Inputs.SCALE):
		set_input_port_default_value(Inputs.SCALE, Vector2(1.0, 1.0))
	if not get_input_port_default_value(Inputs.GRADIENT_ROTATION):
		set_input_port_default_value(Inputs.GRADIENT_ROTATION, 0.0)
