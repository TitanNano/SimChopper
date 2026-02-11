mod ao_baker;
mod building_imports;
mod gltf;
pub mod ui;

use std::num::NonZero;
use std::ops::DerefMut;

use ao_baker::AoBaker;
use gltf::GltfImporter;
use godot::builtin::{GString, StringName, VarDictionary, Variant, VariantType};
use godot::classes::notify::NodeNotification;
use godot::classes::{
    ConfigFile, EditorPlugin, EditorSettings, GDExtensionManager, GltfDocument, IEditorPlugin,
    ProjectSettings,
};
use godot::global::{self, godot_error, godot_print, PropertyHint};
use godot::meta::{AsArg, FromGodot};
use godot::obj::{Base, Gd, NewGd, OnReady, Singleton as _, WithBaseField};
use godot::register::{godot_api, GodotClass};

use building_imports::SetupBuildingImports;

use crate::engine_callable;
use crate::util::variant_type_default_value;

#[derive(GodotClass)]
#[class(tool, base=EditorPlugin, internal)]
struct EditorExtension {
    setup_building_imports: Gd<SetupBuildingImports>,
    gltf_importer: Gd<GltfImporter>,
    ao_baker: OnReady<Gd<AoBaker>>,

    base: Base<EditorPlugin>,
}

const fn new_non_zero(value: u32) -> NonZero<u32> {
    NonZero::new(value).unwrap()
}

#[godot_api]
impl EditorExtension {
    const EDITOR_SETTING_CARGO_PATH: &str = "run/build/cargo_path";

    fn define_project_settings(
        settings: &[(&'static str, VariantType, PropertyHint, &'static str)],
        project_settings: &mut Gd<ProjectSettings>,
    ) {
        for (name, ty, hint, hint_str) in settings {
            Self::define_settings_property(name, *ty, *hint, hint_str, project_settings);
        }
    }

    fn define_settings_property(
        name: &'static str,
        ty: VariantType,
        hint: PropertyHint,
        hint_str: &'static str,
        project_settings: &mut impl EngineSettings,
    ) {
        let default_value = variant_type_default_value(ty);

        if !project_settings.has_setting(name) {
            project_settings.set_setting(name, &default_value);
        }

        project_settings.set_initial_value(&name.into(), &default_value);

        let mut property = VarDictionary::new();

        property.set("name", name);
        property.set("type", ty);
        property.set("hint", hint);
        property.set("hint_string", hint_str);

        project_settings.add_property_info(&property);
    }

    fn get_editor_setting<T: FromGodot>(&mut self, name: &str) -> T {
        self.base_mut()
            .get_editor_interface()
            .expect("editor interface must be available")
            .get_editor_settings()
            .expect("editor settings must be available")
            .get_setting(name)
            .to()
    }
}

#[godot_api]
impl IEditorPlugin for EditorExtension {
    fn init(base: Base<EditorPlugin>) -> Self {
        Self {
            setup_building_imports: SetupBuildingImports::new(
                base.to_init_gd().get_editor_interface(),
            ),
            gltf_importer: GltfImporter::new_gd(),
            ao_baker: OnReady::manual(),
            base,
        }
    }

    fn enter_tree(&mut self) {
        let building_imports = self.setup_building_imports.clone();

        Self::define_project_settings(&AoBaker::SETTINGS, &mut ProjectSettings::singleton());

        self.base_mut().add_tool_menu_item(
            "Setup Building Imports...",
            &engine_callable!(&building_imports, SetupBuildingImports::start),
        );

        GltfDocument::register_gltf_document_extension(&self.gltf_importer);
    }

