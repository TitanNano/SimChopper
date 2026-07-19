use std::backtrace::Backtrace;
use std::borrow::Cow;
use std::collections::HashMap;

use godot::builtin::{Array, Dictionary, GString, StringName, Variant};
use godot::classes::{
    DisplayServer, DpiTexture, FileAccess, IDpiTexture, InputEvent, InputEventJoypadButton,
    InputEventJoypadMotion, InputEventKey, InputMap, Os, ProjectSettings,
};
use godot::global::Key;
use godot::obj::{Base, EngineEnum, Gd, OnEditor, Singleton, WithBaseField};
use godot::prelude::{godot_api, vslice, GodotClass, Var};
use godot::signal::ConnectHandle;
use svgm_core::ast::{Attribute, Document, Element, NodeKind};

use crate::resources::input_device::{ControllerType, DeviceType};
use crate::resources::InputDevice;
use crate::util::{logger, run_on_main};

enum InputTexture {
    Name(String),
    Composed(Vec<Cow<'static, str>>),
}

impl InputTexture {
    fn to_source(&self, texture_dir: &GString) -> Option<GString> {
        match self {
            InputTexture::Name(name) => {
                let texture_path = format!("{texture_dir}/{name}.svg");

                if !FileAccess::file_exists(&texture_path) {
                    logger::warn!("prompt texture {} does not exists!", texture_path);
                    return None;
                }

                let svg = FileAccess::get_file_as_string(&texture_path);

                Some(svg)
            }
            InputTexture::Composed(cows) => {
                let mut composed = Document::new();
                let mut x_offset = 0u32;
                let mut y_height = 0u32;

                let new_root_id = composed.alloc(NodeKind::Element(Element {
                    name: "svg".into(),
                    prefix: None,
                    attributes: Vec::new(),
                    namespaces: Vec::new(),
                }));

                composed.append_child(composed.root, new_root_id);

                for svg_item in cows {
                    let texture_path = format!("{texture_dir}/{svg_item}.svg");

                    if !FileAccess::file_exists(&texture_path) {
                        logger::warn!("prompt texture {} does not exists!", texture_path);
                        return None;
                    }

                    let svg: String = FileAccess::get_file_as_string(&texture_path).into();

                    let doc = svgm_core::parser::parse(&svg).unwrap_or_else(|err| {
                        panic!("unable to parse svg string {svg_item:?}: {err}")
                    });
                    let mut id_map = HashMap::with_capacity(doc.nodes.len());

                    for node_id in doc.traverse() {
                        let node = doc.node(node_id);

                        match &node.kind {
                            NodeKind::Root => {
                                id_map.insert(node_id, new_root_id);
                            }
                            NodeKind::Element(element) => {
                                let mut new_element = Element {
                                    name: element.name.clone(),
                                    prefix: element.prefix.clone(),
                                    attributes: element.attributes.clone(),
                                    namespaces: element.namespaces.clone(),
                                };

                                if new_element.qualified_name() == "svg" {
                                    new_element.name = String::from("g");
                                    new_element.attributes.push(Attribute {
                                        prefix: None,
                                        name: String::from("transform"),
                                        value: format!("translate({x_offset})"),
                                    });
                                    x_offset += new_element
                                        .attr("width")
                                        .map(|width| width.parse::<u32>().unwrap_or_default())
                                        .unwrap_or_default();

                                    y_height = y_height.max(
                                        new_element
                                            .attr("height")
                                            .map(|height| height.parse::<u32>().unwrap_or_default())
                                            .unwrap_or_default(),
                                    );
                                }

                                let new_node_id = composed.alloc(NodeKind::Element(new_element));

                                id_map.insert(node_id, new_node_id);

                                if let Some(parent_id) = node.parent {
                                    let new_parent_id = id_map.get(&parent_id).unwrap();

                                    composed.append_child(*new_parent_id, new_node_id);
                                }
                            }
                            NodeKind::Text(_)
                            | NodeKind::Comment(_)
                            | NodeKind::CData(_)
                            | NodeKind::ProcessingInstruction { .. }
                            | NodeKind::Doctype(_) => (),
                        }
                    }
                }

                let NodeKind::Element(element) = &mut composed.node_mut(new_root_id).kind else {
                    unreachable!();
                };

                element.attributes.push(Attribute {
                    prefix: None,
                    name: "width".into(),
                    value: x_offset.to_string(),
                });

                element.attributes.push(Attribute {
                    prefix: None,
                    name: "height".into(),
                    value: y_height.to_string(),
                });

                let composed_ser = svgm_core::serializer::serialize(&composed);
                let composed_optimized = svgm_core::optimize(&composed_ser).unwrap().data;

                Some(GString::from(&composed_optimized))
            }
        }
    }
}

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

