use std::collections::VecDeque;

use godot::{
    builtin::{Dictionary, GString, Vector2i},
    classes::{
        editor_file_dialog, ConfigFile, DirAccess, EditorFileDialog, EditorInterface, FileAccess,
        MeshInstance3D, Object, PackedScene, ResourceLoader,
    },
    global::Error,
    meta::ToGodot,
    obj::{Base, Gd, GodotClass, NewAlloc, NewGd, WithBaseField},
    register::{godot_api, GodotClass},
};
use pomsky_macro::pomsky;
use regex::Regex;

use crate::{class_callable, util::logger};

#[derive(GodotClass)]
#[class(base = Object, init, tool)]
pub(crate) struct SetupBuildingImports {
    #[var]
    editor: Option<Gd<EditorInterface>>,
    base: Base<Object>,
}

#[godot_api]
impl SetupBuildingImports {
    pub fn new(editor: Option<Gd<EditorInterface>>) -> Gd<Self> {
        Gd::from_init_fn(|base| Self { editor, base })
    }

    #[func]
    pub fn start(&mut self) {
        let Some(editor) = self.editor.as_ref() else {
            logger::error!("Editor is unavailable!");
            return;
        };

        let mut dialog = EditorFileDialog::new_alloc();

        dialog.set_current_dir("res://");
        dialog.set_file_mode(editor_file_dialog::FileMode::OPEN_ANY);
        dialog.set_access(editor_file_dialog::Access::RESOURCES);
        dialog.set_hide_on_ok(true);

        dialog.connect(
            "file_selected",
            &class_callable!(self, Self::on_file_selected),
        );

        dialog.connect(
            "dir_selected",
            &class_callable!(self, Self::on_dir_selected),
        );

        let Some(ui) = editor.get_base_control() else {
            logger::error!("Editor UI is missing!");
            return;
        };

        dialog
            .popup_exclusive_centered_clamped_ex(&ui)
            .minsize(Vector2i::new(2048, 1024))
            .fallback_ratio(0.5)
            .done();
    }

    #[func]
    fn on_file_selected(&mut self, file_path: GString) {
        let Some(editor) = self.editor.as_ref() else {
            logger::error!("Editor is not available!");
            return;
        };

        let import_config_name = format!("{}.import", file_path);

        if !FileAccess::file_exists(&import_config_name) {
            logger::warn!("Resource has never been imported by the editor!");
            return;
        }

        Self::update_import_config(&import_config_name);
        editor
            .get_resource_filesystem()
            .unwrap()
            .reimport_files(&[file_path].into());
    }

    #[func]
    fn on_dir_selected(&mut self, root_dir: GString) {
        let Some(editor) = self.editor.as_ref() else {
            logger::error!("Editor is not available!");
            return;
        };

        if root_dir.is_empty() {
            logger::error!("Directory path is empty!");
            return;
        }

        let mut dir_queue = VecDeque::from([root_dir.clone()]);
        let mut file_queue = Vec::new();
        let pattern = Regex::new(pomsky! { "." ("gltf" | "glb")$ }).expect("unable to fail");

        while let Some(dir_path) = dir_queue.pop_front() {
            logger::info!("Traversing dir \"{}\"...", dir_path);
            let Some(mut dir) = DirAccess::open(&dir_path) else {
                logger::error!("Directory not accessible: {}", root_dir);
                return;
            };

            dir_queue.append(
                &mut dir
                    .get_directories()
                    .to_vec()
                    .into_iter()
                    .map(|dir_name| format!("{}/{}", dir_path, dir_name).into())
                    .collect(),
            );
            file_queue.append(
                &mut dir
                    .get_files()
                    .to_vec()
                    .into_iter()
                    .filter(|file_name| pattern.is_match(&file_name.to_string()))
                    .map(|file_name| GString::from(format!("{}/{}", dir_path, file_name)))
                    .collect(),
            );
        }

        file_queue.iter().for_each(|path| {
            let import_config_name = format!("{}.import", path);

            logger::info!("Processing import config \"{}\"...", import_config_name);

            if !FileAccess::file_exists(&import_config_name) {
                logger::warn!("Resource has never been imported by the editor!");
                return;
            }

            Self::update_import_config(&import_config_name);
        });

        editor
            .get_resource_filesystem()
            .unwrap()
            .reimport_files(&file_queue.as_slice().into());
    }

    fn update_import_config(config_file_name: &str) {
        let mut file = ConfigFile::new_gd();

        let error = file.load(config_file_name);

        if error != Error::OK {
            logger::error!(
                "Failed to read config file {:?}: {}",
                error,
                config_file_name
            );
            return;
        }

        let mut subresources: Dictionary = file
            .get_value_ex("params", "_subresources")
            .default(&Dictionary::new().to_variant())
            .done()
            .try_to()
            .inspect_err(|err| {
                logger::error!("Failed to read subresources as dictionary: {}", err);
            })
            .unwrap_or_default();

        let mut nodes: Dictionary = subresources
            .get("nodes")
            .map(|value| value.try_to())
            .transpose()
            .inspect_err(|err| {
                logger::error!("Failed to read nodes as dictionary: {}", err);
            })
            .ok()
            .flatten()
            .unwrap_or_default();

        let scene = ResourceLoader::singleton()
            .load_ex(&config_file_name.replace(".import", ""))
            .type_hint(&PackedScene::class_name().to_gstring())
            .done();

        let Some(scene) = scene else {
            logger::error!("Failed to load resource!");
            return;
        };

        let scene: Gd<PackedScene> = match scene.try_cast() {
            Ok(scene) => scene,
            Err(_) => {
                logger::error!("Loaded resouce is not of type PackedScene!");
                return;
            }
        };

        let Some(scene_state) = scene.get_state() else {
            logger::error!("Failed to read scene state!");
            return;
        };

        (0..scene_state.get_node_count())
            .filter(|index| {
                scene_state.get_node_type(*index) == MeshInstance3D::class_name().to_string_name()
            })
            .for_each(|index| {
                let path = scene_state.get_node_path(index);
                let mut config: Dictionary = nodes
                    .get(path.clone())
                    .map(|value| value.try_to())
                    .transpose()
                    .inspect_err(|err| {
                        logger::error!("Failed to read node config as dictionary: {}", err);
                    })
                    .ok()
                    .flatten()
                    .unwrap_or_default();

                config.set("generate/occluder", 1);
                config.set("mesh_instance/visibility_range_end", 800.0);
                config.set("mesh_instance/visibility_range_end_margin", 200.0);
                config.set("mesh_instance/visibility_range_fade_mode", 1);

                nodes.set(
                    format!("PATH:{}", path.to_string().trim_start_matches("./")),
                    config,
                );
            });

        subresources.set("nodes", nodes);

        file.set_value("params", "_subresources", &subresources.to_variant());

        let error = file.save(config_file_name);

        if error != Error::OK {
            logger::error!(
                "Failed to write import config file {:?}: {}",
                error,
                config_file_name
            );
        }
    }
}
