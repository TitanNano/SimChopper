###
# Blender GLTF ORM exporter.
#
# This export script prepares blender objects to be exported to GLTF using ORM materials.
# - Marked objects are being "baked" into their final form by applying all modifiers.
# - All materials are baked into Diffuse, ORM and normal textures.
# - Marked objects are being exported to a single gltf file.
# - Empties can be used to export the same mesh as different objects.
#
# Custom Object Properties:
# - gltf_export: Boolean - mark an object for export
# - source: Object ID - The empty with this property will be converted into an object that reuses the mesh of the source.
#
# Render Visibility:
# The render visibility of an object is used to determine if the materials should be baked into an ORM material or not.
# Objects that are hidden from rendering will be exported without their materials.
#
# License: Apache License 2.0
###

import bpy
import sys
from pathlib import Path
from argparse import ArgumentParser
import io_scene_gltf2


TEXTURE_BASE_SIZE = 1024
EXPORT_PROPERTY = "gltf_export"
SOURCE_PROPERTY = "source"

# Find all objects in the scene that are marked for export.
def find_export_objects() -> list[bpy.types.Object]:
    return [obj for obj in bpy.data.objects.values() if obj.get(EXPORT_PROPERTY)]

# Find all objects in the scene that are marked for export.
def find_visible_objects(objects: list[bpy.types.Object]) -> list[bpy.types.Object]:
    return [obj for obj in objects if not obj.hide_render]

# Apply the modifiers of an object.
def apply_modifiers(object: bpy.types.Object) -> bpy.types.Object:
    if object.type == "FONT":
        return
    
    modifiers = object.modifiers.values()

    select_object(object)

    for mod in modifiers:
        bpy.ops.object.modifier_apply(modifier = mod.name, report = True)

    return object

def convert_to_mesh(object: bpy.types.Object):
    select_object(object)

    if object.type not in ["FONT"]:
        return

    bpy.ops.object.convert(target = "MESH", keep_original = False)


# Get rid of export unrelated objects.
def delete_objects_except(keep: list[bpy.types.Object]):
    to_delete = [obj for obj in bpy.data.objects.values() if obj not in keep]

    for obj in to_delete:
        bpy.data.objects.remove(obj, do_unlink = True, do_id_user = True, do_ui_user = True)

# load a new blendfile
def load_blend_file(path: str):
    bpy.ops.wm.open_mainfile(filepath = path)

# create an instance of the gltf output node group in a material
def create_gltf_output_node(material: bpy.types.Material) -> bpy.types.ShaderNode:
    gltf_settings_node_name = io_scene_gltf2.blender.com.material_helpers.get_gltf_node_name()

    if gltf_settings_node_name in bpy.data.node_groups:
        node_group = bpy.data.node_groups[gltf_settings_node_name]
    else:
        node_group = io_scene_gltf2.blender.com.material_helpers.create_settings_group(gltf_settings_node_name)

    new_node = material.node_tree.nodes.new("ShaderNodeGroup")
    new_node.node_tree = bpy.data.node_groups[node_group.name]

    return new_node


def select_object(object: bpy.types.Object):
    bpy.ops.object.select_all(action = "DESELECT")
    object.select_set(True)
    bpy.context.view_layer.objects.active = object


def texture_size(object: bpy.types.Object) -> int:
    if "texture_size" in object:
        return TEXTURE_BASE_SIZE * object["texture_size"]
    else:
        return TEXTURE_BASE_SIZE


