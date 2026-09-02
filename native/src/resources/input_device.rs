use std::borrow::Cow;

use godot::builtin::{Callable, GString, StringName};
use godot::classes::class_macros::sys::VariantType;
use godot::classes::input::MouseMode;
use godot::classes::notify::ObjectNotification;
use godot::classes::{
    match_class, Engine, IResource, Input, InputEvent, InputEventJoypadButton,
    InputEventJoypadMotion, Resource,
};
use godot::init::is_editor_hint;
use godot::meta::conv::ByValue;
use godot::meta::shape::EnumeratorShape;
use godot::meta::{FromGodot, GodotConvert, ToGodot};
use godot::obj::{Base, Gd, Singleton, WithUserSignals};
use godot::prelude::{godot_api, ConvertError, Export, GodotClass};
use godot::register::property::SimpleVar;

use crate::util::logger;

macro_rules! input_axis {
    ($event:ident, $field:expr, $neg:expr, $pos:expr) => {
        if $event.is_action($neg.as_str()) {
            $field.negative = $event
                .get_action_strength_ex($neg.as_str())
                .exact_match(true)
                .done();
        }

        if $event.is_action($pos.as_str()) {
            $field.positive = $event
                .get_action_strength_ex($pos.as_str())
                .exact_match(true)
                .done();
        }
    };
}

macro_rules! input_button {
    ($event:ident, $action:expr, $self:expr => ($signal:ident, $state:ident)) => {
        if $event.is_action($action.as_str()) {
            let is_pressed = $event.is_pressed();
            let diff = $self.$state != is_pressed;
            $self.$state = is_pressed;

            if diff {
                $self.signals().$signal().emit(is_pressed);
            }
        }
    };
}

#[derive(Default)]
struct InputAxis {
    negative: f32,
    positive: f32,
}

impl InputAxis {
    fn get(&self) -> f32 {
        self.positive - self.negative
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DeviceType {
    #[default]
    KeyboardMouse,
    Controller(ControllerType),
}

const DEVICE_TYPE_VARIANTS: &[EnumeratorShape] = &[
    EnumeratorShape::new_string("keyboard_mouse"),
    EnumeratorShape::new_string("controller_play_station"),
    EnumeratorShape::new_string("controller_xbox"),
    EnumeratorShape::new_string("controller_steam"),
    EnumeratorShape::new_string("controller_steam_deck"),
    EnumeratorShape::new_string("controller_switch"),
    EnumeratorShape::new_string("controller_switch_2"),
    EnumeratorShape::new_string("controller_wii"),
    EnumeratorShape::new_string("controller_wii_u"),
    EnumeratorShape::new_string("controller_gamecube"),
    EnumeratorShape::new_string("controller_generic"),
];

impl GodotConvert for DeviceType {
    type Via = GString;

    fn godot_shape() -> godot::meta::shape::GodotShape {
        godot::meta::shape::GodotShape::Enum {
            variant_type: VariantType::STRING,
            enumerators: Cow::Borrowed(DEVICE_TYPE_VARIANTS),
            godot_name: None,
            is_bitfield: false,
        }
    }
}

impl ToGodot for DeviceType {
    type Pass = ByValue;

    fn to_godot(&self) -> godot::meta::ToArg<'_, Self::Via, Self::Pass> {
        let str = match self {
            DeviceType::KeyboardMouse => "keyboard_mouse",
            DeviceType::Controller(ControllerType::PlayStation) => "controller_play_station",
            DeviceType::Controller(ControllerType::Xbox) => "controller_xbox",
            DeviceType::Controller(ControllerType::Steam) => "controller_steam",
            DeviceType::Controller(ControllerType::SteamDeck) => "controller_steam_deck",
            DeviceType::Controller(ControllerType::NintendoSwitch) => "controller_switch",
            DeviceType::Controller(ControllerType::NintendoSwitch2) => "controller_switch_2",
            DeviceType::Controller(ControllerType::NintendoWii) => "controller_wii",
            DeviceType::Controller(ControllerType::NintendoWiiU) => "controller_wii_u",
            DeviceType::Controller(ControllerType::NintendoGamecube) => "controller_gamecube",
            DeviceType::Controller(ControllerType::Generic) => "controller_generic",
        };

        GString::from(str)
    }
}

impl FromGodot for DeviceType {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        match &*via.to_string() {
            "keyboard_mouse" => Ok(DeviceType::KeyboardMouse),
            "controller_play_station" => Ok(DeviceType::Controller(ControllerType::PlayStation)),
            "controller_xbox" => Ok(DeviceType::Controller(ControllerType::Xbox)),
            "controller_steam" => Ok(DeviceType::Controller(ControllerType::Steam)),
            "controller_steam_deck" => Ok(DeviceType::Controller(ControllerType::SteamDeck)),
            "controller_switch" => Ok(DeviceType::Controller(ControllerType::NintendoSwitch)),
            "controller_switch_2" => Ok(DeviceType::Controller(ControllerType::NintendoSwitch2)),
            "controller_wii" => Ok(DeviceType::Controller(ControllerType::NintendoWii)),
            "controller_wii_u" => Ok(DeviceType::Controller(ControllerType::NintendoWiiU)),
            "controller_gamecube" => Ok(DeviceType::Controller(ControllerType::NintendoGamecube)),
            "controller_generic" => Ok(DeviceType::Controller(ControllerType::Generic)),
            _ => Err(ConvertError::new("unknown input device type")),
        }
    }
}

