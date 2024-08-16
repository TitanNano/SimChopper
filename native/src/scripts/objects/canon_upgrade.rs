use anyhow::{bail, Context};
use godot::{
    builtin::{NodePath, StringName},
    engine::{GpuParticles3D, Node3D},
    obj::Gd,
};
use godot_rust_script::{godot_script_impl, GodotScript};

use crate::util::logger;

#[derive(Debug, Default)]
enum CanonMode {
    #[default]
    Inactive,
    Water,
    Teargas,
}

impl TryFrom<u8> for CanonMode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let result = match value {
            0 => Self::Inactive,
            1 => Self::Water,
            2 => Self::Teargas,
            _ => {
                bail!("Failed to parse cannon mode {}, invalid variant!", value);
            }
        };

        Ok(result)
    }
}

impl From<CanonMode> for u8 {
    fn from(value: CanonMode) -> Self {
        match value {
            CanonMode::Inactive => 0,
            CanonMode::Water => 1,
            CanonMode::Teargas => 2,
        }
    }
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
    #[export(enum_options = ["Inactive", "Water", "Teargas"])]
    #[prop(set = Self::set_mode)]
    pub mode: u8,

    #[export(node_path = ["GPUParticles3D"])]
    #[prop(set = Self::set_water_jet_path)]
    pub water_jet_path: NodePath,

    water_jet: Option<Gd<GpuParticles3D>>,

    base: Gd<Node3D>,
}

#[godot_script_impl]
impl CanonUpgrade {
    pub fn _ready(&mut self) {
        self.water_jet = self.base.try_get_node_as(self.water_jet_path.clone());

        if self.water_jet.is_none() {
            logger::error!("Failed to resolve node path: {}", self.water_jet_path);
        }

        self.set_mode(self.mode);
    }

    pub fn set_water_jet_path(&mut self, value: NodePath) {
        self.water_jet_path = value.clone();

        if !self.base.is_node_ready() {
            return;
        }

        self.water_jet = self.base.try_get_node_as(value.clone());

        if self.water_jet.is_none() {
            logger::error!("Failed to resolve node path: {}", value);
        }
    }

    pub fn set_mode(&mut self, value: u8) {
        self.mode = value;

        if !self.base.is_node_ready() {
            return;
        }

        let variant = CanonMode::try_from(value)
            .context("Failed to parse mode!")
            .unwrap_or_else(|err| {
                logger::error!("{:?}", err);
                CanonMode::default()
            });

        let Some(water_jet) = self.water_jet.as_mut() else {
            logger::error!("Water jet node is not available!");
            return;
        };

        water_jet.set_emitting(false);

        match variant {
            CanonMode::Inactive => (),
            CanonMode::Water => water_jet.set_emitting(true),
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

        let current_mode: CanonMode = match self.mode.try_into().context("Current mode is invalid")
        {
            Ok(mode) => mode,
            Err(err) => {
                logger::error!("{:?}", err);
                return;
            }
        };

        match (action, current_mode) {
            (CanonAction::FirePrimary, CanonMode::Inactive) => {
                self.set_mode(CanonMode::Water.into());
            }

            (CanonAction::FireSecondary, CanonMode::Inactive) => {
                self.set_mode(CanonMode::Teargas.into());
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

        let current_mode: CanonMode = match self.mode.try_into().context("Current mode is invalid")
        {
            Ok(mode) => mode,
            Err(err) => {
                logger::error!("{:?}", err);
                return;
            }
        };

        match (action, current_mode) {
            (CanonAction::FireSecondary, CanonMode::Teargas)
            | (CanonAction::FirePrimary, CanonMode::Water) => {
                self.set_mode(CanonMode::Inactive.into());
            }

            _ => (),
        }
    }
}
