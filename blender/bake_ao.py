import bpy
import os
import sys
from pathlib import Path
from argparse import ArgumentParser
from functools import reduce
import mathutils

GLTF_OUTPUT_NODE_GROUP = "glTF Material Output"
GLTF_OUTPUT_OCCLUSION = "Occlusion"


# Function to clear the scene
def clear_scene():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()
    bpy.ops.outliner.orphans_purge()


# Function to import glTF model
def import_gltf(file_path) -> bpy.types.Object:
    bpy.ops.import_scene.gltf(filepath=file_path)

    if "Ground_AO" in bpy.data.objects:
        bpy.data.objects.remove(bpy.data.objects["Ground_AO"])

    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")

    bpy.ops.mesh.remove_doubles()
    bpy.ops.mesh.set_sharpness_by_angle(angle=0.3)
    bpy.ops.mesh.average_normals(average_type="FACE_AREA")

    bpy.ops.object.mode_set(mode="OBJECT")

    return bpy.context.object


def export_gltf(object, file_path):
    bpy.ops.object.select_all(action="SELECT")
    # object.select_set(True)

    texture_dir = Path(
        os.path.relpath("/Users/jovan/SimChopper/resources/Textures/", start=file_path)
    ).joinpath(Path(file_path).stem)

    bpy.ops.export_scene.gltf(
        filepath=file_path,
        export_format="GLTF_SEPARATE",
        export_keep_originals=True,
        export_extras=True,
        use_mesh_edges=True,
        use_selection=True,
        export_texture_dir=str(texture_dir),
    )


# Function to bake ambient occlusion
def bake_ambient_occlusion(obj, output_path, is_orm=False):
    # Ensure the object is selected
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj

    # Create a new material
    mat = bpy.data.materials.new(name="Material")
    mat.use_nodes = True

    # Create a new image for the material
    img = bpy.data.images.new("AO_Map", 2048, 2048, is_data=True, alpha=False)
    img.generated_type = "BLANK"
    img.scale(1024, 1024)

    # Assign the new material to all faces of the object
    obj.data.materials.clear()
    obj.data.materials.append(mat)

    # Assign the new image to the material
    tex_node = mat.node_tree.nodes.new("ShaderNodeTexImage")
    tex_node.image = img
    mat.node_tree.links.new(
        mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"],
        tex_node.outputs["Color"],
    )

    # Configure baking settings
    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.bake_type = "AO"
    bpy.context.scene.cycles.device = "GPU"

    # Bake
    print("starting to bake...")
    bpy.ops.object.bake(
        type="AO", save_mode="EXTERNAL", filepath=output_path, width=1024, height=1024
    )

    if is_orm:
        print("rebaking as ORM texture...")

        orm_node = mat.node_tree.nodes.new("ShaderNodeCombineColor")
        orm_node.inputs['Green'].default_value = 1.0

        mat.node_tree.links.new(
            orm_node.inputs["Red"],
            tex_node.outputs["Color"]
        )
        
        mat.node_tree.links.new(
            mat.node_tree.nodes["Principled BSDF"].inputs["Base Color"],
            orm_node.outputs["Color"],
        )

        # Create a new image for the ORM version
        img = bpy.data.images.new("ORM_Map", 2048, 2048, is_data=True, alpha=False)
        img.generated_type = "BLANK"
        img.scale(1024, 1024)

        orm_tex_node = mat.node_tree.nodes.new("ShaderNodeTexImage")
        orm_tex_node.image = img
        orm_tex_node.select = True

        mat.node_tree.nodes.active = orm_tex_node

        bpy.context.scene.render.bake.use_pass_direct = False
        bpy.context.scene.render.bake.use_pass_indirect = False
        bpy.context.scene.render.bake.use_pass_color = True

        bpy.ops.object.bake(
            type="DIFFUSE", save_mode="INTERNAL", filepath=output_path, width=1024, height=1024
        )
    
    print("saving texture to: " + output_path)
    img.save(filepath=output_path)


# Function to create a new UV map
def create_uv_map(obj):
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)

    if len(obj.data.uv_layers) >= 2:
        obj.data.uv_layers.active_index = 1
        return

    uv_map = obj.data.uv_layers.new(name="AO")
    uv_map.active = True

    obj.data.uv_layers.active_index = 1

    # Enter edit mode
    bpy.ops.object.mode_set(mode="EDIT")

    # Select all faces
    bpy.ops.mesh.select_all(action="SELECT")

    bpy.ops.uv.smart_project(angle_limit=66, island_margin=0)

    bpy.ops.object.mode_set(mode="OBJECT")


# add AO texture to all materials of the object
def setup_ao_map(object: bpy.types.Object, ao_path: str):
    materials = object.data.materials

    gltf_settings_group = get_gltf_settings_node_tree()

    ao_texture = bpy.data.images.load(ao_path, check_existing=True)

    for material in materials:
        if (
            next(
                (
                    node
                    for node in filter(
                        lambda node: node.name.startswith("Group"),
                        material.node_tree.nodes,
                    )
                    if node.node_tree.name == GLTF_OUTPUT_NODE_GROUP
                ),
                None,
            )
            is not None
        ):
            print("material " + material.name + " already has a GLTF AO setup!")
            continue

        uv_node = material.node_tree.nodes.new("ShaderNodeUVMap")
        uv_node.uv_map = object.data.uv_layers.keys()[1]

        ao_node = material.node_tree.nodes.new("ShaderNodeTexImage")
        ao_node.image = ao_texture

        gltf_settings = material.node_tree.nodes.new("ShaderNodeGroup")
        gltf_settings.node_tree = gltf_settings_group

        material.node_tree.links.new(ao_node.inputs["Vector"], uv_node.outputs["UV"])
        material.node_tree.links.new(
            gltf_settings.inputs[GLTF_OUTPUT_OCCLUSION], ao_node.outputs["Color"]
        )
        material.use_backface_culling = True
        material.use_backface_culling_shadow = True
        material.use_backface_culling_lightprobe_volume = True


