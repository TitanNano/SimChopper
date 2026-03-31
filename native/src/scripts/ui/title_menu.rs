use godot::builtin::math::ApproxEq;
use godot::builtin::{GString, Vector2, Vector2i};
use godot::classes::input::MouseMode;
use godot::classes::resource_loader::ThreadLoadStatus;
use godot::classes::{
    window, Animation, AnimationPlayer, BaseButton, Button, Control, DisplayServer, Engine,
    InputEvent, Node3D, PackedScene, ResourceLoader,
};
use godot::global;
use godot::meta::ObjectToOwned;
use godot::obj::{EngineEnum, Gd, Singleton as _};
use godot_rust_script::{godot_script_impl, Context, GodotScript, OnEditor, ScriptExportGroup};
use num::ToPrimitive;

use crate::resources::InputDevice;
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

    #[export]
    pub input_device: OnEditor<Gd<InputDevice>>,

    ready: bool,

    base: Gd<Node3D>,
}

#[godot_script_impl]
impl TitleMenu {
    pub fn _ready(&mut self, mut context: Context<'_, Self>) {
        self.apply_ui_scale();

        let mut start_game = self.start_game.clone();

        // Grab the focus of the first menu entry.
        context.reentrant_scope(self, |_base| {
            start_game.grab_focus();
        });

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

        self.ready = true;
        self.input_device
            .bind_mut()
            .set_mouse_mode(MouseMode::VISIBLE);
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
        if !self.ready {
            return;
        }

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

    fn apply_ui_scale(&self) {
        let mut window = self
            .base
            .get_window()
            .expect("tile menu must be attached to a window!");

        if window.get_mode() == window::Mode::WINDOWED
            && !Engine::singleton().is_embedded_in_editor()
        {
            let scale = DisplayServer::singleton()
                .screen_get_scale_ex()
                .screen(DisplayServer::SCREEN_OF_MAIN_WINDOW)
                .done();

            if scale.approx_eq(&1.0) {
                return;
            }

            let window_size = window.get_size();
            let scaled_window_size = Vector2::new(
                window_size.x.to_f32().unwrap(),
                window_size.y.to_f32().unwrap(),
            ) * scale;
            let window_position = window.get_position();

            logger::debug!(
                "content size: {}, window size: {}",
                window.get_content_scale_size(),
                window_size
            );

            window.set_size(Vector2i::new(
                scaled_window_size.x.to_i32().unwrap(),
                scaled_window_size.y.to_i32().unwrap(),
            ));
            window.set_position(window_position - window_size / 2);
        }
    }

    fn _unhandled_input(&mut self, event: Gd<InputEvent>) {
        self.input_device.bind_mut().capture(event);
    }
}

impl ObjectToOwned<Node3D> for TitleMenu {
    fn object_to_owned(&self) -> Gd<Node3D> {
        self.base.clone()
    }
}
