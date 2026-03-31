###
# Copyright (c) SimChopper; Jovan Gerodetti and contributors.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
###

extends RefCounted

var start: int
var budget: int

func _init(budget: int):
	self.budget = budget
	self.start = Time.get_ticks_msec()


func is_exceded() -> bool:
	return self.elapsed() > self.budget


func elapsed() -> int:
	return Time.get_ticks_msec() - self.start


func restart():
	self.start = Time.get_ticks_msec()
