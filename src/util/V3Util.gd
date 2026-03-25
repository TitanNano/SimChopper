###
# Copyright (c) SimChopper; Jovan Gerodetti and contributors.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
###

extends Object

const XY_PLANE := Vector3.RIGHT + Vector3.UP
const XZ_PLANE := Vector3.RIGHT + Vector3.BACK
const YZ_PLANE := Vector3.UP + Vector3.BACK


static func basis_from_normal(normal: Vector3) -> Basis:
	return Basis(normal.cross(Basis.IDENTITY.z), normal, Basis.IDENTITY.x.cross(normal))