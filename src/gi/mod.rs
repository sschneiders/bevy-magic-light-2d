use bevy::shader::load_shader_library;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_graph::{self, RenderGraph, RenderLabel};
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderContext;
use bevy::render::{Render, RenderApp, RenderSystems, RenderStartup};
use bevy::sprite_render::Material2dPlugin;
use bevy::window::{PrimaryWindow, WindowResized};
use self::pipeline::GiTargets;
use crate::camera_viewer::{setup_camera_viewer, camera_viewer_window_system};
use crate::gi::compositing::{setup_post_processing_camera, CameraTargets, PostProcessingMaterial};
use crate::gi::constants::{POST_PROCESSING_MATERIAL, POST_PROCESSING_RECT};
use crate::gi::pipeline::{
    system_queue_bind_groups,
    system_setup_gi_pipeline,
    GiTargetsWrapper,
    LightPassPipeline,
    LightPassPipelineBindGroups,
};
use crate::gi::pipeline_assets::{
    system_extract_pipeline_assets,
    system_load_embedded_shader_dependencies,
    system_prepare_pipeline_assets,
    EmbeddedShaderDependencies,
    LightPassPipelineAssets,
};
use crate::gi::resource::ComputedTargetSizes;
use crate::prelude::BevyMagicLight2DSettings;

mod constants;
mod pipeline;
mod pipeline_assets;
mod types_gpu;

pub mod compositing;
pub mod render_layer;
pub mod resource;
pub mod types;
pub mod util;

const WORKGROUP_SIZE: u32 = 8;

pub struct BevyMagicLight2DPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct LightPass2DRenderLabel;

impl Plugin for BevyMagicLight2DPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins((
            ExtractResourcePlugin::<GiTargetsWrapper>::default(),
            Material2dPlugin::<PostProcessingMaterial>::default(),
            bevy_egui::EguiPlugin::default(),
        ))
        .init_resource::<CameraTargets>()
        .init_resource::<GiTargetsWrapper>()
        .init_resource::<BevyMagicLight2DSettings>()
        .init_resource::<ComputedTargetSizes>()
        .init_resource::<EmbeddedShaderDependencies>()
        .add_systems(
            PreStartup,
            (
                system_load_embedded_shader_dependencies,
                detect_target_sizes,
                system_setup_gi_pipeline.after(detect_target_sizes),
                setup_post_processing_camera.after(system_setup_gi_pipeline),
            )
                .chain(),
        )
        .add_systems(Startup, setup_camera_viewer)
        .add_systems(PreUpdate, handle_window_resize)
        .add_systems(Last, camera_viewer_window_system);

        load_shader_library!(app, "shaders/gi_attenuation.wgsl");
        load_shader_library!(app, "shaders/gi_camera.wgsl");
        load_shader_library!(app, "shaders/gi_halton.wgsl");
        load_shader_library!(app, "shaders/gi_math.wgsl");
        load_shader_library!(app, "shaders/gi_post_processing.wgsl");
        load_shader_library!(app, "shaders/gi_raymarch.wgsl");
        load_shader_library!(app, "shaders/gi_sdf.wgsl");
        load_shader_library!(app, "shaders/gi_ss_blend.wgsl");
        load_shader_library!(app, "shaders/gi_ss_bounce.wgsl");
        load_shader_library!(app, "shaders/gi_ss_filter.wgsl");
        load_shader_library!(app, "shaders/gi_ss_probe.wgsl");
        load_shader_library!(app, "shaders/gi_types.wgsl");

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(ExtractSchedule, system_extract_pipeline_assets)
            .add_systems(RenderStartup, (
                init_light_pass_pipeline,
                init_light_pass_pipeline_assets,
                init_computed_target_sizes,
            ).chain())
            .add_systems(
                Render,
                (
                    system_prepare_pipeline_assets.in_set(RenderSystems::Prepare),
                    system_queue_bind_groups.in_set(RenderSystems::Queue),
                ),
            );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(LightPass2DRenderLabel, LightPass2DNode::default());
        render_graph.add_node_edge(
            LightPass2DRenderLabel,
            bevy::render::graph::CameraDriverLabel,
        )
    }
}

