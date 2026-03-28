use godot::builtin::GString;
use godot::classes::resource_loader::ThreadLoadStatus;
use godot::classes::{
    Animation, AnimationPlayer, BaseButton, Button, Control, Node3D, PackedScene, ResourceLoader,
};
use godot::global;
use godot::meta::ObjectToOwned;
use godot::obj::{EngineEnum, Gd, Singleton as _};
use godot_rust_script::{godot_script_impl, Context, GodotScript, OnEditor, ScriptExportGroup};

use crate::script_callable;
use crate::util::logger;

#[derive(ScriptExportGroup, Debug, Default)]
struct AnimationsGroup {
    ui_select: OnEditor<Gd<Animation>>,
    ui_activate: OnEditor<Gd<Animation>>,
}

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
struct TitleMenu {
    #[export]
    pub scene_transitions: OnEditor<Gd<AnimationPlayer>>,

    #[export]
    pub start_game: OnEditor<Gd<Button>>,

    #[export]
    pub quit_game: OnEditor<Gd<Button>>,

    #[export(file = ["*.tscn"])]
    pub main_scene: GString,

    #[export]
    pub ui_sounds: OnEditor<Gd<AnimationPlayer>>,

    #[export(flatten)]
    pub animations: AnimationsGroup,

    base: Gd<Node3D>,
}

#[godot_script_impl]
impl TitleMenu {
    pub fn _ready(&mut self) {
        logger::debug!("connecting button signals!");

        self.quit_game
            .signals()
            .pressed()
            .to_untyped()
            .connect(&script_callable!(self, Self::on_quit));

        self.start_game
            .signals()
            .pressed()
            .to_untyped()
            .connect(&script_callable!(self, Self::on_start_game));
    }

    pub fn on_start_game(&mut self) {
        self.scene_transitions
            .play_ex()
            .name("title_screen_ui/fade_out")
            .done();

        let mut resource_loader = ResourceLoader::singleton();
        let load_err = resource_loader.load_threaded_request(&self.main_scene);

        if load_err != global::Error::OK {
            logger::error!("failed to load main scene: {}", load_err.as_str());
            return;
        }

        let scene_path = self.main_scene.clone();
        let mut tree = self.base.get_tree().unwrap();
        let animation_player = self.scene_transitions.clone();

        godot::task::spawn(async move {
            loop {
                match resource_loader.load_threaded_get_status(&scene_path) {
                    ThreadLoadStatus::IN_PROGRESS | ThreadLoadStatus::LOADED
                        if animation_player.is_playing() =>
                    {
                        animation_player
                            .signals()
                            .animation_finished()
                            .to_future()
                            .await;
                    }
                    ThreadLoadStatus::IN_PROGRESS => {
                        tree.signals().process_frame().to_future().await;
                    }

                    ThreadLoadStatus::LOADED => {
                        let scene: Gd<PackedScene> = resource_loader
                            .load_threaded_get(&scene_path)
                            .unwrap()
                            .cast();

                        tree.change_scene_to_packed(Some(&scene));
                        break;
                    }

                    ThreadLoadStatus::INVALID_RESOURCE => {
                        logger::error!("Scene path {} is an invalid resource!", scene_path);
                        break;
                    }

                    ThreadLoadStatus::FAILED => {
                        logger::error!("Failed to load scene: {}", scene_path);
                        break;
                    }

                    _ => unreachable!(),
                }
            }
        });
    }

    pub fn on_quit(&mut self) {
        let mut tree = self.base.get_tree().unwrap();

        tree.quit();
    }

    pub fn on_ui_hover(&mut self, mut target: Gd<Control>, mut context: Context<'_, Self>) {
        // `grab_focus` will end up calling `on_ui_select`.
        context.reentrant_scope(self, |_base| {
            if target
                .clone()
                .try_cast::<BaseButton>()
                .ok()
                .is_some_and(|node| node.is_disabled())
            {
                return;
            }

            target.grab_focus();
        });
    }

    pub fn on_ui_select(&mut self) {
        self.play_ui_sound(&self.animations.ui_select.clone());
    }

    pub fn on_ui_activate(&mut self) {
        self.play_ui_sound(&self.animations.ui_activate.clone());
    }

    fn play_ui_sound(&mut self, animation: &Gd<Animation>) {
        let animation = animation.get_name();

        if !self.ui_sounds.has_animation(animation.arg()) {
            logger::error!(
                "Animation {} does not belong to Animation Player {}",
                animation,
                self.ui_sounds.get_name()
            );
            return;
        }

        self.ui_sounds.play_ex().name(animation.arg()).done();
    }
}

impl ObjectToOwned<Node3D> for TitleMenu {
    fn object_to_owned(&self) -> Gd<Node3D> {
        self.base.clone()
    }
}