def setup_ao_decal(object: bpy.types.Object, ao_path: str):
    material = bpy.data.materials.new("Ground_AO")
    material.use_nodes = True

    object.data.materials.append(material)
    
    ao_texture = bpy.data.images.load(ao_path, check_existing=True)
    ao_node = material.node_tree.nodes.new("ShaderNodeTexImage")
    ao_node.image = ao_texture

    gltf_settings = material.node_tree.nodes.new("ShaderNodeGroup")
    gltf_settings.node_tree = get_gltf_settings_node_tree()


    material.node_tree.links.new(
        gltf_settings.inputs[GLTF_OUTPUT_OCCLUSION], ao_node.outputs["Color"]
    )
    material.use_backface_culling = True
    material.use_backface_culling_shadow = True
    material.use_backface_culling_lightprobe_volume = True


def get_gltf_settings_node_tree() -> bpy.types.NodeTree:
    if GLTF_OUTPUT_NODE_GROUP in bpy.data.node_groups:
        return bpy.data.node_groups[GLTF_OUTPUT_NODE_GROUP]

    new_group = bpy.data.node_groups.new(GLTF_OUTPUT_NODE_GROUP, "ShaderNodeTree")

    new_group.interface.new_socket(GLTF_OUTPUT_OCCLUSION, in_out="INPUT")

    return new_group


def create_ground_ao_catcher(object: bpy.types.Object) -> bpy.types.Object:
    bound_box = [object.matrix_world @ mathutils.Vector(point) for point in object.bound_box]

    max = reduce(
        lambda a, b: [
            a[0] if a[0] > b[0] else b[0],
            a[1] if a[1] > b[1] else b[1],
            a[2] if a[2] > b[2] else b[2],
        ],
        bound_box,
    )

    min = reduce(
        lambda a, b: [
            a[0] if a[0] < b[0] else b[0],
            a[1] if a[1] < b[1] else b[1],
            a[2] if a[2] < b[2] else b[2],
        ],
        bound_box,
    )

    size = [max[0] - min[0],  max[1] - min[1], 0.0]

    bpy.ops.mesh.primitive_plane_add(size = 1.0, calc_uvs = True, scale = size, location = [min[0] + (size[0] * 0.5), min[1] + (size[1] * 0.5), min[2] + (size[2] * 0.5)])

    object = bpy.context.object
    object.scale = [item + 1 for item in size]
    object.name = "Ground_AO"
    object["gd_node"] = "Decal"
    object["lower_fade"] = 9
    object["upper_fade"] = 10
    

    return object


# Main script
if __name__ == "__main__":
    args = []

    if "--" in sys.argv:
        args = sys.argv[sys.argv.index("--") + 1 :]

    argparser = ArgumentParser()
    argparser.add_argument("--file", nargs="*", required=True)
    argparser.add_argument("--ao-tex-dir", required=True)
    argparser.add_argument("--base-path", required=True)

    config = argparser.parse_args(args=args)

    for file in config.file:
        clear_scene()

        print("File|" + file, flush=True)
        print("baking Ambient Occlusion texture for " + file + "...", flush=True)

        # Import the glTF model
        print("Progress|Importing model...", flush=True)
        object = import_gltf(file)

        # Specify the output path for the AO map
        ao_output_path = (
            Path(config.ao_tex_dir)
            .joinpath(Path(file).relative_to(config.base_path))
            .with_suffix(".png")
        )

        ao_ground_output_path = str(ao_output_path.with_stem(ao_output_path.stem + '_ground'))
        ao_output_path = str(ao_output_path)

        print("Progress|Creating UV map...", flush=True)
        create_uv_map(object)
        print("Progress|Baking Ambient Occlusion...", flush=True)
        bake_ambient_occlusion(object, ao_output_path)
        print("Progress|Setting up Ground Ambient Occlusion...", flush=True)

        ground_ao_catcher = create_ground_ao_catcher(object)

        print("Progress|Baking Ground Ambient Occlusion...", flush=True)
        bake_ambient_occlusion(ground_ao_catcher, ao_ground_output_path, is_orm=True)

        print("Ambient Occlusion map baked and saved successfully.", flush=True)

        print("Progress|Clearing Scene...", flush=True)
        clear_scene()

        print("Progress|Re-Importing model...", flush=True)
        object = import_gltf(file)

        print("Progress|Re-Creating UV Map...", flush=True)
        create_uv_map(object)
        print("Progress|Setting up Ambient Occlusion map...", flush=True)
        setup_ao_map(object, ao_output_path)

        ao_ground_object = create_ground_ao_catcher(object)
        print("Progress|Setting up Ground Ambient Occlusion map...", flush=True)
        setup_ao_decal(ao_ground_object, ao_ground_output_path)
        
        file = str(Path(file).with_suffix(".gltf"))

        print("Progress|Exporting modified model...", flush=True)
        export_gltf(object, file)
