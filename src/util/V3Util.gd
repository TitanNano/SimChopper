extends Object

const XY_PLANE := Vector3.RIGHT + Vector3.UP
const XZ_PLANE := Vector3.RIGHT + Vector3.BACK
const YZ_PLANE := Vector3.UP + Vector3.BACK


static func basis_from_normal(normal: Vector3) -> Basis:
	return Basis(normal.cross(Basis.IDENTITY.z), normal, Basis.IDENTITY.x.cross(normal))
