use godot::meta::ToGodot;
use godot::{obj::NewAlloc, tools::load};
use godot_rust_script::godot::classes::Timer;
use godot_rust_script::{
    godot::prelude::{godot_error, Gd, Node3D, NodePath, PackedScene},
    godot_script_impl, GodotScript,
};

use crate::script_callable;

#[derive(Debug, GodotScript)]
#[script(base = Node3D)]
struct CarSpawner {
    default_car: Option<Gd<PackedScene>>,

    #[export]
    pub road_network_path: NodePath,
    timer: Option<Gd<Timer>>,

    base: Gd<Node3D>,
}

const CAR_STATION_WAGON_PATH: &str = "res://resources/Objects/Vehicles/car_station_wagon.tscn";

#[godot_script_impl]
impl CarSpawner {
    pub fn _ready(&mut self) {
        self.default_car = Some(load(CAR_STATION_WAGON_PATH));
    }

    pub fn spawn_car(&mut self) {
        if self.base.get_child_count() > 20 {
            return;
        }

        let inst = self
            .default_car
            .as_ref()
            .expect("failed to load default_car")
            .instantiate();

        let Some(mut inst) = inst else {
            godot_error!("failed to instantiate car scene!");
            return;
        };

        inst.set("road_network_path", &self.road_network_path.to_variant());

        self.base
            .add_child_ex(&inst)
            .force_readable_name(true)
            .done();

        let Some(current_scene) = self
            .base
            .get_tree()
            .and_then(|tree| tree.get_current_scene())
        else {
            godot_error!("there is no active scene!");
            return;
        };

        inst.set_owner(&current_scene);
        inst.call("activate", &[]);
    }

    pub fn start_auto_spawn(&mut self) {
        let timer = match self.timer.as_mut() {
            None => {
                let mut timer = Timer::new_alloc();
                self.timer = Some(timer.clone());

                self.base
                    .add_child_ex(&timer)
                    .force_readable_name(true)
                    .done();

                timer.connect("timeout", &script_callable!(self, Self::start_auto_spawn));

                self.timer.as_mut().unwrap()
            }

            Some(timer) => timer,
        };

        timer.start_ex().time_sec(2.0).done();

        self.spawn_car();
    }

    pub fn stop_auto_spawn(&mut self) {
        let Some(timer) = self.timer.as_mut() else {
            return;
        };

        timer.stop();
    }
}