# create a material that uses Diffuse, ORM and normal textures
def create_orm_material(
    object: bpy.types.Object,
    diffuse_image: bpy.types.Image,
    orm_image: bpy.types.Image,
    normal_image: bpy.types.Image,
    emission_image: bpy.types.Image,
) -> bpy.types.Material:
    material = bpy.data.materials.new(name = "ORM Material")
    material.use_backface_culling = True
    material.use_backface_culling_shadow = True
    material.use_backface_culling_lightprobe_volume = True
    material.use_nodes = True

    bsdf = material.node_tree.nodes["Principled BSDF"]

    # create ORM texture
    orm_texture = material.node_tree.nodes.new("ShaderNodeTexImage")
    orm_texture.image = orm_image
    orm_texture_splitter = material.node_tree.nodes.new("ShaderNodeSeparateColor")

    material.node_tree.links.new(
        orm_texture_splitter.inputs["Color"],
        orm_texture.outputs["Color"],
    )

    # create diffuse texture
    diffuse_texture = material.node_tree.nodes.new("ShaderNodeTexImage")
    diffuse_texture.image = diffuse_image

    material.node_tree.links.new(
        bsdf.inputs["Base Color"],
        diffuse_texture.outputs["Color"],
    )

    # create emission texture
    if emission_image is not None:
        emission_texture = material.node_tree.nodes.new("ShaderNodeTexImage")
        emission_texture.image = emission_image

        material.node_tree.links.new(
            bsdf.inputs["Emission Color"],
            emission_texture.outputs["Color"],
        )

        bsdf.inputs["Emission Strength"].default_value = 1.0

    material.node_tree.links.new(
        bsdf.inputs["Base Color"],
        diffuse_texture.outputs["Color"],
    )

    # create normal map texture
    normal_texture = material.node_tree.nodes.new("ShaderNodeTexImage")
    normal_texture.image = normal_image

    normal_map = material.node_tree.nodes.new("ShaderNodeNormalMap")

    material.node_tree.links.new(
        normal_map.inputs["Color"],
        normal_texture.outputs["Color"],
    )

    material.node_tree.links.new(
        bsdf.inputs["Normal"],
        normal_map.outputs["Normal"],
    )

    # connect ORM channels
    gltf_output = create_gltf_output_node(material)
    
    material.node_tree.links.new(
        gltf_output.inputs["Occlusion"],
        orm_texture_splitter.outputs["Red"],
    )
    
    material.node_tree.links.new(
        bsdf.inputs["Roughness"],
        orm_texture_splitter.outputs["Green"],
    )

    material.node_tree.links.new(
        bsdf.inputs["Metallic"],
        orm_texture_splitter.outputs["Blue"],
    )

    return material

# create a material for baking ORM textures.
def create_orm_bake_material(
    occlusion_image: bpy.types.Image,
    roughness_image: bpy.types.Image,
    metallic_image: bpy.types.Image,
    orm_image: bpy.types.Image,
) -> bpy.types.Material:
    
    material = bpy.data.materials.new(name = "ORM Baker")
    material.use_nodes = True
    bsdf = material.node_tree.nodes["Principled BSDF"]
    
    # create Occlusion texture
    occlusion_texture = material.node_tree.nodes.new("ShaderNodeTexImage")
    occlusion_texture.image = occlusion_image

    # create Roughness texture
    roughness_texture = material.node_tree.nodes.new("ShaderNodeTexImage")
    roughness_texture.image = roughness_image

    # create Metallic texture
    metallic_texture = material.node_tree.nodes.new("ShaderNodeTexImage")
    metallic_texture.image = metallic_image
    
    orm_texture_merger = material.node_tree.nodes.new("ShaderNodeCombineColor")

    material.node_tree.links.new(
        orm_texture_merger.inputs["Red"],
        occlusion_texture.outputs["Color"],
    )

    material.node_tree.links.new(
        orm_texture_merger.inputs["Green"],
        roughness_texture.outputs["Color"],
    )

    material.node_tree.links.new(
        orm_texture_merger.inputs["Blue"],
        metallic_texture.outputs["Color"],
    )

    material.node_tree.links.new(
        bsdf.inputs["Base Color"],
        orm_texture_merger.outputs["Color"],
    )


    orm_texture = material.node_tree.nodes.new("ShaderNodeTexImage")
    orm_texture.image = orm_image
    material.node_tree.nodes.active = orm_texture

    return material


def bake_orm_texture(
    object: bpy.types.Object,
    occlusion_image: bpy.types.Image,
    roughness_image: bpy.types.Image,
    metallic_image: bpy.types.Image
) -> bpy.types.Image:
    size = texture_size(object)
    # create output image
    orm_image = bpy.data.images.new(object.name.replace(" ", "_").lower() + "_orm", size, size, is_data = True, alpha = False)
    orm_image.generated_type = "BLANK"

    material = create_orm_bake_material(occlusion_image, roughness_image, metallic_image, orm_image)

    object.data.materials.append(material)

    # Configure bake settings
    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.device = "GPU"
    bpy.context.scene.cycles.samples = 1
    bpy.context.scene.render.bake.use_pass_direct = False
    bpy.context.scene.render.bake.use_pass_indirect = False
    bpy.context.scene.render.bake.use_pass_color = True

    # activate object
    select_object(object)
    bpy.ops.object.bake(type = "DIFFUSE", save_mode="INTERNAL", width = size, height = size)

    return orm_image


