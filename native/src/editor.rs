mod ao_baker;
mod building_imports;
mod gltf;
pub mod ui;

use std::num::NonZero;

use ao_baker::AoBaker;
use gltf::GltfImporter;
use godot::builtin::{Dictionary, VariantType};
use godot::classes::notify::NodeNotification;
use godot::classes::{EditorPlugin, GltfDocument, IEditorPlugin, ProjectSettings};
use godot::global::PropertyHint;
use godot::obj::{Base, Gd, NewGd, OnReady, WithBaseField};
use godot::register::{godot_api, GodotClass};

use building_imports::SetupBuildingImports;

use crate::engine_callable;
use crate::util::variant_type_default_value;

#[derive(GodotClass)]
#[class(tool, base=EditorPlugin)]
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
    fn define_project_settings(
        settings: &[(&'static str, VariantType, PropertyHint, &'static str)],
        project_settings: &mut Gd<ProjectSettings>,
    ) {
        for (name, ty, hint, hint_str) in settings {
            Self::define_project_settings_property(name, *ty, *hint, hint_str, project_settings);
        }
    }

    fn define_project_settings_property(
        name: &'static str,
        ty: VariantType,
        hint: PropertyHint,
        hint_str: &'static str,
        project_settings: &mut Gd<ProjectSettings>,
    ) {
        let default_value = variant_type_default_value(ty);

        if !project_settings.has_setting(name) {
            project_settings.set_setting(name, &default_value);
        }

        project_settings.set_initial_value(name, &default_value);

        let mut property = Dictionary::new();

        property.set("name", name);
        property.set("type", ty);
        property.set("hint", hint);
        property.set("hint_string", hint_str);

        project_settings.add_property_info(&property);
    }
}

#[godot_api]
impl IEditorPlugin for EditorExtension {
    fn init(base: Base<EditorPlugin>) -> Self {
        Self {
            setup_building_imports: SetupBuildingImports::new(base.to_gd().get_editor_interface()),
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

        let ao_baker = {
            let mut base = self.base_mut();

            AoBaker::new(
                base.get_editor_interface()
                    .expect("editor interface should be available after entering the tree"),
                base.get_tree()
                    .expect("SceneTree should be available after entering the tree"),
            )
        };

        self.ao_baker.init(ao_baker);

        let callable = &engine_callable!(&self.ao_baker, AoBaker::bake);

        self.base_mut()
            .add_tool_menu_item("Bake Ambient Occlusion...", callable);

        GltfDocument::register_gltf_document_extension(&self.gltf_importer);
    }

    fn on_notification(&mut self, what: NodeNotification) {
        if what == NodeNotification::PREDELETE {
            self.setup_building_imports.clone().free();
        }
    }

    fn exit_tree(&mut self) {
        GltfDocument::unregister_gltf_document_extension(&self.gltf_importer);
    }
}
