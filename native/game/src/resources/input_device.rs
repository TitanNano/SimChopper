use godot::builtin::{Callable, StringName};
use godot::classes::input::MouseMode;
use godot::classes::{
    match_class, Engine, IResource, Input, InputEvent, InputEventJoypadButton,
    InputEventJoypadMotion, Resource,
};
use godot::meta::conv::ByValue;
use godot::meta::{FromGodot, GodotConvert, ToGodot};
use godot::obj::{Base, Gd, Singleton, WithUserSignals};
use godot::prelude::{godot_api, ConvertError, GodotClass};
use godot::register::property::SimpleVar;

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

#[derive(Default)]
enum DeviceType {
    #[default]
    KeyboardMouse,
    Controller,
}

#[derive(GodotClass)]
#[class( base = Resource)]
pub(crate) struct InputDevice {
    #[export]
    device_id: i32,

    device_type: DeviceType,
    mouse_mode: MouseMode,

    seperate_climp_axis: bool,

    climb: InputAxis,
    movement: InputAxis,
    strafe: InputAxis,
    turn: InputAxis,

    fire_primary_state: bool,
    fire_secondary_state: bool,

    base: Base<Resource>,
}

#[godot_api]
impl IResource for InputDevice {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            device_id: 0,
            device_type: DeviceType::default(),
            mouse_mode: MouseMode::VISIBLE,
            seperate_climp_axis: true,
            climb: InputAxis::default(),
            movement: InputAxis::default(),
            strafe: InputAxis::default(),
            turn: InputAxis::default(),
            base,
            fire_primary_state: false,
            fire_secondary_state: false,
        }
    }
}

#[godot_api]
impl InputDevice {
    #[signal]
    fn fire_primary(pressed: bool);

    #[signal]
    fn fire_secondary(pressed: bool);

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
    fn strafe_strength(&self) -> f32 {
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
    fn movement_strength(&self) -> f32 {
        self.movement.get()
    }

    #[func]
    fn turn_strength(&self) -> f32 {
        self.turn.get()
    }

    #[func]
    #[expect(clippy::needless_pass_by_value)]
    pub fn capture(&mut self, event: Gd<InputEvent>) {
        if event.get_device() != self.device_id {
            return;
        }

        self.device_type = match_class! { event.clone(),
            _ @ InputEventJoypadButton => DeviceType::Controller,
            _ @ InputEventJoypadMotion => DeviceType::Controller,
            _ => DeviceType::KeyboardMouse,
        };

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

        if !Engine::singleton().is_embedded_in_editor() {
            match self.device_type {
                DeviceType::KeyboardMouse => Input::singleton().set_mouse_mode(self.mouse_mode),
                DeviceType::Controller => Input::singleton().set_mouse_mode(MouseMode::HIDDEN),
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
        }
    }

    #[func]
    pub fn set_mouse_mode(&mut self, mode: MouseMode) {
        self.mouse_mode = mode;
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
    // Buttons
    FirePrimary,
    FireSecondary,
}

impl ButtonAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::FirePrimary => "fire_primary",
            Self::FireSecondary => "fire_secondary",
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
            _ => return Err(ConvertError::new("unknown action type")),
        };

        Ok(parsed)
    }
}