def bake_occlusion_texture(object: bpy.types.Object) -> bpy.types.Image:
    # Configure bake settings
    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.device = "GPU"
    bpy.context.scene.cycles.samples = 30

    size = texture_size(object)
    image = bpy.data.images.new(object.name.replace(" ", "_") + "_occlusion", size, size, is_data = True, alpha = False)
    image.generated_type = "BLANK"

    for material in object.data.materials:
        if material is None:
           continue 
        
        texture = material.node_tree.nodes.new("ShaderNodeTexImage")
        texture.image = image
        material.node_tree.nodes.active = texture

    select_object(object)
    bpy.ops.object.bake(type = "AO", save_mode="INTERNAL", width = size, height = size)

    return image


def bake_roughness_texture(object: bpy.types.Object) -> bpy.types.Image:
    # Configure bake settings
    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.device = "GPU"
    bpy.context.scene.cycles.samples = 1

    size = texture_size(object)
    image = bpy.data.images.new(object.name.replace(" ", "_").lower() + "_roughness", size, size, is_data = True, alpha = False)
    image.generated_type = "BLANK"

    for material in object.data.materials:
        if material is None:
           continue 
        texture = material.node_tree.nodes.new("ShaderNodeTexImage")
        texture.image = image
        material.node_tree.nodes.active = texture

    select_object(object)
    bpy.ops.object.bake(type = "ROUGHNESS", save_mode="INTERNAL", width = size, height = size)

    return image

def bake_diffuse_texture(object: bpy.types.Object) -> bpy.types.Image:
    # Configure bake settings
    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.device = "GPU"
    bpy.context.scene.cycles.samples = 1
    bpy.context.scene.render.bake.use_pass_direct = False
    bpy.context.scene.render.bake.use_pass_indirect = False
    bpy.context.scene.render.bake.use_pass_color = True

    size = texture_size(object)
    image = bpy.data.images.new(object.name.replace(" ", "_").lower() + "_diffuse", size, size, is_data = False, alpha = False)
    image.generated_type = "BLANK"

    metallic_values = []
    metallic_links = []

    for material in object.data.materials:
        if material is None:
           continue 
        texture = material.node_tree.nodes.new("ShaderNodeTexImage")
        texture.image = image
        material.node_tree.nodes.active = texture

        # remove metallic value from material to work around baking issue
        bsdf = material.node_tree.nodes["Principled BSDF"]

        if bsdf.inputs["Metallic"].is_linked:
            link = bsdf.inputs["Metallic"].links[0]
            metallic_links.append(link.from_socket)
            metallic_values.append(None)
            material.node_tree.links.remove(link)
        else:
            metallic_links.append(None)
            metallic_values.append(bsdf.inputs["Metallic"].default_value)
            bsdf.inputs["Metallic"].default_value = 0.0

    bpy.context.view_layer.objects.active = object
    bpy.ops.object.bake(type = "DIFFUSE", save_mode="INTERNAL", width = size, height = size)

    index = -1
    
    # restore metallic values for all materials after baking
    for material in object.data.materials:
        if material is None:
            continue

        index += 1

        print(f"restoring material {index}")

        bsdf = material.node_tree.nodes["Principled BSDF"]

        if metallic_values[index] is not None:
            bsdf.inputs["Metallic"].default_value = metallic_values[index]

        if metallic_links[index] is not None:
            material.node_tree.links.new(
                bsdf.inputs["Metallic"],
                metallic_links[index],
            )

    return image