impl SimpleVar for DeviceType {}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ControllerType {
    PlayStation,
    Xbox,
    Steam,
    SteamDeck,
    NintendoSwitch,
    NintendoSwitch2,
    NintendoWii,
    NintendoWiiU,
    NintendoGamecube,
    Generic,
}

impl ControllerType {
    /// Controller name mapping based on the SDL game controller db.
    ///
    /// <https://github.com/mdqinc/SDL_GameControllerDB/blob/master/gamecontrollerdb.txt>
    fn from_device_id(device_id: i32) -> Self {
        let name = Input::singleton().get_joy_name(device_id).to_string();

        if name.contains("PS4 Controller") || name.contains("PS3 Controller") {
            Self::PlayStation
        } else if name.contains("Xbox") {
            Self::Xbox
        } else if name.contains("Steam Controller") {
            Self::Steam
        } else if name.contains("Steam Deck") {
            Self::SteamDeck
        } else if name.contains("Nintendo Switch 2") {
            Self::NintendoSwitch2
        } else if name.contains("Nintendo Switch") {
            Self::NintendoSwitch
        } else if name.contains("Nintendo Wii U") {
            Self::NintendoWiiU
        } else if name.contains("Nintendo Wii") {
            Self::NintendoWii
        } else if name.contains("Nintendo Gamecube") {
            Self::NintendoGamecube
        } else {
            Self::Generic
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ControllerType::PlayStation => "play_station",
            ControllerType::Xbox => "xbox",
            ControllerType::Steam => "steam",
            ControllerType::SteamDeck => "steam_deck",
            ControllerType::NintendoSwitch => "nintendo_switch",
            ControllerType::NintendoSwitch2 => "nintendo_switch_2",
            ControllerType::NintendoWii => "nintendo_wii",
            ControllerType::NintendoWiiU => "nintendo_wii_u",
            ControllerType::NintendoGamecube => "nintendo_gamecube",
            ControllerType::Generic => "generic",
        }
    }
}

impl Export for DeviceType {}

#[derive(GodotClass)]
#[class( base = Resource, tool)]
#[expect(clippy::struct_excessive_bools)]
pub(crate) struct InputDevice {
    #[export]
    pub device_id: i32,

    device_type: DeviceType,

    #[export]
    #[var(set)]
    editor_device_type: DeviceType,
    mouse_mode: MouseMode,

    seperate_climp_axis: bool,

    climb: InputAxis,
    movement: InputAxis,
    strafe: InputAxis,
    turn: InputAxis,

    fire_primary_state: bool,
    fire_secondary_state: bool,
    ui_tab_prev_state: bool,
    ui_tab_next_state: bool,

    base: Base<Resource>,
}

