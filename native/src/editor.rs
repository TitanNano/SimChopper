mod building_imports;

use godot::builtin::Callable;
use godot::engine::notify::NodeNotification;
use godot::engine::{EditorPlugin, IEditorPlugin};
use godot::obj::{Base, Gd, WithBaseField};
use godot::register::{godot_api, GodotClass};

use building_imports::SetupBuildingImports;

#[derive(GodotClass)]
#[class(editor_plugin, tool, base=EditorPlugin)]
struct EditorExtension {
    setup_building_imports: Gd<SetupBuildingImports>,
    base: Base<EditorPlugin>,
}

#[godot_api]
impl IEditorPlugin for EditorExtension {
    fn init(base: Base<EditorPlugin>) -> Self {
        Self {
            setup_building_imports: SetupBuildingImports::new(base.to_gd().get_editor_interface()),
            base,
        }
    }

    fn enter_tree(&mut self) {
        let building_imports = self.setup_building_imports.clone();

        self.base_mut().add_tool_menu_item(
            "Setup Building Imports...".into(),
            Callable::from_object_method(&building_imports, "start"),
        );
    }

    fn on_notification(&mut self, what: NodeNotification) {
        if what == NodeNotification::PREDELETE {
            self.setup_building_imports.clone().free();
        }
    }
}