    fn ready(&mut self) {
        let editor_interface = self
            .base_mut()
            .get_editor_interface()
            .expect("Editor interface must be available when plugin is ready!");

        let mut editor_settings = editor_interface
            .get_editor_settings()
            .expect("We must have editor settings");

        let ao_baker = {
            let base = self.base();

            AoBaker::new(
                editor_interface.clone(),
                base.get_tree()
                    .expect("SceneTree should be available after entering the tree"),
            )
        };

        self.ao_baker.init(ao_baker);

        let callable = &engine_callable!(&self.ao_baker, AoBaker::bake);

        self.base_mut()
            .add_tool_menu_item("Bake Ambient Occlusion...", callable);

        Self::define_settings_property(
            Self::EDITOR_SETTING_CARGO_PATH,
            VariantType::STRING,
            PropertyHint::GLOBAL_FILE,
            "",
            &mut editor_settings,
        );
    }

    fn on_notification(&mut self, what: NodeNotification) {
        if what == NodeNotification::PREDELETE {
            self.setup_building_imports.clone().free();
        }
    }

    fn exit_tree(&mut self) {
        GltfDocument::unregister_gltf_document_extension(&self.gltf_importer);
    }

    fn build(&mut self) -> bool {
        let cargo_path: GString = self.get_editor_setting(Self::EDITOR_SETTING_CARGO_PATH);

        if cargo_path.is_empty() {
            godot_error!("Editor setting run/build/cargo_path is unset and has to be configured!");
            return false;
        }

        let extension_list = GDExtensionManager::singleton().get_loaded_extensions();

        for path in extension_list.as_slice() {
            let mut config = ConfigFile::new_gd();

            let result = config.load(path);

            if result != global::Error::OK {
                godot_error!("Unable to load gdextension config {}: {:?}", path, result);
                return false;
            }

            let Some(source_dir): Option<GString> =
                config.get_value("build", "source").try_to().ok()
            else {
                continue;
            };

            let output = std::process::Command::new(cargo_path.to_string())
                .arg("build")
                .current_dir(
                    ProjectSettings::singleton()
                        .globalize_path(&source_dir)
                        .to_string(),
                )
                .output()
                .inspect_err(|err| {
                    godot_error!("Failed to run cargo build: {}", err);
                })
                .ok();

            let Some(output) = output else {
                return false;
            };

            String::from_utf8_lossy(&output.stdout)
                .split_terminator('\n')
                .for_each(|line| {
                    godot_print!("{}", line);
                });

            String::from_utf8_lossy(&output.stderr)
                .split_terminator('\n')
                .for_each(|line| {
                    godot_error!("{}", line);
                });

            if !output.status.success() {
                return false;
            }
        }

        true
    }
}

trait EngineSettings {
    fn set_setting(&mut self, name: impl AsArg<GString>, value: &Variant);
    fn has_setting(&mut self, name: impl AsArg<GString>) -> bool;
    fn set_initial_value(&mut self, name: &StringName, value: &Variant);
    fn add_property_info(&mut self, property_info: &VarDictionary);
}

impl EngineSettings for Gd<ProjectSettings> {
    fn set_setting(&mut self, name: impl AsArg<GString>, value: &Variant) {
        self.deref_mut().set_setting(name, value);
    }

    fn has_setting(&mut self, name: impl AsArg<GString>) -> bool {
        self.deref_mut().has_setting(name)
    }

    fn set_initial_value(&mut self, name: &StringName, value: &Variant) {
        self.deref_mut()
            .set_initial_value(&GString::from(name), value);
    }

    fn add_property_info(&mut self, property_info: &VarDictionary) {
        self.deref_mut().add_property_info(property_info);
    }
}

impl EngineSettings for Gd<EditorSettings> {
    fn set_setting(&mut self, name: impl AsArg<GString>, value: &Variant) {
        self.deref_mut().set_setting(name, value);
    }

    fn has_setting(&mut self, name: impl AsArg<GString>) -> bool {
        self.deref_mut().has_setting(name)
    }

    fn set_initial_value(&mut self, name: &StringName, value: &Variant) {
        self.deref_mut().set_initial_value(name, value, false);
    }

    fn add_property_info(&mut self, property_info: &VarDictionary) {
        self.deref_mut().add_property_info(property_info);
    }
}
