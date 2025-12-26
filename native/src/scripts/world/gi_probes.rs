use std::future::IntoFuture;

use godot::builtin::Vector3;
use godot::classes::node::{ProcessMode, ProcessThreadGroup};
use godot::classes::voxel_gi::Subdiv;
use godot::classes::{CameraAttributes, Node3D, VoxelGi, VoxelGiData};
use godot::obj::{Gd, NewAlloc};
use godot_rust_script::{
    godot_script_impl, CastToScript, GodotScript, OnEditor, RsRef, ScriptSignal,
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

    // camera attribues that are used by the scene.
    #[export]
    pub camera_attributes: OnEditor<Gd<CameraAttributes>>,

    // Number of probes that will be used to cover the world with VoxelGI.
    //
    // The probe_count will be squared.
    #[export(range(min = 0.0, max = 20.0, suffix = "²"))]
    pub probe_count: Uf32,

    // Negative offset of the GI probes in number of tiles.
    #[export(range(min = 0.0, max = 200.0, suffix = "Tiles"))]
    pub negative_y_offset: Uf32,

    #[export]
    pub voxel_gi_data: OnEditor<Gd<VoxelGiData>>,

    #[export(storage)]
    pub is_built: bool,

    #[signal("completed_steps")]
    pub build_progress: ScriptSignal<u32>,

    base: Gd<<Self as GodotScript>::Base>,
}

const LOAD_STEP_MULTIPLIER: u32 = 100;

#[godot_script_impl]
impl GiProbes {
    /// Generate and bake GI probes for the scene.
    pub fn build_async(&self, city_size: Uf32, sea_level: Uf32) -> Gd<GodotFuture> {
        let (resolve, future) = godot_future();

        let is_built = self.is_built;
        let world_constants = self.world_constants.clone();
        let probe_count = self.probe_count;
        let negative_y_offset = self.negative_y_offset;
        let camera_attributes = self.camera_attributes.clone();
        let voxel_gi_data = self.voxel_gi_data.clone();
        let tile_size = world_constants.bind().tile_size();
        let mut base = self.base.clone();

        godot::task::spawn(async move {
            if is_built {
                logger::error!("GI Probes are already built! Reset them before rebuilding.");
                return;
            }

            // wait for one frame to clear the borrow of self.
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

            let probe_dimensions = ProbeDimensions::new(tile_size, tiles_per_probe);

            let probe_coordinates = (0..probe_count.into_u32())
                .flat_map(|x| (0..probe_count.into_u32()).map(move |y| (x, y)));

            let world_root = base.get_parent_node_3d().unwrap();
            let scene_tree = base.get_tree().unwrap();
            let mut script: RsRef<Self> = base.to_script();

            let probes = probe_coordinates
                .map(|xy| {
                    create_voxel_gi_probe(
                        &mut base,
                        &probe_dimensions,
                        &voxel_gi_data,
                        &camera_attributes,
                        xy,
                        probe_vertical_offset(
                            sea_level,
                            negative_y_offset,
                            &world_constants.bind(),
                        ),
                    )
                })
                .enumerate();

            let mut generated_probe_count = 0;

            for (index, mut probe) in probes {
                probe.bake_ex().from_node(&world_root).done();

                logger::info!(
                    "built GI probe {} / {}",
                    index + 1,
                    probe_count.into_usize().pow(2)
                );

                script.emit_build_progess(LOAD_STEP_MULTIPLIER);
                generated_probe_count += 1;

                // render one frame to update progress bar.
                scene_tree.signals().process_frame().into_future().await;
            }

            debug_assert_eq!(generated_probe_count, probe_count.into_usize().pow(2));

            script.set_built(true);
            resolve(());
        });

        future
    }

    /// Number of steps that are required to build GI probes for the scene.
    pub fn load_steps(&self) -> u32 {
        self.probe_count.into_u32().pow(2) * LOAD_STEP_MULTIPLIER
    }

    pub fn set_built(&mut self, is_built: bool) {
        self.is_built = is_built;
    }

    pub fn emit_build_progess(&self, count: u32) {
        self.build_progress.emit(count);
    }
}

/// Calculated dimensions of a `VoxelGI` node.
struct ProbeDimensions {
    size: f32,
    extent: f32,
    margin: f32,
}

impl ProbeDimensions {
    fn new(tile_size: u8, tile_count: Uf32) -> Self {
        const MARGIN_TILES: f32 = 2.0;
        const MARGIN_SIDES: f32 = 2.0;

        let size = (Uf32::from(tile_size) * tile_count).into_f32();
        let extent = size / 2.0;

        // probes get an extra margin of 2 tiles on each side so they overlap and blend together.
        let margin = Uf32::from(tile_size).into_f32() * MARGIN_TILES * MARGIN_SIDES;

        Self {
            size,
            extent,
            margin,
        }
    }
}

fn create_voxel_gi_probe(
    node: &mut Gd<Node3D>,
    dimensions: &ProbeDimensions,
    data: &Gd<VoxelGiData>,
    camera_attributes: &Gd<CameraAttributes>,
    xy: (u32, u32),
    height_offset: f32,
) -> Gd<VoxelGi> {
    let mut probe = VoxelGi::new_alloc();
    let probe_data: Gd<VoxelGiData> = data
        .duplicate()
        .expect("Should be possible to duplicate VoxelGIData")
        .cast();

    probe.set_size(Vector3::splat(dimensions.size + dimensions.margin));
    probe.set_subdiv(Subdiv::SUBDIV_64);
    probe.set_camera_attributes(camera_attributes);
    probe.set_probe_data(Some(&probe_data));
    probe.set_process_thread_group(ProcessThreadGroup::SUB_THREAD);
    probe.set_process_mode(ProcessMode::ALWAYS);

    node.add_child(&probe);

    let translate_x = dimensions.size
                    * Uf32::new(xy.0).into_f32()
                    // the initial offset is one probe extend / half the probe size
                    + dimensions.extent;

    let translate_y = dimensions.size
                    * Uf32::new(xy.1).into_f32()
                    // the initial offset is one probe extend / half the probe size
                    + dimensions.extent;

    let translate_z = height_offset + dimensions.extent;

    probe.set_global_position(Vector3::new(translate_x, translate_z, translate_y));
    probe.set_owner(&node.get_tree().unwrap().get_current_scene().unwrap());
    probe
}

#[inline]
fn probe_vertical_offset(
    sea_level: Uf32,
    negative_y_offset: Uf32,
    world_constants: &WorldConstants,
) -> f32 {
    let tile_height = f32::from(world_constants.tile_height());

    (sea_level.into_f32() * tile_height) - (negative_y_offset.into_f32() * tile_height)
}
