use anyhow::{bail, Context};
use godot::builtin::StringName;
use godot::classes::{GpuParticles3D, Node3D};
use godot::obj::Gd;
use godot_rust_script::{godot_script_impl, GodotScript, GodotScriptEnum, OnEditor};

use crate::util::logger;

#[derive(Debug, Default, GodotScriptEnum, Clone, Copy)]
#[script_enum(export)]
pub enum CanonMode {
    #[default]
    Inactive,
    Water,
    Teargas,
}

#[derive(Debug)]
enum CanonAction {
    FirePrimary,
    FireSecondary,
}

impl TryFrom<StringName> for CanonAction {
    type Error = anyhow::Error;

    fn try_from(value: StringName) -> Result<Self, Self::Error> {
        let action = match value.to_string().as_str() {
            "fire_primary" => Self::FirePrimary,
            "fire_secondary" => Self::FireSecondary,
            _ => bail!("Invalid canon action: {}", value),
        };

        Ok(action)
    }
}

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
struct CanonUpgrade {
    #[export]
    #[prop(set = Self::set_mode)]
    pub mode: CanonMode,

    #[export]
    pub water_jet: OnEditor<Gd<GpuParticles3D>>,

    base: Gd<Node3D>,
}

#[godot_script_impl]
impl CanonUpgrade {
    pub fn _ready(&mut self) {
        self.set_mode(self.mode);
    }

    pub fn set_mode(&mut self, value: CanonMode) {
        self.mode = value;

        if !self.base.is_node_ready() {
            return;
        }

        self.water_jet.set_emitting(false);

        match value {
            CanonMode::Inactive => (),
            CanonMode::Water => self.water_jet.set_emitting(true),
            CanonMode::Teargas => (),
        }
    }

    pub fn action_start(&mut self, action: StringName) {
        let action = match CanonAction::try_from(action).context("Failed to parse action string") {
            Ok(action) => action,
            Err(err) => {
                logger::error!("{:?}", err);
                return;
            }
        };

        let current_mode = self.mode;

        match (action, current_mode) {
            (CanonAction::FirePrimary, CanonMode::Inactive) => {
                self.set_mode(CanonMode::Water);
            }

            (CanonAction::FireSecondary, CanonMode::Inactive) => {
                self.set_mode(CanonMode::Teargas);
            }

            _ => (),
        }
    }

    pub fn action_end(&mut self, action: StringName) {
        let action = match CanonAction::try_from(action).context("Failed to parse action string") {
            Ok(action) => action,
            Err(err) => {
                logger::error!("{:?}", err);
                return;
            }
        };

        let current_mode: CanonMode = self.mode;

        match (action, current_mode) {
            (CanonAction::FireSecondary, CanonMode::Teargas)
            | (CanonAction::FirePrimary, CanonMode::Water) => {
                self.set_mode(CanonMode::Inactive);
            }

            _ => (),
        }
    }
}
