use godot::obj::NewAlloc;
use godot_rust_script::{
    godot::{
        engine::{ResourceLoader, Timer},
        prelude::{
            godot_error, Callable, GString, Gd, Node3D, NodePath, PackedScene, StringName, ToGodot,
        },
    },
    godot_script_impl, GodotScript,
};

#[derive(Debug, GodotScript)]
#[script(base = Node3D)]
struct CarSpawner {
    default_car: Option<Gd<PackedScene>>,

    #[export]
    pub road_network_path: NodePath,
    timer: Option<Gd<Timer>>,

    base: Gd<Node3D>,
}

#[godot_script_impl]
impl CarSpawner {
    pub fn _init(&mut self) {
        let mut loader = ResourceLoader::singleton();

        self.default_car = loader
            .load(GString::from(
                "res://resources/Objects/Vehicles/car_station_wagon.tscn",
            ))
            .map(|res| res.cast());
    }

    pub fn spawn_car(&mut self) {
        let inst = self
            .default_car
            .as_ref()
            .expect("failed to load default_car")
            .instantiate();

        let Some(mut inst) = inst else {
            godot_error!("failed to instantiate car scene!");
            return;
        };

        inst.set(
            StringName::from("road_network_path"),
            self.road_network_path.to_variant(),
        );

        if let Some(mut parent) = self.base.get_parent() {
            parent.add_child(inst.clone());
        }

        let Some(current_scene) = self
            .base
            .get_tree()
            .and_then(|tree| tree.get_current_scene())
        else {
            godot_error!("there is no active scene!");
            return;
        };

        inst.set_owner(current_scene);

        let mut inst: Gd<Node3D> = inst.cast();

        inst.global_translate(self.base.get_global_transform().origin);
        inst.call(StringName::from("activate"), &[]);
    }

    pub fn start_auto_spawn(&mut self) {
        let timer = match self.timer.as_mut() {
            None => {
                let mut timer = Timer::new_alloc();
                self.timer = Some(timer.clone());

                self.base
                    .add_child_ex(timer.clone().upcast())
                    .force_readable_name(true)
                    .done();

                timer.connect(
                    StringName::from("timeout"),
                    Callable::from_object_method(&self.base, "start_auto_spawn"),
                );

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