def bake_metallic_texture(object: bpy.types.Object) -> bpy.types.Image:
    # Configure bake settings
    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.device = "GPU"
    bpy.context.scene.cycles.samples = 1

    size = texture_size(object)
    image = bpy.data.images.new(object.name.replace(" ", "_").lower() + "_metallic", size, size, is_data = True, alpha = False)
    image.generated_type = "BLANK"

    for material in object.data.materials:
        if material is None:
           continue 
        texture = material.node_tree.nodes.new("ShaderNodeTexImage")
        texture.image = image
        material.node_tree.nodes.active = texture

        metallic_slot = material.node_tree.nodes["Principled BSDF"].inputs["Metallic"]

        # if the metallic value is a single value we need a value node.
        if not metallic_slot.is_linked:
            metallic_value = material.node_tree.nodes.new("ShaderNodeValue")
            metallic_value.outputs[0].default_value = metallic_slot.default_value

            material.node_tree.links.new(
                material.node_tree.nodes["Material Output"].inputs["Surface"],
                metallic_value.outputs[0],
            )
            continue

        # if there is an incomming connection for the metallic value, we can reroute it.
        material.node_tree.links.new(
            material.node_tree.nodes["Material Output"].inputs["Surface"],
            metallic_slot.links[0].from_socket,
        )

    select_object(object)
    bpy.ops.object.bake(type = "EMIT", save_mode="INTERNAL", width = size, height = size)

    return image


def bake_normal_texture(object: bpy.types.Object) -> bpy.types.Image:
    # Configure bake settings
    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.device = "GPU"
    bpy.context.scene.cycles.samples = 1

    size = texture_size(object)
    image = bpy.data.images.new(object.name.replace(" ", "_").lower() + "_normal", size, size, is_data = True, alpha = False)
    image.generated_type = "BLANK"

    for material in object.data.materials:
        if material is None:
           continue 
        texture = material.node_tree.nodes.new("ShaderNodeTexImage")
        texture.image = image
        material.node_tree.nodes.active = texture

    select_object(object)
    bpy.ops.object.bake(type = "NORMAL", save_mode="INTERNAL", width = size, height = size)

    return image


def bake_emission_texture(object: bpy.types.Object) -> bpy.types.Image:
    # Configure bake settings
    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.device = "GPU"
    bpy.context.scene.cycles.samples = 1

    size = texture_size(object)
    image = bpy.data.images.new(object.name.replace(" ", "_").lower() + "_emission", size, size, is_data = False, alpha = False)
    image.generated_type = "BLANK"
    has_emission = False

    for material in object.data.materials:
        if material is None:
            continue

        texture = material.node_tree.nodes.new("ShaderNodeTexImage")
        texture.image = image
        material.node_tree.nodes.active = texture

        if material.node_tree.nodes["Principled BSDF"].inputs["Emission Strength"].default_value == 0.0:
            continue

        has_emission = True

    if not has_emission:
        return None

    select_object(object)
    bpy.ops.object.bake(type = "EMIT", save_mode="INTERNAL", width = size, height = size)

    return image


def uv_unwrap(object: bpy.types.Object):
    select_object(object)
    
    # Enter edit mode
    bpy.ops.object.mode_set(mode="EDIT")

    # Select all faces
    bpy.ops.mesh.select_all(action="SELECT")

    # bpy.ops.uv.lightmap_pack(PREF_PACK_IN_ONE=True, PREF_NEW_UVLAYER=False, PREF_BOX_DIV=48, PREF_MARGIN_DIV=0.15, PREF_CONTEXT="ALL_FACES")
    bpy.ops.uv.smart_project(angle_limit=0.66, island_margin=0.001, correct_aspect=True)
    bpy.ops.uv.select_all(action='SELECT')
    bpy.ops.uv.pack_islands(rotate=True, rotate_method="AXIS_ALIGNED", scale=True, margin=0.001, shape_method="CONCAVE")

    bpy.ops.object.mode_set(mode="OBJECT")


def clear_materials(object: bpy.types.Object):
    object.data.materials.clear()


def export_gltf(file_path: Path):
    bpy.ops.export_scene.gltf(
        filepath=str(file_path),
        export_format="GLTF_SEPARATE",
        export_keep_originals=False,
        export_extras=True,
        use_mesh_edges=True,
        export_texture_dir="./",
        export_image_quality=0,
        export_image_format="AUTO"
    )


