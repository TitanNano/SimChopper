extends Node

@onready var slot0: AudioStreamPlayer = $Slot0
@onready var slot1: AudioStreamPlayer = $Slot1

var active_slot: AudioStreamPlayer = null
var target_slot: AudioStreamPlayer = null
var elapsed := 0.0

const DURATION = 0.5 #seconds

var playing: bool : get = _get_playing
var track_name: String : get = _get_track_name

func _process(delta):
	if not active_slot or not target_slot:
		return

	if not active_slot.playing or not target_slot.playing:
		target_slot.volume_db = 0
		return

	self.elapsed += delta

	var current_dB = clamp((self.elapsed / DURATION) * -80, -80, 0)

	active_slot.volume_db = current_dB
	target_slot.volume_db = -80 - current_dB

	prints("current_dB: ", target_slot.volume_db, active_slot.volume_db)

	if active_slot.volume_db <= -80:
		active_slot.stop()


func play_track(track: AudioStream):
	self.active_slot = slot0 if slot0.playing else slot1
	self.target_slot = slot1 if slot1 != active_slot else slot0

	self.target_slot.stream = track
	self.target_slot.play(0)

	if not active_slot.playing:
		return

	target_slot.volume_db = -80
	active_slot.volume_db = 0
	elapsed = 0


func _get_playing() -> bool:
	return self.slot0.playing or self.slot1.playing


func _get_track_name() -> String:
	var stream := self.target_slot.stream if self.target_slot else null

	return stream.resource_path if stream else ""