#[derive(Default)]
struct LightPass2DNode {}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
pub fn handle_window_resize(

    mut assets_mesh:     ResMut<Assets<Mesh>>,
    mut assets_material: ResMut<Assets<PostProcessingMaterial>>,
    mut assets_image:    ResMut<Assets<Image>>,

    query_window: Query<&Window, With<PrimaryWindow>>,

        res_plugin_config:      Res<BevyMagicLight2DSettings>,
    mut res_target_sizes:       ResMut<ComputedTargetSizes>,
    mut res_gi_targets_wrapper: ResMut<GiTargetsWrapper>,
    mut res_camera_targets:     ResMut<CameraTargets>,

    mut window_resized_evr: MessageReader<WindowResized>,
) {
    for _ in window_resized_evr.read() {
        let window = query_window
            .single()
            .expect("Expected exactly one primary window");

        *res_target_sizes =
            ComputedTargetSizes::from_window(window, &res_plugin_config.target_scaling_params);

        if !res_target_sizes.is_valid() {
            // Window might be minimized, skip updating resources.
            return;
        }
        
        let _ = assets_mesh.insert(
            POST_PROCESSING_RECT.id(),
            Mesh::from(bevy::math::primitives::Rectangle::new(
                res_target_sizes.primary_target_size.x,
                res_target_sizes.primary_target_size.y,
            )),
        );

        let _ = assets_material.insert(
            POST_PROCESSING_MATERIAL.id(),
            PostProcessingMaterial::create(&res_camera_targets, &res_gi_targets_wrapper),
        );

        *res_gi_targets_wrapper = GiTargetsWrapper{targets: Some(GiTargets::create(&mut assets_image, &res_target_sizes))};
        *res_camera_targets = CameraTargets::create(&mut assets_image, &res_target_sizes);
    }
}

#[rustfmt::skip]
pub fn detect_target_sizes(
        query_window:      Query<&Window, With<PrimaryWindow>>,

        res_plugin_config: Res<BevyMagicLight2DSettings>,
    mut res_target_sizes:  ResMut<ComputedTargetSizes>,
)
{
    let window = query_window.single().expect("Expected exactly one primary window");
    *res_target_sizes = ComputedTargetSizes::from_window(window, &res_plugin_config.target_scaling_params);
}

impl render_graph::Node for LightPass2DNode
{
    fn update(&mut self, _world: &mut World) {}

    #[rustfmt::skip]
    fn run(
        &self,
        _: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if let Some(pipeline_bind_groups) = world.get_resource::<LightPassPipelineBindGroups>() {
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<LightPassPipeline>();
            let target_sizes = world.resource::<ComputedTargetSizes>();

            if let (
                Some(sdf_pipeline),
                Some(ss_probe_pipeline),
                Some(ss_bounce_pipeline),
                Some(ss_blend_pipeline),
                Some(ss_filter_pipeline),
            ) = (
                pipeline_cache.get_compute_pipeline(pipeline.sdf_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_probe_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_bounce_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_blend_pipeline),
                pipeline_cache.get_compute_pipeline(pipeline.ss_filter_pipeline),
            ) {
                let sdf_w = target_sizes.sdf_target_usize.x;
                let sdf_h = target_sizes.sdf_target_usize.y;

                let mut pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor { label: Some("light_pass_2d"), ..default() });

                {
                    let grid_w = sdf_w / WORKGROUP_SIZE;
                    let grid_h = sdf_h / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.sdf_bind_group, &[]);
                    pass.set_pipeline(sdf_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let grid_w = target_sizes.probe_grid_usize.x / WORKGROUP_SIZE;
                    let grid_h = target_sizes.probe_grid_usize.y / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_probe_bind_group, &[]);
                    pass.set_pipeline(ss_probe_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let grid_w = target_sizes.probe_grid_usize.x / WORKGROUP_SIZE;
                    let grid_h = target_sizes.probe_grid_usize.y / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_bounce_bind_group, &[]);
                    pass.set_pipeline(ss_bounce_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let grid_w = target_sizes.probe_grid_usize.x / WORKGROUP_SIZE;
                    let grid_h = target_sizes.probe_grid_usize.y / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_blend_bind_group, &[]);
                    pass.set_pipeline(ss_blend_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }

                {
                    let aligned = util::align_to_work_group_grid(target_sizes.primary_target_isize).as_uvec2();
                    let grid_w = aligned.x / WORKGROUP_SIZE;
                    let grid_h = aligned.y / WORKGROUP_SIZE;
                    pass.set_bind_group(0, &pipeline_bind_groups.ss_filter_bind_group, &[]);
                    pass.set_pipeline(ss_filter_pipeline);
                    pass.dispatch_workgroups(grid_w, grid_h, 1);
                }
            }
        } else {
            log::warn!("Failed to get bind groups");
        }

        Ok(())
    }
}

// RenderStartup initialization functions for Bevy 0.17
fn init_light_pass_pipeline(mut commands: Commands) {
    commands.init_resource::<LightPassPipeline>();
}

fn init_light_pass_pipeline_assets(mut commands: Commands) {
    commands.init_resource::<LightPassPipelineAssets>();
}

fn init_computed_target_sizes(mut commands: Commands) {
    commands.init_resource::<ComputedTargetSizes>();
}