def set_render_visibility(objects: list[bpy.types.Object]):
    for object in bpy.data.objects.values():
        object.hide_render = object not in objects
        object.hide_viewport = object not in objects


def instantiate_empties(objects: list[bpy.types.Object]):
    for (index, object) in enumerate(objects):
        if object.type != "EMPTY":
            continue

        name = object.name
        gd_node = object['gd_node'] if 'gd_node' in object else None
        source = object[SOURCE_PROPERTY]
        location = object.location
        rotation = object.rotation_euler
        rotation_mode = object.rotation_mode
        hide_render = object.hide_render
        hide_viewport = object.hide_viewport

        bpy.data.objects.remove(object, do_unlink=True, do_id_user=True, do_ui_user=True)
        new_object = bpy.data.objects.new(name, source.data)
        bpy.context.scene.collection.objects.link(new_object)
        new_object['gltf_export'] = True
        new_object.location = location
        new_object.rotation_mode = rotation_mode
        new_object.rotation_euler = rotation
        new_object.hide_render = hide_render
        new_object.hide_viewport = hide_viewport

        if gd_node is not None:
            new_object['gd_node'] = gd_node

        objects[index] = new_object


def parse_args() -> dict:
    args = []

    if "--" in sys.argv:
        args = sys.argv[sys.argv.index("--") + 1 :]

    argparser = ArgumentParser()
    argparser.add_argument("--file", nargs="*", required=True)
    argparser.add_argument("--debug", required=False)

    return argparser.parse_args(args=args)


def main():
    config = parse_args()

    for file in config.file:
        print("File|" + file, flush=True)

        load_blend_file(file)
        window = bpy.context.window_manager.windows[0]
        with bpy.context.temp_override(window=window):
            print("Progress|identify export objects...")
            objects = find_export_objects()
            print("Objects|" + str(len(objects)))

            for object in objects:
                if object.type == "EMPTY":
                    continue
                
                print("Progress|convert object \"" + object.name + "\" to mesh...")
                convert_to_mesh(object)

                if config.debug == "convert_mesh":
                    continue
                
                print("Progress|apply modifiers for \"" + object.name + "\"...")
                apply_modifiers(object)

            instantiate_empties(objects)

            if config.debug == "modifiers" or config.debug == "convert_mesh":
                return

            only_visible_objects = find_visible_objects(objects)

            for object in objects:
                if object in only_visible_objects:
                    continue

                object.data.materials.clear()

            set_render_visibility(only_visible_objects)

            for object in only_visible_objects:
                print("Progress|UV unwrap \"" + object.name + "\"...", flush = True)
                uv_unwrap(object)

                if config.debug == "uv":
                    continue

                print("Progress|bake occlusion texture...", flush = True)
                occlusion = bake_occlusion_texture(object)

                if config.debug == "bake_occlusion":
                    continue
                
                print("Progress|bake roughness texture...", flush = True)
                roughness = bake_roughness_texture(object)
                print("Progress|bake diffuse texture...", flush = True)
                diffuse = bake_diffuse_texture(object)

                if config.debug == "bake_diffuse":
                    continue
                
                print("Progress|bake normal texture...", flush = True)
                normal = bake_normal_texture(object)

                if config.debug == "bake_normal":
                    continue
                
                print("Progress|bake emission texture...", flush = True)
                emission = bake_emission_texture(object)

                if config.debug == "bake_emission":
                    continue
                
                print("Progress|bake metallic texture...", flush = True)
                metallic = bake_metallic_texture(object)

                if config.debug == "bake":
                    continue

                print("Progress|bake ORM texture...", flush = True)
                clear_materials(object)
                orm = bake_orm_texture(object, occlusion, roughness, metallic)

                if config.debug == "orm_bake":
                    continue

                print("Progress|create ORM material...", flush = True)
                clear_materials(object)
                orm_material = create_orm_material(object, diffuse, orm, normal, emission)

                object.data.materials.append(orm_material)

            if config.debug:
                return

            delete_objects_except(objects)

            file_path = Path(file)
            gltf_path = file_path.parent.joinpath(f"./{file_path.stem}/scene.gltf")

            for object in objects:
                object.hide_viewport = False

            export_gltf(gltf_path)

# execute entry point
main()
