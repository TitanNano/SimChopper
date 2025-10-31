use godot::builtin::{
    Array, Dictionary, GString, PackedInt32Array, PackedVector3Array, Variant, Vector3,
};
use godot::classes::decal::DecalTexture;
use godot::classes::mesh::ArrayType;
use godot::classes::{
    base_material_3d, ArrayOccluder3D, BaseMaterial3D, CollisionShape3D, ConcavePolygonShape3D,
    Decal, GltfNode, GltfState, IGltfDocumentExtension, Node, Node3D, OccluderInstance3D,
};
use godot::global;
use godot::obj::{EngineEnum, Gd, NewAlloc, NewGd};
use godot::prelude::{godot_api, GodotClass};
use itertools::Itertools;

use crate::util::logger;

#[derive(GodotClass)]
#[class(base = GltfDocumentExtension, tool, init)]
pub struct GltfImporter;

#[derive(Debug)]
enum SupportedNodeTypes {
    Decal,
    Occluder,
    Collision,
    Unsupported,
}

impl From<GString> for SupportedNodeTypes {
    fn from(value: GString) -> Self {
        match String::from(&value).as_str() {
            "Decal" => Self::Decal,
            "Collision" => Self::Collision,
            "Occluder" => Self::Occluder,
            _ => Self::Unsupported,
        }
    }
}

impl SupportedNodeTypes {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Decal => "Decal",
            Self::Collision => "Collision",
            Self::Occluder => "Occluder",
            Self::Unsupported => "UNSUPPORTED",
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

        fix_emissive_materials(&mut state);

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
            .map(|tex_coord| tex_coord.to::<f64>())
            .map(|tex_coord| tex_coord as u64)
            .unwrap_or(0);

        if tex_coord > 0 {
            material
                .cast::<BaseMaterial3D>()
                .set_flag(base_material_3d::Flags::AO_ON_UV2, true);
        }
    });

    Ok(())
}

fn fix_emissive_materials(state: &mut Gd<GltfState>) {
    state.get_materials().iter_shared().for_each(|material| {
        let Ok(mut base_material) = material.try_cast::<BaseMaterial3D>() else {
            logger::warn!("GLTF material is not being imported as BaseMaterial3D");
            return;
        };

        if !base_material.get_feature(base_material_3d::Feature::EMISSION) {
            return;
        }

        base_material.set_emission_energy_multiplier(0.0);
    });
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
        .and_then(|var: Variant| var.try_to::<GString>().ok())
        .map(SupportedNodeTypes::from);

    let Some(node_type) = node_type else {
        return Ok(None);
    };

    let node: Result<Option<Gd<Node3D>>, anyhow::Error> = match node_type {
        SupportedNodeTypes::Decal => create_decal_node(gltf_node, &extras, state),
        SupportedNodeTypes::Collision => create_collision_node(gltf_node, state),
        SupportedNodeTypes::Occluder => create_occluder_node(gltf_node, state),
        SupportedNodeTypes::Unsupported => Ok(None),
    };

    Ok(node?.map(|mut node| {
        node.set_name(&gltf_node.get_name());
        node
    }))
}

fn create_decal_node(
    gltf_node: &mut Gd<GltfNode>,
    extras: &Dictionary,
    state: &mut Gd<GltfState>,
) -> Result<Option<Gd<Node3D>>, anyhow::Error> {
    let mut node = Decal::new_alloc();

    node.set_size(Vector3::ONE);

    let mesh_index = gltf_node.get_mesh();

    let Some(mesh) = state.get_meshes().at(mesh_index as usize).get_mesh() else {
        anyhow::bail!("GltfMesh does not have a ImporterMesh !?");
    };

    if mesh.get_surface_count() != 1 {
        anyhow::bail!("Unable to import mesh with more than one surface as Decal!");
    }

    let material = mesh
        .get_surface_material(0)
        .map(|material| material.cast::<BaseMaterial3D>());

    let Some(material) = material else {
        anyhow::bail!(
            "GLTF node {} has GD node type {} but no materials!",
            gltf_node.get_name(),
            SupportedNodeTypes::Decal.as_str()
        );
    };

    let ao_texture = material.get_texture(base_material_3d::TextureParam::AMBIENT_OCCLUSION);

    let Some(ao_texture) = ao_texture else {
        anyhow::bail!(
            "GLTF node {} has no AO texture, but one was expected for node type {}",
            gltf_node.get_name(),
            SupportedNodeTypes::Decal.as_str()
        );
    };

    node.set_texture(DecalTexture::ORM, &ao_texture);

    if let Some(albedo_texture) = material.get_texture(base_material_3d::TextureParam::ALBEDO) {
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

fn create_collision_node(
    gltf_node: &mut Gd<GltfNode>,
    state: &mut Gd<GltfState>,
) -> Result<Option<Gd<Node3D>>, anyhow::Error> {
    let mesh_index = gltf_node.get_mesh();

    let Some(mesh) = state.get_meshes().at(mesh_index as usize).get_mesh() else {
        anyhow::bail!("GltfMesh does not have a ImporterMesh !?");
    };

    if mesh.get_surface_count() != 1 {
        anyhow::bail!("Unable to import mesh with more than one surface as Collision Shape!");
    }

    let surface = mesh.get_surface_arrays(0);

    let Some(verticies): Option<PackedVector3Array> = surface
        .get(ArrayType::VERTEX.ord() as usize)
        .map(|item| item.to())
    else {
        anyhow::bail!("Unable to get verticies from gltf mesh!");
    };

    let Some(indicies): Option<PackedInt32Array> = surface
        .get(ArrayType::INDEX.ord() as usize)
        .map(|item| item.to())
    else {
        anyhow::bail!("Unable to get indicies from gltf mesh!");
    };

    let mut node = CollisionShape3D::new_alloc();
    let mut mesh = ConcavePolygonShape3D::new_gd();

    mesh.set_faces(
        &indicies
            .as_slice()
            .iter()
            .map(|index| verticies.get(*index as usize).unwrap_or_default())
            .collect(),
    );
    node.set_shape(&mesh);

    Ok(Some(node.upcast()))
}

fn create_occluder_node(
    gltf_node: &mut Gd<GltfNode>,
    state: &mut Gd<GltfState>,
) -> Result<Option<Gd<Node3D>>, anyhow::Error> {
    let mesh_index = gltf_node.get_mesh();

    let Some(mesh) = state.get_meshes().at(mesh_index as usize).get_mesh() else {
        anyhow::bail!("GltfMesh does not have a ImporterMesh !?");
    };

    if mesh.get_surface_count() != 1 {
        anyhow::bail!("Unable to import mesh with more than one surface as Occluder Mesh!");
    }

    let surface = mesh.get_surface_arrays(0);

    let Some(verticies) = surface
        .get(ArrayType::VERTEX.ord() as usize)
        .map(|item| item.to())
    else {
        anyhow::bail!("Unable to get verticies from gltf mesh!");
    };

    let Some(indicies) = surface
        .get(ArrayType::INDEX.ord() as usize)
        .map(|item| item.to())
    else {
        anyhow::bail!("Unable to get indicies from gltf mesh!");
    };

    let mut node = OccluderInstance3D::new_alloc();
    let mut mesh = ArrayOccluder3D::new_gd();

    mesh.set_arrays(&verticies, &indicies);

    node.set_occluder(&mesh);

    Ok(Some(node.upcast()))
}