    is_registered: bool,

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

        if godot::init::is_main_thread() {
            self.refresh_prompt_texture();
        }
    }

    #[func]
    fn set_action(&mut self, action: StringName) {
        self.action = action;

        if godot::init::is_main_thread() {
            self.refresh_prompt_texture();
        }
    }
    #[func]
    fn set_device(&mut self, device: Gd<InputDevice>) {
        Var::var_pub_set(&mut self.device, device);

        if godot::init::is_main_thread() {
            self.register();
            self.is_registered = true;
        }

        if !self.is_registered {
            let id = self.base().instance_id();

            run_on_main(move || {
                let mut gd = Gd::<Self>::from_instance_id(id);

                gd.bind_mut().register();
            });
            self.is_registered = true;
        }
    }

    /// Should only be called from the main thread.
    fn subscribe_device_type_change(&mut self) {
        if let Some(handle) = self.device_type_change_handle.take() {
            handle.disconnect();
        }

        self.device_type_change_handle = Some(
            self.device
                .signals()
                .device_type_changed()
                .connect_other(self, Self::on_device_type_changed),
        );
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

                Some(InputTexture::Name(format!("{}_{}", ty.as_str(), button)))
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
                    ControllerType::Xbox
                } else {
                    ty
                };

                Some(InputTexture::Name(format!(
                    "{}_{input}_{sign}",
                    ty.as_str()
                )))
            }
            DeviceType::KeyboardMouse
                if event.is_class(&InputEventKey::class_id().to_gstring()) =>
            {
                let event = event.cast::<InputEventKey>();
                let mut key = event.get_keycode();

                if key == Key::NONE {
                    key = DisplayServer::singleton()
                        .keyboard_get_keycode_from_physical(event.get_physical_keycode());
                }

                if event.is_alt_pressed()
                    || event.is_ctrl_pressed()
                    || event.is_shift_pressed()
                    || event.is_meta_pressed()
                {
                    let mut textures = Vec::new();

                    if event.is_ctrl_pressed() {
                        textures.push("kbm_ctrl".into());
                    }

                    if event.is_meta_pressed() {
                        textures.push("kbm_meta".into());
                    }

                    if event.is_alt_pressed() {
                        textures.push("kbm_alt".into());
                    }

                    if event.is_shift_pressed() {
                        textures.push("kbm_shift".into());
                    }

                    textures.push(Cow::Owned(format!(
                        "kbm_{}",
                        Os::singleton().get_keycode_string(key).to_lower()
                    )));

                    return Some(InputTexture::Composed(textures));
                }

                Some(InputTexture::Name(format!(
                    "{}_{}",
                    "kbm",
                    Os::singleton().get_keycode_string(key).to_lower()
                )))
            }
            DeviceType::KeyboardMouse | DeviceType::Controller(_) => None,
        });

        let Some(tex) = prompt_id else {
            logger::warn!("unable to determine current prompt texture!");
            self.base_mut().call_deferred("set_source", vslice![""]);
            return;
        };

        let Some(svg) = tex.to_source(&self.texture_dir) else {
            self.base_mut().call_deferred("set_source", vslice![""]);
            return;
        };

        // Call to set_source would be reentrant.
        self.base_mut().set_source(&svg);
        // self.base_mut().call_deferred("set_source", vslice![svg]);
    }

    fn get_action_input_events(&self, action: &StringName) -> Array<Gd<InputEvent>> {
        if godot::init::is_editor_hint() {
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
        if !godot::init::is_main_thread() {
            logger::error!(
                "InputPromptTexture::on_device_type_changed must be called from main-thread!\n{}",
                Backtrace::force_capture()
            );
            return;
        }

        self.refresh_prompt_texture();
    }

    pub fn register(&mut self) {
        self.subscribe_device_type_change();
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
