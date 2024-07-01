use godot::{
	builtin::{NodePath, Transform3D, Vector3, Vector3Axis},
	engine::{light_3d, AnimationPlayer, FogVolume, Node, OmniLight3D},
	obj::Gd,
};
use godot_rust_script::{godot_script_impl, GodotScript};

use crate::util::logger;

#[derive(GodotScript, Debug)]
#[script(base = Node)]
struct FireSpawner {
	#[export(node_path = ["FogVolume"])]
	pub fire_path: NodePath,
	fire: Option<Gd<FogVolume>>,

	#[export(node_path = ["FogVolume"])]
	pub smoke_path: NodePath,
	smoke: Option<Gd<FogVolume>>,

	#[export(node_path = ["OmniLight3D"])]
	pub light_source_path: NodePath,
	light_source: Option<Gd<OmniLight3D>>,

	#[export(node_path = ["AnimationPlayer"])]
	pub audio_player_path: NodePath,
	audio_player: Option<Gd<AnimationPlayer>>,

	base: Gd<Node>,
}

#[godot_script_impl]
impl FireSpawner {
	pub fn _ready(&mut self) {
		logger::debug!("Init Fire spawner...");
		self.fire = self.base.try_get_node_as(self.fire_path.clone());
		self.smoke = self.base.try_get_node_as(self.smoke_path.clone());
		self.audio_player = self.base.try_get_node_as(self.audio_player_path.clone());
		self.light_source = self.base.try_get_node_as(self.light_source_path.clone());

		if let Some(ref mut audio_player) = self.audio_player {
			audio_player.set_current_animation("burning".into());
		} else {
			logger::warn!("No audio player has been setup!");
		}
	}

	pub fn resize(&mut self, size: Vector3) {
		logger::debug!("Resizing fire spawner...");
		let Some(ref mut fire) = self.fire else {
			logger::error!("Failed to resize fire spawner! No fire setup!");
			return;
		};

		let Some(ref mut smoke) = self.smoke else {
			logger::error!("Failed to resize fire spawner! No smoke setup!");
			return;
		};

		let Some(ref mut light_source) = self.light_source else {
			logger::error!("Failed to resize fire spawner! No light source setup!");
			return;
		};

		let smoke_ratio = smoke.get_size() / fire.get_size();
		let fire_size = size * Vector3::new(1.0, 1.5, 1.0);
		let smoke_size = fire_size * smoke_ratio;
		let light_size = size;

		fire.set_size(fire_size);
		fire.set_transform(Transform3D::default().translated(Vector3::new(
			0.0,
			fire_size.y / 2.0 * 0.9,
			0.0,
		)));

		smoke.set_size(smoke_size);
		smoke.set_transform(Transform3D::default().translated(Vector3::new(
			0.0,
			smoke_size.y / 2.0 * 1.2,
			0.0,
		)));

		let light_max_size = match light_size.max_axis().unwrap_or(Vector3Axis::X) {
			Vector3Axis::X => light_size.x,
			Vector3Axis::Y => light_size.y,
			Vector3Axis::Z => light_size.z,
		};

		light_source.set_param(light_3d::Param::RANGE, light_size.length_squared());
		light_source.set_param(light_3d::Param::SIZE, light_max_size);
		light_source.set_transform(Transform3D::default().translated(Vector3::new(
			0.0,
			light_size.y / 2.0,
			0.0,
		)))
	}
}
