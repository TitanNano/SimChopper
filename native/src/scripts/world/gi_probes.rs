use std::future::IntoFuture;

use godot::{
    builtin::Vector3,
    classes::{
        voxel_gi::Subdiv, CameraAttributes, InputEvent, Node3D, ShaderMaterial, VoxelGi,
        VoxelGiData,
    },
    meta::ToGodot,
    obj::{Gd, GodotClass, NewAlloc, NewGd},
};
use godot_rust_script::{
    godot_script_impl, CastToScript, Context, GodotScript, OnEditor, RsRef, ScriptSignal,
};

use crate::{
    resources::WorldConstants,
    util::{
        async_support::{godot_future, GodotFuture},
        logger, Uf32,
    },
};

#[derive(GodotScript, Debug)]
#[script(base = Node3D)]
struct GiProbes {
    #[export]
    pub world_constants: OnEditor<Gd<WorldConstants>>,

    #[export]
    pub camera_attributes: OnEditor<Gd<CameraAttributes>>,

    #[export(range(min = 0.0, max = 20.0))]
    pub probe_count: Uf32,

    #[export(range(min = 0.0, max = 200.0))]
    pub negative_y_offset: Uf32,

    #[export]
    pub ocean_material: OnEditor<Gd<ShaderMaterial>>,

    #[export]
    pub display_debug: bool,

    #[export]
    pub is_built: bool,

    #[signal("completed_steps")]
    pub build_progress: ScriptSignal<u32>,

    probes: Vec<Gd<VoxelGi>>,

    base: Gd<<Self as GodotScript>::Base>,
}

#[godot_script_impl]
impl GiProbes {
    #[expect(clippy::needless_pass_by_value)]
    pub fn _input(&mut self, event: Gd<InputEvent>, mut context: Context<Self>) {
        if event.is_action_pressed("debug_toggle_gi") {
            let visibilty = !self.base.is_visible();

            self.base.set_visible(visibilty);
        }

        if event.is_action_pressed("debug_bake") {
            if !self.is_built {
                return;
            }

            let mut probes: Vec<_> = self.probes.iter().map(std::clone::Clone::clone).collect();

            for probe in &mut probes {
                context.reentrant_scope(self, |base: Gd<Node3D>| {
                    probe
                        .bake_ex()
                        .from_node(&base.get_parent_node_3d().unwrap())
                        .done();
                });
            }
        }
    }

    pub fn reset(&mut self) {
        self.base
            .get_children()
            .iter_shared()
            .for_each(|mut child| {
                debug_assert!(child.is_class(&VoxelGi::class_id().to_gstring()));
                child.queue_free();
            });
    }

    /// Generate and bake GI probes for the scene.
    pub fn build_async(&mut self, city_size: Uf32, sea_level: Uf32) -> Gd<GodotFuture> {
        let (resolve, future) = godot_future();

        let is_built = self.is_built;
        let display_debug = self.display_debug;
        let world_constants = self.world_constants.clone();
        let probe_count = self.probe_count;
        let build_progress = self.build_progress.name().to_string();
        let negative_y_offset = self.negative_y_offset;
        let camera_attributes = self.camera_attributes.clone();
        let ocean_material = self.ocean_material.clone();
        let mut base = self.base.clone();

        godot::task::spawn(async move {
            if is_built {
                logger::error!("GI Probes are already built! Reset them before rebuilding.");
                return;
            }

            // wait for one frame to clear the mutable borrow of self.
            base.get_tree()
                .unwrap()
                .signals()
                .process_frame()
                .into_future()
                .await;

            let tiles_per_probe = city_size / probe_count;

            logger::debug!(
                "number of tiles per GI probe: {tiles_per_probe} ({city_size} / {probe_count})"
            );

            let probe_size =
                (Uf32::from(world_constants.bind().tile_size()) * tiles_per_probe).into_f32();

            let probe_extend = probe_size / 2.0;
            let probe_margin = Uf32::from(world_constants.bind().tile_size()).into_f32() * 4.0;

            let probe_coordinates = (0..probe_count.into_u32())
                .flat_map(|x| (0..probe_count.into_u32()).map(move |y| (x, y)));

            let mut probes = 0;

            for (x, y) in probe_coordinates {
                let mut probe = VoxelGi::new_alloc();
                let mut probe_data = VoxelGiData::new_gd();

                probe_data.set_dynamic_range(1.0);
                probe_data.set_energy(0.3);
                probe_data.set_bias(2.0);
                probe_data.set_normal_bias(1.0);
                probe_data.set_propagation(0.0);
                probe_data.set_use_two_bounces(true);
                probe_data.set_interior(false);

                probe.set_size(Vector3::splat(probe_size + probe_margin));
                probe.set_subdiv(Subdiv::SUBDIV_64);
                probe.set_camera_attributes(&camera_attributes);
                probe.set_probe_data(Some(&probe_data));

                base.add_child(&probe);

                let translate_x = probe_size
                    * Uf32::new(x).into_f32()
                    // the initial offset is one probe extend / half the probe size
                    + probe_extend;

                let translate_y = probe_size
                    * Uf32::new(y).into_f32()
                    // the initial offset is one probe extend / half the probe size
                    + probe_extend;

                let height_offset = (sea_level * world_constants.bind().tile_height().into())
                    .into_f32()
                    + probe_extend
                    - (f32::from(world_constants.bind().tile_height())
                        * negative_y_offset.into_f32());

                probe.set_global_position(Vector3::new(translate_x, height_offset, translate_y));
                probe.set_owner(&base.get_tree().unwrap().get_current_scene().unwrap());

                let world_root = base.get_parent_node_3d().unwrap();

                // VoxelGi::bake tries to call `get_meshes` on all nodes in the tree, also on the `GiProbes` node.
                probe
                    .bake_ex()
                    .from_node(&world_root)
                    .create_visual_debug(display_debug)
                    .done();

                base.emit_signal(&build_progress, &[1.to_variant()]);
                probes += 1;
                logger::info!("built GI probe {probes}");

                let mut base_script: RsRef<GiProbes> = base.to_script();

                base_script.add_probe(probe);

                // give the engine time to update the UI
                base.get_tree()
                    .unwrap()
                    .signals()
                    .process_frame()
                    .to_future()
                    .await;
            }

            debug_assert_eq!(probes, probe_count.into_usize().pow(2));
            let mut script: RsRef<Self> = base.to_script();

            script.set_built(true);
            resolve(());
        });

        future
    }

    pub fn add_probe(&mut self, probe: Gd<VoxelGi>) {
        self.probes.push(probe);
    }

    /// Number of steps that are required to build GI probes for the scene.
    pub fn load_steps(&self) -> u32 {
        self.probe_count.into_u32().pow(2)
    }

    pub fn set_built(&mut self, is_built: bool) {
        self.is_built = is_built;
    }
}