#[godot_api]
impl IResource for InputDevice {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            device_id: 0,
            device_type: DeviceType::default(),
            editor_device_type: DeviceType::KeyboardMouse,
            mouse_mode: MouseMode::VISIBLE,
            seperate_climp_axis: true,
            climb: InputAxis::default(),
            movement: InputAxis::default(),
            strafe: InputAxis::default(),
            turn: InputAxis::default(),
            base,
            fire_primary_state: false,
            fire_secondary_state: false,
            ui_tab_prev_state: false,
            ui_tab_next_state: false,
        }
    }

    fn on_notification(&mut self, notification: ObjectNotification) {
        match notification {
            ObjectNotification::POSTINITIALIZE => logger::debug!("input device post init!"),
            ObjectNotification::EXTENSION_RELOADED => {
                logger::debug!("input device hot-reload complete!");
            }
            _ => (),
        }
    }
}

#[godot_api]
impl InputDevice {
    /// [`DeviceType`] of the input device has changed.
    ///
    /// Either due to input on a different device with the same ID (Keyboard to Controller switch) or because the editor override changed.
    #[signal]
    pub fn device_type_changed();

    #[signal]
    fn fire_primary(pressed: bool);

    #[signal]
    fn fire_secondary(pressed: bool);

    #[signal]
    pub fn ui_tab_prev(pressed: bool);

    #[signal]
    pub fn ui_tab_next(pressed: bool);

    #[func]
    fn climb_strength(&self) -> f32 {
        let climb_strength = self.climb.get();

        if self.seperate_climp_axis {
            let strafe_strength = self.strafe.get();

            if strafe_strength.abs() > climb_strength.abs() {
                return 0.0;
            }
        }

        climb_strength
    }

    #[func]
    pub fn strafe_strength(&self) -> f32 {
        let strafe_strength = self.strafe.get();

        if self.seperate_climp_axis {
            let climb_strength = self.climb.get();

            if climb_strength.abs() > strafe_strength.abs() {
                return 0.0;
            }
        }

        strafe_strength
    }

    #[func]
    pub fn movement_strength(&self) -> f32 {
        self.movement.get()
    }

    #[func]
    fn turn_strength(&self) -> f32 {
        self.turn.get()
    }

    #[func]
    #[expect(clippy::needless_pass_by_value)]
    pub fn capture(&mut self, event: Gd<InputEvent>) {
        let device_id = event.get_device();

        if device_id != self.device_id {
            return;
        }

        let current_device_type = self.device_type;

        self.device_type = match_class! { event.clone(),
            _ @ InputEventJoypadButton => DeviceType::Controller(ControllerType::from_device_id(device_id)),
            _ @ InputEventJoypadMotion => DeviceType::Controller(ControllerType::from_device_id(device_id)),
            _ => DeviceType::KeyboardMouse,
        };

        if current_device_type != self.device_type {
            self.signals().device_type_changed().emit();
        }

        input_axis!(event, self.climb, AxisAction::Land, AxisAction::Rise);
        input_axis!(event, self.movement, AxisAction::Forward, AxisAction::Back);
        input_axis!(
            event,
            self.strafe,
            AxisAction::StrafeLeft,
            AxisAction::StrafeRight
        );
        input_axis!(
            event,
            self.turn,
            AxisAction::TurnRight,
            AxisAction::TurnLeft
        );

        input_button!(event, ButtonAction::FirePrimary, self => (fire_primary, fire_primary_state));
        input_button!(event, ButtonAction::FireSecondary, self => (fire_secondary, fire_secondary_state));
        input_button!(event, ButtonAction::UiTabPrev, self => (ui_tab_prev, ui_tab_prev_state));
        input_button!(event, ButtonAction::UiTabNext, self => (ui_tab_next, ui_tab_next_state));

        if !Engine::singleton().is_embedded_in_editor() {
            match self.device_type {
                DeviceType::KeyboardMouse => Input::singleton().set_mouse_mode(self.mouse_mode),
                DeviceType::Controller(_) => Input::singleton().set_mouse_mode(MouseMode::HIDDEN),
            }
        }
    }

