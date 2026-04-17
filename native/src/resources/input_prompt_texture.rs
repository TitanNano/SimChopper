use godot::builtin::{Array, Dictionary, GString, StringName, Variant};
use godot::classes::{
    DpiTexture, Engine, FileAccess, IDpiTexture, InputEvent, InputEventJoypadButton,
    InputEventJoypadMotion, InputEventKey, InputMap, Os, ProjectSettings,
};
use godot::obj::{Base, EngineEnum, Gd, OnEditor, Singleton, WithBaseField, WithUserSignals};
use godot::prelude::{godot_api, GodotClass, Var};
use godot::signal::ConnectHandle;

use crate::resources::input_device::{ControllerType, DeviceType};
use crate::resources::InputDevice;
use crate::util::logger;

#[derive(GodotClass)]
#[class(base = DpiTexture, init, tool)]
pub(crate) struct InputPromptTexture {
    #[export(dir)]
    #[var(set = set_texture_dir)]
    texture_dir: GString,

    #[export]
    #[var(set = set_device)]
    device: OnEditor<Gd<InputDevice>>,

    device_type_change_handle: Option<ConnectHandle>,

    #[export]
    #[var(hint = INPUT_NAME, set = set_action)]
    action: StringName,

    base: Base<DpiTexture>,
}

#[godot_api]
impl InputPromptTexture {
    #[func]
    fn set_texture_dir(&mut self, dir: GString) {
        self.texture_dir = dir;
        self.refresh_prompt_texture();
    }

    #[func]
    fn set_action(&mut self, action: StringName) {
        self.action = action;
        self.refresh_prompt_texture();
    }
    #[func]
    fn set_device(&mut self, mut device: Gd<InputDevice>) {
        if let Some(handle) = self.device_type_change_handle.take() {
            handle.disconnect();
        }

        self.device_type_change_handle = Some(
            device
                .bind_mut()
                .signals()
                .device_type_changed()
                .connect_other(self, Self::on_device_type_changed),
        );

        Var::var_pub_set(&mut self.device, device);
        self.refresh_prompt_texture();
    }
    fn refresh_prompt_texture(&mut self) {
        if Var::var_get(&self.device).is_none() {
            return;
        }

        if self.action.is_empty() {
            return;
        }

        if self.texture_dir.is_empty() {
            return;
        }

        let device_type = self.device.bind().device_type();
        let events = self.get_action_input_events(&self.action);

        let prompt_id = events.iter_shared().find_map(|event| match device_type {
            DeviceType::Controller(ty)
                if event.is_class(&InputEventJoypadButton::class_id().to_gstring()) =>
            {
                let button = event
                    .cast::<InputEventJoypadButton>()
                    .get_button_index()
                    .as_str()
                    .to_lowercase();

                let ty = if matches!(ty, ControllerType::Generic) {
                    ControllerType::Steam
                } else {
                    ty
                };

                Some(format!("{}_{}", ty.as_str(), button))
            }
            DeviceType::Controller(ty)
                if event.is_class(&InputEventJoypadMotion::class_id().to_gstring()) =>
            {
                let event = event.cast::<InputEventJoypadMotion>();
                let input = event.get_axis().as_str().to_lowercase();
                let sign = if event.get_axis_value().is_sign_negative() {
                    "neg"
                } else {
                    "pos"
                };

                let ty = if matches!(ty, ControllerType::Generic) {
                    ControllerType::Steam
                } else {
                    ty
                };

                Some(format!("{}_{input}_{sign}", ty.as_str()))
            }
            DeviceType::KeyboardMouse
                if event.is_class(&InputEventKey::class_id().to_gstring()) =>
            {
                let key = event.cast::<InputEventKey>().get_key_label();

                Some(format!(
                    "{}_{}",
                    "kbm",
                    Os::singleton().get_keycode_string(key).to_lower()
                ))
            }
            DeviceType::KeyboardMouse | DeviceType::Controller(_) => None,
        });

        let Some(prompt_id) = prompt_id else {
            logger::warn!("unable to determine current prompt id!");
            self.base_mut().set_source("");
            return;
        };

        let texture_path = format!("{}/{}.svg", self.texture_dir, prompt_id);

        if !FileAccess::file_exists(&texture_path) {
            logger::warn!("prompt texture {} does not exists!", texture_path);
            self.base_mut().set_source("");
            return;
        }

        let svg = FileAccess::get_file_as_string(&texture_path);

        self.base_mut().run_deferred_gd(move |mut base| {
            base.set_source(&svg);
        });
    }

    fn get_action_input_events(&self, action: &StringName) -> Array<Gd<InputEvent>> {
        if Engine::singleton().is_editor_hint() {
            let action_var = ProjectSettings::singleton().get_setting(&format!("input/{action}"));

            if action_var.is_nil() {
                logger::warn!("action {} does not exist!", action);
                return Array::new();
            }

            let action = match action_var.try_to::<Dictionary<Variant, Variant>>() {
                Ok(action) => action,
                Err(err) => {
                    logger::error!("{}", err);
                    return Array::new();
                }
            };

            let Some(events) = action.get("events") else {
                logger::error!("input action has no events key!");
                return Array::new();
            };

            return events
                .to::<Array<Variant>>()
                .iter_shared()
                .map(|item| item.to::<Gd<InputEvent>>())
                .collect();
        }

        InputMap::singleton().action_get_events(&self.action)
    }

    fn on_device_type_changed(&mut self) {
        self.refresh_prompt_texture();
    }
}

#[godot_api]
impl IDpiTexture for InputPromptTexture {
    fn get_width(&self) -> i32 {
        64
    }

    fn get_height(&self) -> i32 {
        64
    }
}
