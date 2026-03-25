###
# Copyright (c) SimChopper; Jovan Gerodetti and contributors.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
###

var data: Dictionary

func _init(data: Dictionary):
	self.data = data


func building_id() -> int:
	return self.data.get("building_id")


func size() -> int:
	return self.data.get("size")


func name() -> String:
	return self.data.get("name")


func tile_coords() -> PackedInt32Array:
	return self.data.get("tile_coords")