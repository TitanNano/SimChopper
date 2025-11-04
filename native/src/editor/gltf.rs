use anyhow::Context;
use godot::builtin::{Array, Dictionary, GString, Variant, Vector3};
use godot::classes::decal::DecalTexture;
use godot::classes::{
    base_material_3d, BaseMaterial3D, Decal, GltfNode, GltfState, IGltfDocumentExtension, Node,
    Node3D,
};
use godot::global;
use godot::obj::{Gd, NewAlloc};
use godot::prelude::{godot_api, GodotClass};
use itertools::Itertools;

use crate::util::logger;

#[derive(GodotClass)]
#[class(base = GltfDocumentExtension, tool, init)]
pub struct GltfImporter;

#[derive(Debug)]
enum SupportedNodeTypes {
    Decal,
    Unsupported,
}

impl From<GString> for SupportedNodeTypes {
    fn from(value: GString) -> Self {
        match String::from(&value).as_str() {
            "Decal" => Self::Decal,
            _ => Self::Unsupported,
        }
    }
}

#[godot_api]
impl IGltfDocumentExtension for GltfImporter {
    fn import_post_parse(&mut self, state: Option<Gd<GltfState>>) -> global::Error {
        let Some(mut state) = state else {
            return global::Error::ERR_INVALID_DATA;
        };

        if let Err(err) = fix_ao_uv2(&mut state) {
            return err;
        }

        global::Error::OK
    }

    fn generate_scene_node(
        &mut self,
        state: Option<Gd<GltfState>>,
        gltf_node: Option<Gd<GltfNode>>,
        _parent: Option<Gd<Node>>,
    ) -> Option<Gd<Node3D>> {
        let Some(mut state) = state else {
            logger::error!("generate_scene_node called with null GltfState!");
            return None;
        };

        let Some(mut gltf_node) = gltf_node else {
            logger::error!("generate_scene_node called with null GltfNode!");
            return None;
        };

        match use_gd_node(&mut gltf_node, &mut state) {
            Ok(node) => node,
            Err(err) => {
                logger::error!("Failed to generate GD Node: {:?}", err);
                None
            }
        }
    }
}

fn fix_ao_uv2(state: &mut Gd<GltfState>) -> Result<(), global::Error> {
    let materials = state.get_materials();

    if materials.is_empty() {
        logger::info!("GLTF model does not contain materials!");
        return Ok(());
    }

    let Some(raw_materials) = state.get_json().get("materials") else {
        logger::error!(
            "GLTF model does not contain a materials array, but materials have been imported!"
        );

        return Err(global::Error::FAILED);
    };

    let raw_materials: Array<Variant> = raw_materials.to();

    materials.iter_shared().for_each(|material| {
        let raw_material = raw_materials
            .iter_shared()
            .find(|mat| {
                let Some(name) = mat.to::<Dictionary>().get("name") else {
                    logger::debug!("raw material doesn't have a name!");
                    return false;
                };

                material.get_name() == name.to()
            })
            .map(|mat| mat.to::<Dictionary>());

        let Some(raw_material) = raw_material else {
            logger::error!("Unable to locate raw material in GLTF model!");
            return;
        };

        let tex_coord = raw_material
            .get("occlusionTexture")
            .and_then(|occlusion_tex| occlusion_tex.to::<Dictionary>().get("texCoord"))
            .map_or(0.0, |tex_coord| tex_coord.to::<f64>());

        if tex_coord > 0.0 {
            material
                .cast::<BaseMaterial3D>()
                .set_flag(base_material_3d::Flags::AO_ON_UV2, true);
        }
    });

    Ok(())
}

fn use_gd_node(
    gltf_node: &mut Gd<GltfNode>,
    state: &mut Gd<GltfState>,
) -> Result<Option<Gd<Node3D>>, anyhow::Error> {
    let Some((index, _)) = state
        .get_nodes()
        .iter_shared()
        .find_position(|item| item == gltf_node)
    else {
        anyhow::bail!("GltfNode is not in state!? {}", gltf_node.get_name());
    };

    let Some(gltf_raw_nodes) = state.get_json().get("nodes") else {
        anyhow::bail!("gltf JSON does not contain nodes...");
    };

    let extras = gltf_raw_nodes
        .to::<Array<Variant>>()
        .at(index)
        .to::<Dictionary>()
        .get("extras")
        .map(|var| var.to::<Dictionary>());

    let Some(extras) = extras else {
        return Ok(None);
    };

    let node_type = extras
        .get("gd_node")
        .map(|var| var.to::<GString>())
        .map(SupportedNodeTypes::from);

    let Some(node_type) = node_type else {
        return Ok(None);
    };

    match node_type {
        SupportedNodeTypes::Decal => {
            let mut node = Decal::new_alloc();

            node.set_size(Vector3::ONE);

            let mesh_index = gltf_node.get_mesh();

            let Some(mesh) = state
                .get_meshes()
                .at(mesh_index.try_into().context(
                    "Failed to convert mesh index to usize, it should never be negative",
                )?)
                .get_mesh()
            else {
                anyhow::bail!("GltfMesh does not have a ImporterMesh !?");
            };

            if mesh.get_surface_count() != 1 {
                anyhow::bail!("Unable to import mesh with more than one surface as Decal!");
            }

            let material = mesh
                .get_surface_material(0)
                .map(godot::prelude::Gd::cast::<BaseMaterial3D>);

            let Some(material) = material else {
                anyhow::bail!(
                    "GLTF node {} has GD node type {:?} but no materials!",
                    gltf_node.get_name(),
                    node_type
                );
            };

            let ao_texture =
                material.get_texture(base_material_3d::TextureParam::AMBIENT_OCCLUSION);

            let Some(ao_texture) = ao_texture else {
                anyhow::bail!(
                    "GLTF node {} has no AO texture, but one was expected for node type {:?}",
                    gltf_node.get_name(),
                    node_type
                );
            };

            node.set_texture(DecalTexture::ORM, &ao_texture);

            if let Some(albedo_texture) =
                material.get_texture(base_material_3d::TextureParam::ALBEDO)
            {
                node.set_texture(DecalTexture::ALBEDO, &albedo_texture);
            } else {
                node.set_texture(DecalTexture::ALBEDO, &ao_texture);
                node.set_albedo_mix(0.0);
            }

            if let Some(upper_fade) = extras.get("upper_fade") {
                node.set_upper_fade(upper_fade.to());
            }

            if let Some(lower_fade) = extras.get("lower_fade") {
                node.set_lower_fade(lower_fade.to());
            }

            Ok(Some(node.upcast()))
        }

        SupportedNodeTypes::Unsupported => Ok(None),
    }
}