    #[func]
    #[expect(clippy::needless_pass_by_value)]
    fn subscribe(&mut self, action_type: ButtonAction, handler: Callable) -> godot::global::Error {
        match action_type {
            ButtonAction::FirePrimary => {
                self.signals().fire_primary().to_untyped().connect(&handler)
            }
            ButtonAction::FireSecondary => self
                .signals()
                .fire_secondary()
                .to_untyped()
                .connect(&handler),

            ButtonAction::UiTabPrev => self.signals().ui_tab_prev().to_untyped().connect(&handler),
            ButtonAction::UiTabNext => self.signals().ui_tab_next().to_untyped().connect(&handler),
        }
    }

    #[func]
    #[expect(clippy::needless_pass_by_value)]
    fn unsubscribe(&mut self, action_type: ButtonAction, handler: Callable) {
        match action_type {
            ButtonAction::FirePrimary => self
                .signals()
                .fire_primary()
                .to_untyped()
                .disconnect(&handler),
            ButtonAction::FireSecondary => self
                .signals()
                .fire_secondary()
                .to_untyped()
                .disconnect(&handler),
            ButtonAction::UiTabPrev => self
                .signals()
                .ui_tab_prev()
                .to_untyped()
                .disconnect(&handler),
            ButtonAction::UiTabNext => self
                .signals()
                .ui_tab_next()
                .to_untyped()
                .disconnect(&handler),
        }
    }

    #[func]
    pub fn set_mouse_mode(&mut self, mode: MouseMode) {
        self.mouse_mode = mode;
    }

    pub fn device_type(&self) -> DeviceType {
        if is_editor_hint() {
            self.editor_device_type
        } else {
            self.device_type
        }
    }

    #[func]
    fn set_editor_device_type(&mut self, device_type: DeviceType) {
        let previous_device_type = self.editor_device_type;

        self.editor_device_type = device_type;

        if device_type != previous_device_type {
            self.signals().device_type_changed().emit();
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum AxisAction {
    // Axes
    Forward,
    Back,
    StrafeLeft,
    StrafeRight,
    TurnLeft,
    TurnRight,
    Rise,
    Land,
}

impl AxisAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Forward => "forward",
            Self::Back => "back",
            Self::StrafeLeft => "strafe_left",
            Self::StrafeRight => "strafe_right",
            Self::TurnLeft => "turn_left",
            Self::TurnRight => "turn_right",
            Self::Rise => "rise",
            Self::Land => "land",
        }
    }
}

impl GodotConvert for AxisAction {
    type Via = StringName;

    fn godot_shape() -> godot::meta::shape::GodotShape {
        Self::Via::godot_shape()
    }
}

impl SimpleVar for AxisAction {}
impl ToGodot for AxisAction {
    type Pass = ByValue;

    fn to_godot(&self) -> godot::meta::ToArg<'_, Self::Via, Self::Pass> {
        StringName::from(self.as_str())
    }
}

impl FromGodot for AxisAction {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        let parsed = match via.to_string().as_str() {
            "forward" => Self::Forward,
            "back" => Self::Back,
            "strafe_left" => Self::StrafeLeft,
            "strafe_right" => Self::StrafeRight,
            "turn_left" => Self::TurnLeft,
            "turn_right" => Self::TurnRight,
            "rise" => Self::Rise,
            "land" => Self::Land,
            _ => return Err(ConvertError::new("unknown action type")),
        };

        Ok(parsed)
    }
}

#[derive(Clone, Copy, Debug)]
enum ButtonAction {
    FirePrimary,
    FireSecondary,
    UiTabPrev,
    UiTabNext,
}

impl ButtonAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::FirePrimary => "fire_primary",
            Self::FireSecondary => "fire_secondary",
            Self::UiTabPrev => "ui_tab_prev",
            Self::UiTabNext => "ui_tab_next",
        }
    }
}

impl GodotConvert for ButtonAction {
    type Via = StringName;

    fn godot_shape() -> godot::meta::shape::GodotShape {
        Self::Via::godot_shape()
    }
}

impl FromGodot for ButtonAction {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        let parsed = match via.to_string().as_str() {
            "fire_primary" => Self::FirePrimary,
            "fire_secondary" => Self::FireSecondary,
            "ui_tab_prev" => Self::UiTabPrev,
            "ui_tab_next" => Self::UiTabNext,
            _ => return Err(ConvertError::new("unknown action type")),
        };

        Ok(parsed)
    }
}
