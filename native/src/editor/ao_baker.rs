use std::{
    io::BufReader,
    process::{Command, Stdio},
};

use godot::{
    builtin::{GString, PackedStringArray, VariantType},
    classes::{EditorInterface, ProjectSettings, RefCounted, SceneTree},
    global::PropertyHint,
    obj::{Base, Gd, Singleton},
    prelude::{godot_api, GodotClass},
    task,
};

use crate::{
    editor::{
        new_non_zero,
        ui::{ForgroundProcess, ProgressDialog},
    },
    util::logger,
};

#[derive(GodotClass)]
#[class(base = RefCounted, no_init)]
pub struct AoBaker {
    editor_interface: Gd<EditorInterface>,
    scene_tree: Gd<SceneTree>,
    base: Base<RefCounted>,
}

#[godot_api]
impl AoBaker {
    const AO_BAKE_MODEL_BASE_DIR: &str = "editor/baking/ambient_occlusion_model_base_path";
    const AO_BAKE_INCLUDE_LIST: &str = "editor/baking/ambient_occlusion_models";
    const AO_BAKE_TEXTURE_DIR: &str = "editor/baking/ambient_occlusion_textures";

    pub const SETTINGS: [(&'static str, VariantType, PropertyHint, &'static str); 3] = [
        (
            Self::AO_BAKE_MODEL_BASE_DIR,
            VariantType::STRING,
            PropertyHint::DIR,
            "",
        ),
        (
            Self::AO_BAKE_TEXTURE_DIR,
            VariantType::STRING,
            PropertyHint::DIR,
            "",
        ),
        (
            Self::AO_BAKE_INCLUDE_LIST,
            VariantType::STRING,
            PropertyHint::TYPE_STRING,
            // VARIANT_TYPE_STRING/PROPERTY_HINT_DIR:
            "4/14:",
        ),
    ];

    pub fn new(editor_interface: Gd<EditorInterface>, scene_tree: Gd<SceneTree>) -> Gd<Self> {
        Gd::from_init_fn(|base| Self {
            editor_interface,
            scene_tree,
            base,
        })
    }

    #[func]
    pub fn bake(&mut self) {
        let tree = &self.scene_tree;
        let editor_interface = &self.editor_interface;

        task::spawn(Self::async_bake(tree.clone(), editor_interface.clone()));
    }

    #[expect(clippy::too_many_lines)]
    async fn async_bake(tree: Gd<SceneTree>, editor_interface: Gd<EditorInterface>) {
        use std::io::BufRead;

        let editor_settings = editor_interface
            .get_editor_settings()
            .expect("Editor settings must be there");

        let blender_path_variant =
            editor_settings.get_setting("filesystem/import/blender/blender_path");

        if blender_path_variant.is_nil() {
            logger::error!("Unable to get blender path!");
            return;
        }

        let blender_path = blender_path_variant.to::<GString>();
        let project_settings = ProjectSettings::singleton();
        let ambient_occlusion_textures = project_settings.globalize_path(
            &project_settings
                .get_setting(Self::AO_BAKE_TEXTURE_DIR)
                .to::<GString>(),
        );
        let base_path = project_settings.globalize_path(
            &project_settings
                .get_setting(Self::AO_BAKE_MODEL_BASE_DIR)
                .to::<GString>(),
        );

        let blender_script = project_settings.globalize_path("res://blender/bake_ao.py");

        let model_paths: Vec<_> = project_settings
            .get_setting(Self::AO_BAKE_INCLUDE_LIST)
            .to::<PackedStringArray>()
            .to_vec()
            .into_iter()
            .filter_map(|path| {
                let system_path = project_settings.globalize_path(&path);

                std::fs::read_dir(system_path.to_string())
                    .inspect_err(|err| {
                        logger::error!(
                            "unable to read directory: {}.\nError: {}",
                            system_path,
                            err
                        );
                    })
                    .ok()
            })
            .flatten()
            .filter_map(|entry| {
                entry
                    .inspect_err(|err| logger::error!("Failed to enumerate directory entry: {err}"))
                    .ok()
            })
            .filter(|entry| {
                entry.metadata().is_ok_and(|metadata| metadata.is_file())
                    && entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| ["glb", "gltf"].contains(&ext))
            })
            .map(|entry| entry.path())
            .collect();

        let mut dialog = ProgressDialog::new(&ForgroundProcess {
            title: "Ambient Occlusion Baking",
            tasks: new_non_zero(
                model_paths
                    .len()
                    .try_into()
                    .expect("model_paths should fit into a u32"),
            ),
            task_title: "Bake Mesh",
            steps: new_non_zero(11),
        });
        dialog.bind_mut().popup();

        let mut blender_command = Command::new(blender_path.to_string());

        blender_command
            .arg("--background")
            .arg("--python")
            .arg(blender_script.to_string())
            .arg("--")
            .arg("--ao-tex-dir")
            .arg(ambient_occlusion_textures.to_string())
            .arg("--base-path")
            .arg(base_path.to_string())
            .arg("--file")
            .args(model_paths)
            .stdout(Stdio::piped())
            .stdin(Stdio::null());

        logger::debug!("invoking blender: {:?}", blender_command);

        let blender = blender_command.spawn();

        let mut blender = match blender {
            Ok(child) => child,
            Err(err) => {
                logger::error!("Failed to spawn blender process: {}", err);
                return;
            }
        };

        let stdout = blender.stdout.take().unwrap();

        let mut stdreader = BufReader::new(stdout);
        let mut read_buffer = String::new();
        let next_frame = tree.signals().process_frame();

        loop {
            match blender.try_wait() {
                // We are not done yet.
                Ok(None) => (),

                // we are done and it was a success
                Ok(Some(status)) if status.success() => {
                    break;
                }

                // we are done but blender failed.
                Ok(Some(status)) => {
                    logger::error!(
                        "Blender exited with status {}",
                        status.code().unwrap_or_default()
                    );
                    break;
                }

                // getting the status didn't work
                Err(err) => {
                    logger::error!("Failed to check blender exit status: {err}");
                    break;
                }
            }

            read_buffer.clear();
            let line_error = stdreader.read_line(&mut read_buffer);

            if let Err(err) = line_error {
                logger::error!("failed to read from blender stdout: {err}");
                continue;
            }

            if read_buffer.starts_with("Progress|") {
                let label = read_buffer.split('|').nth(1).unwrap_or_default();

                dialog.bind_mut().push_task_step(label);
                logger::info!("Blender: {read_buffer}");
            }

            if read_buffer.starts_with("File|") {
                let label = read_buffer.split('|').nth(1).unwrap_or_default();

                dialog.bind_mut().push_task(label);
                logger::info!("Blender: {read_buffer}");
            }

            let _: () = next_frame.to_future().await;
        }

        dialog.queue_free();
    }
}
